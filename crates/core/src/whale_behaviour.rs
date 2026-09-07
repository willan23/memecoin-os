use crate::whale::{WhaleEvent, WhaleKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BehaviourPattern {
    Unknown,
    Dormant,
    Accumulation,
    Distribution,
    Mixed,
    Rotation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhaleBehaviour {
    pub pattern: BehaviourPattern,
    pub in_events: u32,
    pub out_events: u32,
    pub cex_events: u32,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub note: String,
}

/// Aggregate indexed whale events. Empty = UNKNOWN, not "no whales".
pub fn summarize(events: &[WhaleEvent]) -> WhaleBehaviour {
    if events.is_empty() {
        return WhaleBehaviour {
            pattern: BehaviourPattern::Unknown,
            in_events: 0,
            out_events: 0,
            cex_events: 0,
            confidence: 0.0,
            evidence: vec!["No indexed whale/CEX events. Holder count is not behaviour.".into()],
            note: "MISSING — not dormant by default. Not identity. Not a trade signal.".into(),
        };
    }
    let inn = events.iter().filter(|e| e.direction == "in").count() as u32;
    let out = events.iter().filter(|e| e.direction == "out").count() as u32;
    let cex = events.iter().filter(|e| e.exchange.is_some()).count() as u32;

    let pattern = if inn + out == 0 {
        BehaviourPattern::Dormant
    } else if inn > 0 && out > 0 && (inn as i32 - out as i32).abs() <= 1 {
        if cex > 0 {
            BehaviourPattern::Rotation
        } else {
            BehaviourPattern::Mixed
        }
    } else if inn > out {
        BehaviourPattern::Accumulation
    } else {
        BehaviourPattern::Distribution
    };

    WhaleBehaviour {
        pattern,
        in_events: inn,
        out_events: out,
        cex_events: cex,
        confidence: 0.55_f64.min(0.3 + 0.05 * events.len() as f64),
        evidence: vec![
            format!("n={}", events.len()),
            "≥1% share is a whale candidate, not a person".into(),
        ],
        note: "Observed sequence of indexed transfers. Not a buy/sell recommendation.".into(),
    }
}

pub fn from_json_rows(rows: &[serde_json::Value]) -> WhaleBehaviour {
    let events: Vec<WhaleEvent> = rows
        .iter()
        .filter_map(|r| {
            let direction = r.get("direction")?.as_str()?.to_string();
            Some(WhaleEvent {
                kind: WhaleKind::WhaleTransfer,
                direction,
                tx_hash: r.get("tx_hash").and_then(|v| v.as_str()).unwrap_or("").into(),
                chain_id: r.get("chain_id").and_then(|v| v.as_str()).unwrap_or("").into(),
                from_address: r.get("from_address").and_then(|v| v.as_str()).unwrap_or("").into(),
                to_address: r.get("to_address").and_then(|v| v.as_str()).unwrap_or("").into(),
                wallet: r.get("wallet").and_then(|v| v.as_str()).unwrap_or("").into(),
                exchange: r.get("exchange").and_then(|v| v.as_str()).map(|s| s.to_string()),
                amount_raw: r.get("amount_raw").and_then(|v| v.as_str()).unwrap_or("").into(),
                amount_usd: r.get("amount_usd").and_then(|v| v.as_f64()),
                confidence: r.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5),
                evidence: vec!["from persisted whale_movements".into()],
            })
        })
        .collect();
    summarize(&events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_unknown_not_zero_whales() {
        let b = summarize(&[]);
        assert_eq!(b.pattern, BehaviourPattern::Unknown);
        assert!(b.note.contains("MISSING"));
    }

    fn ev(direction: &str, exchange: Option<&str>) -> WhaleEvent {
        WhaleEvent {
            kind: WhaleKind::WhaleTransfer,
            direction: direction.into(),
            tx_hash: "0x1".into(),
            chain_id: "1".into(),
            from_address: "0xa".into(),
            to_address: "0xb".into(),
            wallet: "0xa".into(),
            exchange: exchange.map(|s| s.into()),
            amount_raw: "1".into(),
            amount_usd: Some(1.0),
            confidence: 0.5,
            evidence: vec![],
        }
    }

    #[test]
    fn two_in_is_accumulation() {
        let b = summarize(&[ev("in", None), ev("in", None)]);
        assert_eq!(b.pattern, BehaviourPattern::Accumulation);
    }
}
