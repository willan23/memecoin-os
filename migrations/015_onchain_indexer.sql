-- Phase 2: transfer cursor, graph edges, clusters. Reuses whale_movements (006).

CREATE TABLE IF NOT EXISTS indexer_cursors (
    chain_id        TEXT NOT NULL,
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    last_block      BIGINT NOT NULL DEFAULT 0,
    last_run        TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_error      TEXT,
    source          TEXT NOT NULL DEFAULT 'none',
    PRIMARY KEY (chain_id, token_id)
);

CREATE TABLE IF NOT EXISTS graph_edges (
    chain_id        TEXT NOT NULL,
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    from_address    TEXT NOT NULL,
    to_address      TEXT NOT NULL,
    relation        TEXT NOT NULL,
    evidence        JSONB NOT NULL DEFAULT '[]'::jsonb,
    confidence      DOUBLE PRECISION NOT NULL,
    source          TEXT NOT NULL,
    tx_count        INT NOT NULL DEFAULT 1,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (chain_id, token_id, from_address, to_address, relation)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_whale_movements_tx_dir
    ON whale_movements (token_id, tx_hash, direction, wallet);

CREATE TABLE IF NOT EXISTS wallet_clusters (
    cluster_id      TEXT NOT NULL,
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    chain_id        TEXT NOT NULL,
    members         JSONB NOT NULL,
    edge_count      INT NOT NULL,
    confidence      DOUBLE PRECISION NOT NULL,
    evidence        JSONB NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (token_id, chain_id, cluster_id)
);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('jobs.whale_engine', TRUE, 'Classify indexed transfers only'),
    ('jobs.wallet_clusters', TRUE, 'Skip CEX/burn hubs; correlation ≠ identity')
ON CONFLICT (key) DO NOTHING;

UPDATE feature_flags
SET enabled = TRUE,
    reason = 'Phase 2 cursor — no-op without RPC_ETHEREUM or ETHERSCAN_API_KEY'
WHERE key = 'jobs.indexer';
