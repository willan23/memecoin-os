use crate::evidence::EvidenceChunk;
use crate::models::{DataState, ObservationCard, TokenSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    Market,
    Onchain,
    Risk,
    Development,
    Community,
    Narrative,
}

impl AgentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AgentKind::Market => "market-agent/v1",
            AgentKind::Onchain => "onchain-agent/v1",
            AgentKind::Risk => "risk-agent/v1",
            AgentKind::Development => "development-agent/v1",
            AgentKind::Community => "community-agent/v1",
            AgentKind::Narrative => "narrative-agent/v1",
        }
    }
}

pub fn intent_agents(question: &str) -> Vec<AgentKind> {
    let q = question.to_lowercase();
    let mut out = Vec::new();
    if q.contains("liq") || q.contains("pool") || q.contains("volume") || q.contains("market") {
        out.push(AgentKind::Market);
    }
    if q.contains("holder") || q.contains("whale") || q.contains("onchain") || q.contains("transfer") {
        out.push(AgentKind::Onchain);
    }
    if q.contains("risk") {
        out.push(AgentKind::Risk);
    }
    if q.contains("dev") || q.contains("github") || q.contains("commit") {
        out.push(AgentKind::Development);
    }
    if q.contains("social") || q.contains("community") || q.contains("mention") {
        out.push(AgentKind::Community);
    }
    if q.contains("narrative") || q.contains("meme") || q.contains("theme") {
        out.push(AgentKind::Narrative);
    }
    if out.is_empty() {
        out = vec![
            AgentKind::Market,
            AgentKind::Onchain,
            AgentKind::Risk,
            AgentKind::Development,
            AgentKind::Community,
            AgentKind::Narrative,
        ];
    }
    out
}

pub fn run(kind: AgentKind, snap: &TokenSnapshot, chunks: &[&EvidenceChunk]) -> ObservationCard {
    match kind {
        AgentKind::Market => market(snap, chunks),
        AgentKind::Onchain => onchain(snap, chunks),
        AgentKind::Risk => risk(snap),
        AgentKind::Development => development(snap),
        AgentKind::Community => community(snap),
        AgentKind::Narrative => narrative(snap),
    }
}

fn card(
    observation: String,
    evidence: Vec<String>,
    confidence: f64,
    interpretation: String,
    risk: Option<String>,
    data_state: DataState,
    model_id: &str,
) -> ObservationCard {
    ObservationCard {
        observation,
        evidence,
        confidence,
        interpretation,
        risk,
        data_state,
        model_id: Some(model_id.into()),
    }
}

fn market(s: &TokenSnapshot, chunks: &[&EvidenceChunk]) -> ObservationCard {
    let cite = chunks
        .iter()
        .filter(|c| c.source == "market" || c.source == "liquidity")
        .map(|c| c.id.clone())
        .collect::<Vec<_>>();
    if s.market.data_state.present() && s.liquidity.data_state.present() {
        card(
            format!(
                "Market live: volume_24h={} liquidity_usd={} pools={}",
                s.market.volume_24h_usd, s.liquidity.liquidity_usd, s.liquidity.pool_count
            ),
            cite,
            s.market.provenance.confidence.min(s.liquidity.provenance.confidence),
            "Spot and pool observations only — not a price target.".into(),
            Some("Depth can vanish quickly".into()),
            s.market.data_state.clone(),
            AgentKind::Market.as_str(),
        )
    } else {
        card(
            "INSUFFICIENT_EVIDENCE for market/liquidity.".into(),
            vec!["INSUFFICIENT_EVIDENCE".into()],
            0.0,
            "Missing source is not zero activity.".into(),
            None,
            DataState::Missing,
            AgentKind::Market.as_str(),
        )
    }
}

fn onchain(s: &TokenSnapshot, chunks: &[&EvidenceChunk]) -> ObservationCard {
    let cite = chunks
        .iter()
        .filter(|c| c.source == "onchain")
        .map(|c| c.id.clone())
        .collect::<Vec<_>>();
    if s.onchain.data_state.present() {
        card(
            format!(
                "On-chain: holders={} transfers_24h={} (0 transfers means unindexed, not idle, if indexer off)",
                s.onchain.holders, s.onchain.transfers_24h
            ),
            cite,
            s.onchain.provenance.confidence,
            "Holder structure is not identity and not a trade signal.".into(),
            Some("Concentration can be hidden without distribution sample".into()),
            s.onchain.data_state.clone(),
            AgentKind::Onchain.as_str(),
        )
    } else {
        card(
            "INSUFFICIENT_EVIDENCE for holders/transfers.".into(),
            vec!["INSUFFICIENT_EVIDENCE".into()],
            0.0,
            "Holders are not 0.".into(),
            None,
            DataState::Missing,
            AgentKind::Onchain.as_str(),
        )
    }
}

fn risk(s: &TokenSnapshot) -> ObservationCard {
    card(
        format!("Composite risk {:.0} ({:?})", s.risk.score, s.risk.level),
        vec![format!("risk_score={:.0} level={:?}", s.risk.score, s.risk.level)],
        s.risk.confidence,
        s.risk.why.clone(),
        Some("Risk score is not a ban or a buy rating".into()),
        s.data_state.clone(),
        AgentKind::Risk.as_str(),
    )
}

fn development(s: &TokenSnapshot) -> ObservationCard {
    if s.development.data_state.present() {
        card(
            format!("Development commits_30d={}", s.development.commits_30d),
            vec![format!("commits_30d={}", s.development.commits_30d)],
            s.development.provenance.confidence,
            "Public repo activity, not shipping quality.".into(),
            None,
            s.development.data_state.clone(),
            AgentKind::Development.as_str(),
        )
    } else {
        card(
            "INSUFFICIENT_EVIDENCE for development.".into(),
            vec!["INSUFFICIENT_EVIDENCE".into()],
            0.0,
            "No public GitHub linked or provider off.".into(),
            None,
            DataState::Missing,
            AgentKind::Development.as_str(),
        )
    }
}

fn community(s: &TokenSnapshot) -> ObservationCard {
    if s.social.data_state.present() {
        card(
            format!("Community organicness={:.2}", s.social.organicness),
            vec![format!("organicness={:.2}", s.social.organicness)],
            s.social.provenance.confidence,
            "Quality over raw mention count.".into(),
            None,
            s.social.data_state.clone(),
            AgentKind::Community.as_str(),
        )
    } else {
        card(
            "INSUFFICIENT_EVIDENCE — social firehose not licensed.".into(),
            vec!["INSUFFICIENT_EVIDENCE".into()],
            0.0,
            "Mentions are unknown, not zero.".into(),
            None,
            DataState::Missing,
            AgentKind::Community.as_str(),
        )
    }
}

fn narrative(s: &TokenSnapshot) -> ObservationCard {
    card(
        format!("Declared narratives: {}", s.token.narratives.join(", ")),
        vec![format!("narratives {}", s.token.narratives.join(","))],
        0.45,
        "Registry tags + later radar. Not social velocity.".into(),
        None,
        DataState::Live,
        AgentKind::Narrative.as_str(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::{missing_development, missing_liquidity, missing_market, missing_onchain, missing_social};
    use crate::models::{
        DailyBriefing, EcosystemScore, Genome, RiskLevel, RiskReport, TokenStatus, TokenSummary,
        VerificationLevel,
    };
    use chrono::Utc;

    fn empty_snap() -> TokenSnapshot {
        TokenSnapshot {
            token: TokenSummary {
                id: "x".into(),
                symbol: "X".into(),
                name: "X".into(),
                status: TokenStatus::Listed,
                verification: VerificationLevel::Unverified,
                narratives: vec!["meme".into()],
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
                value: 0.0,
                confidence: 0.0,
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
                score: 0.0,
                level: RiskLevel::Unknown,
                confidence: 0.0,
                as_of: Utc::now(),
                factors: vec![],
                why: "t".into(),
            },
            growth: vec![],
            timeline: vec![],
            briefing: DailyBriefing {
                token_id: "x".into(),
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
    fn community_is_insufficient_without_firehose() {
        let s = empty_snap();
        let o = run(AgentKind::Community, &s, &[]);
        assert_eq!(o.data_state, DataState::Missing);
        assert!(o.observation.contains("INSUFFICIENT_EVIDENCE"));
        assert_eq!(o.confidence, 0.0);
    }
}
