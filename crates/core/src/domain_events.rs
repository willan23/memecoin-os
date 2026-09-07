use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Canonical domain-event envelope. Idempotent on `fingerprint`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomainEvent {
    pub event_id: String,
    pub event_type: EventType,
    pub entity_id: String,
    pub chain_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub detected_at: DateTime<Utc>,
    pub source: String,
    pub confidence: f64,
    pub payload: serde_json::Value,
    pub schema_version: u32,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    TokenDiscovered,
    TokenVerified,
    LiquidityUpdated,
    HolderSnapshotCreated,
    WhaleMovementDetected,
    SocialSpikeDetected,
    RiskChanged,
    ScoreUpdated,
    RecommendationGenerated,
    PoolDiscovered,
    AnomalyDetected,
    AnomalyResolved,
    WalletProfileUpdated,
    GenomeUpdated,
    BaselineComputed,
}

impl EventType {
    pub fn as_str(self) -> &'static str {
        match self {
            EventType::TokenDiscovered => "token_discovered",
            EventType::TokenVerified => "token_verified",
            EventType::LiquidityUpdated => "liquidity_updated",
            EventType::HolderSnapshotCreated => "holder_snapshot_created",
            EventType::WhaleMovementDetected => "whale_movement_detected",
            EventType::SocialSpikeDetected => "social_spike_detected",
            EventType::RiskChanged => "risk_changed",
            EventType::ScoreUpdated => "score_updated",
            EventType::RecommendationGenerated => "recommendation_generated",
            EventType::PoolDiscovered => "pool_discovered",
            EventType::AnomalyDetected => "anomaly_detected",
            EventType::AnomalyResolved => "anomaly_resolved",
            EventType::WalletProfileUpdated => "wallet_profile_updated",
            EventType::GenomeUpdated => "genome_updated",
            EventType::BaselineComputed => "baseline_computed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "token_discovered" => EventType::TokenDiscovered,
            "token_verified" => EventType::TokenVerified,
            "liquidity_updated" => EventType::LiquidityUpdated,
            "holder_snapshot_created" => EventType::HolderSnapshotCreated,
            "whale_movement_detected" => EventType::WhaleMovementDetected,
            "social_spike_detected" => EventType::SocialSpikeDetected,
            "risk_changed" => EventType::RiskChanged,
            "score_updated" => EventType::ScoreUpdated,
            "recommendation_generated" => EventType::RecommendationGenerated,
            "pool_discovered" => EventType::PoolDiscovered,
            "anomaly_detected" => EventType::AnomalyDetected,
            "anomaly_resolved" => EventType::AnomalyResolved,
            "wallet_profile_updated" => EventType::WalletProfileUpdated,
            "genome_updated" => EventType::GenomeUpdated,
            "baseline_computed" => EventType::BaselineComputed,
            _ => return None,
        })
    }
}

pub fn fingerprint(event_type: EventType, entity_id: &str, key: &str) -> String {
    let mut h = Sha256::new();
    h.update(event_type.as_str().as_bytes());
    h.update(b"|");
    h.update(entity_id.as_bytes());
    h.update(b"|");
    h.update(key.as_bytes());
    hex::encode(h.finalize())
}

pub fn emit(
    event_type: EventType,
    entity_id: impl Into<String>,
    chain_id: Option<String>,
    source: impl Into<String>,
    confidence: f64,
    idempotency_key: &str,
    payload: serde_json::Value,
) -> DomainEvent {
    let entity_id = entity_id.into();
    let now = Utc::now();
    DomainEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type,
        fingerprint: fingerprint(event_type, &entity_id, idempotency_key),
        entity_id,
        chain_id,
        occurred_at: now,
        detected_at: now,
        source: source.into(),
        confidence,
        payload,
        schema_version: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_idempotent() {
        let a = fingerprint(EventType::PoolDiscovered, "pepe", "0xpair");
        let b = fingerprint(EventType::PoolDiscovered, "pepe", "0xpair");
        let c = fingerprint(EventType::PoolDiscovered, "pepe", "0xother");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn emit_sets_schema_and_roundtrips_type() {
        let ev = emit(
            EventType::ScoreUpdated,
            "pepe",
            Some("ethereum".into()),
            "scoring",
            0.8,
            "2026-08-30",
            serde_json::json!({"health": 52.0}),
        );
        assert_eq!(ev.schema_version, 1);
        assert_eq!(EventType::parse(ev.event_type.as_str()), Some(EventType::ScoreUpdated));
    }
}
