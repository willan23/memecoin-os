use crate::db::{self, ProjectProfile};
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use memecoin_os_core::models::TokenDefinition;
use serde::Deserialize;
use serde_json::{json, Value};

const DISCLAIMER: &str =
    "PROJECT CLAIM is a label, not VERIFIED. VERIFIED ≠ SAFE. Paying never changes a score.";

#[derive(Deserialize, Default)]
pub struct ClaimBody {
    pub claimant_label: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct ProfileBody {
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub discord: Option<String>,
    pub github: Option<String>,
    pub docs: Option<String>,
    pub roadmap: Option<String>,
}

fn merge_url(
    incoming: &Option<String>,
    previous: Option<&str>,
) -> Result<Option<String>, (StatusCode, String)> {
    match incoming {
        Some(v) => official_url(Some(v)),
        None => Ok(previous.filter(|s| !s.is_empty()).map(|s| s.to_string())),
    }
}

fn official_url(raw: Option<&str>) -> Result<Option<String>, (StatusCode, String)> {
    let Some(t) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if t.len() > 500 {
        return Err((StatusCode::BAD_REQUEST, "url too long".into()));
    }
    let lower = t.to_ascii_lowercase();
    if !lower.starts_with("https://") {
        return Err((StatusCode::BAD_REQUEST, "official links must be https".into()));
    }
    if lower.contains("javascript:") || lower.contains("data:") {
        return Err((StatusCode::BAD_REQUEST, "invalid url".into()));
    }
    Ok(Some(t.to_string()))
}

fn official_from(
    def: &TokenDefinition,
    row: Option<&ProjectProfile>,
    observed: &[memecoin_os_core::models::OfficialLink],
) -> Value {
    json!({
        "website": pick(row.and_then(|r| r.website.as_deref()), def.website.as_deref(), find_link(observed, "website")),
        "twitter": pick(row.and_then(|r| r.twitter.as_deref()), def.socials.twitter.as_deref(), find_link(observed, "twitter")),
        "telegram": pick(row.and_then(|r| r.telegram.as_deref()), def.socials.telegram.as_deref(), find_link(observed, "telegram")),
        "discord": pick(row.and_then(|r| r.discord.as_deref()), def.socials.discord.as_deref(), find_link(observed, "discord")),
        "github": pick(row.and_then(|r| r.github.as_deref()), def.socials.github.as_deref(), find_link(observed, "github")),
        "docs": row.and_then(|r| r.docs.clone()).unwrap_or_else(|| "MISSING".into()),
        "roadmap": row.and_then(|r| r.roadmap.clone()).unwrap_or_else(|| "MISSING".into()),
        "observed": observed
    })
}

fn find_link<'a>(links: &'a [memecoin_os_core::models::OfficialLink], kind: &str) -> Option<&'a str> {
    links
        .iter()
        .find(|l| l.kind.eq_ignore_ascii_case(kind))
        .map(|l| l.url.as_str())
}

fn pick(profile: Option<&str>, registry: Option<&str>, observed: Option<&str>) -> Value {
    if let Some(v) = profile.filter(|s| !s.is_empty()) {
        return json!(v);
    }
    if let Some(v) = registry.filter(|s| !s.is_empty()) {
        return json!(v);
    }
    if let Some(v) = observed.filter(|s| !s.is_empty()) {
        return json!(v);
    }
    json!("MISSING")
}

fn claim_json(row: Option<&ProjectProfile>) -> Value {
    match row {
        Some(r) if r.claim_status == "claimed" => json!({
            "status": "claimed",
            "label": "PROJECT CLAIM",
            "claimant_label": r.claimant_label,
            "claimed_at": r.claimed_at
        }),
        Some(r) => json!({
            "status": r.claim_status,
            "label": r.claim_status.to_uppercase(),
            "claimant_label": r.claimant_label,
            "claimed_at": r.claimed_at
        }),
        None => json!({
            "status": "unclaimed",
            "label": "UNCLAIMED",
            "claimant_label": null,
            "claimed_at": null
        }),
    }
}

pub async fn list_projects(State(st): State<AppState>) -> Json<Value> {
    let snaps = st.snapshots.read().await;
    let mut claims = std::collections::HashMap::new();
    if let Some(pool) = &st.pool {
        if let Ok(rows) = db::list_project_claim_status(pool).await {
            for (id, status) in rows {
                claims.insert(id, status);
            }
        }
    }
    let mut projects: Vec<Value> = snaps
        .values()
        .map(|s| {
            let status = claims
                .get(&s.token.id)
                .cloned()
                .unwrap_or_else(|| "unclaimed".into());
            json!({
                "id": s.token.id,
                "symbol": s.token.symbol,
                "name": s.token.name,
                "claim_status": status,
                "verification": s.token.verification,
                "health": s.scores.value,
                "risk": s.risk.score,
                "data_state": s.data_state
            })
        })
        .collect();
    projects.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });
    Json(json!({
        "projects": projects,
        "disclaimer": DISCLAIMER
    }))
}

pub async fn get_project(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snaps.get(&id).cloned().ok_or(StatusCode::NOT_FOUND)?;
    drop(snaps);
    let reg = st.registry.read().await;
    let def = reg.get(&id).map_err(|_| StatusCode::NOT_FOUND)?.clone();
    drop(reg);
    let row = if let Some(pool) = &st.pool {
        db::get_project_profile(pool, &id).await.ok().flatten()
    } else {
        None
    };
    let computed = if let Some(pool) = &st.pool {
        db::get_verification(pool, &id).await.ok().flatten()
    } else {
        None
    };
    Ok(Json(json!({
        "token_id": id,
        "symbol": s.token.symbol,
        "name": s.token.name,
        "claim": claim_json(row.as_ref()),
        "official": official_from(&def, row.as_ref(), &s.liquidity.official_links),
        "verification": {
            "declared": s.token.verification,
            "computed": computed.as_ref().and_then(|v| v.get("level")).cloned().unwrap_or(json!("unverified")),
            "disclaimer": "VERIFIED ≠ SAFE. Computed from public evidence. Claim does not upgrade this."
        },
        "health": s.scores.value,
        "risk": s.risk.score,
        "data_state": s.data_state,
        "disclaimer": DISCLAIMER
    })))
}

pub async fn claim(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ClaimBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    crate::auth::deny_unless_operator(&headers)?;
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    ensure_token_row(&st, pool, &id).await?;
    if let Ok(Some(existing)) = db::get_project_profile(pool, &id).await {
        if existing.claim_status == "claimed" {
            return Err((StatusCode::CONFLICT, "already claimed".into()));
        }
    }
    let label = body
        .claimant_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(80).collect::<String>());
    if let Some(l) = &label {
        if l.contains('<') || l.contains('>') {
            return Err((StatusCode::BAD_REQUEST, "invalid claimant label".into()));
        }
    }
    let row = db::upsert_project_claim(pool, &id, "claimed", label.as_deref())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(json!({
        "claim": claim_json(Some(&row)),
        "verification_unchanged": true,
        "score_unchanged": true,
        "disclaimer": DISCLAIMER
    })))
}

pub async fn unclaim(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    crate::auth::deny_unless_operator(&headers)?;
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    ensure_token_row(&st, pool, &id).await?;
    let row = db::upsert_project_claim(pool, &id, "withdrawn", None)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(json!({
        "claim": claim_json(Some(&row)),
        "verification_unchanged": true,
        "score_unchanged": true,
        "disclaimer": DISCLAIMER
    })))
}

pub async fn patch_profile(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ProfileBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    crate::auth::deny_unless_operator(&headers)?;
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    ensure_token_row(&st, pool, &id).await?;
    let existing = db::get_project_profile(pool, &id).await.ok().flatten();
    let website = merge_url(&body.website, existing.as_ref().and_then(|r| r.website.as_deref()))?;
    let twitter = merge_url(&body.twitter, existing.as_ref().and_then(|r| r.twitter.as_deref()))?;
    let telegram = merge_url(&body.telegram, existing.as_ref().and_then(|r| r.telegram.as_deref()))?;
    let discord = merge_url(&body.discord, existing.as_ref().and_then(|r| r.discord.as_deref()))?;
    let github = merge_url(&body.github, existing.as_ref().and_then(|r| r.github.as_deref()))?;
    let docs = merge_url(&body.docs, existing.as_ref().and_then(|r| r.docs.as_deref()))?;
    let roadmap = merge_url(&body.roadmap, existing.as_ref().and_then(|r| r.roadmap.as_deref()))?;
    let row = db::upsert_project_links(
        pool,
        &id,
        website.as_deref(),
        twitter.as_deref(),
        telegram.as_deref(),
        discord.as_deref(),
        github.as_deref(),
        docs.as_deref(),
        roadmap.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let mut def = {
        let reg = st.registry.read().await;
        reg.get(&id).map_err(|_| (StatusCode::NOT_FOUND, "unknown token".into()))?.clone()
    };
    if body.website.is_some() {
        def.website = website.clone();
    }
    if body.twitter.is_some() {
        def.socials.twitter = twitter.clone();
    }
    if body.telegram.is_some() {
        def.socials.telegram = telegram.clone();
    }
    if body.github.is_some() {
        def.socials.github = github.clone();
    }
    if body.discord.is_some() {
        def.socials.discord = discord.clone();
    }
    db::upsert_token(pool, &def)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    st.registry
        .write()
        .await
        .insert_runtime(def.clone())
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    if let Some(s) = st.snapshots.write().await.get_mut(&id) {
        s.token.website = def.website.clone();
    }

    Ok(Json(json!({
        "official": official_from(&def, Some(&row), &[]),
        "claim": claim_json(Some(&row)),
        "verification_unchanged": true,
        "score_unchanged": true,
        "disclaimer": DISCLAIMER
    })))
}

async fn ensure_token_row(
    st: &AppState,
    pool: &sqlx::PgPool,
    id: &str,
) -> Result<(), (StatusCode, String)> {
    if !st.snapshots.read().await.contains_key(id) {
        return Err((StatusCode::NOT_FOUND, "unknown token".into()));
    }
    let def = st
        .registry
        .read()
        .await
        .get(id)
        .map_err(|_| (StatusCode::NOT_FOUND, "unknown token".into()))?
        .clone();
    db::upsert_token(pool, &def)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(())
}

fn catalog_row(kind: &str, label: &str, live: bool, reason: &str) -> Value {
    json!({
        "kind": kind,
        "label": label,
        "availability": if live { "live" } else { "missing" },
        "reason": reason
    })
}

pub async fn project_alerts(State(st): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let snaps = st.snapshots.read().await;
    let s = snaps.get(&id).cloned().ok_or(StatusCode::NOT_FOUND)?;
    drop(snaps);
    let mut alerts = Vec::new();
    let mut hooks = Vec::new();
    if let Some(pool) = &st.pool {
        if let Ok(rows) = db::load_alerts_for_token(pool, &id, 50).await {
            alerts = rows;
        }
        if let Ok(all) = db::list_webhooks(pool).await {
            hooks = all
                .into_iter()
                .filter(|h| h.token_id.as_deref() == Some(&id) || h.token_id.is_none())
                .collect::<Vec<_>>();
        }
    }
    if alerts.is_empty() {
        alerts = st.engine.alerts_with_prev(&s, None);
    }
    let social_live = s.social.data_state.present();
    let onchain_live = s.onchain.data_state.present();
    let liq_live = s.liquidity.data_state.present();
    Ok(Json(json!({
        "token_id": id,
        "alerts": alerts,
        "catalog": [
            catalog_row("liquidity_anomaly", "Liquidity anomaly", liq_live, if liq_live { "live liquidity snapshot" } else { "liquidity MISSING" }),
            catalog_row("holder_concentration_change", "Holder concentration change", onchain_live, if onchain_live { "live holder snapshot" } else { "holders MISSING" }),
            catalog_row("community_growth_anomaly", "Community growth anomaly", social_live, "Unique accounts 24h — licensed firehose only"),
            catalog_row("risk_signal", "Risk signal", s.risk.confidence > 0.0, "High/critical band from public evidence"),
            catalog_row("risk_change", "Risk band change", s.risk.confidence > 0.0, "Compared to previous snapshot"),
            catalog_row("narrative_acceleration", "Narrative acceleration", false, "mention_velocity MISSING without a social licence"),
            catalog_row("contract_event", "Contract event", false, "No verified contract-event indexer on this token"),
            catalog_row("exchange_listing_change", "Exchange listing change", false, "No listing feed connected"),
            catalog_row("social_anomaly", "Social anomaly", social_live, "Organicness / bot — licensed firehose only"),
        ],
        "webhooks": hooks,
        "disclaimer": "Project alerts are detection only. Not buy signals. Missing sources stay MISSING."
    })))
}
