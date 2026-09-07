pub mod auth;
pub mod cache;
pub mod db;
pub mod jobs;
pub mod migrate;
pub mod onchain_job;
pub mod ecosystem_job;
pub mod queue;
pub mod sso;
pub mod tenant;
pub mod twin_api;
pub mod product;
pub mod llm;
pub mod webhooks;
pub mod exchange;
pub mod project;
pub mod connect;
pub mod community;
pub mod mcp;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::Extension;
use axum::response::Redirect;
use crate::tenant::TenantCtx;
use axum::http::{HeaderMap, HeaderValue, Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use memecoin_os_core::models::{TokenSnapshot, TokenSummary};
use memecoin_os_core::providers::CoinGeckoProvider;
use memecoin_os_core::{onboard_definition, IntelligenceEngine, OnboardRequest, TokenRegistry};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<RwLock<TokenRegistry>>,
    pub engine: Arc<IntelligenceEngine>,
    pub snapshots: Arc<RwLock<HashMap<String, TokenSnapshot>>>,
    pub pool: Option<PgPool>,
    pub redis: Option<ConnectionManager>,
    pub last_job: Arc<RwLock<Option<serde_json::Value>>>,
    pub data_mode: String,
    pub event_bus: tokio::sync::broadcast::Sender<String>,
}

#[derive(Deserialize)]
struct CompareQuery {
    ids: String,
}
#[derive(Deserialize)]
struct AskBody {
    question: String,
    token_id: Option<String>,
}
#[derive(Deserialize)]
struct HistoryQuery {
    range: Option<String>,
}
#[derive(Deserialize)]
struct WebhookBody {
    kind: String,
    url: String,
    chat_id: Option<String>,
    token_id: Option<String>,
    digest_daily: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct Overview {
    tracked_tokens: usize,
    total_market_cap_usd: f64,
    global_volume_24h_usd: f64,
    active_ecosystems: usize,
    top_momentum: Vec<TokenSummary>,
    highest_health: Vec<HealthRow>,
    highest_risk: Vec<RiskRow>,
    tokens: Vec<TokenCard>,
    disclaimer: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct TokenCard {
    token: TokenSummary,
    health: f64,
    health_confidence: f64,
    risk: f64,
    risk_level: String,
    price_usd: f64,
    market_cap_usd: f64,
    volume_24h_usd: f64,
    change_24h_pct: f64,
    liquidity_usd: f64,
    holders: u64,
    data_state: String,
    market_state: String,
    holders_state: String,
}

#[derive(Serialize, Deserialize)]
struct HealthRow {
    id: String,
    symbol: String,
    value: f64,
}
#[derive(Serialize, Deserialize)]
struct RiskRow {
    id: String,
    symbol: String,
    value: f64,
    level: String,
}
#[derive(Serialize)]
struct RankingSet {
    health: Vec<HealthRow>,
    momentum: Vec<HealthRow>,
    risk: Vec<RiskRow>,
    development: Vec<HealthRow>,
    community: Vec<HealthRow>,
    liquidity: Vec<HealthRow>,
}

pub async fn run() {
    let _ = dotenvy::from_filename(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.env"));
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,memecoin_os_api=debug".into()),
        )
        .init();

    let state = boot_state().await;
    jobs::spawn(state.clone());
    tokio::spawn(connect::register_telegram_webhook());
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-operator-key"),
            axum::http::HeaderName::from_static("x-community-session"),
        ])
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| allowed_origin(origin)));
    let app = router(state.clone())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(32 * 1024));
    let addr = bind_addr();
    tracing::info!("MemeCoin OS API listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/status", get(status_page))
        .route("/v1/overview", get(overview))
        .route("/v1/overview.csv", get(overview_csv))
        .route("/v1/tokens", get(list_tokens).post(onboard))
        .route("/v1/tokens/{id}", get(get_token))
        .route("/v1/tokens/{id}/market", get(part_market))
        .route("/v1/tokens/{id}/onchain", get(part_onchain))
        .route("/v1/tokens/{id}/social", get(part_social))
        .route("/v1/tokens/{id}/development", get(part_dev))
        .route("/v1/tokens/{id}/risk", get(part_risk))
        .route("/v1/tokens/{id}/growth", get(part_growth))
        .route("/v1/tokens/{id}/intelligence", get(part_intel))
        .route("/v1/tokens/{id}/timeline", get(part_timeline))
        .route("/v1/tokens/{id}/briefing", get(part_briefing))
        .route("/v1/tokens/{id}/briefing.md", get(briefing_md))
        .route("/v1/tokens/{id}/genome", get(part_genome))
        .route("/v1/tokens/{id}/history", get(token_history))
        .route("/v1/tokens/{id}/history.csv", get(token_history_csv))
        .route("/v1/compare", get(compare))
        .route("/v1/rankings", get(rankings))
        .route("/v1/alerts", get(alerts))
        .route("/v1/alerts/rules", get(alert_rules).patch(patch_alert_rule))
        .route("/v1/alerts/{id}/ack", post(ack_alert))
        .route("/v1/webhooks", get(list_hooks).post(create_hook))
        .route("/v1/webhooks/{id}", delete(delete_hook))
        .route("/v1/flags", get(flags))
        .route("/v1/flags/{key}", patch(patch_flag))
        .route("/v1/keys", get(list_keys).post(create_key))
        .route("/v1/keys/{id}", delete(revoke_key))
        .route("/v1/tokens/{id}/refresh", post(refresh_one))
        .route("/v1/tokens/{id}/baselines", get(part_baselines))
        .route("/v1/tokens/{id}/pools", get(part_pools))
        .route("/v1/tokens/{id}/wallets", get(part_wallets))
        .route("/v1/tokens/{id}/domain-events", get(part_domain_events))
        .route("/v1/tokens/{id}/transfers", get(part_transfers))
        .route("/v1/tokens/{id}/whale-events", get(part_whale_events))
        .route("/v1/tokens/{id}/clusters", get(part_clusters))
        .route("/v1/indexer", get(indexer_status))
        .route("/v1/discovery", get(part_discovery))
        .route("/v1/narratives", get(part_narratives))
        .route("/v1/genome-clusters", get(part_genome_clusters))
        .route("/v1/tokens/{id}/verification", get(part_verification))
        .route("/v1/jobs/refresh", post(refresh_all))
        .route("/v1/ai/query", post(ask))
        .route("/v1/research", post(research))
        .route("/v1/tokens/{id}/research.md", get(research_md))
        .route("/v1/tokens/{id}/research.json", get(research_json))
        .route("/v1/tenants", get(list_tenants).post(create_tenant))
        .route("/v1/branding", get(product::current_branding))
        .route("/v1/tenants/{id}/branding", get(product::get_branding).patch(product::patch_branding))
        .route("/v1/community", get(community_desk))
        .route("/v1/community/session", post(community::create_session))
        .route("/v1/community/rooms", get(community::list_rooms))
        .route("/v1/community/rooms/{id}/messages", get(community::list_messages).post(community::post_message))
        .route("/v1/connect", get(connect::status))
        .route("/v1/connect/discord", get(connect::discord_start))
        .route("/v1/connect/discord/callback", get(connect::discord_callback))
        .route("/v1/connect/telegram", get(connect::telegram_start))
        .route("/v1/connect/telegram/inbound", post(connect::telegram_inbound))
        .route("/v1/watchlist", get(product::get_watchlist).put(product::put_watchlist))
        .route("/v1/billing", get(billing_status))
        .route("/v1/billing/stripe/webhook", post(product::stripe_webhook))
        .route("/v1/queue", get(queue_status))
        .route("/v1/admin/storage", get(storage_status))
        .route("/v1/admin/timeseries", get(timeseries_status))
        .route("/v1/auth/sso", get(sso_status))
        .route("/v1/auth/oidc/login", get(oidc_login))
        .route("/v1/auth/oidc/callback", get(oidc_callback))
        .route("/v2/discovery", get(part_discovery))
        .route("/v2/narratives", get(part_narratives))
        .route("/v2/research", post(research))
        .route("/v2/intelligence", post(ask))
        .route("/v2/clusters", get(part_genome_clusters))
        .route("/v2/ecosystems", get(twin_api::list_ecosystems))
        .route("/v2/ecosystems/compare", get(twin_api::compare_ecosystems))
        .route("/v2/briefing", get(twin_api::daily_briefing))
        .route("/v2/events", get(twin_api::event_stream))
        .route("/v2/ecosystems/{id}", get(twin_api::get_ecosystem))
        .route("/v2/ecosystems/{id}/twin", get(twin_api::twin_now))
        .route("/v2/ecosystems/{id}/twin/state", get(twin_api::twin_state))
        .route("/v2/ecosystems/{id}/twin/history", get(twin_api::twin_history))
        .route("/v2/ecosystems/{id}/twin/graph", get(twin_api::twin_graph))
        .route("/v2/ecosystems/{id}/twin/evolution", get(twin_api::twin_evolution))
        .route("/v2/ecosystems/{id}/genome", get(twin_api::twin_genome))
        .route("/v2/ecosystems/{id}/lifecycle", get(twin_api::twin_lifecycle))
        .route("/v2/ecosystems/{id}/anomalies", get(twin_api::twin_anomalies))
        .route("/v2/ecosystems/{id}/evidence", get(twin_api::twin_evidence))
        .route("/v2/ecosystems/{id}/similar", get(twin_api::twin_similar))
        .route("/v2/ecosystems/{id}/twin/replay", get(twin_api::twin_replay))
        .route("/v2/ecosystems/{id}/whales", get(twin_api::whale_behaviour))
        .route("/v2/simulations", post(twin_api::create_simulation))
        .route("/v2/simulations/{id}", get(twin_api::get_simulation))
        .route("/v2/copilot", post(twin_api::copilot))
        .route("/v1/health", get(exchange::health))
        .route("/v1/ecosystems", get(exchange::list_ecosystems))
        .route("/v1/ecosystems/{id}", get(exchange::get_ecosystem))
        .route("/v1/ecosystems/{id}/asset", get(exchange::asset))
        .route("/v1/ecosystems/{id}/twin", get(exchange::twin))
        .route("/v1/ecosystems/{id}/risk", get(exchange::risk))
        .route("/v1/ecosystems/{id}/narratives", get(exchange::narratives))
        .route("/v1/ecosystems/{id}/genome", get(exchange::genome))
        .route("/v1/ecosystems/{id}/similar", get(exchange::similar))
        .route("/v1/ecosystems/{id}/history", get(exchange::history))
        .route("/v1/ecosystems/{id}/claims", get(exchange::claims))
        .route("/v1/mcp", get(mcp::get_manifest).post(mcp::rpc))
        .route("/v1/projects", get(project::list_projects))
        .route("/v1/projects/{id}", get(project::get_project))
        .route("/v1/projects/{id}/claim", post(project::claim))
        .route("/v1/projects/{id}/unclaim", post(project::unclaim))
        .route("/v1/projects/{id}/profile", patch(project::patch_profile))
        .route("/v1/projects/{id}/alerts", get(project::project_alerts))
        .route_layer(middleware::from_fn_with_state(state.clone(), rate_mw))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_mw))
        .with_state(state)
}

async fn boot_state() -> AppState {
    let registry_path = std::env::var("TOKEN_REGISTRY_PATH").unwrap_or_else(|_| "tokens".into());
    let mut registry = TokenRegistry::load(&registry_path).unwrap_or_else(|e| {
        tracing::error!(error = %e, path = %registry_path, "failed to load token registry");
        std::process::exit(1);
    });
    let engine = Arc::new(IntelligenceEngine::new(Arc::new(CoinGeckoProvider::new())));
    let data_mode = format!("{:?}", engine.mode()).to_lowercase();
    let pool = connect_postgres().await;
    if let Some(pool) = &pool {
        if let Err(e) = migrate::run(pool).await {
            tracing::error!(error = %e, "migrations failed — continuing degraded");
        } else {
            match db::load_token_definitions(pool).await {
                Ok(stored) => {
                    for def in stored {
                        if registry.contains(&def.token_id) {
                            continue;
                        }
                        match registry.insert_runtime(def.clone()) {
                            Ok(()) => tracing::info!(token = %def.token_id, "hydrated token from postgres"),
                            Err(e) => tracing::warn!(token = %def.token_id, error = %e, "skip stored token definition"),
                        }
                    }
                }
                Err(e) => tracing::warn!(error = %e, "could not load token definitions from postgres"),
            }
            for def in registry.list() {
                let _ = db::upsert_token(pool, def).await;
            }
        }
    }
    let redis = match std::env::var("REDIS_URL") {
        Ok(url) => cache::connect(&url).await,
        Err(_) => None,
    };
    let mut map = HashMap::new();
    if let Some(pool) = &pool {
        if let Ok(existing) = db::load_latest_snapshots(pool).await {
            for s in existing {
                map.insert(s.token.id.clone(), s);
            }
        }
    }
    for def in registry.list() {
        let prior = map.get(&def.token_id).cloned();
        let onchain_ready = prior
            .as_ref()
            .map(|s| s.onchain.data_state.present())
            .unwrap_or(false);
        if onchain_ready {
            continue;
        }
        match engine.snapshot_with_prior(def, prior.as_ref()).await {
            Ok(s) => {
                tracing::info!(token = %s.token.id, health = s.scores.value, state = s.data_state.as_str(), "indexed");
                if let Some(pool) = &pool {
                    let _ = db::persist_snapshot(pool, &s).await;
                }
                map.insert(s.token.id.clone(), s);
            }
            Err(e) => tracing::error!(token = %def.token_id, error = %e, "snapshot failed"),
        }
    }
    let (event_bus, _) = tokio::sync::broadcast::channel(64);
    AppState {
        registry: Arc::new(RwLock::new(registry)),
        engine,
        snapshots: Arc::new(RwLock::new(map)),
        pool,
        redis,
        last_job: Arc::new(RwLock::new(None)),
        data_mode,
        event_bus,
    }
}

async fn connect_postgres() -> Option<PgPool> {
    let opts = match postgres_connect_options() {
        Ok(o) => o,
        Err(e) => {
            tracing::error!(error = %e, "invalid DATABASE_URL");
            return None;
        }
    };
    let timeout_secs = if std::env::var("K_SERVICE").is_ok() { 30 } else { 3 };
    match tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        PgPoolOptions::new()
            .max_connections(8)
            .acquire_timeout(Duration::from_secs(timeout_secs))
            .connect_with(opts),
    )
    .await
    {
        Ok(Ok(pool)) => Some(pool),
        Ok(Err(e)) => {
            tracing::error!(error = %e, "postgres unavailable — in-memory degraded mode");
            None
        }
        Err(_) => {
            tracing::error!("postgres connect timed out — in-memory degraded mode");
            None
        }
    }
}

fn postgres_connect_options() -> Result<PgConnectOptions, String> {
    if let Ok(conn) = std::env::var("CLOUD_SQL_CONNECTION_NAME") {
        let conn = conn.trim();
        if !conn.is_empty() {
            let socket = if conn.starts_with("/cloudsql/") {
                conn.to_string()
            } else {
                format!("/cloudsql/{conn}")
            };
            let user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".into());
            let pass = std::env::var("DATABASE_PASSWORD")
                .map_err(|_| "DATABASE_PASSWORD required with CLOUD_SQL_CONNECTION_NAME".to_string())?;
            let db = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "memecoin_os".into());
            return Ok(PgConnectOptions::new()
                .socket(&socket)
                .username(&user)
                .password(&pass)
                .database(&db)
                .ssl_mode(sqlx::postgres::PgSslMode::Disable));
        }
    }
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL unset".to_string())?;
    PgConnectOptions::from_str(&url).map_err(|e| e.to_string())
}

fn bind_addr() -> SocketAddr {
    let port: u16 = std::env::var("PORT")
        .ok()
        .or_else(|| std::env::var("API_PORT").ok())
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let default_host = if std::env::var("K_SERVICE").is_ok() {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };
    let host = std::env::var("API_HOST").unwrap_or_else(|_| default_host.into());
    format!("{host}:{port}").parse().expect("bind addr")
}

fn allowed_origin(origin: &HeaderValue) -> bool {
    let o = origin.as_bytes();
    if o.starts_with(b"http://localhost") || o.starts_with(b"http://127.0.0.1") {
        return true;
    }
    const HOSTS: &[&[u8]] = &[
        b"https://memecoin-os.web.app",
        b"https://memecoin-os.firebaseapp.com",
    ];
    if HOSTS.contains(&o) {
        return true;
    }
    std::env::var("PUBLIC_WEB_ORIGIN")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| o == s.as_bytes())
        .unwrap_or(false)
}

async fn auth_mw(State(st): State<AppState>, mut req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();
    if req.method() == Method::OPTIONS
        || path == "/health"
        || path == "/status"
        || path.starts_with("/v1/auth/")
        || path == "/v2/events"
        || path == "/v1/billing/stripe/webhook"
    {
        return next.run(req).await;
    }
    let bearer = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());
    if bearer.is_none() && !auth::auth_required() {
        if let Some(pool) = &st.pool {
            if let Ok(id) = db::default_tenant_id(pool).await {
                req.extensions_mut().insert(TenantCtx::local_default(id));
            }
        }
        return next.run(req).await;
    }
    let Some(token) = bearer else {
        return (StatusCode::UNAUTHORIZED, "missing api key").into_response();
    };
    let Some(pool) = &st.pool else {
        if auth::auth_required() {
            return (StatusCode::SERVICE_UNAVAILABLE, "auth requires postgres").into_response();
        }
        return next.run(req).await;
    };
    let Some(prefix) = auth::prefix_of(&token) else {
        return (StatusCode::UNAUTHORIZED, "invalid api key").into_response();
    };
    match db::lookup_api_key_hash(pool, &prefix).await {
        Ok(Some((hash, id, tenant_id, role))) if hash == auth::hash_secret(&token) => {
            let _ = db::touch_api_key(pool, id).await;
            req.extensions_mut().insert(TenantCtx {
                tenant_id,
                role,
                key_id: Some(id),
                subject: prefix,
            });
            next.run(req).await
        }
        _ => (StatusCode::UNAUTHORIZED, "invalid api key").into_response(),
    }
}

async fn rate_mw(State(st): State<AppState>, req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("local")
        .to_string();
    if path == "/health" || path == "/status" || path.starts_with("/v1/auth/") || path == "/v2/events" || path == "/v1/billing/stripe/webhook" {
        return next.run(req).await;
    }
    if let Some(redis) = &st.redis {
        let mut r = redis.clone();
        let (key, limit) = if path.starts_with("/v1/ai/") || path.starts_with("/v2/intelligence") {
            (format!("rl:ai:{ip}"), 20)
        } else {
            (format!("rl:api:{ip}"), 180)
        };
        if !cache::rate_limit(&mut r, &key, limit, 60).await {
            return (StatusCode::TOO_MANY_REQUESTS, "rate limit").into_response();
        }
    } else if !cache::memory_rate_limit(&ip, &path, 180, 60) {
        return (StatusCode::TOO_MANY_REQUESTS, "rate limit").into_response();
    }
    if let Some(pool) = &st.pool {
        if let Some(ctx) = req.extensions().get::<TenantCtx>().cloned() {
            let metric = memecoin_os_core::billing::metric_for_path(&path);
            if let Ok(used) = db::increment_usage(pool, ctx.tenant_id, metric).await {
                if metering_enforce() {
                    let plan = memecoin_os_core::billing::PlanId::parse(
                        &db::tenant_plan(pool, ctx.tenant_id).await.unwrap_or_else(|_| "free".into()),
                    );
                    let limit = memecoin_os_core::billing::limit_for(plan, metric);
                    if !memecoin_os_core::billing::allowed(used - 1, limit) {
                        return (StatusCode::PAYMENT_REQUIRED, "plan quota exceeded").into_response();
                    }
                }
            }
        }
    }
    next.run(req).await
}

fn metering_enforce() -> bool {
    matches!(
        std::env::var("METERING_ENFORCE").unwrap_or_default().to_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

async fn health(State(st): State<AppState>) -> Json<serde_json::Value> {
    Json(status_payload(&st).await)
}
async fn status_page(State(st): State<AppState>) -> Json<serde_json::Value> {
    Json(status_payload(&st).await)
}

pub(crate) async fn status_payload(st: &AppState) -> serde_json::Value {
    let snaps = st.snapshots.read().await;
    let mut live = 0;
    let mut missing = 0;
    let mut conflict = 0;
    let mut stale = 0;
    for s in snaps.values() {
        match s.data_state.as_str() {
            "live" | "recent" => live += 1,
            "missing" => missing += 1,
            "conflict" => conflict += 1,
            "stale" => stale += 1,
            _ => {}
        }
    }
    drop(snaps);
    let mut pg_ok = false;
    let mut pg_ms = None;
    if let Some(pool) = &st.pool {
        let start = std::time::Instant::now();
        pg_ok = sqlx::query("SELECT 1").execute(pool).await.is_ok();
        pg_ms = Some(start.elapsed().as_millis() as u64);
    }
    let mut redis_ok = false;
    let mut redis_ms = None;
    if let Some(r) = &st.redis {
        let mut c = r.clone();
        if let Some(ms) = cache::ping_ms(&mut c).await {
            redis_ok = true;
            redis_ms = Some(ms);
        }
    }
    let providers = if let Some(pool) = &st.pool {
        db::provider_health_json(pool).await.unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let last_job = if let Some(pool) = &st.pool {
        db::last_job(pool).await.ok().flatten()
    } else {
        st.last_job.read().await.clone()
    };
    let degraded = !pg_ok || st.data_mode == "simulation";
    serde_json::json!({
        "ok": true,
        "service": "memecoin-os-api",
        "degraded": degraded,
        "data_mode": st.data_mode,
        "postgres": { "ok": pg_ok, "latency_ms": pg_ms },
        "redis": { "ok": redis_ok, "latency_ms": redis_ms },
        "providers": providers,
        "last_job": last_job,
        "tokens": { "live": live, "missing": missing, "conflict": conflict, "stale": stale },
        "twin": {
            "ecosystems": live + missing + conflict + stale,
            "sse": "/v2/events"
        },
        "execute": false,
        "policy": { "read": true, "analyze": true, "recommend": true, "propose": true, "execute": false }
    })
}

fn cards(snaps: &HashMap<String, TokenSnapshot>) -> Vec<TokenCard> {
    let mut cards: Vec<_> = snaps
        .values()
        .map(|s| TokenCard {
            token: s.token.clone(),
            health: s.scores.value,
            health_confidence: s.scores.confidence,
            risk: s.risk.score,
            risk_level: format!("{:?}", s.risk.level),
            price_usd: s.market.price_usd,
            market_cap_usd: s.market.market_cap_usd,
            volume_24h_usd: s.market.volume_24h_usd,
            change_24h_pct: s.market.change_24h_pct,
            liquidity_usd: s.liquidity.liquidity_usd,
            holders: s.onchain.holders,
            data_state: s.data_state.as_str().into(),
            market_state: s.market.data_state.as_str().into(),
            holders_state: s.onchain.data_state.as_str().into(),
        })
        .collect();
    cards.sort_by(|a, b| b.health.partial_cmp(&a.health).unwrap());
    cards
}

fn dim(g: &memecoin_os_core::models::Genome, id: &str) -> f64 {
    g.dimensions.iter().find(|d| d.id == id).map(|d| d.value).unwrap_or(0.0)
}

async fn overview(
    State(st): State<AppState>,
    ctx: Option<Extension<TenantCtx>>,
    Query(q): Query<product::ScopeQuery>,
) -> Json<Overview> {
    let scope = product::watchlist_scope(&st, ctx.as_ref().map(|e| &e.0), q.scope.as_deref()).await;
    let snaps = st.snapshots.read().await;
    let filtered = product::filter_snaps(&snaps, &scope);
    Json(build_overview(&filtered))
}

fn build_overview(snaps: &HashMap<String, TokenSnapshot>) -> Overview {
    let list: Vec<_> = snaps.values().cloned().collect();
    let total_mcap = list.iter().filter(|s| s.market.data_state.present()).map(|s| s.market.market_cap_usd).sum();
    let vol = list.iter().filter(|s| s.market.data_state.present()).map(|s| s.market.volume_24h_usd).sum();
    let mut health: Vec<_> = list.iter().map(|s| HealthRow { id: s.token.id.clone(), symbol: s.token.symbol.clone(), value: s.scores.value }).collect();
    health.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
    let mut risk: Vec<_> = list.iter().map(|s| RiskRow { id: s.token.id.clone(), symbol: s.token.symbol.clone(), value: s.risk.score, level: format!("{:?}", s.risk.level) }).collect();
    risk.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
    let mut mom = list.clone();
    mom.sort_by(|a, b| dim(&b.genome, "social_momentum").partial_cmp(&dim(&a.genome, "social_momentum")).unwrap());
    Overview {
        tracked_tokens: list.len(),
        total_market_cap_usd: total_mcap,
        global_volume_24h_usd: vol,
        active_ecosystems: list.iter().filter(|s| s.onchain.data_state.present() && s.onchain.active_holders_30d > 1000).count(),
        top_momentum: mom.iter().take(5).map(|s| s.token.clone()).collect(),
        highest_health: health.into_iter().take(5).collect(),
        highest_risk: risk.into_iter().take(5).collect(),
        tokens: cards(snaps),
        disclaimer: "Analytics only. Not financial advice. MISSING means no live source.".into(),
    }
}

async fn overview_csv(State(st): State<AppState>) -> impl IntoResponse {
    let snaps = st.snapshots.read().await;
    let mut out = String::from("id,symbol,chain,price_usd,health,risk,risk_level,market_cap_usd,volume_24h,data_state,holders_state\n");
    for t in cards(&snaps) {
        out.push_str(&format!("{},{},{},{:.12},{:.1},{:.1},{},{:.2},{:.2},{},{}\n", t.token.id, t.token.symbol, t.token.primary_chain, t.price_usd, t.health, t.risk, t.risk_level, t.market_cap_usd, t.volume_24h_usd, t.data_state, t.holders_state));
    }
    ([(axum::http::header::CONTENT_TYPE, "text/csv")], out)
}

async fn list_tokens(State(st): State<AppState>) -> Json<Vec<TokenCard>> {
    Json(cards(&*st.snapshots.read().await))
}

fn find_official(snap: Option<&TokenSnapshot>, kind: &str) -> Option<String> {
    snap.and_then(|s| {
        s.liquidity
            .official_links
            .iter()
            .find(|l| l.kind.eq_ignore_ascii_case(kind))
            .map(|l| l.url.clone())
    })
}

async fn community_desk(State(st): State<AppState>) -> Json<serde_json::Value> {
    let reg = st.registry.read().await;
    let snaps = st.snapshots.read().await;
    let mut chat_counts = std::collections::HashMap::new();
    if let Some(pool) = &st.pool {
        if let Ok(rows) = db::community_message_counts(pool).await {
            for (id, all, day) in rows {
                chat_counts.insert(id, (all, day));
            }
        }
    }
    let tokens: Vec<serde_json::Value> = reg
        .list()
        .into_iter()
        .map(|def| {
            let social = snaps.get(&def.token_id).map(|s| &s.social);
            let chat = chat_counts.get(&def.token_id).copied().unwrap_or((0, 0));
            serde_json::json!({
                "id": def.token_id,
                "symbol": def.symbol,
                "name": def.name,
                "website": def.website.clone().or_else(|| find_official(snaps.get(&def.token_id), "website")),
                "socials": {
                    "twitter": def.socials.twitter.clone().or_else(|| find_official(snaps.get(&def.token_id), "twitter")),
                    "telegram": def.socials.telegram.clone().or_else(|| find_official(snaps.get(&def.token_id), "telegram")),
                    "discord": def.socials.discord.clone().or_else(|| find_official(snaps.get(&def.token_id), "discord")),
                    "github": def.socials.github.clone().or_else(|| find_official(snaps.get(&def.token_id), "github"))
                },
                "mentions_24h": if social.map(|s| s.data_state.present()).unwrap_or(false) {
                    serde_json::json!(social.map(|s| s.mentions_24h))
                } else {
                    serde_json::Value::Null
                },
                "social_state": social.map(|s| s.data_state.as_str()).unwrap_or("missing"),
                "chat_state": "live",
                "chat_messages": chat.0,
                "chat_messages_24h": chat.1,
                "note": if social.map(|s| s.data_state.present()).unwrap_or(false) {
                    "Live mentions from licensed firehose. First-party chat is also live."
                } else {
                    "First-party chat is live. Licensed mention firehose is not connected."
                }
            })
        })
        .collect();
    let chat_totals = if let Some(pool) = &st.pool {
        db::community_totals(pool).await.unwrap_or((0, 0))
    } else {
        (0, 0)
    };
    Json(serde_json::json!({
        "tokens": tokens,
        "chat": { "live": true, "messages": chat_totals.0, "messages_24h": chat_totals.1 },
        "note": "First-party chat is live. Official registry links stay as-is. Licensed mention firehose is not connected. No invented Discord/Telegram chatter."
    }))
}

async fn snap(st: &AppState, id: &str) -> Result<TokenSnapshot, StatusCode> {
    st.snapshots.read().await.get(id).cloned().ok_or(StatusCode::NOT_FOUND)
}

async fn get_token(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<TokenSnapshot>, StatusCode> {
    let mut s = snap(&st, &id).await?;
    if let Some(pool) = &st.pool {
        jobs::overlay_history(pool, &mut s).await;
    }
    Ok(Json(s))
}

macro_rules! part {
    ($name:ident, $field:ident) => {
        async fn $name(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
            let s = snap(&st, &id).await?;
            Ok(Json(serde_json::to_value(&s.$field).unwrap()))
        }
    };
}
part!(part_market, market);
part!(part_onchain, onchain);
part!(part_social, social);
part!(part_dev, development);
part!(part_risk, risk);
part!(part_growth, growth);
part!(part_genome, genome);

async fn part_intel(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = snap(&st, &id).await?;
    Ok(Json(serde_json::json!({"observations": s.observations, "briefing": s.briefing, "scores": s.scores, "policy": { "execute": false }})))
}

async fn part_timeline(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(pool) = &st.pool {
        if let Ok(ev) = db::events(pool, &id, 50).await {
            if !ev.is_empty() {
                return Ok(Json(serde_json::to_value(&ev).unwrap()));
            }
        }
    }
    Ok(Json(serde_json::to_value(&snap(&st, &id).await?.timeline).unwrap()))
}

async fn part_briefing(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(pool) = &st.pool {
        if let Ok(Some(b)) = db::load_briefing(pool, &id).await {
            return Ok(Json(serde_json::to_value(&b).unwrap()));
        }
    }
    Ok(Json(serde_json::to_value(&snap(&st, &id).await?.briefing).unwrap()))
}

async fn briefing_md(State(st): State<AppState>, Path(id): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let s = snap(&st, &id).await?;
    let b = s.briefing;
    let md = format!(
        "# {}\n\n**Health** {:.0} · confidence {:.0}% · state `{}`\n\n## Changes\n{}\n\n## Risks\n{}\n\n## Opportunities\n{}\n\n## Evidence\n{}\n\nEXECUTE is disabled.\n",
        b.headline, b.ecosystem_health, b.confidence * 100.0, s.data_state.as_str(),
        b.changes.iter().map(|c| format!("- {c}")).collect::<Vec<_>>().join("\n"),
        b.risks.iter().map(|c| format!("- {c}")).collect::<Vec<_>>().join("\n"),
        b.opportunities.iter().map(|c| format!("- {c}")).collect::<Vec<_>>().join("\n"),
        b.evidence.iter().map(|c| format!("- {c}")).collect::<Vec<_>>().join("\n"),
    );
    Ok(([(axum::http::header::CONTENT_TYPE, "text/markdown")], md))
}

fn parse_range(range: &str) -> i32 {
    match range { "90d" => 90, "7d" => 7, _ => 30 }
}

async fn part_baselines(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"token_id": id, "baselines": [], "note": "Postgres unavailable"}));
    };
    match db::list_baselines(pool, &id).await {
        Ok(rows) => Json(serde_json::json!({"token_id": id, "baselines": rows})),
        Err(_) => Json(serde_json::json!({"token_id": id, "baselines": [], "error": "query failed"})),
    }
}

async fn part_pools(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    if let Some(pool) = &st.pool {
        if let Ok(rows) = db::list_pools(pool, &id).await {
            if !rows.is_empty() {
                return Json(serde_json::json!({"token_id": id, "pools": rows}));
            }
        }
    }
    let pools = snap(&st, &id).await.map(|s| s.liquidity.pools).unwrap_or_default();
    Json(serde_json::json!({"token_id": id, "pools": pools, "persisted": false}))
}

async fn part_wallets(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"token_id": id, "wallets": [], "note": "Postgres unavailable — classifications not persisted"}));
    };
    match db::list_wallet_profiles(pool, &id).await {
        Ok(rows) => Json(serde_json::json!({"token_id": id, "wallets": rows, "disclaimer": "Classification is evidence-based, not identity."})),
        Err(_) => Json(serde_json::json!({"token_id": id, "wallets": []})),
    }
}

async fn part_domain_events(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"token_id": id, "events": []}));
    };
    match db::list_domain_events(pool, &id, 50).await {
        Ok(rows) => Json(serde_json::json!({"token_id": id, "events": rows})),
        Err(_) => Json(serde_json::json!({"token_id": id, "events": []})),
    }
}

async fn part_transfers(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"token_id": id, "transfers": [], "note": "Postgres unavailable — transfers MISSING"}));
    };
    match db::list_transfers(pool, &id, 50).await {
        Ok(rows) => Json(serde_json::json!({
            "token_id": id,
            "transfers": rows,
            "note": if rows.is_empty() { "No indexed transfers. Set RPC_ETHEREUM or ETHERSCAN_API_KEY; holder counts are not transfers." } else { "Indexed ERC-20 Transfer logs only." }
        })),
        Err(_) => Json(serde_json::json!({"token_id": id, "transfers": []})),
    }
}

async fn part_whale_events(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"token_id": id, "events": [], "note": "Postgres unavailable"}));
    };
    match db::list_whale_movements(pool, &id, 50).await {
        Ok(rows) => Json(serde_json::json!({
            "token_id": id,
            "events": rows,
            "disclaimer": "Whale/CEX labels are evidence-based. Not identity. Not a trade signal."
        })),
        Err(_) => Json(serde_json::json!({"token_id": id, "events": []})),
    }
}

async fn part_discovery(State(st): State<AppState>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"candidates": [], "note": "Postgres unavailable", "disclaimer": "Discovery never auto-verifies."}));
    };
    let known: std::collections::HashSet<String> = {
        let reg = st.registry.read().await;
        reg.list()
            .into_iter()
            .flat_map(|d| {
                d.chains
                    .iter()
                    .map(|c| c.contracts.token.address.to_lowercase())
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    match db::list_discovery(pool).await {
        Ok(rows) => {
            let candidates: Vec<serde_json::Value> = rows
                .into_iter()
                .filter_map(|mut row| {
                    let addr = row.get("address")?.as_str()?.to_lowercase();
                    if memecoin_os_core::discovery::is_quote_asset(&addr)
                        || memecoin_os_core::cex::is_hub(&addr)
                    {
                        return None;
                    }
                    row["already_tracked"] = serde_json::json!(known.contains(&addr));
                    Some(row)
                })
                .collect();
            Json(serde_json::json!({
                "candidates": candidates,
                "disclaimer": "Growth does not promote VERIFIED. VERIFIED ≠ SAFE. Track writes UNVERIFIED only."
            }))
        }
        Err(_) => Json(serde_json::json!({"candidates": []})),
    }
}

async fn part_narratives(State(st): State<AppState>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"narratives": [], "note": "Postgres unavailable — social velocity stays MISSING"}));
    };
    match db::list_narratives(pool).await {
        Ok(rows) => Json(serde_json::json!({"narratives": rows, "disclaimer": "mention_velocity is MISSING without a licensed social provider."})),
        Err(_) => Json(serde_json::json!({"narratives": []})),
    }
}

async fn part_genome_clusters(State(st): State<AppState>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"clusters": []}));
    };
    match db::list_genome_clusters(pool).await {
        Ok(rows) => Json(serde_json::json!({"clusters": rows, "disclaimer": "Name is not a clustering feature."})),
        Err(_) => Json(serde_json::json!({"clusters": []})),
    }
}

async fn part_verification(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    if let Some(pool) = &st.pool {
        if let Ok(Some(row)) = db::get_verification(pool, &id).await {
            return Json(row);
        }
    }
    Json(serde_json::json!({"token_id": id, "level": "unverified", "disclaimer": "VERIFIED ≠ SAFE", "note": "No persisted report yet"}))
}

async fn part_clusters(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"token_id": id, "clusters": []}));
    };
    match db::list_clusters(pool, &id).await {
        Ok(rows) => Json(serde_json::json!({"token_id": id, "clusters": rows})),
        Err(_) => Json(serde_json::json!({"token_id": id, "clusters": []})),
    }
}

async fn indexer_status(State(st): State<AppState>) -> Json<serde_json::Value> {
    let chains = ["ethereum", "arbitrum", "bsc", "base", "polygon", "optimism", "avalanche"];
    let cursors = if let Some(pool) = &st.pool {
        db::list_indexer_cursors(pool).await.unwrap_or_default()
    } else {
        vec![]
    };
    let etherscan = std::env::var("ETHERSCAN_API_KEY").map(|s| !s.trim().is_empty()).unwrap_or(false);
    let rows: Vec<_> = chains
        .iter()
        .map(|c| {
            let key = format!("RPC_{}", c.to_uppercase());
            let configured = std::env::var(&key).map(|s| !s.trim().is_empty()).unwrap_or(false);
            serde_json::json!({
                "chain_id": c,
                "rpc_configured": configured,
                "etherscan_fallback": etherscan,
                "transfers_indexed": configured || etherscan,
                "note": if configured {
                    "RPC set — cursor advances on token refresh"
                } else if etherscan {
                    "No RPC — Etherscan tokentx fallback (recent page only)"
                } else {
                    "RPC unset — transfers MISSING"
                }
            })
        })
        .collect();
    Json(serde_json::json!({
        "indexers": rows,
        "cursors": cursors,
        "solana": product::indexer_solana_json(),
        "execute": false
    }))
}

async fn token_history(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<HistoryQuery>) -> Result<Json<serde_json::Value>, StatusCode> {
    let days = parse_range(q.range.as_deref().unwrap_or("30d"));
    let Some(pool) = &st.pool else {
        return Ok(Json(serde_json::json!({"token_id": id, "range_days": days, "points": [], "note": "Postgres unavailable"})));
    };
    let points = db::history(pool, &id, days).await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(serde_json::json!({"token_id": id, "range_days": days, "points": points, "simulated_excluded": true})))
}

async fn token_history_csv(State(st): State<AppState>, Path(id): Path<String>, Query(q): Query<HistoryQuery>) -> Result<impl IntoResponse, StatusCode> {
    let days = parse_range(q.range.as_deref().unwrap_or("30d"));
    let mut out = String::from("t,price_usd,market_cap_usd,volume_24h_usd,liquidity_usd,health,risk,data_state\n");
    if let Some(pool) = &st.pool {
        if let Ok(points) = db::history(pool, &id, days).await {
            for p in points {
                out.push_str(&format!("{},{},{},{},{},{},{},{}\n", p.t.to_rfc3339(), p.price_usd.unwrap_or(0.0), p.market_cap_usd.unwrap_or(0.0), p.volume_24h_usd.unwrap_or(0.0), p.liquidity_usd.unwrap_or(0.0), p.health.unwrap_or(0.0), p.risk.unwrap_or(0.0), p.data_state));
            }
        }
    }
    Ok(([(axum::http::header::CONTENT_TYPE, "text/csv")], out))
}

async fn compare(State(st): State<AppState>, Query(q): Query<CompareQuery>) -> Result<Json<serde_json::Value>, StatusCode> {
    let ids: Vec<_> = q.ids.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let snaps = st.snapshots.read().await;
    let selected: Vec<_> = ids.iter().filter_map(|id| snaps.get(id).cloned()).collect();
    if selected.is_empty() { return Err(StatusCode::NOT_FOUND); }
    Ok(Json(serde_json::json!({"tokens": selected, "interpretation": "Normalized genomes. Not transferable strategies.", "algorithm": { "health": "ecosystem-health/1.0.0", "genome": "1.0.0" }})))
}

async fn rankings(State(st): State<AppState>) -> Json<RankingSet> {
    let list: Vec<_> = st.snapshots.read().await.values().cloned().collect();
    fn health_rank(list: &[TokenSnapshot], f: impl Fn(&TokenSnapshot) -> f64) -> Vec<HealthRow> {
        let mut rows: Vec<_> = list.iter().map(|s| HealthRow { id: s.token.id.clone(), symbol: s.token.symbol.clone(), value: f(s) }).collect();
        rows.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        rows
    }
    Json(RankingSet {
        health: health_rank(&list, |s| s.scores.value),
        momentum: health_rank(&list, |s| dim(&s.genome, "social_momentum")),
        risk: {
            let mut rows: Vec<_> = list.iter().map(|s| RiskRow { id: s.token.id.clone(), symbol: s.token.symbol.clone(), value: s.risk.score, level: format!("{:?}", s.risk.level) }).collect();
            rows.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
            rows
        },
        development: health_rank(&list, |s| dim(&s.genome, "development")),
        community: health_rank(&list, |s| dim(&s.genome, "community")),
        liquidity: health_rank(&list, |s| dim(&s.genome, "liquidity")),
    })
}

async fn alerts(State(st): State<AppState>) -> Json<serde_json::Value> {
    if let Some(pool) = &st.pool {
        if let Ok(stored) = db::load_alerts(pool, 100).await {
            if !stored.is_empty() {
                return Json(serde_json::json!({ "alerts": stored }));
            }
        }
    }
    let list: Vec<_> = st.snapshots.read().await.values().cloned().collect();
    Json(serde_json::json!({ "alerts": st.engine.alerts_from(&list) }))
}

async fn grounded_report(st: &AppState, question: &str, focus: Option<&str>) -> memecoin_os_core::research::ResearchReport {
    let list: Vec<_> = st.snapshots.read().await.values().cloned().collect();
    let report = memecoin_os_core::research::run(question, &list, focus);
    llm::polish(report).await
}

async fn ask(State(st): State<AppState>, Json(body): Json<AskBody>) -> Json<memecoin_os_core::ai::AgentAnswer> {
    let report = grounded_report(&st, &body.question, body.token_id.as_deref()).await;
    Json(memecoin_os_core::research::into_answer(report))
}

async fn research(State(st): State<AppState>, Json(body): Json<AskBody>) -> Json<serde_json::Value> {
    let report = grounded_report(&st, &body.question, body.token_id.as_deref()).await;
    if let Some(pool) = &st.pool {
        if db::flag_enabled(pool, "jobs.research").await {
            let _ = db::insert_research_report(pool, body.token_id.as_deref(), &body.question, &report).await;
        }
    }
    Json(serde_json::to_value(&report).unwrap_or(serde_json::json!({"error": "serialize"})))
}

async fn research_md(State(st): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    if let Some(pool) = &st.pool {
        if let Ok(Some((md, _))) = db::latest_research(pool, &id).await {
            return (
                [(axum::http::header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
                md,
            );
        }
    }
    let report = grounded_report(
        &st,
        "Produce a grounded research report for this ecosystem.",
        Some(&id),
    )
    .await;
    (
        [(axum::http::header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        report.markdown,
    )
}

async fn research_json(State(st): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    if let Some(pool) = &st.pool {
        if let Ok(Some((_, payload))) = db::latest_research(pool, &id).await {
            return Json(payload);
        }
    }
    let report = grounded_report(
        &st,
        "Produce a grounded research report for this ecosystem.",
        Some(&id),
    )
    .await;
    Json(serde_json::to_value(&report).unwrap_or(serde_json::json!({})))
}

async fn onboard(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<OnboardRequest>,
) -> Result<Json<TokenSnapshot>, (StatusCode, String)> {
    auth::deny_unless_operator(&headers)?;
    {
        let addr = req.address.trim().to_lowercase();
        let reg = st.registry.read().await;
        let already = reg.list().into_iter().any(|d| {
            d.chains
                .iter()
                .any(|c| c.contracts.token.address.eq_ignore_ascii_case(&addr))
        });
        if already {
            return Err((StatusCode::CONFLICT, "contract already tracked".into()));
        }
    }
    let def = onboard_definition(req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let snap = st.engine.snapshot(&def).await.map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    if let Some(pool) = &st.pool {
        db::upsert_token(pool, &def).await.map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
        db::persist_snapshot(pool, &snap).await.map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    }
    st.registry.write().await.insert_runtime(def).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    st.snapshots.write().await.insert(snap.token.id.clone(), snap.clone());
    Ok(Json(snap))
}

async fn list_hooks(State(st): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(pool) = &st.pool else { return Ok(Json(serde_json::json!({ "webhooks": [], "degraded": true }))); };
    let hooks = db::list_webhooks(pool).await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(serde_json::json!({ "webhooks": hooks })))
}

async fn create_hook(State(st): State<AppState>, headers: HeaderMap, Json(body): Json<WebhookBody>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    auth::deny_unless_operator(&headers)?;
    let Some(pool) = &st.pool else { return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into())); };
    if body.kind != "discord" && body.kind != "telegram" {
        return Err((StatusCode::BAD_REQUEST, "kind must be discord or telegram".into()));
    }
    if !(body.url.starts_with("https://") || body.url.starts_with("http://127.0.0.1")) {
        return Err((StatusCode::BAD_REQUEST, "webhook url must be https (or local test)".into()));
    }
    let hook = db::insert_webhook(pool, &body.kind, &body.url, body.chat_id.as_deref(), body.token_id.as_deref(), body.digest_daily.unwrap_or(false))
        .await.map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(serde_json::json!(hook)))
}

async fn delete_hook(State(st): State<AppState>, Path(id): Path<String>) -> Result<StatusCode, StatusCode> {
    let Some(pool) = &st.pool else { return Err(StatusCode::SERVICE_UNAVAILABLE); };
    let uuid: Uuid = id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    db::delete_webhook(pool, uuid).await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) fn env_flag(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

fn env_flags() -> serde_json::Value {
    serde_json::json!([
        {"key": "provider.coingecko", "enabled": env_flag("FEATURE_COINGECKO", true), "reason": "env"},
        {"key": "provider.dexscreener", "enabled": env_flag("FEATURE_DEXSCREENER", true), "reason": "env"},
        {"key": "provider.github", "enabled": env_flag("FEATURE_GITHUB", true), "reason": "env"},
        {"key": "jobs.snapshot_refresh", "enabled": true, "reason": "env"},
        {"key": "jobs.briefing", "enabled": true, "reason": "env"},
        {"key": "jobs.webhooks", "enabled": true, "reason": "env"},
        {"key": "provider.holders", "enabled": env_flag("FEATURE_HOLDERS", true), "reason": "env"},
        {"key": "provider.social", "enabled": env_flag("FEATURE_SOCIAL", false), "reason": "live only with SOCIAL_FIREHOSE_URL + KEY"},
        {"key": "writes.operator_key", "enabled": auth::operator_key_configured(), "reason": "POST /v1/tokens, claim/profile, key/webhook create require x-operator-key"},
        {"key": "provider.solana", "enabled": true, "reason": "adapter; RPC only if SOLANA_RPC_URL"},
        {"key": "plane.white_label", "enabled": true, "reason": "tenant branding"},
        {"key": "plane.watchlist", "enabled": true, "reason": "watchlist isolation"},
        {"key": "digital_twin_webgl", "enabled": true, "reason": "WebGL projection"},
        {"key": "plane.mcp", "enabled": true, "reason": "GET/POST /v1/mcp read-only"},
        {"key": "jobs.baselines", "enabled": true, "reason": "env"},
        {"key": "jobs.pool_discovery", "enabled": true, "reason": "env"},
        {"key": "jobs.indexer", "enabled": env_flag("FEATURE_ONCHAIN_INDEXER", true), "reason": "env — no-op without RPC or Etherscan"},
        {"key": "jobs.whale_engine", "enabled": true, "reason": "env"},
        {"key": "jobs.wallet_clusters", "enabled": true, "reason": "env"},
        {"key": "jobs.discovery", "enabled": true, "reason": "env"},
        {"key": "jobs.verification", "enabled": true, "reason": "env"},
        {"key": "jobs.narratives", "enabled": true, "reason": "env"},
        {"key": "jobs.genome_clusters", "enabled": true, "reason": "env"},
        {"key": "plane.ai", "enabled": env_flag("FEATURE_AI_RESEARCH", true), "reason": "env"},
        {"key": "provider.nvidia", "enabled": llm::nvidia_configured(), "reason": "live polish only when NVIDIA_API_KEY is set; grounded report always wins"},
        {"key": "jobs.queue", "enabled": true, "reason": "env"},
        {"key": "plane.billing", "enabled": true, "reason": "env"},
        {"key": "plane.sso", "enabled": true, "reason": "env"},
        {"key": "digital_twin", "enabled": true, "reason": "env"},
        {"key": "digital_twin_3d", "enabled": true, "reason": "env — projection of Twin model"},
        {"key": "knowledge_graph", "enabled": true, "reason": "env"},
        {"key": "ecosystem_lifecycle", "enabled": true, "reason": "env"},
        {"key": "ecosystem_similarity", "enabled": true, "reason": "env"},
        {"key": "simulation_engine", "enabled": true, "reason": "env"},
    ])
}

async fn flags(State(st): State<AppState>) -> Json<serde_json::Value> {
    if let Some(pool) = &st.pool {
        if let Ok(v) = db::flags(pool).await {
            if v.as_array().map(|a| !a.is_empty()).unwrap_or(false) {
                return Json(v);
            }
        }
    }
    Json(env_flags())
}

#[derive(Deserialize)]
struct FlagBody {
    enabled: bool,
    reason: Option<String>,
}

async fn patch_flag(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(body): Json<FlagBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    if key.is_empty() || key.len() > 64 || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-') {
        return Err((StatusCode::BAD_REQUEST, "invalid flag key".into()));
    }
    db::set_flag(pool, &key, body.enabled, body.reason.as_deref())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(serde_json::json!({"key": key, "enabled": body.enabled})))
}

#[derive(Deserialize)]
struct KeyBody {
    name: Option<String>,
    role: Option<String>,
}

async fn list_keys(State(st): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(pool) = &st.pool else {
        return Ok(Json(serde_json::json!({"keys": [], "degraded": true})));
    };
    let keys = db::list_api_keys(pool).await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(serde_json::json!({"keys": keys, "auth_required": auth::auth_required()})))
}

async fn create_key(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<KeyBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    auth::deny_unless_operator(&headers)?;
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    let role = body.role.unwrap_or_else(|| "api_client".into());
    let allowed = ["platform_admin", "analyst", "developer", "community_manager", "viewer", "api_client"];
    if !allowed.contains(&role.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "role not allow-listed".into()));
    }
    let tenant = db::default_tenant_id(pool)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let (plaintext, prefix, hash) = auth::generate_key();
    let name = body.name.unwrap_or_else(|| "operator".into());
    let row = db::insert_api_key(pool, tenant, &name, &prefix, &hash, &role)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let _ = db::insert_audit(
        pool,
        "api-key",
        "create_key",
        &row.id.to_string(),
        serde_json::json!({"name": name, "role": role, "prefix": prefix}),
    )
    .await;
    Ok(Json(serde_json::json!({
        "id": row.id,
        "name": row.name,
        "prefix": row.prefix,
        "role": row.role,
        "secret": plaintext,
        "note": "Copy the secret now. It is not stored in plaintext."
    })))
}

async fn revoke_key(State(st): State<AppState>, Path(id): Path<String>) -> Result<StatusCode, StatusCode> {
    let Some(pool) = &st.pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let uuid: Uuid = id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    db::revoke_api_key(pool, uuid)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn alert_rules(State(st): State<AppState>) -> Json<serde_json::Value> {
    if let Some(pool) = &st.pool {
        if let Ok(rules) = db::list_alert_rules(pool).await {
            return Json(serde_json::json!({"rules": rules}));
        }
    }
    Json(serde_json::json!({"rules": [], "degraded": true}))
}

#[derive(Deserialize)]
struct RuleBody {
    kind: String,
    enabled: bool,
}

async fn patch_alert_rule(
    State(st): State<AppState>,
    Json(body): Json<RuleBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    db::set_alert_rule(pool, &body.kind, body.enabled)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(serde_json::json!({"kind": body.kind, "enabled": body.enabled})))
}

async fn ack_alert(State(st): State<AppState>, Path(id): Path<String>) -> Result<StatusCode, StatusCode> {
    let Some(pool) = &st.pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let uuid: Uuid = id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    db::ack_alert(pool, uuid)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn refresh_one(
    State(st): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TokenSnapshot>, (StatusCode, String)> {
    jobs::refresh_token(&st, &id)
        .await
        .map(Json)
        .map_err(|(c, e)| (c, e))
}

async fn refresh_all(State(st): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    jobs::run_once(&st)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(st.last_job.read().await.clone().unwrap_or(serde_json::json!({"ok": true}))))
}

#[derive(Deserialize)]
struct TenantBody {
    slug: String,
    name: String,
    plan: Option<String>,
}

#[derive(Deserialize)]
struct OidcCb {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct TimeseriesQuery {
    days: Option<i32>,
}

fn is_platform_admin(ctx: Option<&TenantCtx>) -> bool {
    ctx.map(|c| c.role == "platform_admin").unwrap_or(false)
}

fn valid_tenant_slug(s: &str) -> bool {
    let b = s.as_bytes();
    (2..=48).contains(&b.len())
        && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

async fn list_tenants(State(st): State<AppState>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"tenants": [], "degraded": true}));
    };
    match db::list_tenants(pool).await {
        Ok(tenants) => Json(serde_json::json!({"tenants": tenants})),
        Err(_) => Json(serde_json::json!({"tenants": [], "degraded": true})),
    }
}

async fn create_tenant(
    State(st): State<AppState>,
    ctx: Option<Extension<TenantCtx>>,
    Json(body): Json<TenantBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if !is_platform_admin(ctx.as_ref().map(|e| &e.0)) {
        return Err((StatusCode::FORBIDDEN, "platform_admin required".into()));
    }
    if !valid_tenant_slug(&body.slug) {
        return Err((StatusCode::BAD_REQUEST, "slug must be 2-48 [a-zA-Z0-9-]".into()));
    }
    let plan = body.plan.unwrap_or_else(|| "free".into());
    let allowed = ["free", "pro", "research", "growth", "enterprise", "api"];
    if !allowed.contains(&plan.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown plan".into()));
    }
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    let id = db::insert_tenant(pool, &body.slug, &body.name, &plan)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let actor = ctx.as_ref().map(|e| e.0.subject.as_str()).unwrap_or("local-operator");
    let _ = db::insert_audit(
        pool,
        actor,
        "create_tenant",
        &id.to_string(),
        serde_json::json!({"slug": body.slug, "plan": plan}),
    )
    .await;
    Ok(Json(serde_json::json!({
        "id": id,
        "slug": body.slug,
        "name": body.name,
        "plan": plan,
        "note": "Token intelligence stays shared. Isolation is keys, usage, queue, members."
    })))
}

async fn billing_status(
    State(st): State<AppState>,
    ctx: Option<Extension<TenantCtx>>,
) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({
            "degraded": true,
            "metering_enforce": metering_enforce(),
            "stripe": memecoin_os_core::stripe_connect::status_json(),
            "note": "No Stripe charge. Quotas apply only when METERING_ENFORCE=true."
        }));
    };
    let tenant_id = match ctx.as_ref() {
        Some(e) => e.0.tenant_id,
        None => match db::default_tenant_id(pool).await {
            Ok(id) => id,
            Err(_) => {
                return Json(serde_json::json!({"degraded": true, "metering_enforce": metering_enforce()}));
            }
        },
    };
    let plan_s = db::tenant_plan(pool, tenant_id).await.unwrap_or_else(|_| "free".into());
    let plan = memecoin_os_core::billing::PlanId::parse(&plan_s);
    let limits = plan.limits();
    let usage = db::usage_today(pool, tenant_id).await.unwrap_or_default();
    let usage_map: serde_json::Map<String, serde_json::Value> = usage
        .into_iter()
        .map(|(k, v)| (k, serde_json::json!(v)))
        .collect();
    Json(serde_json::json!({
        "tenant_id": tenant_id,
        "plan": plan.as_str(),
        "metering_enforce": metering_enforce(),
        "limits": {
            "api_requests_day": limits.api_requests_day,
            "ai_requests_day": limits.ai_requests_day,
            "research_day": limits.research_day,
            "tokens_monitored": limits.tokens_monitored,
            "unlimited": -1
        },
        "usage_today": usage_map,
        "stripe": memecoin_os_core::stripe_connect::status_json(),
        "note": "Meters always. Stripe charges only when STRIPE_* env is set. -1 = unlimited."
    }))
}

async fn queue_status(State(st): State<AppState>) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"degraded": true, "pending": 0, "locked": 0}));
    };
    match db::queue_stats(pool).await {
        Ok(stats) => Json(serde_json::json!({
            "pending": stats.get("pending"),
            "locked": stats.get("locked"),
            "flag": "jobs.queue",
            "note": "Kinds refresh_token|domain_event|research complete as recorded work; no invented side effects."
        })),
        Err(_) => Json(serde_json::json!({"degraded": true})),
    }
}

async fn storage_status(State(st): State<AppState>) -> Json<serde_json::Value> {
    let path = std::env::var("OBJECT_STORE_PATH").unwrap_or_else(|_| "data/raw".into());
    let blobs = match &st.pool {
        Some(pool) => db::object_blob_count(pool).await.unwrap_or(0),
        None => 0,
    };
    let raw_on = match &st.pool {
        Some(pool) => db::flag_enabled(pool, "jobs.raw_store").await,
        None => false,
    };
    Json(serde_json::json!({
        "path": path,
        "blobs": blobs,
        "raw_store_enabled": raw_on,
        "note": "Content SHA-256. jobs.raw_store is off by default so refresh does not fill disk."
    }))
}

async fn timeseries_status(
    State(st): State<AppState>,
    Query(q): Query<TimeseriesQuery>,
) -> Json<serde_json::Value> {
    let days = q.days.unwrap_or(90).clamp(7, 3650);
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({"degraded": true, "days": days, "older_than": 0}));
    };
    let older = db::count_old_snapshots(pool, days).await.unwrap_or(0);
    Json(serde_json::json!({
        "days": days,
        "older_than": older,
        "brin": ["market_snapshots.as_of", "liquidity_snapshots.as_of"],
        "timescale": "not_yet",
        "note": "GET is preview only. No rows are deleted. Timescale deferred — BRIN p95 acceptable until a written benchmark says otherwise. See docs/TIMESERIES_BENCHMARK.md."
    }))
}

async fn sso_status() -> Json<serde_json::Value> {
    match sso::config_env() {
        Some((issuer, _, _, redirect_uri, scopes)) => Json(serde_json::json!({
            "configured": true,
            "issuer": issuer,
            "redirect_uri": redirect_uri,
            "scopes": scopes,
            "provider_hint": if issuer.contains("accounts.google.com") { "google" } else { "oidc" }
        })),
        None => Json(serde_json::json!({
            "configured": false,
            "note": "Set OIDC_ISSUER, OIDC_CLIENT_ID, OIDC_CLIENT_SECRET. Google: issuer https://accounts.google.com. No local fake users."
        })),
    }
}

async fn oidc_login(State(st): State<AppState>) -> Response {
    let cfg = match sso::resolve().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::SERVICE_UNAVAILABLE, e).into_response(),
    };
    let Some(pool) = &st.pool else {
        return (StatusCode::SERVICE_UNAVAILABLE, "postgres required").into_response();
    };
    let state = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();
    if db::insert_oidc_state(pool, &state, &nonce).await.is_err() {
        return (StatusCode::BAD_GATEWAY, "oidc state").into_response();
    }
    Redirect::temporary(&sso::authorize_url(&cfg, &state, &nonce)).into_response()
}

async fn oidc_callback(State(st): State<AppState>, Query(q): Query<OidcCb>) -> Response {
    let origin = std::env::var("PUBLIC_WEB_ORIGIN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://memecoin-os.web.app".into())
        .trim_end_matches('/')
        .to_string();
    if let Some(err) = q.error {
        return Redirect::temporary(&format!("{origin}/login/?error={}", urlencoding_basic(&err)))
            .into_response();
    }
    let cfg = match sso::resolve().await {
        Ok(c) => c,
        Err(_) => {
            return Redirect::temporary(&format!("{origin}/login/?error=oidc_not_configured"))
                .into_response();
        }
    };
    let Some(pool) = &st.pool else {
        return Redirect::temporary(&format!("{origin}/login/?error=no_db")).into_response();
    };
    let (Some(code), Some(state)) = (q.code, q.state) else {
        return Redirect::temporary(&format!("{origin}/login/?error=missing_code")).into_response();
    };
    match db::take_oidc_state(pool, &state).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Redirect::temporary(&format!("{origin}/login/?error=bad_state")).into_response();
        }
        Err(_) => {
            return Redirect::temporary(&format!("{origin}/login/?error=state_db")).into_response();
        }
    }
    let tokens = match sso::exchange_code(&cfg, &code).await {
        Ok(t) => t,
        Err(_) => {
            return Redirect::temporary(&format!("{origin}/login/?error=token")).into_response();
        }
    };
    let Some(access) = tokens.access_token.filter(|s| !s.is_empty()) else {
        return Redirect::temporary(&format!("{origin}/login/?error=no_access")).into_response();
    };
    let info = match sso::userinfo(&cfg, &access).await {
        Ok(u) => u,
        Err(_) => {
            return Redirect::temporary(&format!("{origin}/login/?error=userinfo")).into_response();
        }
    };
    let Some(sub) = info.sub.filter(|s| !s.is_empty()) else {
        return Redirect::temporary(&format!("{origin}/login/?error=no_sub")).into_response();
    };
    let tenant = match db::default_tenant_id(pool).await {
        Ok(id) => id,
        Err(_) => {
            return Redirect::temporary(&format!("{origin}/login/?error=tenant")).into_response();
        }
    };
    let user_id = match db::upsert_user(pool, tenant, &sub, info.email.as_deref()).await {
        Ok(id) => id,
        Err(_) => {
            return Redirect::temporary(&format!("{origin}/login/?error=user")).into_response();
        }
    };
    let session_plain = format!("sess_{}", Uuid::new_v4().simple());
    let session_hash = auth::hash_secret(&session_plain);
    if db::insert_session(pool, user_id, &session_hash).await.is_err() {
        return Redirect::temporary(&format!("{origin}/login/?error=session")).into_response();
    }
    // Session shown once on /login; stored hashed server-side. Not a trade credential.
    Redirect::temporary(&format!(
        "{origin}/login/?ok=1&session={session_plain}&email={}",
        urlencoding_basic(info.email.as_deref().unwrap_or(""))
    ))
    .into_response()
}

fn urlencoding_basic(s: &str) -> String {
    s.bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => vec![b as char],
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_slug_rules() {
        assert!(valid_tenant_slug("acme"));
        assert!(valid_tenant_slug("acme-1"));
        assert!(!valid_tenant_slug("a"));
        assert!(!valid_tenant_slug("-acme"));
        assert!(!valid_tenant_slug("acme-"));
        assert!(!valid_tenant_slug("acme/os"));
    }

    #[test]
    fn local_operator_is_platform_admin() {
        let ctx = TenantCtx::local_default(Uuid::nil());
        assert!(is_platform_admin(Some(&ctx)));
        assert!(!is_platform_admin(None));
    }
}
