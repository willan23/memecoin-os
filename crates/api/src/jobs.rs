use crate::cache;
use crate::db;
use crate::webhooks;
use crate::AppState;
use axum::http::StatusCode;
use chrono::Utc;
use memecoin_os_core::models::{TokenDefinition, TokenSnapshot};
use std::time::{Duration, Instant};

pub fn spawn(state: AppState) {
    if let Some(pool) = state.pool.clone() {
        crate::queue::spawn(pool);
    }
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(180));
        loop {
            tick.tick().await;
            if let Err(e) = run_once(&state).await {
                tracing::error!(error = %e, "snapshot refresh job failed");
            }
        }
    });
}

pub async fn run_once(state: &AppState) -> Result<(), String> {
    if let Some(pool) = &state.pool {
        if !db::flag_enabled(pool, "jobs.snapshot_refresh").await {
            return Ok(());
        }
    }
    if let Some(redis) = &state.redis {
        let mut r = redis.clone();
        if !cache::acquire_lease(&mut r, "lock:snapshot_refresh", 120).await {
            return Ok(());
        }
    }

    let started = Utc::now();
    let clock = Instant::now();
    let mut tokens_ok = 0i32;
    let mut tokens_err = 0i32;
    let mut last_err: Option<String> = None;
    let market_start = Instant::now();
    let defs = {
        let reg = state.registry.read().await;
        reg.list().into_iter().cloned().collect::<Vec<_>>()
    };
    let max_per_hour: i64 = std::env::var("TOKEN_REFRESH_PER_HOUR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12);
    let prev_map = state.snapshots.read().await.clone();
    let mut next = prev_map.clone();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .ok();

    for def in &defs {
        if let Some(redis) = &state.redis {
            let mut r = redis.clone();
            if !cache::quota_ok(&mut r, &def.token_id, max_per_hour).await {
                continue;
            }
        }
        let prev = prev_map.get(&def.token_id);
        match state.engine.snapshot_with_prior(def, prev).await {
            Ok(mut snap) => {
                persist_refresh(state, def, &mut snap, prev, http.as_ref()).await;
                next.insert(snap.token.id.clone(), snap);
                tokens_ok += 1;
            }
            Err(e) => {
                tokens_err += 1;
                last_err = Some(e.to_string());
            }
        }
    }

    let market_ms = market_start.elapsed().as_millis() as u64;
    *state.snapshots.write().await = next;
    if let Some(pool) = &state.pool {
        let snaps = state.snapshots.read().await;
        let refs: Vec<_> = snaps.values().collect();
        crate::ecosystem_job::run_global(pool, &refs).await;
    }
    let lag_ms = clock.elapsed().as_millis() as i32;
    let spans = serde_json::json!({
        "market_refresh_ms": market_ms,
        "persist_ms": lag_ms.saturating_sub(market_ms as i32),
        "event_emission": true
    });
    if let Some(pool) = &state.pool {
        let _ = db::insert_job_run(
            pool,
            "snapshot_refresh",
            started,
            last_err.is_none(),
            lag_ms,
            tokens_ok,
            tokens_err,
            last_err.as_deref(),
            spans,
        )
        .await;
    }
    *state.last_job.write().await = Some(serde_json::json!({
        "job_name": "snapshot_refresh",
        "lag_ms": lag_ms,
        "tokens_ok": tokens_ok,
        "tokens_err": tokens_err,
        "finished_at": Utc::now(),
    }));
    tracing::info!(lag_ms, tokens_ok, tokens_err, "snapshot refresh complete");
    Ok(())
}

pub async fn refresh_token(
    state: &AppState,
    id: &str,
) -> Result<TokenSnapshot, (StatusCode, String)> {
    let def = {
        let reg = state.registry.read().await;
        reg.get(id)
            .map(|d| d.clone())
            .map_err(|_| (StatusCode::NOT_FOUND, "token not found".into()))?
    };
    if let Some(redis) = &state.redis {
        let mut r = redis.clone();
        let max_per_hour: i64 = std::env::var("TOKEN_REFRESH_PER_HOUR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(12);
        if !cache::quota_ok(&mut r, &def.token_id, max_per_hour).await {
            return Err((StatusCode::TOO_MANY_REQUESTS, "token refresh quota exceeded".into()));
        }
    }
    let prev = state.snapshots.read().await.get(id).cloned();
    let mut snap = state
        .engine
        .snapshot_with_prior(&def, prev.as_ref())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .ok();
    persist_refresh(state, &def, &mut snap, prev.as_ref(), http.as_ref()).await;
    state
        .snapshots
        .write()
        .await
        .insert(snap.token.id.clone(), snap.clone());
    Ok(snap)
}

async fn persist_refresh(
    state: &AppState,
    def: &TokenDefinition,
    snap: &mut TokenSnapshot,
    prev: Option<&TokenSnapshot>,
    http: Option<&reqwest::Client>,
) {
    let Some(pool) = &state.pool else {
        return;
    };
    let _ = db::upsert_token(pool, def).await;
    overlay_history(pool, snap).await;
    crate::onchain_job::run(pool, def, snap).await;
    let _ = db::persist_snapshot(pool, snap).await;
    if db::flag_enabled(pool, "jobs.raw_store").await {
        if let Ok(bytes) = serde_json::to_vec(&snap) {
            let store = memecoin_os_core::storage::LocalFsStore::from_env();
            if let Ok(obj) = memecoin_os_core::storage::ObjectStore::put(&store, &bytes, "snapshot") {
                let _ = db::upsert_object_blob(pool, &obj.hash, "snapshot", obj.bytes as i32, &obj.path).await;
            }
        }
    }
    if db::flag_enabled(pool, "jobs.queue").await {
        if let Ok(tenant) = db::default_tenant_id(pool).await {
            let _ = crate::queue::enqueue(
                pool,
                tenant,
                "domain_event",
                serde_json::json!({"token_id": snap.token.id, "kind": "snapshot_persisted"}),
            )
            .await;
        }
    }
    crate::twin_api::persist_twin(pool, snap).await;
    crate::twin_api::emit(state, "twin.updated", &snap.token.id);
    let rl = if snap.market.data_state.as_str() == "stale" { 1 } else { 0 };
    let _ = db::upsert_provider_health(
        pool,
        "coingecko",
        snap.market.data_state.present(),
        None,
        rl,
        Some(snap.market.provenance.freshness_secs as i32),
        Some(snap.market.data_state.as_str()),
    )
    .await;
    let _ = db::upsert_provider_health(
        pool,
        "dexscreener",
        snap.liquidity.data_state.present(),
        None,
        0,
        Some(snap.liquidity.provenance.freshness_secs as i32),
        Some(snap.liquidity.data_state.as_str()),
    )
    .await;
    let _ = db::upsert_provider_health(
        pool,
        "github",
        snap.development.data_state.present(),
        None,
        0,
        Some(snap.development.provenance.freshness_secs as i32),
        Some(snap.development.data_state.as_str()),
    )
    .await;
    if snap.onchain.data_state.present() || snap.onchain.provenance.provider != "none" {
        let _ = db::upsert_provider_health(
            pool,
            snap.onchain.provenance.provider.as_str(),
            snap.onchain.data_state.present(),
            None,
            0,
            Some(snap.onchain.provenance.freshness_secs as i32),
            Some(snap.onchain.data_state.as_str()),
        )
        .await;
    }
    if db::flag_enabled(pool, "jobs.briefing").await {
        let _ = db::upsert_briefing(pool, &snap.briefing).await;
    }
    let alerts = state.engine.alerts_with_prev(snap, prev);
    let hooks = db::list_webhooks(pool).await.unwrap_or_default();
    let send = db::flag_enabled(pool, "jobs.webhooks").await;
    for a in alerts {
        if !db::alert_rule_enabled(pool, &a.kind).await {
            continue;
        }
        if let Ok(true) = db::insert_alert_if_new(pool, &a).await {
            if send {
                if let Some(c) = http {
                    webhooks::deliver(pool, c, &hooks, &a).await;
                }
            }
        }
    }
    persist_intelligence(pool, snap).await;
    let known: Vec<String> = {
        let reg = state.registry.read().await;
        reg.list()
            .iter()
            .filter_map(|d| d.primary_chain().map(|c| c.contracts.token.address.clone()))
            .collect()
    };
    crate::ecosystem_job::run_token(pool, def, snap, &known).await;
}

fn series_since<F>(hist: &[db::HistoryPoint], hours: i64, pick: F) -> Vec<f64>
where
    F: Fn(&db::HistoryPoint) -> Option<f64>,
{
    let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours);
    hist.iter()
        .filter(|p| p.t >= cutoff)
        .filter_map(pick)
        .filter(|v| *v > 0.0)
        .collect()
}

async fn persist_intelligence(pool: &sqlx::PgPool, snap: &TokenSnapshot) {
    use memecoin_os_core::baseline::{self, BaselineWindow};
    use memecoin_os_core::domain_events::{self, EventType};
    use memecoin_os_core::wallet;

    if db::flag_enabled(pool, "jobs.claims_persist").await {
        let claims = memecoin_os_core::claims::from_snapshot(snap);
        let _ = db::replace_claims(pool, &snap.token.id, &claims).await;
    }

    let chain = snap.token.primary_chain.clone();
    let hour = chrono::Utc::now().format("%Y%m%d%H").to_string();

    if snap.market.data_state.present() {
        let ev = domain_events::emit(
            EventType::ScoreUpdated,
            &snap.token.id,
            Some(chain.clone()),
            "scoring",
            snap.scores.confidence,
            &format!("score:{hour}"),
            serde_json::json!({"health": snap.scores.value, "data_state": snap.data_state.as_str()}),
        );
        let _ = db::insert_domain_event_if_new(pool, &ev).await;
    }

    if db::flag_enabled(pool, "jobs.pool_discovery").await && snap.liquidity.data_state.present() {
        for p in &snap.liquidity.pools {
            let _ = db::upsert_pool(pool, &snap.token.id, p).await;
            let ev = domain_events::emit(
                EventType::PoolDiscovered,
                &snap.token.id,
                Some(p.chain_id.clone()),
                "dexscreener",
                snap.liquidity.provenance.confidence,
                &p.pair_address,
                serde_json::json!({"pair": p.pair_address, "dex": p.dex, "liquidity_usd": p.liquidity_usd}),
            );
            let _ = db::insert_domain_event_if_new(pool, &ev).await;
        }
    }

    if snap.onchain.data_state.present() {
        for h in &snap.onchain.top_holders {
            let profile = wallet::classify_from_share(&h.address, &chain, &snap.token.id, h.share_pct);
            let _ = db::upsert_wallet_profile(pool, &profile).await;
        }
    }

    if !db::flag_enabled(pool, "jobs.baselines").await {
        return;
    }
    let Ok(hist) = db::history(pool, &snap.token.id, 90).await else {
        return;
    };
    let mut computed = Vec::new();
    for w in BaselineWindow::all() {
        let prices = series_since(&hist, w.hours(), |p| p.price_usd);
        let vols = series_since(&hist, w.hours(), |p| p.volume_24h_usd);
        let liqs = series_since(&hist, w.hours(), |p| p.liquidity_usd);
        for (metric, series) in [
            ("price_usd", prices.as_slice()),
            ("volume_24h_usd", vols.as_slice()),
            ("liquidity_usd", liqs.as_slice()),
        ] {
            let b = baseline::compute(metric, w, series);
            let _ = db::upsert_baseline(pool, &snap.token.id, &b).await;
            computed.push(b);
        }
    }
    for a in baseline::anomalies(&computed) {
        let ev = domain_events::emit(
            EventType::AnomalyDetected,
            &snap.token.id,
            Some(chain.clone()),
            "baseline",
            a.confidence,
            &format!("anomaly:{}:{}:{}", a.metric, a.window, hour),
            serde_json::to_value(&a).unwrap_or(serde_json::json!({})),
        );
        let _ = db::insert_domain_event_if_new(pool, &ev).await;
    }
}

pub async fn overlay_history(pool: &sqlx::PgPool, snap: &mut TokenSnapshot) {
    if let Ok(hist) = db::history(pool, &snap.token.id, 7).await {
        snap.price_series = hist
            .iter()
            .filter_map(|p| {
                p.price_usd.map(|v| memecoin_os_core::models::SparkPoint {
                    t: p.t.to_rfc3339(),
                    v,
                })
            })
            .collect();
        snap.health_series = hist
            .iter()
            .filter_map(|p| {
                p.health.map(|v| memecoin_os_core::models::SparkPoint {
                    t: p.t.to_rfc3339(),
                    v,
                })
            })
            .collect();
    }
    if let Ok(ev) = db::events(pool, &snap.token.id, 24).await {
        if !ev.is_empty() {
            snap.timeline = ev;
        }
    }
}
