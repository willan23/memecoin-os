use crate::db;
use crate::tenant::TenantCtx;
use crate::AppState;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Extension;
use axum::Json;
use memecoin_os_core::branding::{self, TenantBranding};
use memecoin_os_core::stripe_connect;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ScopeQuery {
    pub scope: Option<String>,
}

/// None = show all (local operator). Some(ids) = watchlist filter.
pub async fn watchlist_scope(
    st: &AppState,
    ctx: Option<&TenantCtx>,
    scope: Option<&str>,
) -> Option<HashSet<String>> {
    if !crate::auth::auth_required() {
        return None;
    }
    if matches!(scope, Some("all")) && ctx.map(|c| c.role == "platform_admin").unwrap_or(false) {
        return None;
    }
    let pool = st.pool.as_ref()?;
    let tenant = ctx?.tenant_id;
    match db::list_watchlist(pool, tenant).await {
        Ok(ids) => Some(ids.into_iter().collect()),
        Err(_) => Some(HashSet::new()),
    }
}

pub fn filter_snaps<'a, T>(
    snaps: &'a HashMap<String, T>,
    scope: &Option<HashSet<String>>,
) -> HashMap<String, T>
where
    T: Clone,
{
    match scope {
        None => snaps.clone(),
        Some(ids) => snaps
            .iter()
            .filter(|(k, _)| ids.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    }
}

pub async fn current_branding(
    State(st): State<AppState>,
    ctx: Option<Extension<TenantCtx>>,
) -> Json<serde_json::Value> {
    let Some(pool) = &st.pool else {
        return Json(serde_json::json!({
            "branding": {},
            "note": "Default chrome. Postgres down — no persisted branding."
        }));
    };
    let tenant = match ctx.as_ref() {
        Some(e) => e.0.tenant_id,
        None => match db::default_tenant_id(pool).await {
            Ok(id) => id,
            Err(_) => {
                return Json(serde_json::json!({"branding": {}}));
            }
        },
    };
    let branding = db::tenant_branding(pool, tenant).await.unwrap_or_else(|_| serde_json::json!({}));
    Json(serde_json::json!({
        "tenant_id": tenant,
        "branding": branding,
        "note": "Chrome only. Twin facts stay shared."
    }))
}

pub async fn get_branding(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
    ctx: Option<Extension<TenantCtx>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = st.pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    if !can_read_tenant(ctx.as_ref().map(|e| &e.0), id) {
        return Err(StatusCode::FORBIDDEN);
    }
    let branding = db::tenant_branding(pool, id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"tenant_id": id, "branding": branding})))
}

pub async fn patch_branding(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
    ctx: Option<Extension<TenantCtx>>,
    Json(body): Json<TenantBranding>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = st.pool.as_ref().ok_or((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()))?;
    if !can_write_tenant(ctx.as_ref().map(|e| &e.0), id) {
        return Err((StatusCode::FORBIDDEN, "admin or owner of this tenant".into()));
    }
    let clean = branding::validate(&body).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let val = serde_json::to_value(&clean).unwrap_or_else(|_| serde_json::json!({}));
    db::set_tenant_branding(pool, id, &val)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({"tenant_id": id, "branding": clean})))
}

#[derive(Deserialize)]
pub struct WatchlistBody {
    pub token_ids: Vec<String>,
}

async fn tenant_or_default(
    pool: &sqlx::PgPool,
    ctx: Option<&Extension<TenantCtx>>,
) -> Result<Uuid, StatusCode> {
    if let Some(e) = ctx {
        return Ok(e.0.tenant_id);
    }
    db::default_tenant_id(pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

pub async fn get_watchlist(
    State(st): State<AppState>,
    ctx: Option<Extension<TenantCtx>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = st.pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let tenant = tenant_or_default(pool, ctx.as_ref()).await?;
    let ids = db::list_watchlist(pool, tenant).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "tenant_id": tenant,
        "token_ids": ids,
        "note": "Watchlist is a personal filter. Snapshots stay shared. Server-side isolation only when AUTH_REQUIRED."
    })))
}

pub async fn put_watchlist(
    State(st): State<AppState>,
    ctx: Option<Extension<TenantCtx>>,
    Json(body): Json<WatchlistBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = st.pool.as_ref().ok_or((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()))?;
    let tenant = tenant_or_default(pool, ctx.as_ref())
        .await
        .map_err(|s| (s, "tenant required".into()))?;
    let plan_s = db::tenant_plan(pool, tenant).await.unwrap_or_else(|_| "free".into());
    let limits = memecoin_os_core::billing::PlanId::parse(&plan_s).limits();
    if limits.tokens_monitored >= 0 && body.token_ids.len() as i64 > limits.tokens_monitored {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            format!("watchlist exceeds plan cap {}", limits.tokens_monitored),
        ));
    }
    let mut ids = body.token_ids;
    ids.sort();
    ids.dedup();
    db::replace_watchlist(pool, tenant, &ids)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({"token_ids": ids})))
}

pub async fn stripe_webhook(
    State(st): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let secret = std::env::var("STRIPE_WEBHOOK_SECRET").ok().filter(|s| !s.is_empty());
    let Some(secret) = secret else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let sig = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let raw = String::from_utf8_lossy(&body);
    if !stripe_connect::verify_webhook(&secret, sig, &raw) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
    let kind = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let obj = v.get("data").and_then(|d| d.get("object")).cloned().unwrap_or(serde_json::json!({}));
    let tenant_meta = obj
        .get("metadata")
        .and_then(|m| m.get("tenant_id"))
        .and_then(|t| t.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let price = obj
        .get("items")
        .and_then(|i| i.get("data"))
        .and_then(|a| a.get(0))
        .and_then(|i| i.get("price"))
        .and_then(|p| p.get("id"))
        .and_then(|id| id.as_str())
        .or_else(|| obj.get("plan").and_then(|p| p.get("id")).and_then(|id| id.as_str()));
    let mut applied = false;
    if matches!(kind, "customer.subscription.updated" | "checkout.session.completed" | "customer.subscription.deleted") {
        if let (Some(pool), Some(tenant), Some(price)) = (&st.pool, tenant_meta, price) {
            if kind == "customer.subscription.deleted" {
                let _ = db::set_tenant_plan(pool, tenant, "free").await;
                applied = true;
            } else if let Some(plan) = stripe_connect::plan_for_price_id(price) {
                let _ = db::set_tenant_plan(pool, tenant, plan.as_str()).await;
                applied = true;
            }
        }
    }
    Ok(Json(serde_json::json!({
        "received": true,
        "applied": applied,
        "note": "Plan changes only from verified webhook + known Price id. No invented customers."
    })))
}

fn can_read_tenant(ctx: Option<&TenantCtx>, id: Uuid) -> bool {
    match ctx {
        Some(c) if c.role == "platform_admin" => true,
        Some(c) => c.tenant_id == id,
        None => !crate::auth::auth_required(),
    }
}

fn can_write_tenant(ctx: Option<&TenantCtx>, id: Uuid) -> bool {
    match ctx {
        Some(c) if c.role == "platform_admin" || c.role == "owner" || c.role == "admin" => {
            c.role == "platform_admin" || c.tenant_id == id
        }
        _ => false,
    }
}

pub fn indexer_solana_json() -> serde_json::Value {
    let a = memecoin_os_core::chain::SolanaAdapter::from_env();
    serde_json::json!({
        "family": "solana",
        "configured": a.configured(),
        "note": "Live RPC only if SOLANA_RPC_URL is set. Holders/transfers remain MISSING."
    })
}
