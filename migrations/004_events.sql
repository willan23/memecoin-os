CREATE TABLE IF NOT EXISTS token_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    kind            TEXT NOT NULL,
    title           TEXT NOT NULL,
    delta           TEXT,
    confidence      DOUBLE PRECISION NOT NULL DEFAULT 0,
    source          TEXT NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_token_events_token_time
    ON token_events (token_id, occurred_at DESC);
