use crate::db;
use memecoin_os_core::domain_events::{self, EventType};
use memecoin_os_core::indexer::{self, ChainIndexer, EvmRpcIndexer, IndexedTransfer};
use memecoin_os_core::models::{TokenDefinition, TokenSnapshot};
use memecoin_os_core::wallet::{self, WalletClassification};
use memecoin_os_core::{cex, cluster, whale};
use sqlx::PgPool;
use std::collections::HashSet;

pub async fn run(pool: &PgPool, def: &TokenDefinition, snap: &mut TokenSnapshot) {
    if !db::flag_enabled(pool, "jobs.indexer").await {
        return;
    }
    if std::env::var("FEATURE_ONCHAIN_INDEXER")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        return;
    }

    let Some(binding) = def.primary_chain() else {
        return;
    };
    let chain = binding.id.as_str();
    let token = binding.contracts.token.address.as_str();
    let decimals = binding.contracts.token.decimals;
    if !indexer::is_evm_address(token) {
        return;
    }

    let (transfers, source, last_block, err) = fetch_transfers(pool, snap, chain, token).await;
    if let Err(e) = db::upsert_indexer_cursor(
        pool,
        chain,
        &snap.token.id,
        last_block,
        &source,
        err.as_deref(),
    )
    .await
    {
        tracing::warn!(error = %e, "indexer cursor persist failed");
    }
    if transfers.is_empty() {
        overlay_stats(pool, snap).await;
        return;
    }

    let whales: HashSet<String> = snap
        .onchain
        .top_holders
        .iter()
        .filter(|h| h.share_pct >= 1.0)
        .map(|h| h.address.to_lowercase())
        .collect();
    let pools: HashSet<String> = snap
        .liquidity
        .pools
        .iter()
        .map(|p| p.pair_address.to_lowercase())
        .collect();

    let parsed: Vec<Option<f64>> = transfers
        .iter()
        .map(|t| whale::amount_token_units(&t.amount_raw, decimals))
        .collect();
    let present: Vec<f64> = parsed.iter().copied().flatten().collect();
    let large_local = whale::large_indices(&present);
    let mut large = HashSet::new();
    let mut j = 0usize;
    for (i, p) in parsed.iter().enumerate() {
        if p.is_some() {
            if large_local.contains(&j) {
                large.insert(i);
            }
            j += 1;
        }
    }
    let price = if snap.market.data_state.present() {
        snap.market.price_usd
    } else {
        0.0
    };

    let whale_on = db::flag_enabled(pool, "jobs.whale_engine").await;
    let cluster_on = db::flag_enabled(pool, "jobs.wallet_clusters").await;

    for (i, t) in transfers.iter().enumerate() {
        let _ = db::insert_transfer(pool, &snap.token.id, t).await;
        let usd = whale::amount_usd(&t.amount_raw, decimals, price);
        if whale_on {
            let events = whale::classify(t, &whales, &pools, usd, large.contains(&i));
            for ev in events {
                if let Ok(true) = db::insert_whale_movement(pool, &snap.token.id, &ev).await {
                    let de = domain_events::emit(
                        EventType::WhaleMovementDetected,
                        &snap.token.id,
                        Some(chain.into()),
                        "indexer",
                        ev.confidence,
                        &format!("{}:{}:{:?}", ev.tx_hash, ev.direction, ev.kind),
                        serde_json::to_value(&ev).unwrap_or(serde_json::json!({})),
                    );
                    let _ = db::insert_domain_event_if_new(pool, &de).await;
                }
                if ev.kind == whale::WhaleKind::CexDeposit || ev.kind == whale::WhaleKind::CexWithdrawal {
                    let profile = wallet::WalletProfile {
                        address: ev.wallet.clone(),
                        chain_id: chain.into(),
                        classification: WalletClassification::Cex,
                        confidence: cex::label_confidence(),
                        evidence: ev.evidence.clone(),
                        token_id: Some(snap.token.id.clone()),
                        share_pct: None,
                    };
                    let _ = db::upsert_wallet_profile(pool, &profile).await;
                }
            }
        }
        if !cex::is_hub(&t.from_address) && !cex::is_hub(&t.to_address) {
            let _ = db::upsert_graph_edge(pool, &snap.token.id, chain, &t.from_address, &t.to_address).await;
        }
    }

    if cluster_on {
        if let Ok(edges) = db::list_graph_edges(pool, &snap.token.id).await {
            let skip: HashSet<String> = edges
                .iter()
                .flat_map(|(a, b)| [a.clone(), b.clone()])
                .filter(|a| cex::is_hub(a))
                .collect();
            let clustered = cluster::cluster(&edges, &skip);
            let _ = db::replace_clusters(pool, &snap.token.id, chain, &clustered).await;
        }
    }

    overlay_stats(pool, snap).await;
}

async fn fetch_transfers(
    pool: &PgPool,
    snap: &TokenSnapshot,
    chain: &str,
    token: &str,
) -> (Vec<IndexedTransfer>, String, i64, Option<String>) {
    if let Some(ix) = EvmRpcIndexer::for_chain(chain) {
        if ix.rpc_configured() {
            match ix.get_block_height().await {
                Ok(Some(tip)) => {
                    let last = db::indexer_cursor(pool, chain, &snap.token.id)
                        .await
                        .ok()
                        .flatten()
                        .map(|b| b as u64)
                        .filter(|b| *b > 0);
                    if let Some(w) = indexer::next_window(last, tip) {
                        match ix.index_transfers(token, w.from_block, w.to_block).await {
                            Ok(rows) => {
                                return (rows, "rpc".into(), w.to_block as i64, None);
                            }
                            Err(e) => {
                                return (vec![], "rpc".into(), last.unwrap_or(0) as i64, Some(e.to_string()));
                            }
                        }
                    }
                    return (vec![], "rpc".into(), last.unwrap_or(0) as i64, None);
                }
                Ok(None) => {}
                Err(e) => {
                    return (vec![], "rpc".into(), 0, Some(e.to_string()));
                }
            }
        }
    }

    let key = std::env::var("ETHERSCAN_API_KEY").ok().filter(|s| !s.trim().is_empty());
    if let Some(key) = key {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build();
        if let Ok(client) = client {
            match indexer::etherscan_recent_transfers(&client, chain, token, &key).await {
                Ok(rows) => {
                    let max_block = rows.iter().map(|t| t.block_number).max().unwrap_or(0) as i64;
                    return (rows, "etherscan".into(), max_block, None);
                }
                Err(e) => return (vec![], "etherscan".into(), 0, Some(e.to_string())),
            }
        }
    }

    (vec![], "none".into(), 0, None)
}

async fn overlay_stats(pool: &PgPool, snap: &mut TokenSnapshot) {
    let Ok(stats) = db::transfer_stats_24h(pool, &snap.token.id).await else {
        return;
    };
    if stats.transfers_24h == 0 {
        return;
    }
    snap.onchain.transfers_24h = stats.transfers_24h;
    snap.onchain.unique_senders_24h = stats.unique_senders_24h;
    snap.onchain.large_transfers_24h = stats.large_transfers_24h;
    if stats.cex_in_usd > 0.0 {
        snap.onchain.exchange_inflow_usd = stats.cex_in_usd;
    }
    if stats.cex_out_usd > 0.0 {
        snap.onchain.exchange_outflow_usd = stats.cex_out_usd;
    }
}
