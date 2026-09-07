use crate::models::TokenSnapshot;
use crate::twin::TwinState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Assumptions {
    #[serde(default)]
    pub liquidity_pct: f64,
    #[serde(default)]
    pub holder_growth_pct: f64,
    #[serde(default)]
    pub whale_selling_pct: f64,
    #[serde(default)]
    pub developer_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sensitivity {
    pub variable: String,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationResult {
    pub ecosystem_id: String,
    pub assumptions: Assumptions,
    pub current_health: f64,
    pub projected_health: f64,
    pub current_risk: f64,
    pub projected_risk: f64,
    pub delta_health: f64,
    pub delta_risk: f64,
    pub confidence: f64,
    pub sensitivity: Vec<Sensitivity>,
    pub banner: String,
    pub notes: Vec<String>,
    pub projected_twin_note: String,
}

/// Clone Twin metrics, apply named % shocks. Not a price forecast.
pub fn run(s: &TokenSnapshot, twin: &TwinState, a: Assumptions) -> SimulationResult {
    let health = s.scores.value;
    let risk = s.risk.score;
    let liq_w = if s.liquidity.data_state.present() { 0.12 } else { 0.0 };
    let hold_w = if s.onchain.data_state.present() { 0.08 } else { 0.0 };
    let whale_w = if twin.whale_state.data_state.present() { 0.10 } else { 0.0 };
    let dev_w = if s.development.data_state.present() { 0.05 } else { 0.0 };

    let d_h = liq_w * a.liquidity_pct + hold_w * a.holder_growth_pct - whale_w * a.whale_selling_pct
        + dev_w * a.developer_pct;
    let projected_health = (health + d_h).clamp(0.0, 100.0);
    let projected_risk = (risk - 0.05 * a.liquidity_pct + 0.08 * a.whale_selling_pct).clamp(0.0, 100.0);

    let mut n_apply = 0u32;
    if a.liquidity_pct.abs() > 0.0 {
        n_apply += 1;
    }
    if a.holder_growth_pct.abs() > 0.0 {
        n_apply += 1;
    }
    if a.whale_selling_pct.abs() > 0.0 {
        n_apply += 1;
    }
    if a.developer_pct.abs() > 0.0 {
        n_apply += 1;
    }
    let confidence = (s.scores.confidence - 0.12 * n_apply as f64).clamp(0.1, 0.7);

    let mut notes = vec![
        "SIMULATION — NOT A PREDICTION".into(),
        "Price is not a simulated variable.".into(),
        "ceteris paribus: other Twin fields stay as observed.".into(),
    ];
    if !s.liquidity.data_state.present() && a.liquidity_pct.abs() > 0.0 {
        notes.push("Liquidity shock ignored — field is MISSING.".into());
    }
    if !twin.whale_state.data_state.present() && a.whale_selling_pct.abs() > 0.0 {
        notes.push("Whale shock ignored — no evidenced whale behaviour.".into());
    }

    SimulationResult {
        ecosystem_id: twin.ecosystem_id.clone(),
        assumptions: a.clone(),
        current_health: health,
        projected_health,
        current_risk: risk,
        projected_risk,
        delta_health: projected_health - health,
        delta_risk: projected_risk - risk,
        confidence,
        sensitivity: vec![
            Sensitivity {
                variable: "liquidity".into(),
                impact: if liq_w > 0.0 { "HIGH".into() } else { "NONE (missing)".into() },
            },
            Sensitivity {
                variable: "holder_growth".into(),
                impact: if hold_w > 0.0 { "MEDIUM".into() } else { "NONE (missing)".into() },
            },
            Sensitivity {
                variable: "narrative".into(),
                impact: "NONE (social firehose MISSING)".into(),
            },
            Sensitivity {
                variable: "developer".into(),
                impact: if dev_w > 0.0 { "LOW".into() } else { "NONE (missing)".into() },
            },
        ],
        banner: "SIMULATION / NOT A PREDICTION".into(),
        notes,
        projected_twin_note: format!(
            "Projected health {projected_health:.1} (was {health:.1}). EXECUTE remains off."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::{missing_development, missing_liquidity, missing_market, missing_onchain, missing_social};
    use crate::models::{
        DailyBriefing, DataState, EcosystemScore, Genome, RiskLevel, RiskReport, TokenStatus, TokenSummary,
        VerificationLevel,
    };
    use crate::twin::from_snapshot;
    use chrono::Utc;

    fn snap() -> TokenSnapshot {
        TokenSnapshot {
            token: TokenSummary {
                id: "pepe".into(),
                symbol: "PEPE".into(),
                name: "Pepe".into(),
                status: TokenStatus::Listed,
                verification: VerificationLevel::Unverified,
                narratives: vec![],
                primary_chain: "ethereum".into(),
                website: None,
                description: None,
                contract: None,
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
                value: 40.0,
                confidence: 0.5,
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
                score: 30.0,
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
                ecosystem_health: 40.0,
                headline: String::new(),
                changes: vec![],
                risks: vec![],
                opportunities: vec![],
                investigation: String::new(),
                confidence: 0.5,
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
    fn missing_liquidity_does_not_move_health_from_liq_shock() {
        let s = snap();
        let t = from_snapshot("pepe", &s);
        let r = run(
            &s,
            &t,
            Assumptions {
                liquidity_pct: 25.0,
                ..Default::default()
            },
        );
        assert_eq!(r.projected_health, r.current_health);
        assert!(r.banner.contains("NOT A PREDICTION"));
        assert!(!r.notes.iter().any(|n| n.to_lowercase().contains("price will")));
    }
}
