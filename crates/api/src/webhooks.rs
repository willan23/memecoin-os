use crate::db;
use memecoin_os_core::models::{Alert, WebhookEndpoint};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn deliver(
    pool: &PgPool,
    client: &reqwest::Client,
    hooks: &[WebhookEndpoint],
    alert: &Alert,
) {
    let alert_uuid: Uuid = alert.id.parse().unwrap_or_else(|_| Uuid::new_v4());
    for hook in hooks.iter().filter(|h| h.enabled) {
        if let Some(tid) = &hook.token_id {
            if tid != &alert.token_id {
                continue;
            }
        }
        let text = format!(
            "[{}] {} — {}\n{}\n{}",
            alert.severity.to_uppercase(),
            alert.kind,
            alert.title,
            alert.body,
            alert.evidence.join(" · ")
        );
        let (ok, status, detail) = match hook.kind.as_str() {
            "telegram" => post_telegram(client, &hook.url, hook.chat_id.as_deref(), &text).await,
            _ => post_discord(client, &hook.url, &text).await,
        };
        if let Err(e) = db::log_delivery(
            pool,
            alert_uuid,
            hook.id.parse().unwrap_or_else(|_| Uuid::new_v4()),
            ok,
            status,
            detail.as_deref(),
        )
        .await
        {
            tracing::warn!(error = %e, "failed to log webhook delivery");
        }
    }
}

pub async fn post_discord(
    client: &reqwest::Client,
    url: &str,
    text: &str,
) -> (bool, Option<i32>, Option<String>) {
    match client
        .post(url)
        .json(&serde_json::json!({ "content": text }))
        .send()
        .await
    {
        Ok(r) => {
            let status = r.status().as_u16() as i32;
            (r.status().is_success(), Some(status), None)
        }
        Err(e) => (false, None, Some(e.to_string())),
    }
}

pub async fn post_telegram(
    client: &reqwest::Client,
    url: &str,
    chat_id: Option<&str>,
    text: &str,
) -> (bool, Option<i32>, Option<String>) {
    let mut body = serde_json::json!({ "text": text });
    if let Some(id) = chat_id {
        body["chat_id"] = serde_json::Value::String(id.to_string());
    }
    match client.post(url).json(&body).send().await {
        Ok(r) => {
            let status = r.status().as_u16() as i32;
            (r.status().is_success(), Some(status), None)
        }
        Err(e) => (false, None, Some(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn discord_payload_hits_mock() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let received = std::sync::Arc::new(tokio::sync::Mutex::new(None::<String>));
        let rec2 = received.clone();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            hyper_util_read(stream, rec2).await;
        });
        let client = reqwest::Client::new();
        let url = format!("http://{addr}/hook");
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        let _ = post_discord(&client, &url, "hello from memecoin os").await;
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        let body = received.lock().await;
        assert!(body.as_ref().map(|s| s.contains("hello")).unwrap_or(true));
    }

    async fn hyper_util_read(
        mut stream: tokio::net::TcpStream,
        rec: std::sync::Arc<tokio::sync::Mutex<Option<String>>>,
    ) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap_or(0);
        let s = String::from_utf8_lossy(&buf[..n]).to_string();
        *rec.lock().await = Some(s);
        let _ = stream
            .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n")
            .await;
    }
}
