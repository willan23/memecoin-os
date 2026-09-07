-- MemeCoin OS transactional schema (PostgreSQL).
-- The MVP API boots with in-memory + YAML registry; this schema is the
-- durable model for Phase 2 persistence.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE tenants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE tokens (
    id              TEXT PRIMARY KEY,
    schema_version  INT NOT NULL DEFAULT 1,
    symbol          TEXT NOT NULL,
    name            TEXT NOT NULL,
    status          TEXT NOT NULL,
    verification    TEXT NOT NULL,
    definition      JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE observations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id),
    kind            TEXT NOT NULL,
    source          TEXT NOT NULL,
    provider        TEXT NOT NULL,
    observed_at     TIMESTAMPTZ NOT NULL,
    ingested_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    freshness_secs  INT,
    confidence      DOUBLE PRECISION NOT NULL,
    validation      TEXT NOT NULL,
    payload         JSONB NOT NULL,
    raw_reference   TEXT
);

CREATE INDEX idx_observations_token_kind_time
    ON observations (token_id, kind, observed_at DESC);

CREATE TABLE scores (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id),
    algorithm       TEXT NOT NULL,
    algorithm_version TEXT NOT NULL,
    as_of           TIMESTAMPTZ NOT NULL,
    score           DOUBLE PRECISION NOT NULL,
    breakdown       JSONB NOT NULL,
    evidence        JSONB NOT NULL,
    UNIQUE (token_id, algorithm, algorithm_version, as_of)
);

CREATE TABLE alerts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id),
    kind            TEXT NOT NULL,
    severity        TEXT NOT NULL,
    title           TEXT NOT NULL,
    body            TEXT NOT NULL,
    evidence        JSONB NOT NULL,
    fired_at        TIMESTAMPTZ NOT NULL,
    acknowledged_at TIMESTAMPTZ
);

CREATE TABLE audit_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor           TEXT NOT NULL,
    action          TEXT NOT NULL,
    resource        TEXT NOT NULL,
    reason          TEXT,
    source          TEXT,
    model           TEXT,
    prompt_version  TEXT,
    tool            TEXT,
    result          TEXT,
    payload         JSONB NOT NULL,
    immutable       BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE api_keys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    prefix          TEXT NOT NULL,
    hash            TEXT NOT NULL,
    role            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at      TIMESTAMPTZ
);
