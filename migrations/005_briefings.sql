CREATE TABLE IF NOT EXISTS daily_briefings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    as_of           DATE NOT NULL,
    generated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    headline        TEXT NOT NULL,
    payload         JSONB NOT NULL,
    UNIQUE (token_id, as_of)
);
