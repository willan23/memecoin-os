use crate::models::{DataState, TokenSnapshot};
use serde::{Deserialize, Serialize};

/// Structured fact retrieved for RAG. Never invented.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceChunk {
    pub id: String,
    pub token_id: String,
    pub source: String,
    pub text: String,
    pub data_state: DataState,
    pub confidence: f64,
}

pub fn corpus(snapshots: &[TokenSnapshot]) -> Vec<EvidenceChunk> {
    let mut out = Vec::new();
    for s in snapshots {
        let id = &s.token.id;
        out.push(chunk(
            id,
            "identity",
            format!(
                "{} {} on {} verification {:?} narratives {}",
                s.token.name,
                s.token.symbol,
                s.token.primary_chain,
                s.token.verification,
                s.token.narratives.join(",")
            ),
            DataState::Live,
            0.9,
        ));
        out.push(state_chunk(
            id,
            "market",
            if s.market.data_state.present() {
                format!(
                    "price_usd={} mcap={} volume_24h={} change_24h={:+.2}% state={}",
                    s.market.price_usd,
                    s.market.market_cap_usd,
                    s.market.volume_24h_usd,
                    s.market.change_24h_pct,
                    s.market.data_state.as_str()
                )
            } else {
                "market INSUFFICIENT_EVIDENCE".into()
            },
            s.market.data_state.clone(),
            s.market.provenance.confidence,
        ));
        out.push(state_chunk(
            id,
            "liquidity",
            if s.liquidity.data_state.present() {
                format!(
                    "liquidity_usd={} pools={} spread_bps={} lp_7d={:+.1}%",
                    s.liquidity.liquidity_usd,
                    s.liquidity.pool_count,
                    s.liquidity.spread_bps,
                    s.liquidity.lp_change_7d_pct
                )
            } else {
                "liquidity INSUFFICIENT_EVIDENCE".into()
            },
            s.liquidity.data_state.clone(),
            s.liquidity.provenance.confidence,
        ));
        out.push(state_chunk(
            id,
            "onchain",
            if s.onchain.data_state.present() {
                format!(
                    "holders={} whales={} top10={:.1}% transfers_24h={}",
                    s.onchain.holders,
                    s.onchain.whale_holders,
                    s.onchain.top10_concentration_pct,
                    s.onchain.transfers_24h
                )
            } else {
                "onchain INSUFFICIENT_EVIDENCE".into()
            },
            s.onchain.data_state.clone(),
            s.onchain.provenance.confidence,
        ));
        out.push(state_chunk(
            id,
            "social",
            if s.social.data_state.present() {
                format!(
                    "organicness={:.2} unique_24h={} bot={:.2}",
                    s.social.organicness, s.social.unique_accounts_24h, s.social.bot_probability
                )
            } else {
                "social INSUFFICIENT_EVIDENCE firehose not licensed".into()
            },
            s.social.data_state.clone(),
            s.social.provenance.confidence,
        ));
        out.push(state_chunk(
            id,
            "development",
            if s.development.data_state.present() {
                format!(
                    "commits_30d={} contributors={} activity={:.0}",
                    s.development.commits_30d,
                    s.development.active_contributors_30d,
                    s.development.activity_score
                )
            } else {
                "development INSUFFICIENT_EVIDENCE".into()
            },
            s.development.data_state.clone(),
            s.development.provenance.confidence,
        ));
        out.push(chunk(
            id,
            "risk",
            format!(
                "risk_score={:.0} level={:?} why={}",
                s.risk.score, s.risk.level, s.risk.why
            ),
            s.data_state.clone(),
            s.risk.confidence,
        ));
        out.push(chunk(
            id,
            "health",
            format!(
                "ecosystem_health={:.0} confidence={:.2} {}",
                s.scores.value, s.scores.confidence, s.scores.why
            ),
            s.data_state.clone(),
            s.scores.confidence,
        ));
        for o in &s.growth {
            out.push(chunk(
                id,
                "opportunity",
                format!("{} evidence={}", o.title, o.evidence.join("; ")),
                DataState::Live,
                o.confidence,
            ));
        }
    }
    out
}

fn chunk(token_id: &str, source: &str, text: String, data_state: DataState, confidence: f64) -> EvidenceChunk {
    EvidenceChunk {
        id: format!("{token_id}:{source}"),
        token_id: token_id.into(),
        source: source.into(),
        text,
        data_state,
        confidence,
    }
}

fn state_chunk(
    token_id: &str,
    source: &str,
    text: String,
    data_state: DataState,
    confidence: f64,
) -> EvidenceChunk {
    chunk(token_id, source, text, data_state, if data_state.present() { confidence } else { 0.0 })
}

/// Lexical RAG over structured facts. Complements Postgres; does not replace it.
pub fn retrieve<'a>(corpus: &'a [EvidenceChunk], query: &str, k: usize) -> Vec<&'a EvidenceChunk> {
    let terms: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() > 2)
        .map(|s| s.to_string())
        .collect();
    let mut scored: Vec<(&EvidenceChunk, f64)> = corpus
        .iter()
        .map(|c| {
            let hay = format!("{} {} {}", c.source, c.text, c.token_id).to_lowercase();
            let mut s = 0.0;
            for t in &terms {
                if hay.contains(t) {
                    s += 1.0;
                }
            }
            if query.to_lowercase().contains(&c.token_id) {
                s += 2.0;
            }
            (c, s)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    if scored.iter().all(|(_, s)| *s == 0.0) {
        return corpus.iter().take(k).collect();
    }
    scored.into_iter().filter(|(_, s)| *s > 0.0).take(k).map(|(c, _)| c).collect()
}

pub fn citations_allowed(evidence: &[String], corpus: &[EvidenceChunk]) -> bool {
    evidence.iter().all(|e| {
        let el = e.to_lowercase();
        el.contains("insufficient_evidence")
            || el.contains("missing")
            || corpus.iter().any(|c| {
                c.text.to_lowercase().contains(&el)
                    || el.contains(&c.source)
                    || c.id == *e
                    || c.text == *e
                    || e == "INSUFFICIENT_EVIDENCE"
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieve_ranks_query_terms() {
        let corpus = vec![
            EvidenceChunk {
                id: "a:market".into(),
                token_id: "pepe".into(),
                source: "market".into(),
                text: "volume_24h=1000 state=live".into(),
                data_state: DataState::Live,
                confidence: 0.8,
            },
            EvidenceChunk {
                id: "a:social".into(),
                token_id: "pepe".into(),
                source: "social".into(),
                text: "social INSUFFICIENT_EVIDENCE firehose not licensed".into(),
                data_state: DataState::Missing,
                confidence: 0.0,
            },
        ];
        let hit = retrieve(&corpus, "social mentions firehose", 2);
        assert_eq!(hit[0].source, "social");
        assert!(citations_allowed(&["INSUFFICIENT_EVIDENCE".into()], &corpus));
        assert!(!citations_allowed(&["secret invented fact".into()], &corpus));
    }
}
