use crate::db;
use crate::twin_api;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use memecoin_os_core::ecosystem;
use memecoin_os_core::models::TokenSnapshot;
use memecoin_os_core::twin;
use memecoin_os_core::twin_similarity;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct RangeQuery {
    pub range: Option<String>,
}

fn envelope(data: Value, freshness: &str, confidence: f64, evidence: Vec<String>) -> Value {
    json!({
        "data": data,
        "timestamp": Utc::now(),
        "freshness": freshness,
        "confidence": confidence,
        "evidence": evidence,
        "powered_by": "MemeCoin OS",
        "execute": false,
        "note": "Exchange Intelligence API. Observable state only. Not a trade signal. VERIFIED ≠ SAFE."
    })
}

fn snap_or_404(snaps: &std::collections::HashMap<String, TokenSnapshot>, id: &str) -> Result<TokenSnapshot, StatusCode> {
    snaps.get(id).cloned().ok_or(StatusCode::NOT_FOUND)
}

pub async fn list_ecosystems(State(st): State<AppState>) -> Json<Value> {
    let snaps = st.snapshots.read().await;
    let rows = twin_api::ecosystems_from(&snaps);
    let freshness = if rows.iter().any(|e| e.data_freshness == "live") {
        "live"
    } else {
        "unknown"
    };
    Json(envelope(
        json!({ "ecosystems": rows }),
        freshness,
        0.7,
        vec!["registry snapshots".into()],
    ))
}

pub async fn get_ecosystem(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    let eco = ecosystem::from_snapshot(&s);
    let asset = asset_card(&s);
    Ok(Json(envelope(
        json!({ "ecosystem": eco, "asset": asset }),
        s.data_state.as_str(),
        eco.confidence,
        vec![format!("snapshot {}", s.as_of)],
    )))
}

pub async fn twin(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    let t = twin::from_snapshot(&id, &s);
    Ok(Json(envelope(
        json!({ "twin": t }),
        &t.freshness,
        t.confidence,
        t.market_state.evidence.clone(),
    )))
}

pub async fn risk(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    Ok(Json(envelope(
        json!({ "risk": s.risk }),
        s.data_state.as_str(),
        s.risk.confidence,
        s.risk.factors.iter().flat_map(|f| f.evidence.clone()).take(8).collect(),
    )))
}

pub async fn narratives(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    let mut rows = Vec::new();
    if let Some(pool) = &st.pool {
        if let Ok(all) = db::list_narratives(pool).await {
            rows = all
                .into_iter()
                .filter(|n| {
                    n.get("token_ids")
                        .and_then(|v| v.as_array())
                        .map(|a| a.iter().any(|x| x.as_str() == Some(&id)))
                        .unwrap_or(false)
                })
                .collect();
        }
    }
    let evidence = if rows.is_empty() {
        s.token.narratives.clone()
    } else {
        vec!["narratives table".into()]
    };
    Ok(Json(envelope(
        json!({
            "declared": s.token.narratives,
            "detected": rows,
            "mention_velocity": "MISSING"
        }),
        s.data_state.as_str(),
        s.scores.confidence,
        evidence,
    )))
}

pub async fn genome(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    Ok(Json(envelope(
        json!({ "genome": s.genome }),
        s.data_state.as_str(),
        s.scores.confidence,
        vec!["token genome dimensions".into()],
    )))
}

pub async fn claims(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    let derived = memecoin_os_core::claims::from_snapshot(&s);
    let persisted = if let Some(pool) = &st.pool {
        db::list_claims(pool, &id).await.unwrap_or_default()
    } else {
        vec![]
    };
    Ok(Json(envelope(
        json!({
            "claims": derived,
            "persisted": persisted,
            "persisted_count": persisted.len(),
            "note": "Live claims are derived from the current snapshot. persisted is the last stored row per claim_id. INSUFFICIENT is not a quiet market. SUPPORTS is not SAFE. VERIFIED ≠ SAFE."
        }),
        s.data_state.as_str(),
        s.scores.confidence,
        vec!["derived from live snapshot layers".into(), "persisted on refresh when jobs.claims_persist is on".into()],
    )))
}

pub async fn similar(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    let others: Vec<_> = snaps.values().cloned().collect();
    Ok(Json(envelope(
        json!({
            "similar": twin_similarity::rank_against(&s, &others),
            "disclaimer": "Structural similarity. Not a claim that performance will repeat."
        }),
        s.data_state.as_str(),
        s.scores.confidence,
        vec!["genome + chain overlap".into()],
    )))
}

pub async fn history(
    State(st): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<RangeQuery>,
) -> Result<Json<Value>, StatusCode> {
    let days = match q.range.as_deref().unwrap_or("30d") {
        "7d" => 7,
        "90d" => 90,
        _ => 30,
    };
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    let points = if let Some(pool) = &st.pool {
        db::history(pool, &id, days).await.map_err(|_| StatusCode::BAD_GATEWAY)?
    } else {
        vec![]
    };
    Ok(Json(envelope(
        json!({ "token_id": id, "range_days": days, "points": points, "simulated_excluded": true }),
        s.data_state.as_str(),
        s.scores.confidence,
        vec!["market_snapshots history".into()],
    )))
}

pub async fn asset(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_or_404(&snaps, &id)?;
    Ok(Json(envelope(
        asset_card(&s),
        s.data_state.as_str(),
        s.scores.confidence,
        vec![format!("snapshot {}", s.as_of)],
    )))
}

pub async fn health(State(st): State<AppState>) -> Json<Value> {
    let data = crate::status_payload(&st).await;
    let freshness = if data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        "live"
    } else {
        "unknown"
    };
    Json(envelope(
        data,
        freshness,
        1.0,
        vec!["process health".into()],
    ))
}

pub(crate) fn asset_card(s: &TokenSnapshot) -> Value {
    let organic = if s.social.data_state.present() {
        json!(s.social.organicness)
    } else {
        json!("MISSING")
    };
    let momentum = if s.market.data_state.present() {
        json!(s.market.change_24h_pct)
    } else {
        json!("MISSING")
    };
    json!({
        "id": s.token.id,
        "symbol": s.token.symbol,
        "name": s.token.name,
        "chain": s.token.primary_chain,
        "contract": s.token.contract,
        "health": s.scores.value,
        "risk": s.risk.score,
        "risk_level": s.risk.level,
        "momentum_24h_pct": momentum,
        "organicness": organic,
        "narrative": s.token.narratives.first().cloned().unwrap_or_else(|| "MISSING".into()),
        "liquidity_usd": if s.liquidity.data_state.present() { json!(s.liquidity.liquidity_usd) } else { json!("MISSING") },
        "holders": if s.onchain.data_state.present() { json!(s.onchain.holders) } else { json!("MISSING") },
        "updated": s.as_of,
        "widget_url": format!("https://memecoin-os.web.app/embed/{}/", s.token.id)
    })
}
