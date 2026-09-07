use crate::models::{TimelineEvent, TokenSnapshot};
use crate::twin::TwinHistoryPoint;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayFrame {
    pub t: DateTime<Utc>,
    pub kind: String,
    pub title: String,
    pub source: String,
    pub evidence: Vec<String>,
}

/// Chronological frames from real timeline + history points. No invented motion.
pub fn frames(snap: &TokenSnapshot, hist: &[TwinHistoryPoint], range_hours: i64) -> Vec<ReplayFrame> {
    let cutoff = Utc::now() - chrono::Duration::hours(range_hours.max(1));
    let mut out = Vec::new();
    for e in &snap.timeline {
        if e.at >= cutoff {
            out.push(ReplayFrame {
                t: e.at,
                kind: e.kind.clone(),
                title: e.title.clone(),
                source: e.source.clone(),
                evidence: vec![format!("confidence={}", e.confidence)],
            });
        }
    }
    for p in hist {
        if p.t >= cutoff && p.data_state != "simulated" {
            out.push(ReplayFrame {
                t: p.t,
                kind: "observation".into(),
                title: format!(
                    "liq={} health={}",
                    p.liquidity_usd.map(|v| format!("{v:.0}")).unwrap_or_else(|| "MISSING".into()),
                    p.health.map(|v| format!("{v:.1}")).unwrap_or_else(|| "MISSING".into())
                ),
                source: "market_snapshots".into(),
                evidence: vec![format!("data_state={}", p.data_state)],
            });
        }
    }
    out.sort_by(|a, b| a.t.cmp(&b.t));
    out
}

pub fn from_timeline_only(events: &[TimelineEvent]) -> Vec<ReplayFrame> {
    let mut out: Vec<_> = events
        .iter()
        .map(|e| ReplayFrame {
            t: e.at,
            kind: e.kind.clone(),
            title: e.title.clone(),
            source: e.source.clone(),
            evidence: vec![],
        })
        .collect();
    out.sort_by(|a, b| a.t.cmp(&b.t));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn simulated_history_is_not_replayed() {
        let now = Utc::now();
        let hist = vec![TwinHistoryPoint {
            t: now - Duration::hours(1),
            liquidity_usd: Some(1.0),
            volume_24h_usd: None,
            health: None,
            risk: None,
            data_state: "simulated".into(),
        }];
        let snap = empty_snap();
        assert!(frames(&snap, &hist, 24).is_empty());
    }

    fn empty_snap() -> TokenSnapshot {
        use crate::quality::{missing_development, missing_liquidity, missing_market, missing_onchain, missing_social};
        use crate::models::{
            DailyBriefing, DataState, EcosystemScore, Genome, RiskLevel, RiskReport, TokenStatus, TokenSummary,
            VerificationLevel,
        };
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
                value: 1.0,
                confidence: 0.1,
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
                score: 1.0,
                level: RiskLevel::Unknown,
                confidence: 0.1,
                as_of: Utc::now(),
                factors: vec![],
                why: "t".into(),
            },
            growth: vec![],
            timeline: vec![],
            briefing: DailyBriefing {
                token_id: "pepe".into(),
                as_of: Utc::now(),
                ecosystem_health: 1.0,
                headline: String::new(),
                changes: vec![],
                risks: vec![],
                opportunities: vec![],
                investigation: String::new(),
                confidence: 0.1,
                evidence: vec![],
            },
            observations: vec![],
            price_series: vec![],
            health_series: vec![],
            as_of: Utc::now(),
            data_state: DataState::Missing,
        }
    }
}
