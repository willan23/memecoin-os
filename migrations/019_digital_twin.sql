-- Digital Twin: ecosystems composed from existing token snapshots.
-- Token intelligence stays the source. Twin is a state model, not a 3D store.

CREATE TABLE IF NOT EXISTS ecosystems (
    id                  TEXT PRIMARY KEY,
    slug                TEXT NOT NULL UNIQUE,
    name                TEXT NOT NULL,
    description         TEXT,
    primary_chain       TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'discovered',
    lifecycle_phase     TEXT NOT NULL DEFAULT 'discovered',
    discovery_score     DOUBLE PRECISION NOT NULL DEFAULT 0,
    intelligence_score  DOUBLE PRECISION NOT NULL DEFAULT 0,
    risk_score          DOUBLE PRECISION NOT NULL DEFAULT 0,
    confidence          DOUBLE PRECISION NOT NULL DEFAULT 0,
    verification_status TEXT NOT NULL DEFAULT 'unverified',
    auto_verified       BOOLEAN NOT NULL DEFAULT FALSE,
    first_seen          TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_updated        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ecosystem_members (
    ecosystem_id    TEXT NOT NULL REFERENCES ecosystems(id) ON DELETE CASCADE,
    entity_kind     TEXT NOT NULL,
    entity_id       TEXT NOT NULL,
    PRIMARY KEY (ecosystem_id, entity_kind, entity_id)
);

CREATE TABLE IF NOT EXISTS twin_states (
    ecosystem_id    TEXT NOT NULL REFERENCES ecosystems(id) ON DELETE CASCADE,
    as_of           TIMESTAMPTZ NOT NULL,
    payload         JSONB NOT NULL,
    PRIMARY KEY (ecosystem_id, as_of)
);

CREATE INDEX IF NOT EXISTS idx_twin_states_asof ON twin_states (ecosystem_id, as_of DESC);

CREATE TABLE IF NOT EXISTS entity_relationships (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ecosystem_id        TEXT NOT NULL REFERENCES ecosystems(id) ON DELETE CASCADE,
    source_entity       TEXT NOT NULL,
    target_entity       TEXT NOT NULL,
    relationship_type   TEXT NOT NULL,
    confidence          DOUBLE PRECISION NOT NULL,
    evidence            JSONB NOT NULL DEFAULT '[]'::jsonb,
    status              TEXT NOT NULL DEFAULT 'observed',
    first_seen          TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS twin_simulations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ecosystem_id    TEXT NOT NULL,
    assumptions     JSONB NOT NULL,
    result          JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('digital_twin', TRUE, 'Ecosystem Twin from snapshots'),
    ('knowledge_graph', TRUE, 'Relational graph in Postgres / memory'),
    ('ecosystem_lifecycle', TRUE, 'Evidence-only lifecycle'),
    ('ecosystem_similarity', TRUE, 'Genome + structure, not price'),
    ('simulation_engine', TRUE, 'What-if on Twin metrics — not a prediction')
ON CONFLICT (key) DO NOTHING;
