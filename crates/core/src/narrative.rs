use crate::models::DataState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NarrativeState {
    Emerging,
    Stable,
    Accelerating,
    Decelerating,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Narrative {
    pub id: String,
    pub label: String,
    pub token_ids: Vec<String>,
    pub state: NarrativeState,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub mention_velocity: DataState,
    pub unique_accounts: DataState,
}

#[derive(Debug, Clone)]
pub struct NarrativeToken {
    pub token_id: String,
    pub tags: Vec<String>,
    pub name: String,
    pub symbol: String,
    pub liquidity_change_pct: Option<f64>,
    pub volume_present: bool,
}

/// Weak name tags — low confidence, never a social observation.
pub fn tags_from_identity(name: &str, symbol: &str, declared: &[String]) -> Vec<(String, String)> {
    let blob = format!("{} {}", name, symbol).to_lowercase();
    let mut out: Vec<(String, String)> = declared
        .iter()
        .filter(|t| !t.is_empty())
        .map(|t| (slug(t), "declared in token registry".into()))
        .collect();
    let heuristics = [
        ("dog", &["doge", "dog", "inu", "shib"][..]),
        ("cat", &["cat", "neko", "kitten"][..]),
        ("frog", &["pepe", "frog", "wojak"][..]),
        ("ai", &["ai", "gpt", "agent"][..]),
        ("politics", &["trump", "boden", "maga", "vote"][..]),
        ("gaming", &["game", "play", "pixel"][..]),
        ("rwa", &["rwa", "real-world"][..]),
    ];
    for (tag, keys) in heuristics {
        if keys.iter().any(|k| blob.contains(k)) {
            out.push((tag.into(), "weak name/symbol heuristic — not social volume".into()));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

pub fn aggregate(tokens: &[NarrativeToken]) -> Vec<Narrative> {
    let mut bags: HashMap<String, Vec<&NarrativeToken>> = HashMap::new();
    let mut why: HashMap<String, Vec<String>> = HashMap::new();
    for t in tokens {
        let tags = tags_from_identity(&t.name, &t.symbol, &t.tags);
        for (id, reason) in tags {
            bags.entry(id.clone()).or_default().push(t);
            why.entry(id).or_default().push(format!("{}: {reason}", t.token_id));
        }
    }

    let mut out = Vec::new();
    for (id, members) in bags {
        let token_ids: Vec<String> = members.iter().map(|m| m.token_id.clone()).collect();
        let changes: Vec<f64> = members.iter().filter_map(|m| m.liquidity_change_pct).collect();
        let any_vol = members.iter().any(|m| m.volume_present);
        let state = if token_ids.len() == 1 {
            NarrativeState::Emerging
        } else if changes.len() >= 2 && changes.iter().all(|c| *c < -5.0) {
            NarrativeState::Decelerating
        } else if changes.len() >= 2 && changes.iter().all(|c| *c > 5.0) && any_vol {
            NarrativeState::Accelerating
        } else {
            NarrativeState::Stable
        };
        let n = token_ids.len();
        let mut evidence = why.get(&id).cloned().unwrap_or_default();
        evidence.push("mention_velocity and unique_accounts stay MISSING without a licensed social provider".into());
        if !changes.is_empty() {
            evidence.push(format!("liquidity change samples={}", changes.len()));
        }
        out.push(Narrative {
            label: id.replace('-', " "),
            id,
            token_ids,
            state,
            confidence: (0.35 + n as f64 * 0.08).min(0.72),
            evidence,
            mention_velocity: DataState::Missing,
            unique_accounts: DataState::Missing,
        });
    }
    out.sort_by(|a, b| b.token_ids.len().cmp(&a.token_ids.len()).then(a.id.cmp(&b.id)));
    out
}

fn slug(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn social_stays_missing_and_pepe_is_frog() {
        let tokens = vec![NarrativeToken {
            token_id: "pepe".into(),
            tags: vec!["meme".into()],
            name: "Pepe".into(),
            symbol: "PEPE".into(),
            liquidity_change_pct: Some(1.0),
            volume_present: true,
        }];
        let n = aggregate(&tokens);
        assert!(n.iter().any(|x| x.id == "frog" && x.state == NarrativeState::Emerging));
        assert!(n.iter().all(|x| x.mention_velocity == DataState::Missing));
    }

    #[test]
    fn accelerating_needs_live_liquidity_growth() {
        let tokens = vec![
            NarrativeToken {
                token_id: "a".into(),
                tags: vec!["ai".into()],
                name: "A".into(),
                symbol: "A".into(),
                liquidity_change_pct: Some(12.0),
                volume_present: true,
            },
            NarrativeToken {
                token_id: "b".into(),
                tags: vec!["ai".into()],
                name: "B".into(),
                symbol: "B".into(),
                liquidity_change_pct: Some(8.0),
                volume_present: true,
            },
        ];
        let n = aggregate(&tokens);
        let ai = n.iter().find(|x| x.id == "ai").unwrap();
        assert_eq!(ai.state, NarrativeState::Accelerating);
    }
}
