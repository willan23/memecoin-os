CREATE TABLE IF NOT EXISTS job_runs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_name        TEXT NOT NULL,
    started_at      TIMESTAMPTZ NOT NULL,
    finished_at     TIMESTAMPTZ,
    ok              BOOLEAN NOT NULL DEFAULT FALSE,
    lag_ms          INT,
    tokens_ok       INT NOT NULL DEFAULT 0,
    tokens_err      INT NOT NULL DEFAULT 0,
    error           TEXT,
    spans           JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_job_runs_name_started
    ON job_runs (job_name, started_at DESC);

CREATE TABLE IF NOT EXISTS provider_health (
    provider        TEXT PRIMARY KEY,
    last_ok_at      TIMESTAMPTZ,
    last_error_at   TIMESTAMPTZ,
    last_status     INT,
    error_count     INT NOT NULL DEFAULT 0,
    rate_limited    INT NOT NULL DEFAULT 0,
    latency_ms      INT,
    detail          TEXT
);
