use crate::models::{
    DevelopmentSnapshot, LiquiditySnapshot, MarketSnapshot, OnchainSnapshot, RiskFactor,
    RiskLevel, RiskReport, SocialSnapshot, TokenDefinition,
};
use crate::scoring::clamp;
use chrono::Utc;

pub const RISK_ALGO: &str = "token-risk";
pub const RISK_VERSION: &str = "1.0.0";

pub fn classify(score: f64) -> RiskLevel {
    match score {
        s if s < 25.0 => RiskLevel::Low,
        s if s < 45.0 => RiskLevel::Moderate,
        s if s < 70.0 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}

fn factor(id: &str, label: &str, score: f64, evidence: Vec<String>) -> RiskFactor {
    RiskFactor {
        id: id.into(),
        label: label.into(),
        score: clamp(score),
        level: classify(score),
        evidence,
    }
}

fn unknown(id: &str, label: &str, why: &str) -> RiskFactor {
    RiskFactor {
        id: id.into(),
        label: label.into(),
        score: 0.0,
        level: RiskLevel::Unknown,
        evidence: vec![why.into()],
    }
}

pub fn assess(
    def: &TokenDefinition,
    market: &MarketSnapshot,
    liq: &LiquiditySnapshot,
    onchain: &OnchainSnapshot,
    social: &SocialSnapshot,
    dev: &DevelopmentSnapshot,
) -> RiskReport {
    let contract = match def.verification {
        crate::models::VerificationLevel::Unverified => 62.0,
        crate::models::VerificationLevel::DataVerified => 38.0,
        crate::models::VerificationLevel::ContractVerified => 28.0,
        crate::models::VerificationLevel::EcosystemVerified => 22.0,
        crate::models::VerificationLevel::CommunityVerified => 18.0,
    };

    let liquidity_risk = clamp(
        (80.0 - liq.liquidity_usd.log10() * 10.0).max(8.0) + (liq.spread_bps / 2.0).min(25.0),
    );
    let concentration = clamp(onchain.top10_concentration_pct * 1.35);
    let whale = clamp(
        onchain.top10_concentration_pct * 0.7
            + onchain.large_transfers_24h as f64 * 3.5
            + if onchain.exchange_inflow_usd > onchain.exchange_outflow_usd * 1.8 {
                12.0
            } else {
                0.0
            },
    );
    let governance = if def.features.governance { 20.0 } else { 48.0 };
    let exchange = if market.volume_24h_usd < 100_000.0 {
        55.0
    } else if market.volume_24h_usd < 1_000_000.0 {
        36.0
    } else {
        18.0
    };
    let social_manip = clamp(
        social.bot_probability * 70.0 + social.spam_ratio * 40.0 - social.organicness * 15.0,
    );
    let development_risk = match dev.last_commit_days {
        Some(d) if d > 120 => 72.0,
        Some(d) if d > 45 => 48.0,
        Some(_) => 22.0,
        None => 40.0,
    };
    let dependency = 28.0;
    let vol = market.change_24h_pct.abs();
    let market_risk = clamp(20.0 + vol * 2.2 + (100.0 + market.ath_distance_pct.unwrap_or(-50.0)).max(0.0) * 0.15);

    let factors = vec![
        factor("contract", "Contract", contract, vec![
            format!("verification={:?}", def.verification),
            format!("primary chain={}", def.primary_chain().map(|c| c.id.as_str()).unwrap_or("unknown")),
        ]),
        if liq.data_state.present() {
            factor("liquidity", "Liquidity", liquidity_risk, vec![
                format!("liquidity_usd={:.0}", liq.liquidity_usd),
                format!("spread_bps={:.0}", liq.spread_bps),
                format!("lp_change_7d={:.1}%", liq.lp_change_7d_pct),
            ])
        } else {
            unknown("liquidity", "Liquidity", "MISSING — no live DEX source")
        },
        if onchain.data_state.present() && onchain.top10_concentration_pct > 0.0 {
            factor("concentration", "Concentration", concentration, vec![
                format!("top10={:.1}%", onchain.top10_concentration_pct),
                format!("top50={:.1}%", onchain.top50_concentration_pct),
            ])
        } else if onchain.data_state.present() {
            unknown(
                "concentration",
                "Concentration",
                "Holder count is live; distribution not sampled",
            )
        } else {
            unknown("concentration", "Concentration", "MISSING — holder indexer not connected")
        },
        if onchain.data_state.present() && onchain.top10_concentration_pct > 0.0 {
            factor("whale", "Whale", whale, vec![
                format!("whale_holders={}", onchain.whale_holders),
                format!("large_transfers_24h={}", onchain.large_transfers_24h),
                format!("cex_inflow={:.0}", onchain.exchange_inflow_usd),
            ])
        } else if onchain.data_state.present() {
            unknown("whale", "Whale", "Holder count is live; whale transfers not indexed")
        } else {
            unknown("whale", "Whale", "MISSING — whale indexer not connected")
        },
        factor("governance", "Governance", governance, vec![
            format!("governance_tracked={}", def.features.governance),
        ]),
        if market.data_state.present() {
            factor("exchange", "Exchange", exchange, vec![
                format!("volume_24h={:.0}", market.volume_24h_usd),
            ])
        } else {
            unknown("exchange", "Exchange", "MISSING — no live market volume")
        },
        if social.data_state.present() {
            factor("social_manipulation", "Social manipulation", social_manip, vec![
                format!("bot_probability={:.2}", social.bot_probability),
                format!("spam_ratio={:.2}", social.spam_ratio),
                format!("organicness={:.2}", social.organicness),
            ])
        } else {
            unknown("social_manipulation", "Social manipulation", "MISSING — social firehose not connected")
        },
        if dev.data_state.present() {
            factor("development", "Development", development_risk, vec![
                format!("last_commit_days={:?}", dev.last_commit_days),
                format!("commits_30d={}", dev.commits_30d),
            ])
        } else {
            unknown("development", "Development", "MISSING — GitHub not configured")
        },
        factor("dependency", "Dependency", dependency, vec![
            "single-chain primary listing risk assessed from definition".into(),
        ]),
        if market.data_state.present() {
            factor("market", "Market", market_risk, vec![
                format!("change_24h={:.2}%", market.change_24h_pct),
                format!("ath_distance={:?}", market.ath_distance_pct),
            ])
        } else {
            unknown("market", "Market", "MISSING — no live price")
        },
    ];

    let known: Vec<_> = factors.iter().filter(|f| f.level != RiskLevel::Unknown).collect();
    let score = if known.is_empty() {
        0.0
    } else {
        known.iter().map(|f| f.score).sum::<f64>() / known.len() as f64
    };
    let level = if known.is_empty() {
        RiskLevel::Unknown
    } else {
        classify(score)
    };
    let confidence = mean(&[
        market.provenance.confidence,
        liq.provenance.confidence,
        onchain.provenance.confidence,
        social.provenance.confidence,
    ]);

    let why = format!(
        "Risk {score:.0} ({level:?}) is a composite of contract, liquidity, concentration, whale, social and development factors. It is not a prediction of price. Highest factor: {}.",
        factors
            .iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            .map(|f| f.label.as_str())
            .unwrap_or("n/a")
    );

    RiskReport {
        algorithm: RISK_ALGO.into(),
        algorithm_version: RISK_VERSION.into(),
        score: clamp(score),
        level,
        confidence,
        as_of: Utc::now(),
        factors,
        why,
    }
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len().max(1) as f64
}

/// Detection-only signals. Never used to reproduce manipulation.
pub fn manipulation_flags(
    liq: &LiquiditySnapshot,
    onchain: &OnchainSnapshot,
    social: &SocialSnapshot,
    market: &MarketSnapshot,
) -> Vec<String> {
    let mut flags = Vec::new();
    if liq.lp_change_7d_pct < -15.0 {
        flags.push("sudden_liquidity_removal".into());
    }
    if social.bot_probability > 0.35 && social.mentions_24h > 200 {
        flags.push("artificial_social_engagement".into());
    }
    if market.volume_24h_usd > liq.liquidity_usd * 8.0 && liq.liquidity_usd > 0.0 {
        flags.push("abnormal_volume_vs_liquidity".into());
    }
    if onchain.top10_concentration_pct > 50.0 {
        flags.push("suspicious_holder_concentration".into());
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_thresholds() {
        assert!(matches!(classify(10.0), RiskLevel::Low));
        assert!(matches!(classify(40.0), RiskLevel::Moderate));
        assert!(matches!(classify(60.0), RiskLevel::High));
        assert!(matches!(classify(90.0), RiskLevel::Critical));
    }

    #[test]
    fn unknown_when_evidence_missing() {
        let def = crate::models::TokenDefinition {
            schema_version: 1,
            token_id: "x".into(),
            symbol: "X".into(),
            name: "X".into(),
            status: crate::models::TokenStatus::Unverified,
            verification: crate::models::VerificationLevel::Unverified,
            category: "memecoin".into(),
            narratives: vec![],
            launch_date: None,
            website: None,
            description: None,
            chains: vec![crate::models::ChainBinding {
                id: "ethereum".into(),
                standard: "erc20".into(),
                is_primary: true,
                contracts: crate::models::ChainContracts {
                    token: crate::models::TokenContract {
                        address: "0x6982508145454Ce325dDbE47a25d4ec3d2311933".into(),
                        decimals: 18,
                        explorer: None,
                    },
                },
            }],
            socials: Default::default(),
            data_sources: crate::models::DataSources {
                onchain: true,
                social: true,
                github: false,
                exchanges: true,
                news: false,
                market: None,
            },
            features: Default::default(),
            supply: Default::default(),
        };
        let report = assess(
            &def,
            &crate::quality::missing_market(),
            &crate::quality::missing_liquidity(),
            &crate::quality::missing_onchain(),
            &crate::quality::missing_social(),
            &crate::quality::missing_development(),
        );
        assert!(report.factors.iter().any(|f| f.level == RiskLevel::Unknown));
        assert!(report.factors.iter().filter(|f| f.id == "whale").all(|f| f.level == RiskLevel::Unknown));
    }

    #[test]
    fn holder_count_without_distribution_stays_unknown() {
        let def = crate::models::TokenDefinition {
            schema_version: 1,
            token_id: "x".into(),
            symbol: "X".into(),
            name: "X".into(),
            status: crate::models::TokenStatus::Listed,
            verification: crate::models::VerificationLevel::DataVerified,
            category: "memecoin".into(),
            narratives: vec![],
            launch_date: None,
            website: None,
            description: None,
            chains: vec![],
            socials: Default::default(),
            data_sources: crate::models::DataSources {
                onchain: true,
                social: false,
                github: false,
                exchanges: false,
                news: false,
                market: None,
            },
            features: Default::default(),
            supply: Default::default(),
        };
        let mut onchain = crate::quality::missing_onchain();
        onchain.holders = 50_000;
        onchain.top10_concentration_pct = 0.0;
        onchain.data_state = crate::models::DataState::Live;
        let report = assess(
            &def,
            &crate::quality::missing_market(),
            &crate::quality::missing_liquidity(),
            &onchain,
            &crate::quality::missing_social(),
            &crate::quality::missing_development(),
        );
        let conc = report.factors.iter().find(|f| f.id == "concentration").unwrap();
        assert_eq!(conc.level, RiskLevel::Unknown);
    }
}
