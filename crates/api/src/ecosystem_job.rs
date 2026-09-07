use crate::db;
use memecoin_os_core::discovery;
use memecoin_os_core::domain_events::{self, EventType};
use memecoin_os_core::genome_cluster::{self, GenomeToken};
use memecoin_os_core::models::{TokenDefinition, TokenSnapshot};
use memecoin_os_core::narrative::{self, NarrativeToken};
use memecoin_os_core::verification;
use sqlx::PgPool;

pub async fn run_token(pool: &PgPool, def: &TokenDefinition, snap: &TokenSnapshot, known_addresses: &[String]) {
    if db::flag_enabled(pool, "jobs.discovery").await && snap.liquidity.data_state.present() {
        for c in &snap.liquidity.counterparts {
            if let Some(a) = discovery::assess(c, known_addresses) {
                let _ = db::upsert_discovery(pool, &a).await;
                let ev = domain_events::emit(
                    EventType::TokenDiscovered,
                    &a.address,
                    Some(a.chain_id.clone()),
                    "dexscreener",
                    a.score,
                    &format!("{}:{}", a.chain_id, a.address),
                    serde_json::json!({"status": a.status.as_str(), "via": a.via_token, "auto_verified": false}),
                );
                let _ = db::insert_domain_event_if_new(pool, &ev).await;
            }
        }
    }
    if db::flag_enabled(pool, "jobs.verification").await {
        let report = verification::assess(def, snap);
        let _ = db::upsert_verification(pool, &report).await;
        let ev = domain_events::emit(
            EventType::TokenVerified,
            &snap.token.id,
            Some(snap.token.primary_chain.clone()),
            "verification",
            0.7,
            &format!("verify:{}:{}", snap.token.id, report.level.as_str()),
            serde_json::to_value(&report).unwrap_or(serde_json::json!({})),
        );
        let _ = db::insert_domain_event_if_new(pool, &ev).await;
    }
}

pub async fn run_global(pool: &PgPool, snaps: &[&TokenSnapshot]) {
    if db::flag_enabled(pool, "jobs.narratives").await {
        let tokens: Vec<NarrativeToken> = snaps
            .iter()
            .map(|s| NarrativeToken {
                token_id: s.token.id.clone(),
                tags: s.token.narratives.clone(),
                name: s.token.name.clone(),
                symbol: s.token.symbol.clone(),
                liquidity_change_pct: if s.liquidity.data_state.present() {
                    Some(s.liquidity.lp_change_7d_pct)
                } else {
                    None
                },
                volume_present: s.market.data_state.present() && s.market.volume_24h_usd > 0.0,
            })
            .collect();
        let rows = narrative::aggregate(&tokens);
        let _ = db::replace_narratives(pool, &rows).await;
    }
    if db::flag_enabled(pool, "jobs.genome_clusters").await {
        let items: Vec<GenomeToken> = snaps
            .iter()
            .map(|s| GenomeToken {
                token_id: s.token.id.clone(),
                symbol: s.token.symbol.clone(),
                chain: s.token.primary_chain.clone(),
                narratives: s.token.narratives.clone(),
                values: s.genome.dimensions.iter().map(|d| d.value).collect(),
            })
            .collect();
        let rows = genome_cluster::cluster(&items, 0.82);
        let _ = db::replace_genome_clusters(pool, &rows).await;
    }
}
