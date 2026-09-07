use sqlx::PgPool;
use std::path::{Path, PathBuf};

const FILES: &[&str] = &[
    "001_init.sql",
    "002_snapshots.sql",
    "003_jobs_health.sql",
    "004_events.sql",
    "005_briefings.sql",
    "006_holders_whales.sql",
    "007_social_posts.sql",
    "008_auth_prep.sql",
    "009_feature_flags.sql",
    "010_schema_migrations.sql",
    "011_historical_queries.sql",
    "012_webhooks.sql",
    "013_ops_complete.sql",
    "014_intelligence_foundation.sql",
    "015_onchain_indexer.sql",
    "016_ecosystem_intelligence.sql",
    "017_research_reports.sql",
    "018_scale_foundation.sql",
    "019_digital_twin.sql",
    "020_next_wave.sql",
    "021_project_claims.sql",
    "022_project_alerts.sql",
    "023_community_connect.sql",
    "024_claims_baselines.sql",
];

pub fn split_sql(sql: &str) -> Vec<String> {
    let without_line_comments: String = sql
        .lines()
        .filter(|l| !l.trim().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let chars: Vec<char> = without_line_comments.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\'' {
            cur.push(ch);
            if in_quote && chars.get(i + 1) == Some(&'\'') {
                cur.push('\'');
                i += 2;
                continue;
            }
            in_quote = !in_quote;
        } else if ch == ';' && !in_quote {
            let t = cur.trim().to_string();
            if !t.is_empty() {
                parts.push(t);
            }
            cur.clear();
        } else {
            cur.push(ch);
        }
        i += 1;
    }
    let t = cur.trim().to_string();
    if !t.is_empty() {
        parts.push(t);
    }
    parts
}

pub fn migrations_dir() -> PathBuf {
    if let Ok(p) = std::env::var("MIGRATIONS_PATH") {
        return PathBuf::from(p);
    }
    let from_manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
    if from_manifest.exists() {
        return from_manifest;
    }
    PathBuf::from("migrations")
}

pub async fn run(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_lock(872134)")
        .execute(pool)
        .await?;
    let result = run_inner(pool).await;
    let _ = sqlx::query("SELECT pg_advisory_unlock(872134)")
        .execute(pool)
        .await;
    result
}

async fn run_inner(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            filename TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )",
    )
    .execute(pool)
    .await?;

    let tokens_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'tokens')",
    )
    .fetch_one(pool)
    .await?;
    if tokens_exists {
        sqlx::query(
            "INSERT INTO schema_migrations (filename) VALUES ('001_init.sql') ON CONFLICT DO NOTHING",
        )
        .execute(pool)
        .await?;
    }

    let dir = migrations_dir();
    for file in FILES {
        let applied: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM schema_migrations WHERE filename = $1)",
        )
        .bind(file)
        .fetch_one(pool)
        .await?;
        if applied {
            continue;
        }
        let path = dir.join(file);
        let sql = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| sqlx::Error::Protocol(format!("read {}: {e}", path.display()).into()))?;
        for stmt in split_sql(&sql) {
            sqlx::query(&stmt).execute(pool).await?;
        }
        sqlx::query("INSERT INTO schema_migrations (filename) VALUES ($1) ON CONFLICT DO NOTHING")
            .bind(file)
            .execute(pool)
            .await?;
        tracing::info!(file, "applied migration");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_statements() {
        let parts = split_sql("CREATE TABLE a (id int);\nCREATE TABLE b (id int);");
        assert_eq!(parts.len(), 2);
        let commented = split_sql("-- registry; this schema is durable\nCREATE TABLE a (id int);");
        assert_eq!(commented, vec!["CREATE TABLE a (id int)"]);
        let quoted = split_sql("INSERT INTO f (reason) VALUES ('a; b');");
        assert_eq!(quoted, vec!["INSERT INTO f (reason) VALUES ('a; b')"]);
    }
}
