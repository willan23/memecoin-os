use crate::genome_cluster::{self, GenomeToken};
use crate::models::TokenSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimilarityReport {
    pub a: String,
    pub b: String,
    pub similarity: f64,
    pub similarities: Vec<String>,
    pub differences: Vec<String>,
    pub confidence: f64,
    pub limitations: Vec<String>,
    pub disclaimer: String,
}

pub fn compare(a: &TokenSnapshot, b: &TokenSnapshot) -> SimilarityReport {
    let ga = genome_token(a);
    let gb = genome_token(b);
    let sim = genome_cluster::similarity(&ga, &gb);
    let mut similarities = Vec::new();
    let mut differences = Vec::new();
    if a.token.primary_chain == b.token.primary_chain {
        similarities.push(format!("same chain {}", a.token.primary_chain));
    } else {
        differences.push(format!(
            "chain {} vs {}",
            a.token.primary_chain, b.token.primary_chain
        ));
    }
    let na: std::collections::HashSet<_> = a.token.narratives.iter().collect();
    let nb: std::collections::HashSet<_> = b.token.narratives.iter().collect();
    for t in na.intersection(&nb) {
        similarities.push(format!("shared narrative {t}"));
    }
    if (a.scores.value - b.scores.value).abs() > 15.0 {
        differences.push(format!(
            "health {:.0} vs {:.0}",
            a.scores.value, b.scores.value
        ));
    }
    SimilarityReport {
        a: a.token.id.clone(),
        b: b.token.id.clone(),
        similarity: (sim * 100.0).round() / 100.0,
        similarities,
        differences,
        confidence: a.scores.confidence.min(b.scores.confidence).min(0.75),
        limitations: vec![
            "Genome + narrative Jaccard + chain. Not market cap.".into(),
            "Does not imply future price or 'repeat performance'.".into(),
        ],
        disclaimer: "Structural similarity only. Not a prediction that one ecosystem will repeat the other.".into(),
    }
}

pub fn rank_against(focus: &TokenSnapshot, others: &[TokenSnapshot]) -> Vec<SimilarityReport> {
    let mut rows: Vec<_> = others
        .iter()
        .filter(|o| o.token.id != focus.token.id)
        .map(|o| compare(focus, o))
        .collect();
    rows.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
    rows
}

fn genome_token(s: &TokenSnapshot) -> GenomeToken {
    GenomeToken {
        token_id: s.token.id.clone(),
        symbol: s.token.symbol.clone(),
        chain: s.token.primary_chain.clone(),
        narratives: s.token.narratives.clone(),
        values: s.genome.dimensions.iter().map(|d| d.value).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::{missing_development, missing_liquidity, missing_market, missing_onchain, missing_social};
    use crate::models::{
        DailyBriefing, DataState, EcosystemScore, Genome, GenomeDimension, RiskLevel, RiskReport, TokenStatus,
        TokenSummary, VerificationLevel,
    };
    use chrono::Utc;

    fn snap(id: &str, chain: &str, vals: Vec<f64>) -> TokenSnapshot {
        TokenSnapshot {
            token: TokenSummary {
                id: id.into(),
                symbol: id.to_uppercase(),
                name: id.into(),
                status: TokenStatus::Listed,
                verification: VerificationLevel::Unverified,
                narratives: vec!["meme".into()],
                primary_chain: chain.into(),
                website: None,
                description: None,
                contract: None,
                chains: vec![chain.into()],
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
                dimensions: vals
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| GenomeDimension {
                        id: format!("d{i}"),
                        label: format!("d{i}"),
                        value,
                    })
                    .collect(),
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
                token_id: id.into(),
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
    fn similar_genomes_do_not_claim_price() {
        let a = snap("a", "ethereum", vec![1.0, 0.0, 0.0]);
        let b = snap("b", "ethereum", vec![1.0, 0.0, 0.0]);
        let r = compare(&a, &b);
        assert!(r.similarity > 0.8);
        assert!(r.disclaimer.contains("Not a prediction"));
    }
}
