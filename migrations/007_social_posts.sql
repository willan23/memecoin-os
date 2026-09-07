CREATE TABLE IF NOT EXISTS social_posts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    platform        TEXT NOT NULL,
    external_id     TEXT NOT NULL,
    posted_at       TIMESTAMPTZ NOT NULL,
    author_key      TEXT,
    text            TEXT,
    lang            TEXT,
    engagement      INT,
    bot_score       DOUBLE PRECISION,
    organic         BOOLEAN,
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (platform, external_id)
);

CREATE INDEX IF NOT EXISTS idx_social_posts_token_time
    ON social_posts (token_id, posted_at DESC);
