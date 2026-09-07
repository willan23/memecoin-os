use crate::models::{DataState, TokenSnapshot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldState {
    pub data_state: DataState,
    pub confidence: f64,
    pub summary: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwinState {
    pub timestamp: DateTime<Utc>,
    pub ecosystem_id: String,
    pub token_id: String,
    pub market_state: FieldState,
    pub liquidity_state: FieldState,
    pub holder_state: FieldState,
    pub wallet_state: FieldState,
    pub whale_state: FieldState,
    pub developer_state: FieldState,
    pub social_state: FieldState,
    pub narrative_state: FieldState,
    pub risk_state: FieldState,
    pub governance_state: FieldState,
    pub network_state: FieldState,
    pub confidence: f64,
    pub freshness: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwinHistoryPoint {
    pub t: DateTime<Utc>,
    pub liquidity_usd: Option<f64>,
    pub volume_24h_usd: Option<f64>,
    pub health: Option<f64>,
    pub risk: Option<f64>,
    pub data_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateDelta {
    pub field: String,
    pub before: String,
    pub after: String,
    pub why: String,
    pub confidence: f64,
}

fn field(state: DataState, confidence: f64, summary: String, evidence: Vec<String>) -> FieldState {
    FieldState {
        data_state: state,
        confidence: if state == DataState::Missing { 0.0 } else { confidence },
        summary: if state == DataState::Missing {
            "MISSING — not zero".into()
        } else {
            summary
        },
        evidence,
    }
}

pub fn from_snapshot(ecosystem_id: &str, s: &TokenSnapshot) -> TwinState {
    let social_missing = s.social.data_state == DataState::Missing;
    TwinState {
        timestamp: s.as_of,
        ecosystem_id: ecosystem_id.into(),
        token_id: s.token.id.clone(),
        market_state: field(
            s.market.data_state,
            s.market.provenance.confidence,
            format!(
                "spot={:.6} vol_24h={:.0} cap={:.0}",
                s.market.price_usd, s.market.volume_24h_usd, s.market.market_cap_usd
            ),
            vec![format!("provider={}", s.market.provenance.provider)],
        ),
        liquidity_state: field(
            s.liquidity.data_state,
            s.liquidity.provenance.confidence,
            format!(
                "usd={:.0} pools={} lp_7d={:.1}%",
                s.liquidity.liquidity_usd, s.liquidity.pool_count, s.liquidity.lp_change_7d_pct
            ),
            vec![format!("provider={}", s.liquidity.provenance.provider)],
        ),
        holder_state: field(
            s.onchain.data_state,
            s.onchain.provenance.confidence,
            format!(
                "holders={} top10={:.1}% (count is not distribution if UNKNOWN)",
                s.onchain.holders, s.onchain.top10_concentration_pct
            ),
            vec![format!("provider={}", s.onchain.provenance.provider)],
        ),
        wallet_state: field(
            if s.onchain.top_holders.is_empty() {
                DataState::Missing
            } else {
                s.onchain.data_state
            },
            if s.onchain.top_holders.is_empty() {
                0.0
            } else {
                s.onchain.provenance.confidence.min(0.7)
            },
            if s.onchain.top_holders.is_empty() {
                "MISSING — no top-holder sample".into()
            } else {
                format!("{} top-holder rows (class, not identity)", s.onchain.top_holders.len())
            },
            vec!["share-based classes only".into()],
        ),
        whale_state: field(
            if s.onchain.transfers_24h == 0 && s.onchain.data_state == DataState::Missing {
                DataState::Missing
            } else if s.onchain.large_transfers_24h > 0 {
                s.onchain.data_state
            } else {
                DataState::Missing
            },
            0.5,
            if s.onchain.large_transfers_24h == 0 {
                "MISSING — no indexed large transfers in this snapshot".into()
            } else {
                format!("large_transfers_24h={}", s.onchain.large_transfers_24h)
            },
            vec!["whale ≠ identity".into()],
        ),
        developer_state: field(
            s.development.data_state,
            s.development.provenance.confidence,
            format!(
                "commits_30d={} contributors={}",
                s.development.commits_30d, s.development.active_contributors_30d
            ),
            vec![format!("provider={}", s.development.provenance.provider)],
        ),
        social_state: field(
            s.social.data_state,
            if social_missing { 0.0 } else { s.social.provenance.confidence },
            if social_missing {
                "INSUFFICIENT_EVIDENCE — no firehose licence".into()
            } else {
                format!("mentions_24h={}", s.social.mentions_24h)
            },
            vec!["social firehose not licensed".into()],
        ),
        narrative_state: field(
            if s.token.narratives.is_empty() {
                DataState::Missing
            } else {
                DataState::Live
            },
            0.4,
            if s.token.narratives.is_empty() {
                "MISSING".into()
            } else {
                format!("registry tags: {}", s.token.narratives.join(", "))
            },
            vec!["tags + heuristics; velocity MISSING without firehose".into()],
        ),
        risk_state: field(
            if s.risk.confidence < 0.15 {
                DataState::Missing
            } else {
                DataState::Live
            },
            s.risk.confidence,
            format!("score={:.1} level={:?}", s.risk.score, s.risk.level),
            vec![s.risk.why.clone()],
        ),
        governance_state: field(
            DataState::Missing,
            0.0,
            "MISSING — no governance indexer".into(),
            vec![],
        ),
        network_state: field(
            DataState::Live,
            0.9,
            format!("chain={}", s.token.primary_chain),
            vec!["from token definition".into()],
        ),
        confidence: s.scores.confidence,
        freshness: s.data_state.as_str().into(),
        note: "Twin is composed from canonical snapshots. Absence is MISSING, not zero.".into(),
    }
}

/// Nearest observation with `t <= at`. Never uses future points.
pub fn reconstruct_at(current: &TwinState, history: &[TwinHistoryPoint], at: DateTime<Utc>) -> TwinState {
    let mut chosen: Option<&TwinHistoryPoint> = None;
    for p in history {
        if p.t <= at && p.data_state != "simulated" {
            chosen = Some(p);
        }
    }
    let Some(p) = chosen else {
        let mut missing = current.clone();
        missing.timestamp = at;
        missing.freshness = "missing".into();
        missing.note = "No historical observation at or before that timestamp. No look-ahead.".into();
        missing.liquidity_state.data_state = DataState::Missing;
        missing.liquidity_state.summary = "MISSING — not reconstructed from the future".into();
        missing.confidence = 0.0;
        return missing;
    };
    let mut out = current.clone();
    out.timestamp = p.t;
    out.freshness = "historical".into();
    if let Some(liq) = p.liquidity_usd {
        out.liquidity_state.summary = format!("usd={liq:.0} (reconstructed ≤ requested time)");
        out.liquidity_state.data_state = DataState::Stale;
        out.liquidity_state.evidence.push(format!("as_of={}", p.t.to_rfc3339()));
    }
    if let Some(h) = p.health {
        out.risk_state.evidence.push(format!("health_then={h:.1}"));
    }
    if let Some(r) = p.risk {
        out.risk_state.summary = format!("score={r:.1} (historical)");
        out.risk_state.data_state = DataState::Stale;
    }
    if let Some(v) = p.volume_24h_usd {
        out.market_state.summary = format!("vol_24h={v:.0} (historical; price not implied)");
        out.market_state.data_state = DataState::Stale;
    }
    out.note = "Reconstructed from observations ≤ timestamp. Not a backtest of price.".into();
    out.confidence = out.confidence.min(0.55);
    out
}

pub fn what_changed(before: &TwinState, after: &TwinState) -> Vec<StateDelta> {
    let pairs = [
        ("liquidity", &before.liquidity_state, &after.liquidity_state),
        ("holders", &before.holder_state, &after.holder_state),
        ("risk", &before.risk_state, &after.risk_state),
        ("development", &before.developer_state, &after.developer_state),
        ("social", &before.social_state, &after.social_state),
        ("market", &before.market_state, &after.market_state),
    ];
    pairs
        .into_iter()
        .filter(|(_, a, b)| a.summary != b.summary || a.data_state != b.data_state)
        .map(|(field, a, b)| StateDelta {
            field: field.into(),
            before: a.summary.clone(),
            after: b.summary.clone(),
            why: format!("{} → {}", a.data_state.as_str(), b.data_state.as_str()),
            confidence: a.confidence.min(b.confidence),
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    pub claim: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub counter_evidence: Vec<String>,
    pub freshness: String,
}

pub fn claims(t: &TwinState) -> Vec<Claim> {
    let mut out = Vec::new();
    if t.liquidity_state.data_state.present() {
        out.push(Claim {
            claim: "Liquidity observation exists on the Twin.".into(),
            confidence: t.liquidity_state.confidence,
            evidence: t.liquidity_state.evidence.clone(),
            counter_evidence: vec![],
            freshness: t.liquidity_state.data_state.as_str().into(),
        });
    }
    if t.social_state.data_state == DataState::Missing {
        out.push(Claim {
            claim: "Social activity is not measured.".into(),
            confidence: 1.0,
            evidence: t.social_state.evidence.clone(),
            counter_evidence: vec![],
            freshness: "missing".into(),
        });
    }
    if t.whale_state.data_state == DataState::Missing {
        out.push(Claim {
            claim: "Whale behaviour is not evidenced on this slice.".into(),
            confidence: 0.9,
            evidence: t.whale_state.evidence.clone(),
            counter_evidence: vec![],
            freshness: "missing".into(),
        });
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NextStep {
    pub title: String,
    pub reason: String,
    pub priority: u8,
    pub evidence: Vec<String>,
    pub confidence: f64,
}

pub fn investigate_next(t: &TwinState) -> Vec<NextStep> {
    let mut steps = Vec::new();
    if t.whale_state.data_state == DataState::Missing {
        steps.push(NextStep {
            title: "Check indexer / RPC for transfers".into(),
            reason: "Whale field is MISSING, not zero activity.".into(),
            priority: 1,
            evidence: t.whale_state.evidence.clone(),
            confidence: 0.7,
        });
    }
    if t.liquidity_state.data_state.present() {
        steps.push(NextStep {
            title: "Inspect pools on the graph".into(),
            reason: "Liquidity is observed; pool edges are the next evidence.".into(),
            priority: 2,
            evidence: t.liquidity_state.evidence.clone(),
            confidence: t.liquidity_state.confidence,
        });
    }
    if t.developer_state.data_state == DataState::Missing {
        steps.push(NextStep {
            title: "Confirm GitHub on the token definition".into(),
            reason: "Development is MISSING until socials.github is set.".into(),
            priority: 3,
            evidence: vec![],
            confidence: 0.8,
        });
    }
    steps.push(NextStep {
        title: "Compare with historical Twin".into(),
        reason: "Reconstruction uses only observations ≤ the cursor.".into(),
        priority: 4,
        evidence: vec![t.note.clone()],
        confidence: 0.6,
    });
    if t.social_state.data_state == DataState::Missing {
        steps.push(NextStep {
            title: "Do not treat mentions as 0".into(),
            reason: "No licensed firehose.".into(),
            priority: 5,
            evidence: t.social_state.evidence.clone(),
            confidence: 1.0,
        });
    }
    steps.sort_by_key(|s| s.priority);
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn reconstruct_ignores_future_and_simulated() {
        let now = Utc::now();
        let current = TwinState {
            timestamp: now,
            ecosystem_id: "pepe".into(),
            token_id: "pepe".into(),
            market_state: field(DataState::Live, 0.8, "now".into(), vec![]),
            liquidity_state: field(DataState::Live, 0.8, "liq now".into(), vec![]),
            holder_state: field(DataState::Missing, 0.0, "x".into(), vec![]),
            wallet_state: field(DataState::Missing, 0.0, "x".into(), vec![]),
            whale_state: field(DataState::Missing, 0.0, "x".into(), vec![]),
            developer_state: field(DataState::Missing, 0.0, "x".into(), vec![]),
            social_state: field(DataState::Missing, 0.0, "x".into(), vec![]),
            narrative_state: field(DataState::Missing, 0.0, "x".into(), vec![]),
            risk_state: field(DataState::Live, 0.5, "risk".into(), vec![]),
            governance_state: field(DataState::Missing, 0.0, "x".into(), vec![]),
            network_state: field(DataState::Live, 0.9, "eth".into(), vec![]),
            confidence: 0.7,
            freshness: "live".into(),
            note: String::new(),
        };
        let hist = vec![
            TwinHistoryPoint {
                t: now + Duration::hours(2),
                liquidity_usd: Some(9_999.0),
                volume_24h_usd: None,
                health: None,
                risk: None,
                data_state: "live".into(),
            },
            TwinHistoryPoint {
                t: now - Duration::hours(6),
                liquidity_usd: Some(100.0),
                volume_24h_usd: Some(10.0),
                health: Some(40.0),
                risk: Some(20.0),
                data_state: "live".into(),
            },
            TwinHistoryPoint {
                t: now - Duration::hours(3),
                liquidity_usd: Some(1.0),
                volume_24h_usd: None,
                health: None,
                risk: None,
                data_state: "simulated".into(),
            },
        ];
        let past = reconstruct_at(&current, &hist, now - Duration::hours(1));
        assert!(past.liquidity_state.summary.contains("100"));
        assert!(!past.liquidity_state.summary.contains("9999"));
        let none = reconstruct_at(&current, &hist, now - Duration::days(30));
        assert_eq!(none.liquidity_state.data_state, DataState::Missing);
    }
}
