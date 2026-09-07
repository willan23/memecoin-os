-- Phase 1 foundation: domain events, pools, wallets, transfers, baselines.

CREATE TABLE IF NOT EXISTS domain_events (
    event_id        UUID PRIMARY KEY,
    event_type      TEXT NOT NULL,
    entity_id       TEXT NOT NULL,
    chain_id        TEXT,
    occurred_at     TIMESTAMPTZ NOT NULL,
    detected_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    source          TEXT NOT NULL,
    confidence      DOUBLE PRECISION NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    schema_version  INT NOT NULL DEFAULT 1,
    fingerprint     TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_domain_events_entity_time
    ON domain_events (entity_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_domain_events_type
    ON domain_events (event_type, occurred_at DESC);

CREATE TABLE IF NOT EXISTS discovered_pools (
    pair_address    TEXT NOT NULL,
    chain_id        TEXT NOT NULL,
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    dex             TEXT,
    liquidity_usd   DOUBLE PRECISION,
    price_usd       DOUBLE PRECISION,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (chain_id, pair_address)
);

CREATE INDEX IF NOT EXISTS idx_discovered_pools_token ON discovered_pools (token_id);

CREATE TABLE IF NOT EXISTS transfers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chain_id        TEXT NOT NULL,
    token_id        TEXT REFERENCES tokens(id) ON DELETE CASCADE,
    tx_hash         TEXT NOT NULL,
    block_number    BIGINT NOT NULL,
    from_address    TEXT NOT NULL,
    to_address      TEXT NOT NULL,
    amount_raw      TEXT NOT NULL,
    ingested_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (chain_id, tx_hash, from_address, to_address, amount_raw)
);

CREATE INDEX IF NOT EXISTS idx_transfers_token_block ON transfers (token_id, block_number DESC);

CREATE TABLE IF NOT EXISTS wallets (
    address         TEXT NOT NULL,
    chain_id        TEXT NOT NULL,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (chain_id, address)
);

CREATE TABLE IF NOT EXISTS wallet_profiles (
    address         TEXT NOT NULL,
    chain_id        TEXT NOT NULL,
    token_id        TEXT REFERENCES tokens(id) ON DELETE CASCADE,
    classification  TEXT NOT NULL,
    confidence      DOUBLE PRECISION NOT NULL,
    share_pct       DOUBLE PRECISION,
    evidence        JSONB NOT NULL DEFAULT '[]'::jsonb,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (chain_id, address, token_id)
);

CREATE TABLE IF NOT EXISTS metric_baselines (
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    metric          TEXT NOT NULL,
    "window"        TEXT NOT NULL,
    n               INT NOT NULL,
    mean            DOUBLE PRECISION NOT NULL,
    stddev          DOUBLE PRECISION NOT NULL,
    last_value      DOUBLE PRECISION NOT NULL,
    z_score         DOUBLE PRECISION,
    data_state      TEXT NOT NULL,
    evidence        JSONB NOT NULL DEFAULT '[]'::jsonb,
    as_of           TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (token_id, metric, "window")
);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('jobs.baselines', TRUE, 'Historical baselines from persisted snapshots'),
    ('jobs.pool_discovery', TRUE, 'Persist DexScreener pools'),
    ('jobs.indexer', FALSE, 'EVM transfer cursor — off until RPC + Phase 2'),
    ('plane.intelligence', TRUE, 'Phase 1 intelligence plane')
ON CONFLICT (key) DO NOTHING;
