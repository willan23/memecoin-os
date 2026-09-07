use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

fn memory_hits() -> &'static Mutex<HashMap<String, (i64, Instant)>> {
    static HITS: OnceLock<Mutex<HashMap<String, (i64, Instant)>>> = OnceLock::new();
    HITS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Process-local limiter when Redis is unset (Cloud Run first deploy).
pub fn memory_rate_limit(ip: &str, path: &str, limit: i64, window_secs: i64) -> bool {
    let key = format!("{ip}:{path}");
    let now = Instant::now();
    let window = Duration::from_secs(window_secs.max(1) as u64);
    let Ok(mut map) = memory_hits().lock() else {
        return true;
    };
    map.retain(|_, (_, t)| now.duration_since(*t) < window);
    let entry = map.entry(key).or_insert((0, now));
    if now.duration_since(entry.1) >= window {
        *entry = (0, now);
    }
    entry.0 += 1;
    entry.0 <= limit
}

pub async fn connect(url: &str) -> Option<ConnectionManager> {
    let client = redis::Client::open(url).ok()?;
    let fut = ConnectionManager::new(client);
    match tokio::time::timeout(Duration::from_secs(3), fut).await {
        Ok(Ok(mut mgr)) => {
            let ping: Result<String, _> = redis::cmd("PING").query_async(&mut mgr).await;
            if ping.ok().as_deref() == Some("PONG") {
                Some(mgr)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub async fn ping_ms(mgr: &mut ConnectionManager) -> Option<u64> {
    let start = std::time::Instant::now();
    let ping: Result<String, _> = redis::cmd("PING").query_async(mgr).await;
    ping.ok().map(|_| start.elapsed().as_millis() as u64)
}

pub async fn cache_get(mgr: &mut ConnectionManager, key: &str) -> Option<String> {
    mgr.get(key).await.ok()
}

pub async fn cache_set(mgr: &mut ConnectionManager, key: &str, val: &str, ttl_secs: u64) {
    let _: Result<(), _> = mgr.set_ex(key, val, ttl_secs).await;
}

pub async fn rate_limit(mgr: &mut ConnectionManager, key: &str, limit: i64, window_secs: i64) -> bool {
    let count: i64 = mgr.incr(key, 1).await.unwrap_or(limit + 1);
    if count == 1 {
        let _: Result<(), _> = mgr.expire(key, window_secs).await;
    }
    count <= limit
}

pub async fn acquire_lease(mgr: &mut ConnectionManager, key: &str, ttl_secs: i64) -> bool {
    let ok: Result<bool, _> = redis::cmd("SET")
        .arg(key)
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(ttl_secs)
        .query_async(mgr)
        .await;
    ok.unwrap_or(false)
}

pub async fn quota_ok(mgr: &mut ConnectionManager, token_id: &str, max_per_hour: i64) -> bool {
    let hour = chrono::Utc::now().format("%Y%m%d%H").to_string();
    let key = format!("quota:refresh:{token_id}:{hour}");
    rate_limit(mgr, &key, max_per_hour, 3600).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_limiter_caps() {
        let ip = format!("test-{}", std::process::id());
        assert!(memory_rate_limit(&ip, "/v1/overview", 2, 60));
        assert!(memory_rate_limit(&ip, "/v1/overview", 2, 60));
        assert!(!memory_rate_limit(&ip, "/v1/overview", 2, 60));
    }
}
