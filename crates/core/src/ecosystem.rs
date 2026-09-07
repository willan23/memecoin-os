use crate::models::{DataState, TokenSnapshot, TokenSummary, VerificationLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Bootstrap: one candidate ecosystem per registry token. Never auto-VERIFIED.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ecosystem {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub primary_chain: String,
    pub status: String,
    pub lifecycle_phase: LifecyclePhase,
    pub discovery_score: f64,
    pub intelligence_score: f64,
    pub risk_score: f64,
    pub confidence: f64,
    pub verification_status: String,
    pub token_ids: Vec<String>,
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub data_freshness: String,
    pub auto_verified: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecyclePhase {
    Unknown,
    Discovered,
    Emerging,
    Formation,
    Growth,
    Expansion,
    Maturation,
    Decline,
    Dormant,
}

impl LifecyclePhase {
    pub fn as_str(self) -> &'static str {
        match self {
            LifecyclePhase::Unknown => "unknown",
            LifecyclePhase::Discovered => "discovered",
            LifecyclePhase::Emerging => "emerging",
            LifecyclePhase::Formation => "formation",
            LifecyclePhase::Growth => "growth",
            LifecyclePhase::Expansion => "expansion",
            LifecyclePhase::Maturation => "maturation",
            LifecyclePhase::Decline => "decline",
            LifecyclePhase::Dormant => "dormant",
        }
    }
}

pub fn from_snapshot(s: &TokenSnapshot) -> Ecosystem {
    let phase = classify(s);
    Ecosystem {
        id: s.token.id.clone(),
        slug: s.token.id.clone(),
        name: s.token.name.clone(),
        description: s.token.description.clone(),
        primary_chain: s.token.primary_chain.clone(),
        status: "discovered".into(),
        lifecycle_phase: phase,
        discovery_score: discovery_score(s),
        intelligence_score: s.scores.value,
        risk_score: s.risk.score,
        confidence: s.scores.confidence.min(s.risk.confidence),
        verification_status: s.token.verification.as_str().into(),
        token_ids: vec![s.token.id.clone()],
        first_seen: s.as_of,
        last_updated: s.as_of,
        data_freshness: s.data_state.as_str().into(),
        auto_verified: false,
    }
}

pub fn from_summary(t: &TokenSummary, as_of: DateTime<Utc>) -> Ecosystem {
    Ecosystem {
        id: t.id.clone(),
        slug: t.id.clone(),
        name: t.name.clone(),
        description: t.description.clone(),
        primary_chain: t.primary_chain.clone(),
        status: "discovered".into(),
        lifecycle_phase: LifecyclePhase::Discovered,
        discovery_score: 0.0,
        intelligence_score: 0.0,
        risk_score: 0.0,
        confidence: 0.0,
        verification_status: t.verification.as_str().into(),
        token_ids: vec![t.id.clone()],
        first_seen: as_of,
        last_updated: as_of,
        data_freshness: "missing".into(),
        auto_verified: false,
    }
}

/// Evidence-only. Social MISSING does not count as growth. VERIFIED ≠ SAFE.
pub fn classify(s: &TokenSnapshot) -> LifecyclePhase {
    let m = s.market.data_state;
    let l = s.liquidity.data_state;
    let h = s.onchain.data_state;
    if !m.present() && !l.present() {
        return LifecyclePhase::Dormant;
    }
    if l.present() && s.liquidity.lp_change_7d_pct <= -15.0 {
        return LifecyclePhase::Decline;
    }
    if s.scores.value < 35.0
        && matches!(
            s.risk.level,
            crate::models::RiskLevel::High | crate::models::RiskLevel::Critical
        )
        && s.risk.confidence > 0.4
    {
        return LifecyclePhase::Decline;
    }
    let verified = !matches!(s.token.verification, VerificationLevel::Unverified);
    let counterparts = !s.liquidity.counterparts.is_empty();
    let live_liq = l == DataState::Live || l == DataState::Recent;
    let live_hold = h == DataState::Live || h == DataState::Recent;
    if live_liq && counterparts && s.scores.value >= 50.0 {
        return LifecyclePhase::Expansion;
    }
    if live_liq && live_hold && s.scores.value >= 50.0 && s.liquidity.lp_change_7d_pct > 0.0 {
        return LifecyclePhase::Growth;
    }
    if live_liq && (s.liquidity.pool_count >= 1 || s.liquidity.liquidity_usd > 0.0) && verified {
        return LifecyclePhase::Formation;
    }
    if live_liq && live_hold {
        return LifecyclePhase::Emerging;
    }
    LifecyclePhase::Discovered
}

pub fn discovery_score(s: &TokenSnapshot) -> f64 {
    let mut n = 0.0;
    let mut w = 0.0;
    if s.liquidity.data_state.present() {
        n += 40.0;
        w += 40.0;
    }
    if s.onchain.data_state.present() {
        n += 30.0;
        w += 30.0;
    }
    if s.development.data_state.present() {
        n += 20.0;
        w += 20.0;
    }
    if s.market.data_state.present() {
        n += 10.0;
        w += 10.0;
    }
    if w == 0.0 {
        return 0.0;
    }
    (n / w) * 100.0 * s.scores.confidence.max(0.2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::{missing_development, missing_liquidity, missing_market, missing_onchain, missing_social};
    use crate::models::{
        DailyBriefing, DataState, EcosystemScore, Genome, RiskLevel, RiskReport, TokenStatus, TokenSummary,
        VerificationLevel,
    };

    fn base() -> TokenSnapshot {
        TokenSnapshot {
            token: TokenSummary {
                id: "pepe".into(),
                symbol: "PEPE".into(),
                name: "Pepe".into(),
                status: TokenStatus::Listed,
                verification: VerificationLevel::Unverified,
                narratives: vec!["meme".into()],
                primary_chain: "ethereum".into(),
                website: None,
                description: None,
                contract: Some("0x6982508145454ce325ddbe47a25d4ec3d2311933".into()),
                chains: vec!["ethereum".into()],
            },
            market: missing_market(),
            liquidity: missing_liquidity(),
            onchain: missing_onchain(),
            social: missing_social(),
            development: missing_development(),
            scores: EcosystemScore {
                algorithm: "t".into(),
                algorithm_version: "1".into(),
                value: 20.0,
                confidence: 0.3,
                as_of: Utc::now(),
                components: vec![],
                why: "t".into(),
            },
            genome: Genome {
                algorithm_version: "1".into(),
                dimensions: vec![],
            },
            risk: RiskReport {
                algorithm: "t".into(),
                algorithm_version: "1".into(),
                score: 10.0,
                level: RiskLevel::Unknown,
                confidence: 0.2,
                as_of: Utc::now(),
                factors: vec![],
                why: "t".into(),
            },
            growth: vec![],
            timeline: vec![],
            briefing: DailyBriefing {
                token_id: "pepe".into(),
                as_of: Utc::now(),
                ecosystem_health: 20.0,
                headline: String::new(),
                changes: vec![],
                risks: vec![],
                opportunities: vec![],
                investigation: String::new(),
                confidence: 0.3,
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
    fn missing_market_and_liq_is_dormant() {
        assert_eq!(classify(&base()), LifecyclePhase::Dormant);
    }

    #[test]
    fn never_auto_verified() {
        let e = from_snapshot(&base());
        assert!(!e.auto_verified);
        assert_eq!(e.status, "discovered");
        assert_eq!(e.id, "pepe");
    }
}
