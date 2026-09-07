CREATE TABLE IF NOT EXISTS holder_snapshots (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id                TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    as_of                   TIMESTAMPTZ NOT NULL,
    chain_id                TEXT NOT NULL,
    holders                 BIGINT NOT NULL,
    active_holders_30d      BIGINT,
    new_holders_7d          BIGINT,
    top10_concentration_pct DOUBLE PRECISION,
    top50_concentration_pct DOUBLE PRECISION,
    whale_holders           BIGINT,
    payload                 JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_holder_snapshots_token_time
    ON holder_snapshots (token_id, as_of DESC);

CREATE TABLE IF NOT EXISTS whale_movements (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    occurred_at     TIMESTAMPTZ NOT NULL,
    chain_id        TEXT NOT NULL,
    tx_hash         TEXT,
    direction       TEXT NOT NULL,
    amount_usd      DOUBLE PRECISION,
    wallet          TEXT,
    exchange        TEXT,
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_whale_movements_token_time
    ON whale_movements (token_id, occurred_at DESC);
