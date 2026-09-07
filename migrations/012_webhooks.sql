CREATE TABLE IF NOT EXISTS webhook_endpoints (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind            TEXT NOT NULL,
    url             TEXT NOT NULL,
    chat_id         TEXT,
    token_id        TEXT REFERENCES tokens(id) ON DELETE CASCADE,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    digest_daily    BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS alert_configs (
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    threshold       JSONB NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (token_id, kind)
);

CREATE TABLE IF NOT EXISTS alert_fingerprints (
    fingerprint     TEXT PRIMARY KEY,
    alert_id        UUID NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
    token_id        TEXT NOT NULL,
    kind            TEXT NOT NULL,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS alert_deliveries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id        UUID NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
    webhook_id      UUID NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
    attempted_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    ok              BOOLEAN NOT NULL,
    status          INT,
    detail          TEXT
);

CREATE INDEX IF NOT EXISTS idx_alert_deliveries_alert
    ON alert_deliveries (alert_id, attempted_at DESC);
