use crate::models::{DataState, TokenSnapshot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Derived evidence graph. Never invents a missing layer into SUPPORTS.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimRelation {
    Supports,
    Contradicts,
    Insufficient,
}

impl ClaimRelation {
    pub fn as_str(self) -> &'static str {
        match self {
            ClaimRelation::Supports => "supports",
            ClaimRelation::Contradicts => "contradicts",
            ClaimRelation::Insufficient => "insufficient",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "supports" => ClaimRelation::Supports,
            "contradicts" => ClaimRelation::Contradicts,
            _ => ClaimRelation::Insufficient,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceClaim {
    pub id: String,
    pub claim: String,
    pub relation: ClaimRelation,
    pub evidence: Vec<String>,
    pub data_state: DataState,
    pub confidence: f64,
    /// Set only when the row was read from Postgres. Live derivation leaves this blank.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stored_at: Option<DateTime<Utc>>,
}

pub fn from_snapshot(s: &TokenSnapshot) -> Vec<EvidenceClaim> {
    let id = &s.token.id;
    vec![
        layer(
            id,
            "tradable_market",
            "Observed spot market (price from a live provider)",
            &s.market.data_state,
            s.market.provenance.confidence,
            vec![
                format!("provider={}", s.market.provenance.provider),
                format!("price_present={}", s.market.price_usd > 0.0),
            ],
        ),
        layer(
            id,
            "tradable_liquidity",
            "Observed DEX liquidity",
            &s.liquidity.data_state,
            s.liquidity.provenance.confidence,
            vec![
                format!("provider={}", s.liquidity.provenance.provider),
                format!("pools={}", s.liquidity.pool_count),
            ],
        ),
        layer(
            id,
            "holder_base",
            "Observed holder count",
            &s.onchain.data_state,
            s.onchain.provenance.confidence,
            vec![
                format!("provider={}", s.onchain.provenance.provider),
                format!("holders={}", s.onchain.holders),
            ],
        ),
        layer(
            id,
            "organic_social",
            "Licensed mention firehose / organicness",
            &s.social.data_state,
            s.social.provenance.confidence,
            vec!["firehose licence required; first-party chat is a different plane".into()],
        ),
        layer(
            id,
            "public_development",
            "Public repository activity",
            &s.development.data_state,
            s.development.provenance.confidence,
            vec!["needs official github URL".into()],
        ),
        EvidenceClaim {
            id: format!("{id}:verified_safe"),
            claim: "VERIFIED implies SAFE".into(),
            relation: ClaimRelation::Contradicts,
            evidence: vec![
                format!("declared={}", format!("{:?}", s.token.verification).to_lowercase()),
                "VERIFIED ≠ SAFE".into(),
            ],
            data_state: DataState::Live,
            confidence: 1.0,
            stored_at: None,
        },
    ]
}

fn layer(
    token_id: &str,
    key: &str,
    claim: &str,
    state: &DataState,
    confidence: f64,
    evidence: Vec<String>,
) -> EvidenceClaim {
    let relation = match state {
        DataState::Live | DataState::Recent | DataState::Stale => ClaimRelation::Supports,
        DataState::Conflict => ClaimRelation::Contradicts,
        DataState::Simulated => ClaimRelation::Insufficient,
        DataState::Missing => ClaimRelation::Insufficient,
    };
    EvidenceClaim {
        id: format!("{token_id}:{key}"),
        claim: claim.into(),
        relation,
        evidence,
        data_state: state.clone(),
        confidence: if matches!(relation, ClaimRelation::Supports) {
            confidence
        } else {
            0.0
        },
        stored_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        DailyBriefing, EcosystemScore, Genome, RiskLevel, RiskReport, TokenStatus, TokenSummary,
        VerificationLevel,
    };
    use crate::quality::{
        missing_development, missing_liquidity, missing_market, missing_onchain, missing_social,
    };
    use chrono::Utc;

    fn snap() -> TokenSnapshot {
        let mut market = missing_market();
        market.data_state = DataState::Live;
        market.price_usd = 0.0001;
        TokenSnapshot {
            token: TokenSummary {
                id: "pepe".into(),
                symbol: "PEPE".into(),
                name: "Pepe".into(),
                status: TokenStatus::Listed,
                verification: VerificationLevel::DataVerified,
                narratives: vec![],
                primary_chain: "ethereum".into(),
                website: None,
                description: None,
                contract: None,
                chains: vec!["ethereum".into()],
            },
            market,
            liquidity: missing_liquidity(),
            onchain: missing_onchain(),
            social: missing_social(),
            development: missing_development(),
            scores: EcosystemScore {
                algorithm: "ecosystem-health".into(),
                algorithm_version: "1.0.0".into(),
                value: 0.0,
                confidence: 0.0,
                as_of: Utc::now(),
                components: vec![],
                why: "test".into(),
            },
            genome: Genome {
                algorithm_version: "1.0.0".into(),
                dimensions: vec![],
            },
            risk: RiskReport {
                algorithm: "token-risk".into(),
                algorithm_version: "1.0.0".into(),
                score: 0.0,
                level: RiskLevel::Unknown,
                confidence: 0.0,
                as_of: Utc::now(),
                factors: vec![],
                why: "test".into(),
            },
            growth: vec![],
            timeline: vec![],
            briefing: DailyBriefing {
                token_id: "pepe".into(),
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
            data_state: DataState::Live,
        }
    }

    #[test]
    fn live_market_supports_missing_social_is_insufficient() {
        let claims = from_snapshot(&snap());
        let market = claims.iter().find(|c| c.id.ends_with(":tradable_market")).unwrap();
        let social = claims.iter().find(|c| c.id.ends_with(":organic_social")).unwrap();
        assert_eq!(market.relation, ClaimRelation::Supports);
        assert_eq!(social.relation, ClaimRelation::Insufficient);
        assert!(claims
            .iter()
            .any(|c| c.relation == ClaimRelation::Contradicts && c.id.ends_with(":verified_safe")));
    }
}
