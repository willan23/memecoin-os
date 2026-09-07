use crate::db;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn enqueue(pool: &PgPool, tenant_id: Uuid, kind: &str, payload: serde_json::Value) -> Result<Uuid, sqlx::Error> {
    db::enqueue_work(pool, tenant_id, kind, &payload).await
}

pub async fn claim_and_run(pool: &PgPool) -> Result<Option<String>, sqlx::Error> {
    let Some(job) = db::claim_work(pool).await? else {
        return Ok(None);
    };
    let result = if known_kind(&job.kind) {
        Ok(())
    } else {
        Err(format!("unknown kind {}", job.kind))
    };
    match result {
        Ok(()) => {
            db::complete_work(pool, job.id, None).await?;
            Ok(Some(job.kind))
        }
        Err(e) => {
            db::complete_work(pool, job.id, Some(&e)).await?;
            Ok(Some(format!("failed:{}", job.kind)))
        }
    }
}

pub fn spawn(pool: PgPool) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            tick.tick().await;
            if !db::flag_enabled(&pool, "jobs.queue").await {
                continue;
            }
            match claim_and_run(&pool).await {
                Ok(Some(kind)) => tracing::debug!(kind, "queue job"),
                Ok(None) => {}
                Err(e) => tracing::warn!(error = %e, "queue claim failed"),
            }
        }
    });
}

fn known_kind(kind: &str) -> bool {
    matches!(kind, "refresh_token" | "domain_event" | "research")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_kinds_are_allow_listed() {
        assert!(known_kind("research"));
        assert!(!known_kind("execute_trade"));
    }
}
