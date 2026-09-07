use crate::models::TokenSnapshot;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEntity {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub importance: f64,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relationship_type: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<DateTime<Utc>>,
}

/// Reconstruct edges valid at T. Missing bounds are treated as unbounded.
pub fn valid_at(edges: &[GraphEdge], at: DateTime<Utc>) -> Vec<GraphEdge> {
    edges
        .iter()
        .filter(|e| {
            let from_ok = e.valid_from.map(|f| f <= at).unwrap_or(true);
            let to_ok = e.valid_to.map(|t| t > at).unwrap_or(true);
            from_ok && to_ok
        })
        .cloned()
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwinGraph {
    pub ecosystem_id: String,
    pub entities: Vec<GraphEntity>,
    pub relationships: Vec<GraphEdge>,
    pub note: String,
}

/// Relational projection for 2D. Correlation ≠ identity.
pub fn from_snapshot(ecosystem_id: &str, s: &TokenSnapshot) -> TwinGraph {
    let mut entities = Vec::new();
    let mut relationships = Vec::new();
    let eco = format!("eco:{ecosystem_id}");
    entities.push(ent(&eco, "Ecosystem", &s.token.name, 1.0));
    let tok = format!("tok:{}", s.token.id);
    entities.push(ent(&tok, "Token", &s.token.symbol, 0.9));
    relationships.push(edge(&tok, &eco, "PART_OF", 0.95, vec!["registry member".into()]));

    if let Some(addr) = &s.token.contract {
        let c = format!("ctr:{addr}");
        entities.push(ent(&c, "Contract", &trunc(addr), 0.7));
        relationships.push(edge(&c, &tok, "DEPLOYED", 0.9, vec!["token definition".into()]));
    }

    for (i, p) in s.liquidity.pools.iter().take(8).enumerate() {
        let id = format!("pool:{}:{}", p.chain_id, p.pair_address);
        entities.push(ent(&id, "Pool", p.dex.as_deref().unwrap_or("pool"), 0.5 + (i as f64) * 0.02));
        let conf = if p.liquidity_usd.is_some() { 0.7 } else { 0.4 };
        relationships.push(edge(
            &id,
            &tok,
            "PROVIDES_LIQUIDITY",
            conf,
            vec![format!("pair={}", trunc(&p.pair_address))],
        ));
    }

    for n in &s.token.narratives {
        let id = format!("nar:{n}");
        entities.push(ent(&id, "Narrative", n, 0.4));
        relationships.push(edge(
            &id,
            &eco,
            "ASSOCIATED_WITH",
            0.4,
            vec!["registry tag — not firehose velocity".into()],
        ));
    }

    layout(&mut entities);
    TwinGraph {
        ecosystem_id: ecosystem_id.into(),
        entities,
        relationships,
        note: "Projection of the Twin (2D/3D). POTENTIALLY_ASSOCIATED when identity is not proven. No decorative motion.".into(),
    }
}

fn trunc(s: &str) -> String {
    if s.len() <= 12 {
        s.into()
    } else {
        format!("{}…{}", &s[..6], &s[s.len() - 4..])
    }
}

fn ent(id: &str, kind: &str, label: &str, importance: f64) -> GraphEntity {
    GraphEntity {
        id: id.into(),
        kind: kind.into(),
        label: label.into(),
        importance,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

fn edge(source: &str, target: &str, rel: &str, confidence: f64, evidence: Vec<String>) -> GraphEdge {
    GraphEdge {
        source: source.into(),
        target: target.into(),
        relationship_type: rel.into(),
        confidence,
        evidence,
        status: "observed".into(),
        valid_from: None,
        valid_to: None,
    }
}

fn layout(nodes: &mut [GraphEntity]) {
    if nodes.is_empty() {
        return;
    }
    nodes[0].x = 0.0;
    nodes[0].y = 0.0;
    let n = nodes.len().saturating_sub(1).max(1) as f64;
    for (i, node) in nodes.iter_mut().enumerate().skip(1) {
        let a = (i as f64) / n * std::f64::consts::TAU;
        let r = 120.0 + node.importance * 40.0;
        node.x = (a.cos() * r).round();
        node.y = (a.sin() * r).round();
        node.z = ((node.importance - 0.5) * 70.0).round();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::{missing_development, missing_liquidity, missing_market, missing_onchain, missing_social};
    use crate::models::{
        DailyBriefing, DataState, DiscoveredPool, EcosystemScore, Genome, RiskLevel, RiskReport, TokenStatus,
        TokenSummary, VerificationLevel,
    };
    use chrono::Utc;

    #[test]
    fn token_is_part_of_ecosystem_and_name_is_not_an_edge() {
        let mut s = TokenSnapshot {
            token: TokenSummary {
                id: "pepe".into(),
                symbol: "PEPE".into(),
                name: "Pepe".into(),
                status: TokenStatus::Listed,
                verification: VerificationLevel::Unverified,
                narratives: vec!["frog".into()],
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
                value: 1.0,
                confidence: 0.2,
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
                confidence: 0.2,
                evidence: vec![],
            },
            observations: vec![],
            price_series: vec![],
            health_series: vec![],
            as_of: Utc::now(),
            data_state: DataState::Missing,
        };
        s.liquidity.pools.push(DiscoveredPool {
            chain_id: "ethereum".into(),
            pair_address: "0xabcabcabcabcabcabcabcabcabcabcabcabcabca".into(),
            dex: Some("uniswap".into()),
            liquidity_usd: Some(10_000.0),
            price_usd: None,
        });
        let g = from_snapshot("pepe", &s);
        assert!(g.relationships.iter().any(|e| e.relationship_type == "PART_OF"));
        assert!(g.entities.iter().any(|e| e.kind == "Pool"));
        assert!(g.entities.iter().all(|e| e.kind != "Person"));
    }

    #[test]
    fn expired_edge_is_not_valid_at() {
        let now = Utc::now();
        let e = GraphEdge {
            source: "a".into(),
            target: "b".into(),
            relationship_type: "PART_OF".into(),
            confidence: 0.5,
            evidence: vec![],
            status: "observed".into(),
            valid_from: Some(now - chrono::Duration::hours(2)),
            valid_to: Some(now - chrono::Duration::hours(1)),
        };
        assert!(valid_at(&[e], now).is_empty());
    }
}
