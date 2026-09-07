use crate::models::{
    DevelopmentSnapshot, EcosystemScore, Genome, GenomeDimension, LiquiditySnapshot,
    MarketSnapshot, OnchainSnapshot, ScoreComponent, ScoreDriver, SocialSnapshot,
};
use chrono::Utc;

pub const HEALTH_ALGO: &str = "ecosystem-health";
pub const HEALTH_VERSION: &str = "1.0.0";
pub const GENOME_VERSION: &str = "1.0.0";

pub fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

pub fn market_health(m: &MarketSnapshot, liq: &LiquiditySnapshot) -> f64 {
    let vol_to_mcap = if m.market_cap_usd > 0.0 {
        (m.volume_24h_usd / m.market_cap_usd * 100.0).min(40.0)
    } else {
        0.0
    };
    let liq_score = (liq.liquidity_usd.log10() * 12.0).clamp(0.0, 40.0);
    let vol_score = vol_to_mcap * 0.8;
    let spread_penalty = (liq.spread_bps / 4.0).min(20.0);
    let stability = 20.0 - m.change_24h_pct.abs().min(20.0);
    clamp(liq_score + vol_score + stability - spread_penalty)
}

pub fn onchain_activity(o: &OnchainSnapshot) -> f64 {
    let holder_scale = (o.holders as f64).log10() * 12.0;
    let active_ratio = if o.holders > 0 {
        o.active_holders_30d as f64 / o.holders as f64
    } else {
        0.0
    };
    clamp(holder_scale + active_ratio * 180.0 + (o.transfers_24h as f64).log10() * 8.0)
}

pub fn community(s: &SocialSnapshot, o: &OnchainSnapshot) -> f64 {
    let organic = s.organicness * 35.0;
    let unique = (s.unique_accounts_24h as f64).log10().max(0.0) * 12.0;
    let retention = o.retained_holders_30d_pct * 0.25;
    let bot_pen = s.bot_probability * 25.0;
    clamp(organic + unique + retention + s.engagement_score * 0.25 - bot_pen)
}

pub fn development(d: &DevelopmentSnapshot) -> f64 {
    let recency = match d.last_commit_days {
        Some(days) if days <= 7 => 30.0,
        Some(days) if days <= 30 => 18.0,
        Some(days) if days <= 90 => 8.0,
        Some(_) => 2.0,
        None => 4.0,
    };
    clamp(recency + (d.commits_30d as f64).min(40.0) + (d.active_contributors_30d as f64) * 4.0)
}

pub fn utility(def_narratives: &[String], d: &DevelopmentSnapshot) -> f64 {
    let narrative_bonus = if def_narratives.iter().any(|n| n == "ai" || n == "utility") {
        18.0
    } else {
        8.0
    };
    clamp(narrative_bonus + d.activity_score * 0.4)
}

pub fn adoption(o: &OnchainSnapshot, m: &MarketSnapshot) -> f64 {
    let holders = (o.holders as f64).log10() * 14.0;
    let mcap = (m.market_cap_usd.max(1.0).log10() * 8.0).min(35.0);
    clamp(holders + mcap)
}

pub fn liquidity_score(l: &LiquiditySnapshot) -> f64 {
    let depth = ((l.depth_plus_2pct_usd + l.depth_minus_2pct_usd) / 2.0).max(1.0).log10() * 16.0;
    let spread = 30.0 - (l.spread_bps / 3.0).min(30.0);
    clamp(depth + spread + (l.pool_count as f64).min(10.0) * 2.0)
}

pub fn governance(enabled: bool) -> f64 {
    if enabled {
        45.0
    } else {
        12.0
    }
}

pub fn treasury(enabled: bool) -> f64 {
    if enabled {
        40.0
    } else {
        10.0
    }
}

pub fn social_momentum(s: &SocialSnapshot) -> f64 {
    let unique = (s.unique_accounts_24h as f64).log10().max(0.0) * 18.0;
    let quality = s.engagement_score * 0.3;
    let organic = s.organicness * 25.0;
    let sent = (s.sentiment_net + 1.0) * 10.0;
    let diversity = s.narrative_diversity * 15.0;
    clamp(unique + quality + organic + sent + diversity - s.bot_probability * 20.0)
}

pub fn whale_risk_from_onchain(o: &OnchainSnapshot) -> f64 {
    clamp(o.top10_concentration_pct * 1.1 + o.large_transfers_24h as f64 * 2.0)
}

pub fn ecosystem_health(
    market: &MarketSnapshot,
    liq: &LiquiditySnapshot,
    onchain: &OnchainSnapshot,
    social: &SocialSnapshot,
    dev: &DevelopmentSnapshot,
    narratives: &[String],
    has_gov: bool,
    has_treasury: bool,
    risk_score: f64,
) -> EcosystemScore {
    let mh = market_health(market, liq);
    let liqs = liquidity_score(liq);
    let oc = onchain_activity(onchain);
    let comm = community(social, onchain);
    let devs = development(dev);
    let util = utility(narratives, dev);
    let adop = adoption(onchain, market);
    let gov = governance(has_gov);
    let tre = treasury(has_treasury);
    let risk_component = clamp(100.0 - risk_score);

    let mut components = vec![
        (comp("market", "Market Health", mh, 0.15, market, liq), market.data_state.present() || liq.data_state.present()),
        (comp_simple("liquidity", "Liquidity", liqs, 0.12, vec![
            drv("USD liquidity", liq.liquidity_usd / 10_000.0, "input"),
            drv("spread (bps)", -liq.spread_bps, "penalty"),
        ]), liq.data_state.present()),
        (comp_simple("onchain", "On-chain Activity", oc, 0.12, vec![
            drv("holders", onchain.holders as f64, "input"),
            drv("30d active", onchain.active_holders_30d as f64, "input"),
        ]), onchain.data_state.present()),
        (comp_simple("community", "Community", comm, 0.12, vec![
            drv("organicness", social.organicness * 100.0, "input"),
            drv("bot probability", -social.bot_probability * 100.0, "penalty"),
        ]), social.data_state.present()),
        (comp_simple("development", "Development", devs, 0.10, vec![
            drv("commits 30d", dev.commits_30d as f64, "input"),
            drv("contributors", dev.active_contributors_30d as f64, "input"),
        ]), dev.data_state.present()),
        (comp_simple("utility", "Utility", util, 0.08, vec![
            drv("narrative/utility", util, "input"),
        ]), dev.data_state.present() || !narratives.is_empty()),
        (comp_simple("adoption", "Adoption", adop, 0.08, vec![
            drv("holder base", onchain.holders as f64, "input"),
        ]), onchain.data_state.present() || market.data_state.present()),
        (comp_simple("governance", "Governance", gov, 0.05, vec![
            drv("governance tracked", if has_gov { 1.0 } else { 0.0 }, "input"),
        ]), true),
        (comp_simple("treasury", "Treasury", tre, 0.05, vec![
            drv("treasury tracked", if has_treasury { 1.0 } else { 0.0 }, "input"),
        ]), true),
        (comp_simple("risk", "Risk (inverted)", risk_component, 0.13, vec![
            drv("risk score", -risk_score, "penalty"),
        ]), market.data_state.present() || liq.data_state.present()),
    ];

    let kept: Vec<ScoreComponent> = components
        .drain(..)
        .filter_map(|(c, keep)| {
            if keep {
                Some(c)
            } else {
                let mut c = c;
                c.weight = 0.0;
                c.drivers.push(drv("skipped (MISSING source)", 0.0, "skip"));
                Some(c)
            }
        })
        .collect();
    let active_w: f64 = kept.iter().filter(|c| c.weight > 0.0).map(|c| c.weight).sum();
    let components: Vec<ScoreComponent> = kept
        .into_iter()
        .map(|mut c| {
            if active_w > 0.0 && c.weight > 0.0 {
                c.weight /= active_w;
            }
            c
        })
        .collect();

    let value = components
        .iter()
        .map(|c| c.value * c.weight)
        .sum::<f64>();

    let confidence = mean(&[
        market.provenance.confidence,
        liq.provenance.confidence,
        onchain.provenance.confidence,
        social.provenance.confidence,
        dev.provenance.confidence,
    ]);

    let top = {
        let mut c = components.clone();
        c.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        c
    };

    let why = format!(
        "Health {value:.0} reflects observable ecosystem quality, not expected price. Strongest: {}. Weakest: {}. Confidence {confidence:.0}%.",
        top[0].label,
        top.last().map(|c| c.label.as_str()).unwrap_or("n/a"),
        value = value,
        confidence = confidence * 100.0
    );

    EcosystemScore {
        algorithm: HEALTH_ALGO.into(),
        algorithm_version: HEALTH_VERSION.into(),
        value: clamp(value),
        confidence,
        as_of: Utc::now(),
        components,
        why,
    }
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn drv(label: &str, delta: f64, direction: &str) -> ScoreDriver {
    ScoreDriver {
        label: label.into(),
        delta,
        direction: direction.into(),
    }
}

fn comp_simple(id: &str, label: &str, value: f64, weight: f64, drivers: Vec<ScoreDriver>) -> ScoreComponent {
    ScoreComponent {
        id: id.into(),
        label: label.into(),
        value: clamp(value),
        weight,
        drivers,
    }
}

fn comp(
    id: &str,
    label: &str,
    value: f64,
    weight: f64,
    m: &MarketSnapshot,
    l: &LiquiditySnapshot,
) -> ScoreComponent {
    ScoreComponent {
        id: id.into(),
        label: label.into(),
        value: clamp(value),
        weight,
        drivers: vec![
            drv("24h volume / mcap", m.volume_24h_usd / m.market_cap_usd.max(1.0), "input"),
            drv("liquidity USD", l.liquidity_usd, "input"),
            drv("24h change abs", -m.change_24h_pct.abs(), "penalty"),
        ],
    }
}

pub fn genome(
    market: &MarketSnapshot,
    liq: &LiquiditySnapshot,
    onchain: &OnchainSnapshot,
    social: &SocialSnapshot,
    dev: &DevelopmentSnapshot,
    narratives: &[String],
    has_gov: bool,
    has_treasury: bool,
    risk_score: f64,
) -> Genome {
    let dims = vec![
        dim("narrative", "Narrative", if narratives.iter().any(|n| n == "ai") { 72.0 } else if narratives.len() >= 3 { 64.0 } else { 48.0 }),
        dim("community", "Community", community(social, onchain)),
        dim("utility", "Utility", utility(narratives, dev)),
        dim("development", "Development", development(dev)),
        dim("liquidity", "Liquidity", liquidity_score(liq)),
        dim("exchange_reach", "Exchange Reach", clamp((market.volume_24h_usd / 50_000.0).min(80.0) + if market.market_cap_usd > 20_000_000.0 { 20.0 } else { 8.0 })),
        dim("social_momentum", "Social Momentum", social_momentum(social)),
        dim("holder_retention", "Holder Retention", onchain.retained_holders_30d_pct),
        dim("whale_risk", "Whale Risk", whale_risk_from_onchain(onchain)),
        dim("developer_activity", "Developer Activity", dev.activity_score),
        dim("governance", "Governance", governance(has_gov)),
        dim("treasury", "Treasury", treasury(has_treasury)),
        dim("brand", "Brand Strength", clamp(community(social, onchain) * 0.7 + adoption(onchain, market) * 0.3)),
        dim("ecosystem_activity", "Ecosystem Activity", onchain_activity(onchain)),
        dim("transparency", "Transparency", clamp(55.0 + if has_treasury { 15.0 } else { 0.0 } + if has_gov { 10.0 } else { 0.0 } - if social.bot_probability > 0.25 { 12.0 } else { 0.0 })),
        dim("risk_inverse", "Risk Control", clamp(100.0 - risk_score)),
    ];
    Genome {
        algorithm_version: GENOME_VERSION.into(),
        dimensions: dims,
    }
}

fn dim(id: &str, label: &str, value: f64) -> GenomeDimension {
    GenomeDimension {
        id: id.into(),
        label: label.into(),
        value: clamp(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DataState;

    #[test]
    fn weights_sum_to_one() {
        let sum: f64 = 0.15 + 0.12 + 0.12 + 0.12 + 0.10 + 0.08 + 0.08 + 0.05 + 0.05 + 0.13;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn clamp_bounds() {
        assert_eq!(clamp(-4.0), 0.0);
        assert_eq!(clamp(140.0), 100.0);
    }

    #[test]
    fn skip_missing_and_renormalize() {
        let market = crate::quality::missing_market();
        let liq = crate::quality::missing_liquidity();
        let onchain = crate::quality::missing_onchain();
        let social = crate::quality::missing_social();
        let mut dev = crate::quality::missing_development();
        dev.data_state = DataState::Live;
        dev.commits_30d = 12;
        dev.provenance.confidence = 0.8;
        let score = ecosystem_health(&market, &liq, &onchain, &social, &dev, &["ai".into()], false, false, 40.0);
        let skipped = score.components.iter().filter(|c| c.weight == 0.0).count();
        assert!(skipped >= 3, "missing sources must be skipped");
        let active: f64 = score.components.iter().map(|c| c.weight).sum();
        assert!((active - 1.0).abs() < 1e-6, "weights renormalize to 1, got {active}");
        let dev_w = score.components.iter().find(|c| c.id == "development").unwrap().weight;
        assert!(dev_w > 0.10, "remaining weight concentrates on present sources");
    }
}
