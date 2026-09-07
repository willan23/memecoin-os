use crate::db;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

fn last_post() -> &'static Mutex<HashMap<Uuid, Instant>> {
    static MAP: OnceLock<Mutex<HashMap<Uuid, Instant>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn valid_handle(raw: &str) -> Result<String, (StatusCode, String)> {
    let h = raw.trim();
    if h.len() < 3 || h.len() > 20 {
        return Err((StatusCode::BAD_REQUEST, "handle must be 3–20 characters".into()));
    }
    if !h.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err((StatusCode::BAD_REQUEST, "handle is letters, numbers, underscore".into()));
    }
    let lower = h.to_ascii_lowercase();
    if matches!(lower.as_str(), "admin" | "operator" | "system" | "missing" | "verified") {
        return Err((StatusCode::BAD_REQUEST, "reserved handle".into()));
    }
    Ok(h.to_string())
}

fn valid_body(raw: &str) -> Result<String, (StatusCode, String)> {
    let t = raw.trim();
    if t.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "empty message".into()));
    }
    if t.chars().count() > 500 {
        return Err((StatusCode::BAD_REQUEST, "message too long".into()));
    }
    let lower = t.to_ascii_lowercase();
    if lower.contains("<script") || lower.contains("javascript:") {
        return Err((StatusCode::BAD_REQUEST, "rejected".into()));
    }
    Ok(t.to_string())
}

async fn valid_room(st: &AppState, room: &str) -> Result<String, (StatusCode, String)> {
    if room == "global" {
        return Ok(room.into());
    }
    if st.snapshots.read().await.contains_key(room) {
        return Ok(room.into());
    }
    Err((StatusCode::NOT_FOUND, "unknown room".into()))
}

#[derive(Deserialize)]
pub struct SessionBody {
    pub handle: String,
}

#[derive(Deserialize)]
pub struct PostBody {
    pub body: String,
}

pub async fn create_session(
    State(st): State<AppState>,
    Json(body): Json<SessionBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let handle = if body.handle.trim().is_empty() {
        guest_handle()
    } else {
        valid_handle(&body.handle)?
    };
    open_session(&st, &handle).await
}

fn guest_handle() -> String {
    format!("guest_{}", &Uuid::new_v4().simple().to_string()[..6])
}

async fn open_session(st: &AppState, handle: &str) -> Result<Json<Value>, (StatusCode, String)> {
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    let id = db::insert_community_session(pool, handle)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(json!({
        "session_id": id,
        "handle": handle,
        "live": true,
        "note": "Handle is a display name, not identity. Not VERIFIED."
    })))
}

pub async fn list_rooms(State(st): State<AppState>) -> Json<Value> {
    let snaps = st.snapshots.read().await;
    let mut counts = std::collections::HashMap::new();
    let mut totals = (0i64, 0i64);
    if let Some(pool) = &st.pool {
        if let Ok(rows) = db::community_message_counts(pool).await {
            for (id, all, day) in rows {
                counts.insert(id, (all, day));
            }
        }
        if let Ok(t) = db::community_totals(pool).await {
            totals = t;
        }
    }
    let mut rooms = vec![json!({
        "id": "global",
        "label": "Memecoins — general",
        "messages": counts.get("global").map(|c| c.0).unwrap_or(0),
        "messages_24h": counts.get("global").map(|c| c.1).unwrap_or(0)
    })];
    let mut tokens: Vec<_> = snaps.values().collect();
    tokens.sort_by(|a, b| a.token.symbol.cmp(&b.token.symbol));
    for s in tokens {
        let c = counts.get(&s.token.id).copied().unwrap_or((0, 0));
        rooms.push(json!({
            "id": s.token.id,
            "label": format!("{} · {}", s.token.symbol, s.token.name),
            "messages": c.0,
            "messages_24h": c.1
        }));
    }
    Json(json!({
        "live": true,
        "messages": totals.0,
        "messages_24h": totals.1,
        "rooms": rooms,
        "disclaimer": "First-party chat is live. Empty rooms are empty, not disconnected. Not a social firehose. Not financial advice."
    }))
}

pub async fn list_messages(
    State(st): State<AppState>,
    Path(room): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let room = valid_room(&st, &room).await?;
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    let messages = db::list_community_messages(pool, &room, 80)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(json!({ "room_id": room, "messages": messages })))
}

pub async fn post_message(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(room): Path<String>,
    Json(body): Json<PostBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let room = valid_room(&st, &room).await?;
    let text = valid_body(&body.body)?;
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    let sid = headers
        .get("x-community-session")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim();
    let (session, handle) = if sid.is_empty() {
        let h = guest_handle();
        let id = db::insert_community_session(pool, &h)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
        (id, h)
    } else {
        let session: Uuid = sid
            .parse()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "join with a handle first".into()))?;
        let handle = db::community_session_handle(pool, session)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
            .ok_or((StatusCode::UNAUTHORIZED, "unknown session".into()))?;
        (session, handle)
    };
    {
        let mut map = last_post().lock().unwrap();
        if let Some(prev) = map.get(&session) {
            if prev.elapsed() < Duration::from_secs(2) {
                return Err((StatusCode::TOO_MANY_REQUESTS, "slow down".into()));
            }
        }
        map.insert(session, Instant::now());
    }
    let mut row = db::insert_community_message(pool, &room, session, &handle, &text)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    if let Some(obj) = row.as_object_mut() {
        obj.insert("session_id".into(), json!(session));
    }
    let _ = st.event_bus.send(
        json!({
            "type": "community.message",
            "room_id": room,
            "message": row
        })
        .to_string(),
    );
    Ok(Json(row))
}
