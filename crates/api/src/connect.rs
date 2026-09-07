use crate::db;
use crate::AppState;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

fn public_origin() -> String {
    std::env::var("PUBLIC_WEB_ORIGIN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://memecoin-os.web.app".into())
        .trim_end_matches('/')
        .to_string()
}

fn discord_configured() -> bool {
    env_nonempty("DISCORD_CLIENT_ID") && env_nonempty("DISCORD_CLIENT_SECRET")
}

fn telegram_configured() -> bool {
    env_nonempty("TELEGRAM_BOT_TOKEN") && env_nonempty("TELEGRAM_BOT_USERNAME")
}

fn env_nonempty(key: &str) -> bool {
    std::env::var(key).map(|v| !v.trim().is_empty()).unwrap_or(false)
}

fn env_val(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[derive(Deserialize)]
pub struct ConnectQuery {
    pub token_id: Option<String>,
}

#[derive(Deserialize)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub fn status_payload() -> Value {
    let origin = public_origin();
    json!({
        "discord": {
            "configured": discord_configured(),
            "flow": "oauth_webhook_incoming",
            "start": format!("{origin}/v1/connect/discord"),
            "note": if discord_configured() {
                "Click starts Discord authorize. You pick the channel. We store the webhook URL only.".into()
            } else {
                format!("Set DISCORD_CLIENT_ID and DISCORD_CLIENT_SECRET. Redirect URI must be {origin}/v1/connect/discord/callback")
            }
        },
        "telegram": {
            "configured": telegram_configured(),
            "flow": "bot_start",
            "start": format!("{origin}/v1/connect/telegram"),
            "bot_username": env_val("TELEGRAM_BOT_USERNAME"),
            "note": if telegram_configured() {
                "Click opens the bot. /start authorizes alert delivery to that chat."
            } else {
                "Set TELEGRAM_BOT_TOKEN and TELEGRAM_BOT_USERNAME. Then Telegram can POST /v1/connect/telegram/inbound"
            }
        },
        "disclaimer": "This connects YOUR room for alerts. It is not a firehose and not identity verification."
    })
}

pub async fn status() -> Json<Value> {
    Json(status_payload())
}

pub async fn discord_start(
    State(st): State<AppState>,
    Query(q): Query<ConnectQuery>,
) -> impl IntoResponse {
    if !discord_configured() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "discord oauth not configured",
                "need": ["DISCORD_CLIENT_ID", "DISCORD_CLIENT_SECRET"],
                "redirect_uri": format!("{}/v1/connect/discord/callback", public_origin())
            })),
        )
            .into_response();
    }
    let Some(pool) = &st.pool else {
        return (StatusCode::SERVICE_UNAVAILABLE, "postgres required").into_response();
    };
    let state = Uuid::new_v4().simple().to_string();
    if let Err(e) = db::insert_connect_state(pool, &state, "discord", q.token_id.as_deref()).await {
        return (StatusCode::BAD_GATEWAY, e.to_string()).into_response();
    }
    let client_id = env_val("DISCORD_CLIENT_ID").unwrap();
    let redirect = format!("{}/v1/connect/discord/callback", public_origin());
    let url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=webhook.incoming&state={}",
        urlencoding(&client_id),
        urlencoding(&redirect),
        urlencoding(&state)
    );
    Redirect::temporary(&url).into_response()
}

pub async fn discord_callback(
    State(st): State<AppState>,
    Query(q): Query<OAuthCallback>,
) -> impl IntoResponse {
    let origin = public_origin();
    if q.error.is_some() {
        return Redirect::temporary(&format!("{origin}/community/?connect=denied")).into_response();
    }
    let (Some(code), Some(state)) = (q.code.as_deref(), q.state.as_deref()) else {
        return Redirect::temporary(&format!("{origin}/community/?connect=missing_code")).into_response();
    };
    let Some(pool) = &st.pool else {
        return Redirect::temporary(&format!("{origin}/community/?connect=no_db")).into_response();
    };
    let row = match db::take_connect_state(pool, state).await {
        Ok(Some(r)) if r.kind == "discord" => r,
        _ => return Redirect::temporary(&format!("{origin}/community/?connect=bad_state")).into_response(),
    };
    match exchange_discord(code).await {
        Ok(webhook_url) => {
            if let Err(e) = db::insert_webhook(
                pool,
                "discord",
                &webhook_url,
                None,
                row.token_id.as_deref(),
                false,
            )
            .await
            {
                tracing::warn!(error = %e, "discord webhook persist failed");
                return Redirect::temporary(&format!("{origin}/community/?connect=discord_failed&reason=persist"))
                    .into_response();
            }
            let dest = match row.token_id {
                Some(id) => format!("{origin}/projects/{id}/?connected=discord"),
                None => format!("{origin}/community/?connected=discord"),
            };
            Redirect::temporary(&dest).into_response()
        }
        Err(reason) => {
            tracing::warn!(reason, "discord oauth exchange failed");
            Redirect::temporary(&format!(
                "{origin}/community/?connect=discord_failed&reason={reason}"
            ))
            .into_response()
        }
    }
}

fn webhook_url_ok(url: &str) -> bool {
    url.starts_with("https://discord.com/api/webhooks/")
        || url.starts_with("https://discordapp.com/api/webhooks/")
}

fn webhook_from_token_body(body: &Value) -> Option<String> {
    let wh = body.get("webhook")?;
    if let Some(url) = wh.get("url").and_then(|u| u.as_str()).filter(|u| webhook_url_ok(u)) {
        return Some(url.to_string());
    }
    let id = wh
        .get("id")
        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_u64().map(|n| n.to_string())))?;
    let token = wh.get("token").and_then(|t| t.as_str()).filter(|t| !t.is_empty())?;
    let url = format!("https://discord.com/api/webhooks/{id}/{token}");
    webhook_url_ok(&url).then_some(url)
}

async fn exchange_discord(code: &str) -> Result<String, String> {
    let client_id = env_val("DISCORD_CLIENT_ID").ok_or("missing_client")?;
    let client_secret = env_val("DISCORD_CLIENT_SECRET").ok_or("missing_secret")?;
    let redirect = format!("{}/v1/connect/discord/callback", public_origin());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|_| "http_client".to_string())?;
    let res = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect.as_str()),
        ])
        .send()
        .await
        .map_err(|_| "discord_unreachable".to_string())?;
    let status = res.status();
    let body: Value = res.json().await.map_err(|_| "discord_decode".to_string())?;
    if !status.is_success() {
        let err = body
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("token_http");
        // Common: invalid_client (bad secret), invalid_grant (code reuse), invalid_request (redirect mismatch)
        return Err(err.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(48).collect());
    }
    webhook_from_token_body(&body).ok_or_else(|| {
        let scope = body.get("scope").and_then(|s| s.as_str()).unwrap_or("");
        if scope.contains("webhook.incoming") {
            "no_webhook".into()
        } else {
            "scope_missing_webhook".into()
        }
    })
}

pub async fn telegram_start(
    State(st): State<AppState>,
    Query(q): Query<ConnectQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let username = env_val("TELEGRAM_BOT_USERNAME").ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "telegram bot not configured".into(),
    ))?;
    let Some(pool) = &st.pool else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "postgres required".into()));
    };
    let state = Uuid::new_v4().simple().to_string();
    db::insert_connect_state(pool, &state, "telegram", q.token_id.as_deref())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let bot = username.trim_start_matches('@');
    Ok(Json(json!({
        "url": format!("https://t.me/{bot}?start={state}"),
        "state": state,
        "note": "Open Telegram, tap Start. That chat will receive project alerts."
    })))
}

#[derive(Deserialize)]
pub struct TelegramUpdate {
    pub message: Option<TelegramMessage>,
}

#[derive(Deserialize)]
pub struct TelegramMessage {
    pub chat: TelegramChat,
    pub text: Option<String>,
}

#[derive(Deserialize)]
pub struct TelegramChat {
    pub id: i64,
}

pub async fn telegram_inbound(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(update): Json<TelegramUpdate>,
) -> StatusCode {
    if let Some(expected) = env_val("TELEGRAM_WEBHOOK_SECRET") {
        let got = headers
            .get("x-telegram-bot-api-secret-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if got != expected {
            return StatusCode::UNAUTHORIZED;
        }
    }
    let Some(pool) = &st.pool else {
        return StatusCode::SERVICE_UNAVAILABLE;
    };
    let Some(msg) = update.message else {
        return StatusCode::OK;
    };
    let text = msg.text.unwrap_or_default();
    let state = text
        .strip_prefix("/start")
        .unwrap_or("")
        .trim()
        .to_string();
    if state.is_empty() {
        return StatusCode::OK;
    }
    let Ok(Some(row)) = db::take_connect_state(pool, &state).await else {
        return StatusCode::OK;
    };
    if row.kind != "telegram" {
        return StatusCode::OK;
    }
    let Some(token) = env_val("TELEGRAM_BOT_TOKEN") else {
        return StatusCode::OK;
    };
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let _ = db::insert_webhook(
        pool,
        "telegram",
        &url,
        Some(&msg.chat.id.to_string()),
        row.token_id.as_deref(),
        false,
    )
    .await;
    StatusCode::OK
}

pub async fn register_telegram_webhook() {
    let Some(token) = env_val("TELEGRAM_BOT_TOKEN") else {
        return;
    };
    let url = format!("{}/v1/connect/telegram/inbound", public_origin());
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };
    let mut body = json!({ "url": url });
    if let Some(secret) = env_val("TELEGRAM_WEBHOOK_SECRET") {
        body["secret_token"] = json!(secret);
    }
    match client
        .post(format!("https://api.telegram.org/bot{token}/setWebhook"))
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => tracing::info!("telegram webhook registered"),
        Ok(r) => tracing::warn!(status = %r.status(), "telegram setWebhook failed"),
        Err(e) => tracing::warn!(error = %e, "telegram setWebhook error"),
    }
}

fn urlencoding(s: &str) -> String {
    s.bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => vec![b as char],
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}
