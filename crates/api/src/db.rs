use chrono::{DateTime, Utc};
use memecoin_os_core::models::{Alert, DailyBriefing, TokenDefinition, TokenSnapshot, WebhookEndpoint};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn load_token_definitions(pool: &PgPool) -> Result<Vec<TokenDefinition>, sqlx::Error> {
    let rows: Vec<(serde_json::Value,)> = sqlx::query_as("SELECT definition FROM tokens")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(v,)| serde_json::from_value(v).ok())
        .collect())
}

pub async fn upsert_token(pool: &PgPool, def: &TokenDefinition) -> Result<(), sqlx::Error> {
    let definition = serde_json::to_value(def).unwrap_or(serde_json::Value::Null);
    sqlx::query(
        "INSERT INTO tokens (id, schema_version, symbol, name, status, verification, definition, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, now())
         ON CONFLICT (id) DO UPDATE SET
            schema_version = EXCLUDED.schema_version,
            symbol = EXCLUDED.symbol,
            name = EXCLUDED.name,
            status = EXCLUDED.status,
            verification = EXCLUDED.verification,
            definition = EXCLUDED.definition,
            updated_at = now()",
    )
    .bind(&def.token_id)
    .bind(def.schema_version as i32)
    .bind(&def.symbol)
    .bind(&def.name)
    .bind(format!("{:?}", def.status).to_lowercase())
    .bind(format!("{:?}", def.verification).to_lowercase())
    .bind(definition)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn persist_snapshot(pool: &PgPool, snap: &TokenSnapshot) -> Result<(), sqlx::Error> {
    let payload = serde_json::to_value(snap).unwrap_or(serde_json::Value::Null);
    sqlx::query(
        "INSERT INTO token_snapshots (token_id, as_of, data_state, payload, updated_at)
         VALUES ($1, $2, $3, $4, now())
         ON CONFLICT (token_id) DO UPDATE SET
            as_of = EXCLUDED.as_of,
            data_state = EXCLUDED.data_state,
            payload = EXCLUDED.payload,
            updated_at = now()",
    )
    .bind(&snap.token.id)
    .bind(snap.as_of)
    .bind(snap.data_state.as_str())
    .bind(&payload)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO market_snapshots (token_id, as_of, price_usd, market_cap_usd, volume_24h_usd, change_24h_pct, data_state, payload)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&snap.token.id)
    .bind(snap.as_of)
    .bind(snap.market.price_usd)
    .bind(snap.market.market_cap_usd)
    .bind(snap.market.volume_24h_usd)
    .bind(snap.market.change_24h_pct)
    .bind(snap.market.data_state.as_str())
    .bind(serde_json::to_value(&snap.market).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO liquidity_snapshots (token_id, as_of, liquidity_usd, pool_count, spread_bps, data_state, payload)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&snap.token.id)
    .bind(snap.as_of)
    .bind(snap.liquidity.liquidity_usd)
    .bind(snap.liquidity.pool_count as i32)
    .bind(snap.liquidity.spread_bps)
    .bind(snap.liquidity.data_state.as_str())
    .bind(serde_json::to_value(&snap.liquidity).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO onchain_snapshots (token_id, as_of, holders, data_state, payload) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&snap.token.id)
    .bind(snap.as_of)
    .bind(snap.onchain.holders as i64)
    .bind(snap.onchain.data_state.as_str())
    .bind(serde_json::to_value(&snap.onchain).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO social_snapshots (token_id, as_of, mentions_24h, data_state, payload) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&snap.token.id)
    .bind(snap.as_of)
    .bind(snap.social.mentions_24h as i64)
    .bind(snap.social.data_state.as_str())
    .bind(serde_json::to_value(&snap.social).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO development_snapshots (token_id, as_of, commits_30d, data_state, payload) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&snap.token.id)
    .bind(snap.as_of)
    .bind(snap.development.commits_30d as i64)
    .bind(snap.development.data_state.as_str())
    .bind(serde_json::to_value(&snap.development).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO scores (token_id, algorithm, algorithm_version, as_of, score, breakdown, evidence)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (token_id, algorithm, algorithm_version, as_of) DO NOTHING",
    )
    .bind(&snap.token.id)
    .bind(&snap.scores.algorithm)
    .bind(&snap.scores.algorithm_version)
    .bind(snap.scores.as_of)
    .bind(snap.scores.value)
    .bind(serde_json::to_value(&snap.scores.components).unwrap_or(serde_json::Value::Null))
    .bind(serde_json::json!({ "why": snap.scores.why, "confidence": snap.scores.confidence }))
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO risk_reports (token_id, algorithm, algorithm_version, as_of, score, level, payload)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (token_id, algorithm, algorithm_version, as_of) DO NOTHING",
    )
    .bind(&snap.token.id)
    .bind(&snap.risk.algorithm)
    .bind(&snap.risk.algorithm_version)
    .bind(snap.risk.as_of)
    .bind(snap.risk.score)
    .bind(format!("{:?}", snap.risk.level))
    .bind(serde_json::to_value(&snap.risk).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;

    insert_event(
        pool,
        &snap.token.id,
        "ScoreUpdated",
        &format!("Health {:.0} ({})", snap.scores.value, snap.data_state.as_str()),
        Some(&format!("{:.0}", snap.scores.value)),
        snap.scores.confidence,
        "scoring",
    )
    .await?;
    if snap.liquidity.data_state.present() {
        insert_event(
            pool,
            &snap.token.id,
            "LiquidityUpdated",
            &format!("Liquidity ${:.0}", snap.liquidity.liquidity_usd),
            Some(&format!("{:.0}", snap.liquidity.liquidity_usd)),
            snap.liquidity.provenance.confidence,
            "dexscreener",
        )
        .await?;
    }
    if snap.onchain.data_state.present() {
        sqlx::query(
            "INSERT INTO holder_snapshots (token_id, as_of, chain_id, holders, active_holders_30d, new_holders_7d, top10_concentration_pct, top50_concentration_pct, whale_holders, payload)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&snap.token.id)
        .bind(snap.as_of)
        .bind(&snap.token.primary_chain)
        .bind(snap.onchain.holders as i64)
        .bind(snap.onchain.active_holders_30d as i64)
        .bind(snap.onchain.new_holders_7d)
        .bind(snap.onchain.top10_concentration_pct)
        .bind(snap.onchain.top50_concentration_pct)
        .bind(snap.onchain.whale_holders as i64)
        .bind(serde_json::to_value(&snap.onchain).unwrap_or(serde_json::Value::Null))
        .execute(pool)
        .await?;
        insert_event(
            pool,
            &snap.token.id,
            "HolderSnapshotCreated",
            &format!("Holders {}", snap.onchain.holders),
            Some(&format!("{}", snap.onchain.holders)),
            snap.onchain.provenance.confidence,
            &snap.onchain.provenance.provider,
        )
        .await?;
    }
    Ok(())
}

pub async fn insert_event(
    pool: &PgPool,
    token_id: &str,
    kind: &str,
    title: &str,
    delta: Option<&str>,
    confidence: f64,
    source: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO token_events (token_id, kind, title, delta, confidence, source)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(token_id)
    .bind(kind)
    .bind(title)
    .bind(delta)
    .bind(confidence)
    .bind(source)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_latest_snapshots(pool: &PgPool) -> Result<Vec<TokenSnapshot>, sqlx::Error> {
    let rows: Vec<(serde_json::Value,)> = sqlx::query_as("SELECT payload FROM token_snapshots")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(v,)| serde_json::from_value(v).ok())
        .collect())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoryPoint {
    pub t: DateTime<Utc>,
    pub price_usd: Option<f64>,
    pub market_cap_usd: Option<f64>,
    pub volume_24h_usd: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub health: Option<f64>,
    pub risk: Option<f64>,
    pub data_state: String,
}

pub async fn history(pool: &PgPool, token_id: &str, days: i32) -> Result<Vec<HistoryPoint>, sqlx::Error> {
    let rows: Vec<(DateTime<Utc>, f64, f64, f64, String)> = sqlx::query_as(
        "SELECT as_of, price_usd, market_cap_usd, volume_24h_usd, data_state
         FROM market_snapshots
         WHERE token_id = $1 AND as_of >= now() - ($2 || ' days')::interval
           AND data_state <> 'simulated'
         ORDER BY as_of ASC",
    )
    .bind(token_id)
    .bind(days.to_string())
    .fetch_all(pool)
    .await?;

    let liq: Vec<(DateTime<Utc>, f64)> = sqlx::query_as(
        "SELECT as_of, liquidity_usd FROM liquidity_snapshots
         WHERE token_id = $1 AND as_of >= now() - ($2 || ' days')::interval
           AND data_state <> 'simulated' ORDER BY as_of ASC",
    )
    .bind(token_id)
    .bind(days.to_string())
    .fetch_all(pool)
    .await?;

    let scores: Vec<(DateTime<Utc>, f64)> = sqlx::query_as(
        "SELECT as_of, score FROM scores WHERE token_id = $1 AND as_of >= now() - ($2 || ' days')::interval ORDER BY as_of ASC",
    )
    .bind(token_id)
    .bind(days.to_string())
    .fetch_all(pool)
    .await?;

    let risks: Vec<(DateTime<Utc>, f64)> = sqlx::query_as(
        "SELECT as_of, score FROM risk_reports WHERE token_id = $1 AND as_of >= now() - ($2 || ' days')::interval ORDER BY as_of ASC",
    )
    .bind(token_id)
    .bind(days.to_string())
    .fetch_all(pool)
    .await?;

    let mut out = Vec::new();
    for (t, price, mcap, vol, state) in rows {
        out.push(HistoryPoint {
            t,
            price_usd: Some(price),
            market_cap_usd: Some(mcap),
            volume_24h_usd: Some(vol),
            liquidity_usd: nearest(&liq, t),
            health: nearest(&scores, t),
            risk: nearest(&risks, t),
            data_state: state,
        });
    }
    Ok(out)
}

fn nearest(series: &[(DateTime<Utc>, f64)], t: DateTime<Utc>) -> Option<f64> {
    series
        .iter()
        .min_by_key(|(at, _)| (*at - t).num_seconds().abs())
        .filter(|(at, _)| (*at - t).num_seconds().abs() < 600)
        .map(|(_, v)| *v)
}

pub async fn events(
    pool: &PgPool,
    token_id: &str,
    limit: i64,
) -> Result<Vec<memecoin_os_core::models::TimelineEvent>, sqlx::Error> {
    let rows: Vec<(DateTime<Utc>, String, String, Option<String>, f64, String)> = sqlx::query_as(
        "SELECT occurred_at, kind, title, delta, confidence, source
         FROM token_events WHERE token_id = $1 ORDER BY occurred_at DESC LIMIT $2",
    )
    .bind(token_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(at, kind, title, delta, confidence, source)| {
            memecoin_os_core::models::TimelineEvent {
                at,
                kind,
                title,
                delta,
                confidence,
                source,
            }
        })
        .collect())
}

pub async fn insert_alert_if_new(pool: &PgPool, alert: &Alert) -> Result<bool, sqlx::Error> {
    let id: Uuid = alert.id.parse().unwrap_or_else(|_| Uuid::new_v4());
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM alert_fingerprints WHERE fingerprint = $1)")
            .bind(&alert.fingerprint)
            .fetch_one(pool)
            .await?;
    if exists {
        sqlx::query("UPDATE alert_fingerprints SET last_seen = now() WHERE fingerprint = $1")
            .bind(&alert.fingerprint)
            .execute(pool)
            .await?;
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO alerts (id, token_id, kind, severity, title, body, evidence, fired_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(&alert.token_id)
    .bind(&alert.kind)
    .bind(&alert.severity)
    .bind(&alert.title)
    .bind(&alert.body)
    .bind(serde_json::to_value(&alert.evidence).unwrap_or(serde_json::Value::Null))
    .bind(alert.fired_at)
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO alert_fingerprints (fingerprint, alert_id, token_id, kind) VALUES ($1, $2, $3, $4)",
    )
    .bind(&alert.fingerprint)
    .bind(id)
    .bind(&alert.token_id)
    .bind(&alert.kind)
    .execute(pool)
    .await?;
    Ok(true)
}

pub async fn load_alerts_for_token(pool: &PgPool, token_id: &str, limit: i64) -> Result<Vec<Alert>, sqlx::Error> {
    let rows: Vec<(Uuid, String, String, String, String, String, serde_json::Value, DateTime<Utc>)> =
        sqlx::query_as(
            "SELECT id, token_id, kind, severity, title, body, evidence, fired_at
             FROM alerts WHERE acknowledged_at IS NULL AND token_id = $1
             ORDER BY fired_at DESC LIMIT $2",
        )
        .bind(token_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(id, token_id, kind, severity, title, body, evidence, fired_at)| Alert {
            id: id.to_string(),
            token_id,
            kind,
            severity,
            title,
            body,
            evidence: serde_json::from_value(evidence).unwrap_or_default(),
            fired_at,
            fingerprint: String::new(),
        })
        .collect())
}

pub async fn load_alerts(pool: &PgPool, limit: i64) -> Result<Vec<Alert>, sqlx::Error> {
    let rows: Vec<(Uuid, String, String, String, String, String, serde_json::Value, DateTime<Utc>)> =
        sqlx::query_as(
            "SELECT id, token_id, kind, severity, title, body, evidence, fired_at FROM alerts WHERE acknowledged_at IS NULL ORDER BY fired_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(id, token_id, kind, severity, title, body, evidence, fired_at)| Alert {
            id: id.to_string(),
            token_id,
            kind,
            severity,
            title,
            body,
            evidence: serde_json::from_value(evidence).unwrap_or_default(),
            fired_at,
            fingerprint: String::new(),
        })
        .collect())
}

pub async fn upsert_briefing(pool: &PgPool, briefing: &DailyBriefing) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO daily_briefings (token_id, as_of, headline, payload)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (token_id, as_of) DO UPDATE SET headline = EXCLUDED.headline, payload = EXCLUDED.payload, generated_at = now()",
    )
    .bind(&briefing.token_id)
    .bind(briefing.as_of.date_naive())
    .bind(&briefing.headline)
    .bind(serde_json::to_value(briefing).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_briefing(pool: &PgPool, token_id: &str) -> Result<Option<DailyBriefing>, sqlx::Error> {
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT payload FROM daily_briefings WHERE token_id = $1 ORDER BY as_of DESC LIMIT 1",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(v,)| serde_json::from_value(v).ok()))
}

pub async fn insert_job_run(
    pool: &PgPool,
    job_name: &str,
    started: DateTime<Utc>,
    ok: bool,
    lag_ms: i32,
    tokens_ok: i32,
    tokens_err: i32,
    error: Option<&str>,
    spans: serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO job_runs (job_name, started_at, finished_at, ok, lag_ms, tokens_ok, tokens_err, error, spans)
         VALUES ($1, $2, now(), $3, $4, $5, $6, $7, $8)",
    )
    .bind(job_name)
    .bind(started)
    .bind(ok)
    .bind(lag_ms)
    .bind(tokens_ok)
    .bind(tokens_err)
    .bind(error)
    .bind(spans)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn last_job(pool: &PgPool) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let row: Option<(String, DateTime<Utc>, Option<DateTime<Utc>>, bool, Option<i32>, i32, i32)> =
        sqlx::query_as(
            "SELECT job_name, started_at, finished_at, ok, lag_ms, tokens_ok, tokens_err FROM job_runs ORDER BY started_at DESC LIMIT 1",
        )
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(name, started, finished, ok, lag, tok_ok, tok_err)| {
        serde_json::json!({
            "job_name": name, "started_at": started, "finished_at": finished,
            "ok": ok, "lag_ms": lag, "tokens_ok": tok_ok, "tokens_err": tok_err,
        })
    }))
}

pub async fn provider_health_json(pool: &PgPool) -> Result<serde_json::Value, sqlx::Error> {
    let rows: Vec<(String, Option<DateTime<Utc>>, Option<i32>, i32, Option<i32>, Option<String>)> =
        sqlx::query_as(
            "SELECT provider, last_ok_at, last_status, rate_limited, latency_ms, detail FROM provider_health",
        )
        .fetch_all(pool)
        .await?;
    let mut map = serde_json::Map::new();
    for (provider, last_ok, status, rl, lat, detail) in rows {
        map.insert(
            provider,
            serde_json::json!({"last_ok_at": last_ok, "last_status": status, "rate_limited": rl, "latency_ms": lat, "detail": detail}),
        );
    }
    Ok(serde_json::Value::Object(map))
}

pub async fn list_webhooks(pool: &PgPool) -> Result<Vec<WebhookEndpoint>, sqlx::Error> {
    let rows: Vec<(Uuid, String, String, Option<String>, Option<String>, bool, bool)> = sqlx::query_as(
        "SELECT id, kind, url, chat_id, token_id, enabled, digest_daily FROM webhook_endpoints ORDER BY created_at",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, kind, url, chat_id, token_id, enabled, digest_daily)| WebhookEndpoint {
            id: id.to_string(),
            kind,
            url,
            chat_id,
            token_id,
            enabled,
            digest_daily,
        })
        .collect())
}

pub async fn insert_webhook(
    pool: &PgPool,
    kind: &str,
    url: &str,
    chat_id: Option<&str>,
    token_id: Option<&str>,
    digest_daily: bool,
) -> Result<WebhookEndpoint, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO webhook_endpoints (id, kind, url, chat_id, token_id, digest_daily) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(kind)
    .bind(url)
    .bind(chat_id)
    .bind(token_id)
    .bind(digest_daily)
    .execute(pool)
    .await?;
    Ok(WebhookEndpoint {
        id: id.to_string(),
        kind: kind.into(),
        url: url.into(),
        chat_id: chat_id.map(|s| s.to_string()),
        token_id: token_id.map(|s| s.to_string()),
        enabled: true,
        digest_daily,
    })
}

pub async fn delete_webhook(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM webhook_endpoints WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn log_delivery(
    pool: &PgPool,
    alert_id: Uuid,
    webhook_id: Uuid,
    ok: bool,
    status: Option<i32>,
    detail: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO alert_deliveries (alert_id, webhook_id, ok, status, detail) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(alert_id)
    .bind(webhook_id)
    .bind(ok)
    .bind(status)
    .bind(detail)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn flags(pool: &PgPool) -> Result<serde_json::Value, sqlx::Error> {
    let rows: Vec<(String, bool, Option<String>)> =
        sqlx::query_as("SELECT key, enabled, reason FROM feature_flags ORDER BY key")
            .fetch_all(pool)
            .await?;
    Ok(serde_json::json!(rows
        .into_iter()
        .map(|(k, e, r)| serde_json::json!({"key": k, "enabled": e, "reason": r}))
        .collect::<Vec<_>>()))
}

pub async fn upsert_provider_health(
    pool: &PgPool,
    provider: &str,
    ok: bool,
    status: Option<i32>,
    rate_limited: i32,
    latency_ms: Option<i32>,
    detail: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO provider_health (provider, last_ok_at, last_error_at, last_status, error_count, rate_limited, latency_ms, detail)
         VALUES ($1, CASE WHEN $2 THEN now() ELSE NULL END, CASE WHEN $2 THEN NULL ELSE now() END, $3, CASE WHEN $2 THEN 0 ELSE 1 END, $4, $5, $6)
         ON CONFLICT (provider) DO UPDATE SET
            last_ok_at = CASE WHEN EXCLUDED.last_ok_at IS NOT NULL THEN EXCLUDED.last_ok_at ELSE provider_health.last_ok_at END,
            last_error_at = CASE WHEN $2 THEN provider_health.last_error_at ELSE now() END,
            last_status = COALESCE(EXCLUDED.last_status, provider_health.last_status),
            error_count = CASE WHEN $2 THEN 0 ELSE provider_health.error_count + 1 END,
            rate_limited = provider_health.rate_limited + EXCLUDED.rate_limited,
            latency_ms = COALESCE(EXCLUDED.latency_ms, provider_health.latency_ms),
            detail = EXCLUDED.detail",
    )
    .bind(provider)
    .bind(ok)
    .bind(status)
    .bind(rate_limited)
    .bind(latency_ms)
    .bind(detail)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn flag_enabled(pool: &PgPool, key: &str) -> bool {
    sqlx::query_scalar("SELECT enabled FROM feature_flags WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(true)
}

pub async fn set_flag(pool: &PgPool, key: &str, enabled: bool, reason: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO feature_flags (key, enabled, reason, updated_at)
         VALUES ($1, $2, $3, now())
         ON CONFLICT (key) DO UPDATE SET enabled = EXCLUDED.enabled, reason = COALESCE(EXCLUDED.reason, feature_flags.reason), updated_at = now()",
    )
    .bind(key)
    .bind(enabled)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn default_tenant_id(pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    let id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM tenants WHERE slug = 'default' LIMIT 1")
        .fetch_optional(pool)
        .await?;
    if let Some(id) = id {
        return Ok(id);
    }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO tenants (id, slug, name) VALUES ($1, 'default', 'Local operator') ON CONFLICT (slug) DO NOTHING")
        .bind(id)
        .execute(pool)
        .await?;
    let id: Uuid = sqlx::query_scalar("SELECT id FROM tenants WHERE slug = 'default' LIMIT 1")
        .fetch_one(pool)
        .await?;
    Ok(id)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApiKeyRow {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
}

pub async fn insert_api_key(
    pool: &PgPool,
    tenant_id: Uuid,
    name: &str,
    prefix: &str,
    hash: &str,
    role: &str,
) -> Result<ApiKeyRow, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO api_keys (id, tenant_id, prefix, hash, role, name) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(tenant_id)
    .bind(prefix)
    .bind(hash)
    .bind(role)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(ApiKeyRow {
        id: id.to_string(),
        name: name.into(),
        prefix: prefix.into(),
        role: role.into(),
        created_at: Utc::now(),
        revoked: false,
    })
}

pub async fn list_api_keys(pool: &PgPool) -> Result<Vec<ApiKeyRow>, sqlx::Error> {
    let rows: Vec<(Uuid, String, String, String, DateTime<Utc>, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT id, name, prefix, role, created_at, revoked_at FROM api_keys ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, prefix, role, created_at, revoked_at)| ApiKeyRow {
            id: id.to_string(),
            name,
            prefix,
            role,
            created_at,
            revoked: revoked_at.is_some(),
        })
        .collect())
}

pub async fn revoke_api_key(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let n = sqlx::query("UPDATE api_keys SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn lookup_api_key_hash(
    pool: &PgPool,
    prefix: &str,
) -> Result<Option<(String, Uuid, Uuid, String)>, sqlx::Error> {
    let row: Option<(String, Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT hash, id, tenant_id, role FROM api_keys WHERE prefix = $1 AND revoked_at IS NULL LIMIT 1",
    )
    .bind(prefix)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn touch_api_key(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn alert_rule_enabled(pool: &PgPool, kind: &str) -> bool {
    sqlx::query_scalar("SELECT enabled FROM alert_rules WHERE kind = $1")
        .bind(kind)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(true)
}

pub async fn list_alert_rules(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(String, bool, Option<String>)> =
        sqlx::query_as("SELECT kind, enabled, reason FROM alert_rules ORDER BY kind")
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(kind, enabled, reason)| serde_json::json!({"kind": kind, "enabled": enabled, "reason": reason}))
        .collect())
}

pub async fn set_alert_rule(pool: &PgPool, kind: &str, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO alert_rules (kind, enabled, updated_at) VALUES ($1, $2, now())
         ON CONFLICT (kind) DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = now()",
    )
    .bind(kind)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_domain_event_if_new(
    pool: &PgPool,
    ev: &memecoin_os_core::domain_events::DomainEvent,
) -> Result<bool, sqlx::Error> {
    let id: Uuid = ev.event_id.parse().unwrap_or_else(|_| Uuid::new_v4());
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM domain_events WHERE fingerprint = $1)")
            .bind(&ev.fingerprint)
            .fetch_one(pool)
            .await?;
    if exists {
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO domain_events (event_id, event_type, entity_id, chain_id, occurred_at, detected_at, source, confidence, payload, schema_version, fingerprint)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (fingerprint) DO NOTHING",
    )
    .bind(id)
    .bind(ev.event_type.as_str())
    .bind(&ev.entity_id)
    .bind(&ev.chain_id)
    .bind(ev.occurred_at)
    .bind(ev.detected_at)
    .bind(&ev.source)
    .bind(ev.confidence)
    .bind(&ev.payload)
    .bind(ev.schema_version as i32)
    .bind(&ev.fingerprint)
    .execute(pool)
    .await?;
    Ok(true)
}

pub async fn upsert_pool(
    pool: &PgPool,
    token_id: &str,
    p: &memecoin_os_core::models::DiscoveredPool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO discovered_pools (pair_address, chain_id, token_id, dex, liquidity_usd, price_usd)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (chain_id, pair_address) DO UPDATE SET
            token_id = EXCLUDED.token_id,
            dex = COALESCE(EXCLUDED.dex, discovered_pools.dex),
            liquidity_usd = EXCLUDED.liquidity_usd,
            price_usd = EXCLUDED.price_usd,
            last_seen = now()",
    )
    .bind(&p.pair_address)
    .bind(&p.chain_id)
    .bind(token_id)
    .bind(&p.dex)
    .bind(p.liquidity_usd)
    .bind(p.price_usd)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_pools(
    pool: &PgPool,
    token_id: &str,
) -> Result<Vec<memecoin_os_core::models::DiscoveredPool>, sqlx::Error> {
    let rows: Vec<(String, String, Option<String>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT pair_address, chain_id, dex, liquidity_usd, price_usd FROM discovered_pools WHERE token_id = $1 ORDER BY liquidity_usd DESC NULLS LAST",
    )
    .bind(token_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(pair_address, chain_id, dex, liquidity_usd, price_usd)| {
            memecoin_os_core::models::DiscoveredPool {
                pair_address,
                chain_id,
                dex,
                liquidity_usd,
                price_usd,
            }
        })
        .collect())
}

pub async fn upsert_wallet_profile(
    pool: &PgPool,
    p: &memecoin_os_core::wallet::WalletProfile,
) -> Result<(), sqlx::Error> {
    let Some(token_id) = &p.token_id else {
        return Ok(());
    };
    sqlx::query(
        "INSERT INTO wallets (address, chain_id) VALUES ($1, $2)
         ON CONFLICT (chain_id, address) DO UPDATE SET last_seen = now()",
    )
    .bind(&p.address)
    .bind(&p.chain_id)
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO wallet_profiles (address, chain_id, token_id, classification, confidence, share_pct, evidence)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (chain_id, address, token_id) DO UPDATE SET
            classification = EXCLUDED.classification,
            confidence = EXCLUDED.confidence,
            share_pct = EXCLUDED.share_pct,
            evidence = EXCLUDED.evidence,
            updated_at = now()",
    )
    .bind(&p.address)
    .bind(&p.chain_id)
    .bind(token_id)
    .bind(format!("{:?}", p.classification).to_lowercase())
    .bind(p.confidence)
    .bind(p.share_pct)
    .bind(serde_json::to_value(&p.evidence).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_wallet_profiles(pool: &PgPool, token_id: &str) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(String, String, String, f64, Option<f64>, serde_json::Value)> = sqlx::query_as(
        "SELECT address, chain_id, classification, confidence, share_pct, evidence
         FROM wallet_profiles WHERE token_id = $1 ORDER BY share_pct DESC NULLS LAST",
    )
    .bind(token_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(address, chain_id, classification, confidence, share_pct, evidence)| {
            serde_json::json!({
                "address": address, "chain_id": chain_id, "classification": classification,
                "confidence": confidence, "share_pct": share_pct, "evidence": evidence
            })
        })
        .collect())
}

pub async fn upsert_baseline(
    pool: &PgPool,
    token_id: &str,
    b: &memecoin_os_core::baseline::MetricBaseline,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO metric_baselines (token_id, metric, \"window\", n, mean, stddev, last_value, z_score, mad, percentile_25, percentile_75, data_state, evidence, as_of)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now())
         ON CONFLICT (token_id, metric, \"window\") DO UPDATE SET
            n = EXCLUDED.n, mean = EXCLUDED.mean, stddev = EXCLUDED.stddev,
            last_value = EXCLUDED.last_value, z_score = EXCLUDED.z_score,
            mad = EXCLUDED.mad, percentile_25 = EXCLUDED.percentile_25, percentile_75 = EXCLUDED.percentile_75,
            data_state = EXCLUDED.data_state, evidence = EXCLUDED.evidence, as_of = now()",
    )
    .bind(token_id)
    .bind(&b.metric)
    .bind(&b.window)
    .bind(b.n as i32)
    .bind(b.mean)
    .bind(b.stddev)
    .bind(b.last)
    .bind(b.z_score)
    .bind(b.mad)
    .bind(b.percentile_25)
    .bind(b.percentile_75)
    .bind(b.data_state.as_str())
    .bind(serde_json::to_value(&b.evidence).unwrap_or(serde_json::Value::Null))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_baselines(
    pool: &PgPool,
    token_id: &str,
) -> Result<Vec<memecoin_os_core::baseline::MetricBaseline>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        i32,
        f64,
        f64,
        f64,
        Option<f64>,
        f64,
        Option<f64>,
        Option<f64>,
        String,
        serde_json::Value,
    )> = sqlx::query_as(
        "SELECT metric, \"window\", n, mean, stddev, last_value, z_score, mad, percentile_25, percentile_75, data_state, evidence
         FROM metric_baselines WHERE token_id = $1 ORDER BY metric, \"window\"",
    )
    .bind(token_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(metric, window, n, mean, stddev, last, z_score, mad, p25, p75, data_state, evidence)| {
            memecoin_os_core::baseline::MetricBaseline {
                metric,
                window,
                n: n as u32,
                mean,
                stddev,
                last,
                z_score,
                mad,
                percentile_25: p25,
                percentile_75: p75,
                data_state: memecoin_os_core::models::DataState::parse(&data_state),
                evidence: serde_json::from_value(evidence).unwrap_or_default(),
            }
        })
        .collect())
}

pub async fn replace_claims(
    pool: &PgPool,
    token_id: &str,
    claims: &[memecoin_os_core::claims::EvidenceClaim],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM evidence_claims WHERE token_id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    for c in claims {
        sqlx::query(
            "INSERT INTO evidence_claims (token_id, claim_id, claim, relation, evidence, data_state, confidence, as_of)
             VALUES ($1, $2, $3, $4, $5, $6, $7, now())",
        )
        .bind(token_id)
        .bind(&c.id)
        .bind(&c.claim)
        .bind(c.relation.as_str())
        .bind(serde_json::to_value(&c.evidence).unwrap_or(serde_json::Value::Null))
        .bind(c.data_state.as_str())
        .bind(c.confidence)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn list_claims(
    pool: &PgPool,
    token_id: &str,
) -> Result<Vec<memecoin_os_core::claims::EvidenceClaim>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        String,
        serde_json::Value,
        String,
        f64,
        DateTime<Utc>,
    )> = sqlx::query_as(
        "SELECT claim_id, claim, relation, evidence, data_state, confidence, as_of
         FROM evidence_claims WHERE token_id = $1 ORDER BY claim_id",
    )
    .bind(token_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, claim, relation, evidence, data_state, confidence, as_of)| {
            memecoin_os_core::claims::EvidenceClaim {
                id,
                claim,
                relation: memecoin_os_core::claims::ClaimRelation::parse(&relation),
                evidence: serde_json::from_value(evidence).unwrap_or_default(),
                data_state: memecoin_os_core::models::DataState::parse(&data_state),
                confidence,
                stored_at: Some(as_of),
            }
        })
        .collect())
}

pub async fn list_domain_events(
    pool: &PgPool,
    entity_id: &str,
    limit: i64,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(
        Uuid,
        String,
        String,
        Option<String>,
        DateTime<Utc>,
        String,
        f64,
        serde_json::Value,
        String,
    )> = sqlx::query_as(
        "SELECT event_id, event_type, entity_id, chain_id, occurred_at, source, confidence, payload, fingerprint
         FROM domain_events WHERE entity_id = $1 ORDER BY occurred_at DESC LIMIT $2",
    )
    .bind(entity_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(event_id, event_type, entity_id, chain_id, occurred_at, source, confidence, payload, fingerprint)| {
            serde_json::json!({
                "event_id": event_id, "event_type": event_type, "entity_id": entity_id,
                "chain_id": chain_id, "occurred_at": occurred_at, "source": source,
                "confidence": confidence, "payload": payload, "fingerprint": fingerprint
            })
        })
        .collect())
}

pub async fn insert_transfer(
    pool: &PgPool,
    token_id: &str,
    t: &memecoin_os_core::indexer::IndexedTransfer,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO transfers (chain_id, token_id, tx_hash, block_number, from_address, to_address, amount_raw)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (chain_id, tx_hash, from_address, to_address, amount_raw) DO NOTHING",
    )
    .bind(&t.chain_id)
    .bind(token_id)
    .bind(&t.tx_hash)
    .bind(t.block_number as i64)
    .bind(&t.from_address)
    .bind(&t.to_address)
    .bind(&t.amount_raw)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn list_transfers(pool: &PgPool, token_id: &str, limit: i64) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(String, i64, String, String, String, String)> = sqlx::query_as(
        "SELECT chain_id, block_number, tx_hash, from_address, to_address, amount_raw
         FROM transfers WHERE token_id = $1 ORDER BY block_number DESC LIMIT $2",
    )
    .bind(token_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(chain_id, block_number, tx_hash, from_address, to_address, amount_raw)| {
            serde_json::json!({
                "chain_id": chain_id, "block_number": block_number, "tx_hash": tx_hash,
                "from": from_address, "to": to_address, "amount_raw": amount_raw
            })
        })
        .collect())
}

#[derive(Debug, Default)]
pub struct TransferStats {
    pub transfers_24h: u64,
    pub unique_senders_24h: u64,
    pub cex_in_count: u64,
    pub cex_out_count: u64,
    pub cex_in_usd: f64,
    pub cex_out_usd: f64,
    pub large_transfers_24h: u64,
}

pub async fn transfer_stats_24h(pool: &PgPool, token_id: &str) -> Result<TransferStats, sqlx::Error> {
    let row: Option<(i64, i64)> = sqlx::query_as(
        "SELECT COUNT(*)::bigint, COUNT(DISTINCT from_address)::bigint
         FROM transfers WHERE token_id = $1 AND ingested_at > now() - interval '24 hours'",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await?;
    let (n, senders) = row.unwrap_or((0, 0));
    let cex: Option<(i64, i64, f64, f64, i64)> = sqlx::query_as(
        "SELECT
            COUNT(*) FILTER (WHERE direction = 'cex_in')::bigint,
            COUNT(*) FILTER (WHERE direction = 'cex_out')::bigint,
            COALESCE(SUM(amount_usd) FILTER (WHERE direction = 'cex_in'), 0),
            COALESCE(SUM(amount_usd) FILTER (WHERE direction = 'cex_out'), 0),
            COUNT(*) FILTER (WHERE payload ? 'kind')::bigint
         FROM whale_movements
         WHERE token_id = $1 AND occurred_at > now() - interval '24 hours'",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await?;
    let (cin, cout, cin_usd, cout_usd, large) = cex.unwrap_or((0, 0, 0.0, 0.0, 0));
    Ok(TransferStats {
        transfers_24h: n.max(0) as u64,
        unique_senders_24h: senders.max(0) as u64,
        cex_in_count: cin.max(0) as u64,
        cex_out_count: cout.max(0) as u64,
        cex_in_usd: cin_usd,
        cex_out_usd: cout_usd,
        large_transfers_24h: large.max(0) as u64,
    })
}

pub async fn insert_whale_movement(
    pool: &PgPool,
    token_id: &str,
    ev: &memecoin_os_core::whale::WhaleEvent,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO whale_movements (token_id, occurred_at, chain_id, tx_hash, direction, amount_usd, wallet, exchange, payload)
         VALUES ($1, now(), $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (token_id, tx_hash, direction, wallet) DO NOTHING",
    )
    .bind(token_id)
    .bind(&ev.chain_id)
    .bind(&ev.tx_hash)
    .bind(&ev.direction)
    .bind(ev.amount_usd)
    .bind(&ev.wallet)
    .bind(&ev.exchange)
    .bind(serde_json::to_value(ev).unwrap_or(serde_json::json!({})))
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn list_whale_movements(pool: &PgPool, token_id: &str, limit: i64) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(DateTime<Utc>, String, Option<String>, String, Option<f64>, Option<String>, Option<String>, serde_json::Value)> = sqlx::query_as(
        "SELECT occurred_at, chain_id, tx_hash, direction, amount_usd, wallet, exchange, payload
         FROM whale_movements WHERE token_id = $1 ORDER BY occurred_at DESC LIMIT $2",
    )
    .bind(token_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(occurred_at, chain_id, tx_hash, direction, amount_usd, wallet, exchange, payload)| {
            serde_json::json!({
                "occurred_at": occurred_at, "chain_id": chain_id, "tx_hash": tx_hash,
                "direction": direction, "amount_usd": amount_usd, "wallet": wallet,
                "exchange": exchange, "payload": payload
            })
        })
        .collect())
}

pub async fn upsert_graph_edge(
    pool: &PgPool,
    token_id: &str,
    chain_id: &str,
    from: &str,
    to: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO graph_edges (chain_id, token_id, from_address, to_address, relation, evidence, confidence, source)
         VALUES ($1, $2, $3, $4, 'transferred_to', '[]'::jsonb, 0.4, 'indexer')
         ON CONFLICT (chain_id, token_id, from_address, to_address, relation) DO UPDATE SET
            tx_count = graph_edges.tx_count + 1,
            last_seen = now()",
    )
    .bind(chain_id)
    .bind(token_id)
    .bind(from)
    .bind(to)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_graph_edges(pool: &PgPool, token_id: &str) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT from_address, to_address FROM graph_edges WHERE token_id = $1 AND relation = 'transferred_to'",
    )
    .bind(token_id)
    .fetch_all(pool)
    .await
}

pub async fn replace_clusters(
    pool: &PgPool,
    token_id: &str,
    chain_id: &str,
    clusters: &[memecoin_os_core::cluster::WalletCluster],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM wallet_clusters WHERE token_id = $1 AND chain_id = $2")
        .bind(token_id)
        .bind(chain_id)
        .execute(pool)
        .await?;
    for c in clusters {
        sqlx::query(
            "INSERT INTO wallet_clusters (cluster_id, token_id, chain_id, members, edge_count, confidence, evidence)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&c.cluster_id)
        .bind(token_id)
        .bind(chain_id)
        .bind(serde_json::to_value(&c.members).unwrap_or(serde_json::json!([])))
        .bind(c.edge_count as i32)
        .bind(c.confidence)
        .bind(serde_json::to_value(&c.evidence).unwrap_or(serde_json::json!([])))
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn list_clusters(pool: &PgPool, token_id: &str) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(String, String, serde_json::Value, i32, f64, serde_json::Value)> = sqlx::query_as(
        "SELECT cluster_id, chain_id, members, edge_count, confidence, evidence
         FROM wallet_clusters WHERE token_id = $1 ORDER BY jsonb_array_length(members) DESC",
    )
    .bind(token_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(cluster_id, chain_id, members, edge_count, confidence, evidence)| {
            serde_json::json!({
                "cluster_id": cluster_id, "chain_id": chain_id, "members": members,
                "edge_count": edge_count, "confidence": confidence, "evidence": evidence,
                "disclaimer": "shared transfer path is not identity"
            })
        })
        .collect())
}

pub async fn indexer_cursor(pool: &PgPool, chain_id: &str, token_id: &str) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT last_block FROM indexer_cursors WHERE chain_id = $1 AND token_id = $2")
        .bind(chain_id)
        .bind(token_id)
        .fetch_optional(pool)
        .await
}

pub async fn upsert_indexer_cursor(
    pool: &PgPool,
    chain_id: &str,
    token_id: &str,
    last_block: i64,
    source: &str,
    last_error: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO indexer_cursors (chain_id, token_id, last_block, source, last_error, last_run)
         VALUES ($1, $2, $3, $4, $5, now())
         ON CONFLICT (chain_id, token_id) DO UPDATE SET
            last_block = EXCLUDED.last_block,
            source = EXCLUDED.source,
            last_error = EXCLUDED.last_error,
            last_run = now()",
    )
    .bind(chain_id)
    .bind(token_id)
    .bind(last_block)
    .bind(source)
    .bind(last_error)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_indexer_cursors(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(String, String, i64, DateTime<Utc>, Option<String>, String)> = sqlx::query_as(
        "SELECT chain_id, token_id, last_block, last_run, last_error, source FROM indexer_cursors",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(chain_id, token_id, last_block, last_run, last_error, source)| {
            serde_json::json!({
                "chain_id": chain_id, "token_id": token_id, "last_block": last_block,
                "last_run": last_run, "last_error": last_error, "source": source
            })
        })
        .collect())
}

pub async fn upsert_discovery(
    pool: &PgPool,
    a: &memecoin_os_core::discovery::DiscoveryAssessment,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO discovered_tokens (chain_id, address, symbol, name, pair_address, dex, liquidity_usd, via_token, status, score, evidence, auto_verified)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,false)
         ON CONFLICT (chain_id, address) DO UPDATE SET
            symbol = COALESCE(EXCLUDED.symbol, discovered_tokens.symbol),
            name = COALESCE(EXCLUDED.name, discovered_tokens.name),
            pair_address = COALESCE(EXCLUDED.pair_address, discovered_tokens.pair_address),
            dex = COALESCE(EXCLUDED.dex, discovered_tokens.dex),
            liquidity_usd = EXCLUDED.liquidity_usd,
            via_token = EXCLUDED.via_token,
            status = EXCLUDED.status,
            score = EXCLUDED.score,
            evidence = EXCLUDED.evidence,
            last_seen = now()",
    )
    .bind(&a.chain_id)
    .bind(&a.address)
    .bind(&a.symbol)
    .bind(&a.name)
    .bind(&a.pair_address)
    .bind(&a.dex)
    .bind(a.liquidity_usd)
    .bind(&a.via_token)
    .bind(a.status.as_str())
    .bind(a.score)
    .bind(serde_json::to_value(&a.evidence).unwrap_or(serde_json::json!([])))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_discovery(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(String, String, Option<String>, Option<String>, String, f64, bool, serde_json::Value, Option<f64>)> = sqlx::query_as(
        "SELECT chain_id, address, symbol, name, status, score, auto_verified, evidence, liquidity_usd
         FROM discovered_tokens
         ORDER BY score DESC, last_seen DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(chain_id, address, symbol, name, status, score, auto_verified, evidence, liquidity_usd)| {
            serde_json::json!({
                "chain_id": chain_id, "address": address, "symbol": symbol, "name": name,
                "status": status, "score": score, "auto_verified": auto_verified,
                "liquidity_usd": liquidity_usd, "evidence": evidence
            })
        })
        .collect())
}

pub async fn upsert_verification(
    pool: &PgPool,
    r: &memecoin_os_core::verification::VerificationReport,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO verification_reports (token_id, level, declared_level, reasons, evidence, last_verified_at, disclaimer)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         ON CONFLICT (token_id) DO UPDATE SET
            level = EXCLUDED.level, declared_level = EXCLUDED.declared_level,
            reasons = EXCLUDED.reasons, evidence = EXCLUDED.evidence,
            last_verified_at = EXCLUDED.last_verified_at, disclaimer = EXCLUDED.disclaimer",
    )
    .bind(&r.token_id)
    .bind(r.level.as_str())
    .bind(r.declared_level.as_str())
    .bind(serde_json::to_value(&r.reasons).unwrap_or(serde_json::json!([])))
    .bind(serde_json::to_value(&r.evidence).unwrap_or(serde_json::json!([])))
    .bind(r.last_verified_at)
    .bind(&r.disclaimer)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_verification(pool: &PgPool, token_id: &str) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let row: Option<(String, String, String, serde_json::Value, serde_json::Value, DateTime<Utc>, String)> = sqlx::query_as(
        "SELECT token_id, level, declared_level, reasons, evidence, last_verified_at, disclaimer
         FROM verification_reports WHERE token_id = $1",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(token_id, level, declared_level, reasons, evidence, last_verified_at, disclaimer)| {
        serde_json::json!({
            "token_id": token_id, "level": level, "declared_level": declared_level,
            "reasons": reasons, "evidence": evidence, "last_verified_at": last_verified_at,
            "disclaimer": disclaimer
        })
    }))
}

#[derive(Debug, Clone)]
pub struct ProjectProfile {
    pub token_id: String,
    pub claim_status: String,
    pub claimant_label: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub discord: Option<String>,
    pub github: Option<String>,
    pub docs: Option<String>,
    pub roadmap: Option<String>,
}

fn map_profile(
    token_id: String,
    claim_status: String,
    claimant_label: Option<String>,
    claimed_at: Option<DateTime<Utc>>,
    website: Option<String>,
    twitter: Option<String>,
    telegram: Option<String>,
    discord: Option<String>,
    github: Option<String>,
    docs: Option<String>,
    roadmap: Option<String>,
) -> ProjectProfile {
    ProjectProfile {
        token_id,
        claim_status,
        claimant_label,
        claimed_at,
        website,
        twitter,
        telegram,
        discord,
        github,
        docs,
        roadmap,
    }
}

pub async fn get_project_profile(pool: &PgPool, token_id: &str) -> Result<Option<ProjectProfile>, sqlx::Error> {
    let row: Option<(
        String,
        String,
        Option<String>,
        Option<DateTime<Utc>>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT token_id, claim_status, claimant_label, claimed_at, website, twitter, telegram, discord, github, docs, roadmap
         FROM project_profiles WHERE token_id = $1",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(a, b, c, d, e, f, g, h, i, j, k)| map_profile(a, b, c, d, e, f, g, h, i, j, k)))
}

pub async fn list_project_claim_status(pool: &PgPool) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as("SELECT token_id, claim_status FROM project_profiles")
        .fetch_all(pool)
        .await
}

pub async fn upsert_project_claim(
    pool: &PgPool,
    token_id: &str,
    status: &str,
    claimant_label: Option<&str>,
) -> Result<ProjectProfile, sqlx::Error> {
    sqlx::query(
        "INSERT INTO project_profiles (token_id, claim_status, claimant_label, claimed_at, updated_at)
         VALUES ($1, $2, $3, CASE WHEN $2 = 'claimed' THEN now() ELSE NULL END, now())
         ON CONFLICT (token_id) DO UPDATE SET
            claim_status = EXCLUDED.claim_status,
            claimant_label = EXCLUDED.claimant_label,
            claimed_at = CASE
                WHEN EXCLUDED.claim_status = 'claimed' THEN COALESCE(project_profiles.claimed_at, now())
                ELSE project_profiles.claimed_at
            END,
            updated_at = now()",
    )
    .bind(token_id)
    .bind(status)
    .bind(claimant_label)
    .execute(pool)
    .await?;
    Ok(get_project_profile(pool, token_id)
        .await?
        .expect("project profile after claim"))
}

pub async fn upsert_project_links(
    pool: &PgPool,
    token_id: &str,
    website: Option<&str>,
    twitter: Option<&str>,
    telegram: Option<&str>,
    discord: Option<&str>,
    github: Option<&str>,
    docs: Option<&str>,
    roadmap: Option<&str>,
) -> Result<ProjectProfile, sqlx::Error> {
    sqlx::query(
        "INSERT INTO project_profiles (token_id, website, twitter, telegram, discord, github, docs, roadmap, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
         ON CONFLICT (token_id) DO UPDATE SET
            website = EXCLUDED.website,
            twitter = EXCLUDED.twitter,
            telegram = EXCLUDED.telegram,
            discord = EXCLUDED.discord,
            github = EXCLUDED.github,
            docs = EXCLUDED.docs,
            roadmap = EXCLUDED.roadmap,
            updated_at = now()",
    )
    .bind(token_id)
    .bind(website)
    .bind(twitter)
    .bind(telegram)
    .bind(discord)
    .bind(github)
    .bind(docs)
    .bind(roadmap)
    .execute(pool)
    .await?;
    Ok(get_project_profile(pool, token_id)
        .await?
        .expect("project profile after links"))
}

pub async fn replace_narratives(
    pool: &PgPool,
    rows: &[memecoin_os_core::narrative::Narrative],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM narratives").execute(pool).await?;
    for n in rows {
        sqlx::query(
            "INSERT INTO narratives (id, label, state, confidence, token_ids, evidence, mention_velocity, unique_accounts)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(&n.id)
        .bind(&n.label)
        .bind(format!("{:?}", n.state).to_lowercase())
        .bind(n.confidence)
        .bind(serde_json::to_value(&n.token_ids).unwrap_or(serde_json::json!([])))
        .bind(serde_json::to_value(&n.evidence).unwrap_or(serde_json::json!([])))
        .bind(n.mention_velocity.as_str())
        .bind(n.unique_accounts.as_str())
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn list_narratives(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(String, String, String, f64, serde_json::Value, serde_json::Value, String, String)> = sqlx::query_as(
        "SELECT id, label, state, confidence, token_ids, evidence, mention_velocity, unique_accounts FROM narratives ORDER BY jsonb_array_length(token_ids) DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, label, state, confidence, token_ids, evidence, mention_velocity, unique_accounts)| {
            serde_json::json!({
                "id": id, "label": label, "state": state, "confidence": confidence,
                "token_ids": token_ids, "evidence": evidence,
                "mention_velocity": mention_velocity, "unique_accounts": unique_accounts
            })
        })
        .collect())
}

pub async fn replace_genome_clusters(
    pool: &PgPool,
    rows: &[memecoin_os_core::genome_cluster::GenomeCluster],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM genome_clusters").execute(pool).await?;
    for c in rows {
        sqlx::query(
            "INSERT INTO genome_clusters (cluster_id, label, token_ids, confidence, evidence)
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(&c.cluster_id)
        .bind(&c.label)
        .bind(serde_json::to_value(&c.token_ids).unwrap_or(serde_json::json!([])))
        .bind(c.confidence)
        .bind(serde_json::to_value(&c.evidence).unwrap_or(serde_json::json!([])))
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn list_genome_clusters(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(String, String, serde_json::Value, f64, serde_json::Value)> = sqlx::query_as(
        "SELECT cluster_id, label, token_ids, confidence, evidence FROM genome_clusters ORDER BY jsonb_array_length(token_ids) DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(cluster_id, label, token_ids, confidence, evidence)| {
            serde_json::json!({
                "cluster_id": cluster_id, "label": label, "token_ids": token_ids,
                "confidence": confidence, "evidence": evidence
            })
        })
        .collect())
}

pub async fn insert_research_report(
    pool: &PgPool,
    token_id: Option<&str>,
    question: &str,
    report: &memecoin_os_core::research::ResearchReport,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO research_reports (token_id, question, markdown, payload, confidence, model_id, generated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id",
    )
    .bind(token_id)
    .bind(question)
    .bind(&report.markdown)
    .bind(serde_json::to_value(report).unwrap_or(serde_json::json!({})))
    .bind(report.confidence)
    .bind(&report.model_id)
    .bind(report.generated_at)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn latest_research(pool: &PgPool, token_id: &str) -> Result<Option<(String, serde_json::Value)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT markdown, payload FROM research_reports WHERE token_id = $1 ORDER BY generated_at DESC LIMIT 1",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone)]
pub struct WorkJob {
    pub id: Uuid,
    pub kind: String,
    pub payload: serde_json::Value,
}

pub async fn enqueue_work(
    pool: &PgPool,
    tenant_id: Uuid,
    kind: &str,
    payload: &serde_json::Value,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO work_queue (id, tenant_id, kind, payload) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(tenant_id)
    .bind(kind)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn claim_work(pool: &PgPool) -> Result<Option<WorkJob>, sqlx::Error> {
    let row: Option<(Uuid, String, serde_json::Value)> = sqlx::query_as(
        "UPDATE work_queue SET locked_until = now() + interval '30 seconds', attempts = attempts + 1
         WHERE id = (
            SELECT id FROM work_queue
            WHERE done_at IS NULL
              AND (locked_until IS NULL OR locked_until < now())
              AND available_at <= now()
            ORDER BY available_at
            FOR UPDATE SKIP LOCKED
            LIMIT 1
         )
         RETURNING id, kind, payload",
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id, kind, payload)| WorkJob { id, kind, payload }))
}

pub async fn complete_work(pool: &PgPool, id: Uuid, err: Option<&str>) -> Result<(), sqlx::Error> {
    if err.is_some() {
        sqlx::query("UPDATE work_queue SET last_error = $2, locked_until = now() + interval '15 seconds' WHERE id = $1")
            .bind(id)
            .bind(err)
            .execute(pool)
            .await?;
    } else {
        sqlx::query("UPDATE work_queue SET done_at = now(), last_error = NULL WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn queue_stats(pool: &PgPool) -> Result<serde_json::Value, sqlx::Error> {
    let row: (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*) FILTER (WHERE done_at IS NULL)::bigint, COUNT(*) FILTER (WHERE done_at IS NULL AND locked_until > now())::bigint FROM work_queue",
    )
    .fetch_one(pool)
    .await?;
    Ok(serde_json::json!({"pending": row.0, "locked": row.1}))
}

pub async fn increment_usage(pool: &PgPool, tenant_id: Uuid, metric: &str) -> Result<i64, sqlx::Error> {
    let used: i64 = sqlx::query_scalar(
        "INSERT INTO usage_meters (tenant_id, metric, window_start, used)
         VALUES ($1, $2, CURRENT_DATE, 1)
         ON CONFLICT (tenant_id, metric, window_start) DO UPDATE SET used = usage_meters.used + 1
         RETURNING used",
    )
    .bind(tenant_id)
    .bind(metric)
    .fetch_one(pool)
    .await?;
    Ok(used)
}

pub async fn usage_today(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<(String, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT metric, used FROM usage_meters WHERE tenant_id = $1 AND window_start = CURRENT_DATE",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
}

pub async fn tenant_plan(pool: &PgPool, tenant_id: Uuid) -> Result<String, sqlx::Error> {
    let p: Option<String> = sqlx::query_scalar("SELECT plan FROM tenants WHERE id = $1")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?;
    Ok(p.unwrap_or_else(|| "free".into()))
}

pub async fn tenant_branding(pool: &PgPool, tenant_id: Uuid) -> Result<serde_json::Value, sqlx::Error> {
    let row: Option<serde_json::Value> = sqlx::query_scalar("SELECT branding FROM tenants WHERE id = $1")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.unwrap_or_else(|| serde_json::json!({})))
}

pub async fn set_tenant_branding(
    pool: &PgPool,
    tenant_id: Uuid,
    branding: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tenants SET branding = $2 WHERE id = $1")
        .bind(tenant_id)
        .bind(branding)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_watchlist(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT token_id FROM tenant_watchlist WHERE tenant_id = $1 ORDER BY token_id")
        .bind(tenant_id)
        .fetch_all(pool)
        .await
}

pub async fn replace_watchlist(
    pool: &PgPool,
    tenant_id: Uuid,
    token_ids: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM tenant_watchlist WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await?;
    for id in token_ids {
        sqlx::query("INSERT INTO tenant_watchlist (tenant_id, token_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(tenant_id)
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn set_tenant_plan(pool: &PgPool, tenant_id: Uuid, plan: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tenants SET plan = $2 WHERE id = $1")
        .bind(tenant_id)
        .bind(plan)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_tenants(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(Uuid, String, String, String, String, serde_json::Value)> = sqlx::query_as(
        "SELECT id, slug, name, plan, status, branding FROM tenants ORDER BY slug",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, slug, name, plan, status, branding)| {
            serde_json::json!({"id": id, "slug": slug, "name": name, "plan": plan, "status": status, "branding": branding})
        })
        .collect())
}

pub async fn insert_tenant(pool: &PgPool, slug: &str, name: &str, plan: &str) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO tenants (id, slug, name, plan) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(slug)
        .bind(name)
        .bind(plan)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn upsert_user(
    pool: &PgPool,
    tenant_id: Uuid,
    subject: &str,
    email: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (tenant_id, subject, email, role, last_login)
         VALUES ($1, $2, $3, 'viewer', now())
         ON CONFLICT (tenant_id, subject) DO UPDATE SET email = EXCLUDED.email, last_login = now()
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(subject)
    .bind(email)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn insert_session(pool: &PgPool, user_id: Uuid, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, now() + interval '12 hours')",
    )
    .bind(user_id)
    .bind(token_hash)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_oidc_state(pool: &PgPool, state: &str, nonce: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO oidc_states (state, nonce, expires_at) VALUES ($1, $2, now() + interval '10 minutes')")
        .bind(state)
        .bind(nonce)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn take_oidc_state(pool: &PgPool, state: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("DELETE FROM oidc_states WHERE state = $1 AND expires_at > now() RETURNING nonce")
        .bind(state)
        .fetch_optional(pool)
        .await
}

pub async fn upsert_object_blob(pool: &PgPool, hash: &str, provider: &str, bytes: i32, path: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO object_blobs (hash, provider, bytes, path) VALUES ($1, $2, $3, $4) ON CONFLICT (hash) DO NOTHING",
    )
    .bind(hash)
    .bind(provider)
    .bind(bytes)
    .bind(path)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn object_blob_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*)::bigint FROM object_blobs")
        .fetch_one(pool)
        .await
}

pub async fn count_old_snapshots(pool: &PgPool, days: i32) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM market_snapshots WHERE as_of < now() - make_interval(days => $1)",
    )
    .bind(days)
    .fetch_one(pool)
    .await
}

pub async fn prune_old_snapshots(pool: &PgPool, days: i32) -> Result<u64, sqlx::Error> {
    let n = sqlx::query("DELETE FROM market_snapshots WHERE as_of < now() - make_interval(days => $1)")
        .bind(days)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n)
}

pub async fn insert_audit(
    pool: &PgPool,
    actor: &str,
    action: &str,
    resource: &str,
    payload: serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO audit_events (actor, action, resource, payload) VALUES ($1, $2, $3, $4)",
    )
    .bind(actor)
    .bind(action)
    .bind(resource)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn ack_alert(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let n = sqlx::query("UPDATE alerts SET acknowledged_at = now() WHERE id = $1 AND acknowledged_at IS NULL")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn upsert_ecosystem(pool: &PgPool, e: &memecoin_os_core::ecosystem::Ecosystem) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO ecosystems (id, slug, name, description, primary_chain, status, lifecycle_phase,
            discovery_score, intelligence_score, risk_score, confidence, verification_status, auto_verified, last_updated)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,false,now())
         ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            lifecycle_phase = EXCLUDED.lifecycle_phase,
            discovery_score = EXCLUDED.discovery_score,
            intelligence_score = EXCLUDED.intelligence_score,
            risk_score = EXCLUDED.risk_score,
            confidence = EXCLUDED.confidence,
            verification_status = EXCLUDED.verification_status,
            last_updated = now()",
    )
    .bind(&e.id)
    .bind(&e.slug)
    .bind(&e.name)
    .bind(&e.description)
    .bind(&e.primary_chain)
    .bind(&e.status)
    .bind(e.lifecycle_phase.as_str())
    .bind(e.discovery_score)
    .bind(e.intelligence_score)
    .bind(e.risk_score)
    .bind(e.confidence)
    .bind(&e.verification_status)
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO ecosystem_members (ecosystem_id, entity_kind, entity_id) VALUES ($1, 'token', $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(&e.id)
    .bind(&e.id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_twin_state(pool: &PgPool, ecosystem_id: &str, payload: &serde_json::Value) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO twin_states (ecosystem_id, as_of, payload) VALUES ($1, now(), $2)
         ON CONFLICT (ecosystem_id, as_of) DO UPDATE SET payload = EXCLUDED.payload",
    )
    .bind(ecosystem_id)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn replace_relationships(
    pool: &PgPool,
    ecosystem_id: &str,
    edges: &[memecoin_os_core::twin_graph::GraphEdge],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE entity_relationships SET valid_to = now()
         WHERE ecosystem_id = $1 AND valid_to IS NULL",
    )
    .bind(ecosystem_id)
    .execute(pool)
    .await?;
    for e in edges {
        sqlx::query(
            "INSERT INTO entity_relationships (ecosystem_id, source_entity, target_entity, relationship_type, confidence, evidence, status, valid_from, evidence_ids)
             VALUES ($1,$2,$3,$4,$5,$6,$7, now(), $8)",
        )
        .bind(ecosystem_id)
        .bind(&e.source)
        .bind(&e.target)
        .bind(&e.relationship_type)
        .bind(e.confidence)
        .bind(serde_json::json!(e.evidence))
        .bind(&e.status)
        .bind(serde_json::json!(e.evidence))
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn relationships_at(
    pool: &PgPool,
    ecosystem_id: &str,
    at: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<memecoin_os_core::twin_graph::GraphEdge>, sqlx::Error> {
    let rows: Vec<(String, String, String, f64, serde_json::Value, String, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as(
            "SELECT source_entity, target_entity, relationship_type, confidence, evidence, status, valid_from, valid_to
             FROM entity_relationships
             WHERE ecosystem_id = $1 AND valid_from <= $2 AND (valid_to IS NULL OR valid_to > $2)",
        )
        .bind(ecosystem_id)
        .bind(at)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(source, target, relationship_type, confidence, evidence, status, valid_from, valid_to)| {
            memecoin_os_core::twin_graph::GraphEdge {
                source,
                target,
                relationship_type,
                confidence,
                evidence: evidence
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                status,
                valid_from,
                valid_to,
            }
        })
        .collect())
}

pub async fn insert_simulation(pool: &PgPool, ecosystem_id: &str, assumptions: &serde_json::Value, result: &serde_json::Value) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO twin_simulations (id, ecosystem_id, assumptions, result) VALUES ($1,$2,$3,$4)")
        .bind(id)
        .bind(ecosystem_id)
        .bind(assumptions)
        .bind(result)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn get_simulation(pool: &PgPool, id: Uuid) -> Result<Option<serde_json::Value>, sqlx::Error> {
    sqlx::query_scalar("SELECT result FROM twin_simulations WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub struct ConnectState {
    pub kind: String,
    pub token_id: Option<String>,
}

pub async fn insert_connect_state(
    pool: &PgPool,
    state: &str,
    kind: &str,
    token_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM connect_states WHERE created_at < now() - interval '30 minutes'")
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO connect_states (state, kind, token_id) VALUES ($1, $2, $3)")
        .bind(state)
        .bind(kind)
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn take_connect_state(pool: &PgPool, state: &str) -> Result<Option<ConnectState>, sqlx::Error> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "DELETE FROM connect_states WHERE state = $1 RETURNING kind, token_id",
    )
    .bind(state)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(kind, token_id)| ConnectState { kind, token_id }))
}

pub async fn insert_community_session(pool: &PgPool, handle: &str) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO community_sessions (id, handle) VALUES ($1, $2)")
        .bind(id)
        .bind(handle)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn community_session_handle(pool: &PgPool, id: Uuid) -> Result<Option<String>, sqlx::Error> {
    sqlx::query("UPDATE community_sessions SET last_seen = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query_scalar("SELECT handle FROM community_sessions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_community_message(
    pool: &PgPool,
    room_id: &str,
    session_id: Uuid,
    handle: &str,
    body: &str,
) -> Result<serde_json::Value, sqlx::Error> {
    let id = Uuid::new_v4();
    let at = Utc::now();
    sqlx::query(
        "INSERT INTO community_messages (id, room_id, session_id, handle, body, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(room_id)
    .bind(session_id)
    .bind(handle)
    .bind(body)
    .bind(at)
    .execute(pool)
    .await?;
    Ok(serde_json::json!({
        "id": id,
        "room_id": room_id,
        "handle": handle,
        "body": body,
        "created_at": at
    }))
}

pub async fn list_community_messages(
    pool: &PgPool,
    room_id: &str,
    limit: i64,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(Uuid, String, String, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, room_id, handle, body, created_at FROM community_messages
         WHERE room_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(room_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .rev()
        .map(|(id, room_id, handle, body, created_at)| {
            serde_json::json!({
                "id": id,
                "room_id": room_id,
                "handle": handle,
                "body": body,
                "created_at": created_at
            })
        })
        .collect())
}

pub async fn community_message_counts(pool: &PgPool) -> Result<Vec<(String, i64, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT room_id,
                count(*)::bigint,
                count(*) FILTER (WHERE created_at > now() - interval '24 hours')::bigint
         FROM community_messages
         GROUP BY room_id",
    )
    .fetch_all(pool)
    .await
}

pub async fn community_totals(pool: &PgPool) -> Result<(i64, i64), sqlx::Error> {
    let row: (i64, i64) = sqlx::query_as(
        "SELECT count(*)::bigint,
                count(*) FILTER (WHERE created_at > now() - interval '24 hours')::bigint
         FROM community_messages",
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}
