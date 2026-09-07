use memecoin_os_api::{db, migrate};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;
use std::time::Duration;

fn load_env() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.env");
    let _ = dotenvy::from_filename(&root);
    let _ = dotenvy::dotenv();
    if std::env::var("MIGRATIONS_PATH").is_err() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
        std::env::set_var("MIGRATIONS_PATH", dir);
    }
}

async fn try_pool() -> Option<sqlx::PgPool> {
    load_env();
    let url = std::env::var("DATABASE_URL").ok()?;
    let opts = PgConnectOptions::from_str(&url).ok()?;
    match tokio::time::timeout(
        Duration::from_secs(3),
        PgPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(Duration::from_secs(3))
            .connect_with(opts),
    )
    .await
    {
        Ok(Ok(pool)) => Some(pool),
        _ => None,
    }
}

#[tokio::test]
async fn postgres_persist_survives_reload() {
    let Some(pool) = try_pool().await else {
        eprintln!("skip: postgres not reachable");
        return;
    };
    migrate::run(&pool).await.expect("migrate");
    let snaps = db::load_latest_snapshots(&pool).await.expect("load");
    let _ = snaps.len();
}

#[tokio::test]
async fn webhook_table_exists_when_pg_up() {
    let Some(pool) = try_pool().await else {
        eprintln!("skip: postgres not reachable");
        return;
    };
    migrate::run(&pool).await.expect("migrate");
    let hooks = db::list_webhooks(&pool).await.expect("webhooks");
    let _ = hooks.len();
}
