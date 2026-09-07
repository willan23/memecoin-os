use crate::db;
use crate::product;
use crate::tenant::TenantCtx;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::Extension;
use axum::http::StatusCode;
use axum::Json;
use chrono::{Duration, Utc};
use memecoin_os_core::models::TokenSnapshot;
use memecoin_os_core::research;
use memecoin_os_core::twin::{self, TwinHistoryPoint};
use memecoin_os_core::{ecosystem, twin_graph, twin_sim, twin_similarity};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TwinQuery {
    pub at: Option<String>,
    pub ago: Option<String>,
}

#[derive(Deserialize)]
pub struct SimBody {
    pub ecosystem_id: String,
    #[serde(default)]
    pub liquidity_pct: f64,
    #[serde(default)]
    pub holder_growth_pct: f64,
    #[serde(default)]
    pub whale_selling_pct: f64,
    #[serde(default)]
    pub developer_pct: f64,
}

#[derive(Deserialize)]
pub struct CopilotBody {
    pub question: String,
    pub ecosystem_id: Option<String>,
}

fn snap_for<'a>(st: &'a std::collections::HashMap<String, TokenSnapshot>, id: &str) -> Option<&'a TokenSnapshot> {
    st.get(id)
}

pub fn ecosystems_from(snaps: &std::collections::HashMap<String, TokenSnapshot>) -> Vec<ecosystem::Ecosystem> {
    let mut rows: Vec<_> = snaps.values().map(ecosystem::from_snapshot).collect();
    rows.sort_by(|a, b| b.intelligence_score.partial_cmp(&a.intelligence_score).unwrap());
    rows
}

pub async fn list_ecosystems(
    State(st): State<AppState>,
    ctx: Option<Extension<TenantCtx>>,
    Query(q): Query<product::ScopeQuery>,
) -> Json<serde_json::Value> {
    let scope = product::watchlist_scope(&st, ctx.as_ref().map(|e| &e.0), q.scope.as_deref()).await;
    let snaps = st.snapshots.read().await;
    let filtered = product::filter_snaps(&snaps, &scope);
    let rows = ecosystems_from(&filtered);
    Json(serde_json::json!({
        "ecosystems": rows,
        "scope": q.scope.unwrap_or_else(|| if crate::auth::auth_required() { "watchlist".into() } else { "all".into() }),
        "note": "One candidate ecosystem per registry token. auto_verified is always false. VERIFIED ≠ SAFE. Watchlist filters lists only."
    }))
}

pub async fn get_ecosystem(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let eco = ecosystem::from_snapshot(s);
    let twin = twin::from_snapshot(&eco.id, s);
    Ok(Json(serde_json::json!({
        "ecosystem": eco,
        "twin": twin,
        "disclaimer": "Digital Twin of observed state. Not a trading signal."
    })))
}

pub async fn twin_now(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::json!({
        "twin": twin::from_snapshot(&id, s),
        "mode": "current"
    })))
}

pub async fn twin_state(
    State(st): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<TwinQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let current = twin::from_snapshot(&id, s);
    let at = parse_at(&q);
    if at.is_none() {
        return Ok(Json(serde_json::json!({"twin": current, "mode": "current"})));
    }
    let at = at.unwrap();
    let hist = load_hist(&st, &id).await;
    let reconstructed = twin::reconstruct_at(&current, &hist, at);
    Ok(Json(serde_json::json!({
        "twin": reconstructed,
        "mode": "historical",
        "requested_at": at,
        "look_ahead": false
    })))
}

pub async fn twin_history(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    if st.snapshots.read().await.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let hist = load_hist(&st, &id).await;
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "points": hist,
        "simulated_excluded": true,
        "note": "Observations only. No future rows."
    })))
}

pub async fn twin_graph(
    State(st): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<TwinQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let mut g = twin_graph::from_snapshot(&id, s);
    if let Some(at) = parse_at(&q) {
        if let Some(pool) = &st.pool {
            if let Ok(edges) = db::relationships_at(pool, &id, at).await {
                if !edges.is_empty() {
                    g.relationships = edges;
                } else {
                    g.relationships = twin_graph::valid_at(&g.relationships, at);
                }
            }
        } else {
            g.relationships = twin_graph::valid_at(&g.relationships, at);
        }
        g.note = format!("Graph at {}. Expired edges omitted. Not Neo4j.", at);
    }
    Ok(Json(serde_json::json!(g)))
}

pub async fn twin_evolution(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let current = twin::from_snapshot(&id, s);
    let hist = load_hist(&st, &id).await;
    let at = Utc::now() - Duration::hours(24);
    let before = twin::reconstruct_at(&current, &hist, at);
    let changed = twin::what_changed(&before, &current);
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "window": "24h",
        "before": before,
        "after": current,
        "what_changed": changed,
        "note": "Observed sequence. Causality is not assumed."
    })))
}

pub async fn twin_genome(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "genome": s.genome,
        "lifecycle": ecosystem::classify(s),
        "note": "Token genome reused as ecosystem fingerprint until multi-token graphs exist."
    })))
}

pub async fn twin_lifecycle(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let phase = ecosystem::classify(s);
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "phase": phase,
        "trigger": "classified from live snapshot fields only",
        "evidence": [
            format!("liquidity={}", s.liquidity.data_state.as_str()),
            format!("holders={}", s.onchain.data_state.as_str()),
            format!("health={:.1}", s.scores.value)
        ],
        "confidence": s.scores.confidence,
        "note": "Social MISSING does not promote GROWTH. VERIFIED ≠ SAFE."
    })))
}

pub async fn twin_anomalies(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let mut sequence = Vec::new();
    if s.liquidity.data_state.present() && s.liquidity.lp_change_7d_pct.abs() >= 10.0 {
        sequence.push(serde_json::json!({"step": "liquidity", "observed": s.liquidity.lp_change_7d_pct}));
    }
    if s.risk.confidence > 0.4 && s.risk.score >= 60.0 {
        sequence.push(serde_json::json!({"step": "risk", "observed": s.risk.score}));
    }
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "observed_sequence": sequence,
        "causality": "not_asserted",
        "note": "Empty sequence means no evidenced anomaly — not 'zero risk'."
    })))
}

pub async fn twin_evidence(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let t = twin::from_snapshot(&id, s);
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "claims": twin::claims(&t),
        "investigate_next": twin::investigate_next(&t)
    })))
}

pub async fn twin_similar(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let others: Vec<_> = snaps.values().cloned().collect();
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "similar": twin_similarity::rank_against(s, &others),
        "disclaimer": "Structural similarity. Not a claim that performance will repeat."
    })))
}

pub async fn create_simulation(
    State(st): State<AppState>,
    Json(body): Json<SimBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = {
        let snaps = st.snapshots.read().await;
        let s = snap_for(&snaps, &body.ecosystem_id).ok_or(StatusCode::NOT_FOUND)?;
        let t = twin::from_snapshot(&body.ecosystem_id, s);
        twin_sim::run(
            s,
            &t,
            twin_sim::Assumptions {
                liquidity_pct: body.liquidity_pct,
                holder_growth_pct: body.holder_growth_pct,
                whale_selling_pct: body.whale_selling_pct,
                developer_pct: body.developer_pct,
            },
        )
    };
    let mut id: Option<Uuid> = None;
    if let Some(pool) = &st.pool {
        if let Ok(v) = serde_json::to_value(&result) {
            let assumptions = serde_json::json!({
                "liquidity_pct": body.liquidity_pct,
                "holder_growth_pct": body.holder_growth_pct,
                "whale_selling_pct": body.whale_selling_pct,
                "developer_pct": body.developer_pct
            });
            id = db::insert_simulation(pool, &body.ecosystem_id, &assumptions, &v).await.ok();
        }
    }
    Ok(Json(serde_json::json!({
        "id": id,
        "result": result
    })))
}

pub async fn get_simulation(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let uuid: Uuid = id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let Some(pool) = &st.pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    match db::get_simulation(pool, uuid).await {
        Ok(Some(v)) => Ok(Json(v)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}

pub async fn copilot(State(st): State<AppState>, Json(body): Json<CopilotBody>) -> Json<serde_json::Value> {
    let snaps = st.snapshots.read().await;
    let list: Vec<_> = snaps.values().cloned().collect();
    let focus = body.ecosystem_id.as_deref();
    let report = crate::llm::polish(research::run(&body.question, &list, focus)).await;
    let next = focus
        .and_then(|id| snaps.get(id).map(|s| (id, s)))
        .map(|(id, s)| twin::investigate_next(&twin::from_snapshot(id, s)))
        .unwrap_or_default();
    Json(serde_json::json!({
        "research": report,
        "investigate_next": next,
        "execute": false,
        "note": "Copilot reads Twin/snapshots. It does not call providers or execute trades."
    }))
}

fn parse_at(q: &TwinQuery) -> Option<chrono::DateTime<Utc>> {
    if let Some(at) = &q.at {
        return chrono::DateTime::parse_from_rfc3339(at)
            .ok()
            .map(|d| d.with_timezone(&Utc));
    }
    match q.ago.as_deref() {
        Some("1h") => Some(Utc::now() - Duration::hours(1)),
        Some("6h") => Some(Utc::now() - Duration::hours(6)),
        Some("24h") | Some("1d") => Some(Utc::now() - Duration::hours(24)),
        Some("7d") => Some(Utc::now() - Duration::days(7)),
        Some("30d") => Some(Utc::now() - Duration::days(30)),
        Some("90d") => Some(Utc::now() - Duration::days(90)),
        _ => None,
    }
}

async fn load_hist(st: &AppState, id: &str) -> Vec<TwinHistoryPoint> {
    let Some(pool) = &st.pool else {
        return vec![];
    };
    match db::history(pool, id, 90).await {
        Ok(points) => points
            .into_iter()
            .map(|p| TwinHistoryPoint {
                t: p.t,
                liquidity_usd: p.liquidity_usd,
                volume_24h_usd: p.volume_24h_usd,
                health: p.health,
                risk: p.risk,
                data_state: p.data_state,
            })
            .collect(),
        Err(_) => vec![],
    }
}

pub async fn persist_twin(pool: &sqlx::PgPool, s: &TokenSnapshot) {
    let eco = ecosystem::from_snapshot(s);
    if db::upsert_ecosystem(pool, &eco).await.is_err() {
        return;
    }
    let t = twin::from_snapshot(&eco.id, s);
    if let Ok(v) = serde_json::to_value(&t) {
        let _ = db::insert_twin_state(pool, &eco.id, &v).await;
    }
    let g = twin_graph::from_snapshot(&eco.id, s);
    let _ = db::replace_relationships(pool, &eco.id, &g.relationships).await;
}

pub fn emit(st: &AppState, kind: &str, ecosystem_id: &str) {
    let payload = serde_json::json!({
        "type": kind,
        "ecosystem_id": ecosystem_id,
        "at": Utc::now()
    })
    .to_string();
    let _ = st.event_bus.send(payload);
}

pub async fn event_stream(
    State(st): State<AppState>,
) -> axum::response::sse::Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>>
{
    use axum::response::sse::{Event, KeepAlive, Sse};
    use tokio_stream::wrappers::BroadcastStream;
    use tokio_stream::StreamExt;
    let rx = st.event_bus.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|m| match m {
        Ok(data) => Some(Ok(Event::default().event("message").data(data))),
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default().text("ping"))
}

#[derive(Deserialize)]
pub struct IdsQuery {
    pub ids: String,
}

#[derive(Deserialize)]
pub struct ReplayQuery {
    pub range: Option<String>,
}

pub async fn compare_ecosystems(
    State(st): State<AppState>,
    Query(q): Query<IdsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ids: Vec<_> = q
        .ids
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if ids.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let snaps = st.snapshots.read().await;
    let a = snap_for(&snaps, &ids[0]).ok_or(StatusCode::NOT_FOUND)?;
    let b = snap_for(&snaps, &ids[1]).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::json!({
        "a": { "ecosystem": ecosystem::from_snapshot(a), "twin": twin::from_snapshot(&ids[0], a) },
        "b": { "ecosystem": ecosystem::from_snapshot(b), "twin": twin::from_snapshot(&ids[1], b) },
        "similarity": twin_similarity::compare(a, b),
        "disclaimer": "Structural Twin compare. Not a prediction that one repeats the other."
    })))
}

pub async fn twin_replay(
    State(st): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<ReplayQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snap_for(&snaps, &id).ok_or(StatusCode::NOT_FOUND)?;
    let hours = match q.range.as_deref() {
        Some("7d") => 24 * 7,
        Some("30d") => 24 * 30,
        _ => 24,
    };
    let hist = load_hist(&st, &id).await;
    let frames = memecoin_os_core::twin_replay::frames(s, &hist, hours);
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "range_hours": hours,
        "frames": frames,
        "note": "Only persisted observations and timeline events. Empty means no evidenced history, not zero activity."
    })))
}

pub async fn whale_behaviour(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    if st.snapshots.read().await.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let rows = match &st.pool {
        Some(pool) => db::list_whale_movements(pool, &id, 80).await.unwrap_or_default(),
        None => vec![],
    };
    Ok(Json(serde_json::json!({
        "ecosystem_id": id,
        "behaviour": memecoin_os_core::whale_behaviour::from_json_rows(&rows),
    })))
}

pub async fn daily_briefing(State(st): State<AppState>) -> Json<serde_json::Value> {
    let snaps = st.snapshots.read().await;
    let mut items: Vec<_> = snaps
        .values()
        .map(|s| {
            let eco = ecosystem::from_snapshot(s);
            serde_json::json!({
                "ecosystem_id": eco.id,
                "name": eco.name,
                "lifecycle": eco.lifecycle_phase,
                "headline": s.briefing.headline,
                "changes": s.briefing.changes,
                "risks": s.briefing.risks,
                "health": s.scores.value,
                "confidence": s.briefing.confidence,
                "data_state": s.data_state,
            })
        })
        .collect();
    items.sort_by(|a, b| {
        let bh = b.get("health").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let ah = a.get("health").and_then(|v| v.as_f64()).unwrap_or(0.0);
        bh.partial_cmp(&ah).unwrap_or(std::cmp::Ordering::Equal)
    });
    Json(serde_json::json!({
        "title": "Daily ecosystem intelligence",
        "items": items,
        "note": "From Twin snapshots. Social remains MISSING without a firehose licence. EXECUTE off."
    }))
}
