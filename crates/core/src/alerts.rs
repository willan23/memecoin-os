use crate::models::{Alert, DataState, TokenSnapshot};
use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn fingerprint(token_id: &str, kind: &str, key: &str) -> String {
    let mut h = Sha256::new();
    h.update(token_id.as_bytes());
    h.update(b"|");
    h.update(kind.as_bytes());
    h.update(b"|");
    h.update(key.as_bytes());
    hex::encode(h.finalize())
}

pub fn hour_bucket(now: chrono::DateTime<Utc>) -> String {
    now.format("%Y%m%d%H").to_string()
}

/// Threshold evaluation. Never fires on MISSING sources.
pub fn evaluate(snap: &TokenSnapshot, prev: Option<&TokenSnapshot>) -> Vec<Alert> {
    let mut out = Vec::new();
    let hour = hour_bucket(Utc::now());

    if snap.market.data_state == DataState::Conflict {
        out.push(make(
            snap,
            "price_conflict",
            "high",
            format!("{} provider prices disagree", snap.token.symbol),
            "Relative divergence exceeded 25%. Treat price as DATA_CONFLICT until sources agree.".into(),
            vec![
                snap.market.provenance.provider.clone(),
                format!("data_state={}", snap.market.data_state.as_str()),
            ],
            fingerprint(&snap.token.id, "price_conflict", &hour),
        ));
    }

    // Freshness is about stale market/liquidity (or a stale rollup), never about
    // expected MISSING holders/social zeros.
    if snap.market.data_state == DataState::Stale
        || snap.liquidity.data_state == DataState::Stale
        || snap.data_state == DataState::Stale
    {
        out.push(make(
            snap,
            "data_freshness",
            "medium",
            format!("{} data is not live", snap.token.symbol),
            format!(
                "Rolled-up data_state={} market={} liquidity={}",
                snap.data_state.as_str(),
                snap.market.data_state.as_str(),
                snap.liquidity.data_state.as_str()
            ),
            vec!["freshness window exceeded".into()],
            fingerprint(&snap.token.id, "data_freshness", &hour),
        ));
    }

    if let Some(prev) = prev {
        if snap.liquidity.data_state.present() && prev.liquidity.data_state.present() {
            let prev_liq = prev.liquidity.liquidity_usd.max(1.0);
            let drop = (prev_liq - snap.liquidity.liquidity_usd) / prev_liq;
            if drop >= 0.15 {
                out.push(make(
                    snap,
                    "liquidity_anomaly",
                    "high",
                    format!("{} liquidity dropped {:.0}%", snap.token.symbol, drop * 100.0),
                    format!(
                        "Liquidity ${:.0} → ${:.0}",
                        prev.liquidity.liquidity_usd, snap.liquidity.liquidity_usd
                    ),
                    vec![format!("pool_count={}", snap.liquidity.pool_count)],
                    fingerprint(&snap.token.id, "liquidity_anomaly", &hour),
                ));
            }
        }
        if snap.market.data_state.present()
            && prev.market.data_state.present()
            && (snap.scores.value - prev.scores.value).abs() >= 5.0
        {
            out.push(make(
                snap,
                "score_change",
                "medium",
                format!(
                    "{} health {:+.0}",
                    snap.token.symbol,
                    snap.scores.value - prev.scores.value
                ),
                format!("{:.0} → {:.0}", prev.scores.value, snap.scores.value),
                vec![format!(
                    "algo {} {}",
                    snap.scores.algorithm, snap.scores.algorithm_version
                )],
                fingerprint(&snap.token.id, "score_change", &hour),
            ));
        }
        if format!("{:?}", snap.risk.level) != format!("{:?}", prev.risk.level) {
            out.push(make(
                snap,
                "risk_change",
                "high",
                format!(
                    "{} risk {:?} → {:?}",
                    snap.token.symbol, prev.risk.level, snap.risk.level
                ),
                snap.risk.why.clone(),
                vec![format!("score={:.0}", snap.risk.score)],
                fingerprint(&snap.token.id, "risk_change", &hour),
            ));
        }
        let now_phase = crate::ecosystem::classify(snap);
        let prev_phase = crate::ecosystem::classify(prev);
        if now_phase != prev_phase {
            out.push(make(
                snap,
                "lifecycle_transition",
                "medium",
                format!(
                    "{} lifecycle {:?} → {:?}",
                    snap.token.symbol, prev_phase, now_phase
                ),
                "Classified from observed Twin fields. Not a price regime.".into(),
                vec![
                    format!("from={}", prev_phase.as_str()),
                    format!("to={}", now_phase.as_str()),
                ],
                fingerprint(&snap.token.id, "lifecycle_transition", &hour),
            ));
        }
        if snap.onchain.data_state.present() && prev.onchain.data_state.present() {
            let delta = snap.onchain.top10_concentration_pct - prev.onchain.top10_concentration_pct;
            if delta.abs() >= 5.0 {
                out.push(make(
                    snap,
                    "holder_concentration_change",
                    "high",
                    format!(
                        "{} top10 concentration {:+.1}pp",
                        snap.token.symbol, delta
                    ),
                    format!(
                        "{:.1}% → {:.1}%",
                        prev.onchain.top10_concentration_pct, snap.onchain.top10_concentration_pct
                    ),
                    vec![
                        format!("holders={}", snap.onchain.holders),
                        format!("data_state={}", snap.onchain.data_state.as_str()),
                    ],
                    fingerprint(&snap.token.id, "holder_concentration_change", &hour),
                ));
            }
        }
        if snap.social.data_state.present() && prev.social.data_state.present() {
            let prev_acc = prev.social.unique_accounts_24h.max(1) as f64;
            let growth = (snap.social.unique_accounts_24h as f64 - prev_acc) / prev_acc;
            if growth.abs() >= 0.5 {
                out.push(make(
                    snap,
                    "community_growth_anomaly",
                    "medium",
                    format!(
                        "{} unique accounts 24h {:+.0}%",
                        snap.token.symbol,
                        growth * 100.0
                    ),
                    format!(
                        "{} → {}",
                        prev.social.unique_accounts_24h, snap.social.unique_accounts_24h
                    ),
                    vec![format!("organicness={:.2}", snap.social.organicness)],
                    fingerprint(&snap.token.id, "community_growth_anomaly", &hour),
                ));
            }
            let org_drop = prev.social.organicness - snap.social.organicness;
            let bot_jump = snap.social.bot_probability - prev.social.bot_probability;
            if org_drop >= 0.15 || bot_jump >= 0.2 {
                out.push(make(
                    snap,
                    "social_anomaly",
                    "high",
                    format!("{} social quality shifted", snap.token.symbol),
                    format!(
                        "organicness {:.2}→{:.2} bot {:.2}→{:.2}",
                        prev.social.organicness,
                        snap.social.organicness,
                        prev.social.bot_probability,
                        snap.social.bot_probability
                    ),
                    vec![snap.social.provenance.provider.clone()],
                    fingerprint(&snap.token.id, "social_anomaly", &hour),
                ));
            }
        }
        if snap.development.data_state.present()
            && prev.development.data_state.present()
            && snap.development.commits_30d > prev.development.commits_30d.saturating_add(10)
        {
            out.push(make(
                snap,
                "development_spike",
                "low",
                format!("{} development spike", snap.token.symbol),
                format!(
                    "commits_30d {} → {}",
                    prev.development.commits_30d, snap.development.commits_30d
                ),
                vec![snap.development.provenance.provider.clone()],
                fingerprint(&snap.token.id, "development_spike", &hour),
            ));
        }
    }

    if snap.liquidity.data_state.present() && snap.liquidity.lp_change_7d_pct <= -5.0 {
        out.push(make(
            snap,
            "liquidity_anomaly",
            "high",
            format!("{} liquidity deteriorated", snap.token.symbol),
            format!("LP 7d {:+.1}%", snap.liquidity.lp_change_7d_pct),
            vec![format!("liquidity_usd={:.0}", snap.liquidity.liquidity_usd)],
            fingerprint(&snap.token.id, "liquidity_anomaly", "lp7d"),
        ));
    }

    if matches!(snap.risk.level, crate::models::RiskLevel::High | crate::models::RiskLevel::Critical)
        && snap.risk.confidence > 0.0
    {
        out.push(make(
            snap,
            "risk_signal",
            "high",
            format!("{} risk {:?}", snap.token.symbol, snap.risk.level),
            snap.risk.why.clone(),
            vec![format!("score={:.0}", snap.risk.score)],
            fingerprint(&snap.token.id, "risk_signal", &format!("{:?}", snap.risk.level)),
        ));
    }

    if snap.development.data_state.present() && snap.development.last_commit_days.unwrap_or(0) > 45 {
        out.push(make(
            snap,
            "development_silence",
            "medium",
            format!("{} development quiet", snap.token.symbol),
            format!("last commit {:?} days ago", snap.development.last_commit_days),
            vec!["github snapshot".into()],
            fingerprint(&snap.token.id, "development_silence", &hour),
        ));
    }

    out.sort_by(|a, b| b.fired_at.cmp(&a.fired_at));
    out
}

fn make(
    snap: &TokenSnapshot,
    kind: &str,
    severity: &str,
    title: String,
    body: String,
    evidence: Vec<String>,
    fingerprint: String,
) -> Alert {
    Alert {
        id: Uuid::new_v4().to_string(),
        token_id: snap.token.id.clone(),
        kind: kind.into(),
        severity: severity.into(),
        title,
        body,
        evidence,
        fired_at: Utc::now(),
        fingerprint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_stable() {
        assert_eq!(
            fingerprint("pepe", "price_conflict", "2026082617"),
            fingerprint("pepe", "price_conflict", "2026082617")
        );
        assert_ne!(
            fingerprint("pepe", "price_conflict", "a"),
            fingerprint("pepe", "price_conflict", "b")
        );
    }

    fn missing_heavy_snap() -> TokenSnapshot {
        use crate::models::{
            DailyBriefing, EcosystemScore, Genome, RiskLevel, RiskReport, TokenStatus, TokenSummary,
            VerificationLevel,
        };
        use crate::quality::{
            missing_development, missing_liquidity, missing_market, missing_onchain, missing_social,
        };
        let mut liq = missing_liquidity();
        liq.lp_change_7d_pct = -12.0;
        liq.liquidity_usd = 0.0;
        TokenSnapshot {
            token: TokenSummary {
                id: "pepe".into(),
                symbol: "PEPE".into(),
                name: "Pepe".into(),
                status: TokenStatus::Listed,
                verification: VerificationLevel::DataVerified,
                narratives: vec![],
                primary_chain: "ethereum".into(),
                website: None,
                description: None,
                contract: None,
                chains: vec!["ethereum".into()],
            },
            market: missing_market(),
            liquidity: liq,
            onchain: missing_onchain(),
            social: missing_social(),
            development: missing_development(),
            scores: EcosystemScore {
                algorithm: "ecosystem-health".into(),
                algorithm_version: "1.0.0".into(),
                value: 0.0,
                confidence: 0.0,
                as_of: Utc::now(),
                components: vec![],
                why: "missing".into(),
            },
            genome: Genome {
                algorithm_version: "1.0.0".into(),
                dimensions: vec![],
            },
            risk: RiskReport {
                algorithm: "token-risk".into(),
                algorithm_version: "1.0.0".into(),
                score: 0.0,
                level: RiskLevel::Unknown,
                confidence: 0.0,
                as_of: Utc::now(),
                factors: vec![],
                why: "missing".into(),
            },
            growth: vec![],
            timeline: vec![],
            briefing: DailyBriefing {
                token_id: "pepe".into(),
                as_of: Utc::now(),
                ecosystem_health: 0.0,
                headline: String::new(),
                changes: vec![],
                risks: vec![],
                opportunities: vec![],
                investigation: String::new(),
                confidence: 0.0,
                evidence: vec![],
            },
            observations: vec![],
            price_series: vec![],
            health_series: vec![],
            as_of: Utc::now(),
            data_state: DataState::Missing,
        }
    }

    #[test]
    fn missing_zeros_do_not_fire_liquidity_or_freshness() {
        let snap = missing_heavy_snap();
        let alerts = evaluate(&snap, None);
        assert!(
            !alerts.iter().any(|a| a.kind == "liquidity_anomaly"),
            "LP 7d on MISSING zeros must not alert: {alerts:?}"
        );
        assert!(
            !alerts.iter().any(|a| a.kind == "data_freshness"),
            "expected MISSING holders/social must not look like stale: {alerts:?}"
        );
        let prev = missing_heavy_snap();
        let delta = evaluate(&snap, Some(&prev));
        assert!(!delta.iter().any(|a| a.kind == "score_change"));
        assert!(!delta.iter().any(|a| a.kind == "holder_concentration_change"));
        assert!(!delta.iter().any(|a| a.kind == "community_growth_anomaly"));
        assert!(!delta.iter().any(|a| a.kind == "social_anomaly"));
    }

    #[test]
    fn holder_concentration_fires_only_when_onchain_present() {
        let prev = missing_heavy_snap();
        let mut now = missing_heavy_snap();
        now.onchain.data_state = DataState::Live;
        now.onchain.top10_concentration_pct = 42.0;
        now.onchain.holders = 1000;
        let mut prev = prev;
        prev.onchain.data_state = DataState::Live;
        prev.onchain.top10_concentration_pct = 30.0;
        prev.onchain.holders = 900;
        let alerts = evaluate(&now, Some(&prev));
        assert!(alerts.iter().any(|a| a.kind == "holder_concentration_change"));
        let missing_prev = missing_heavy_snap();
        let no_fire = evaluate(&now, Some(&missing_prev));
        assert!(!no_fire.iter().any(|a| a.kind == "holder_concentration_change"));
    }

    #[test]
    fn stale_market_does_fire_freshness() {
        let mut snap = missing_heavy_snap();
        snap.market.data_state = DataState::Stale;
        snap.data_state = DataState::Stale;
        let alerts = evaluate(&snap, None);
        assert!(alerts.iter().any(|a| a.kind == "data_freshness"));
    }

    #[test]
    fn lifecycle_phase_change_fires() {
        let prev = missing_heavy_snap();
        let mut now = missing_heavy_snap();
        now.market.data_state = DataState::Live;
        let alerts = evaluate(&now, Some(&prev));
        assert!(
            alerts.iter().any(|a| a.kind == "lifecycle_transition"),
            "dormant → discovered should alert: {alerts:?}"
        );
    }
}
