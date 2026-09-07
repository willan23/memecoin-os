-- Phase 3: discovery, verification, narratives, genome clusters.

CREATE TABLE IF NOT EXISTS discovered_tokens (
    chain_id        TEXT NOT NULL,
    address         TEXT NOT NULL,
    symbol          TEXT,
    name            TEXT,
    pair_address    TEXT,
    dex             TEXT,
    liquidity_usd   DOUBLE PRECISION,
    via_token       TEXT NOT NULL,
    status          TEXT NOT NULL,
    score           DOUBLE PRECISION NOT NULL,
    evidence        JSONB NOT NULL DEFAULT '[]'::jsonb,
    auto_verified   BOOLEAN NOT NULL DEFAULT FALSE,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (chain_id, address)
);

CREATE TABLE IF NOT EXISTS verification_reports (
    token_id            TEXT PRIMARY KEY REFERENCES tokens(id) ON DELETE CASCADE,
    level               TEXT NOT NULL,
    declared_level      TEXT NOT NULL,
    reasons             JSONB NOT NULL,
    evidence            JSONB NOT NULL,
    last_verified_at    TIMESTAMPTZ NOT NULL,
    disclaimer          TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS narratives (
    id              TEXT PRIMARY KEY,
    label           TEXT NOT NULL,
    state           TEXT NOT NULL,
    confidence      DOUBLE PRECISION NOT NULL,
    token_ids       JSONB NOT NULL,
    evidence        JSONB NOT NULL,
    mention_velocity TEXT NOT NULL DEFAULT 'missing',
    unique_accounts  TEXT NOT NULL DEFAULT 'missing',
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS genome_clusters (
    cluster_id      TEXT PRIMARY KEY,
    label           TEXT NOT NULL,
    token_ids       JSONB NOT NULL,
    confidence      DOUBLE PRECISION NOT NULL,
    evidence        JSONB NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('jobs.discovery', TRUE, 'Dex pair counterparts only'),
    ('jobs.verification', TRUE, 'Evidence levels; never auto community-verified'),
    ('jobs.narratives', TRUE, 'Registry + weak name tags; social stays MISSING'),
    ('jobs.genome_clusters', TRUE, 'Genome + narrative + chain; not name')
ON CONFLICT (key) DO NOTHING;
