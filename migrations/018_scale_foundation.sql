-- Phase 5: tenants/plans, metering, queue, object meta, SSO users, time-series prune.
-- Token intelligence stays shared public data. Isolation is for keys, usage, queue, members.

ALTER TABLE tenants ADD COLUMN IF NOT EXISTS plan TEXT NOT NULL DEFAULT 'free';
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'active';

CREATE TABLE IF NOT EXISTS usage_meters (
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    metric          TEXT NOT NULL,
    window_start    DATE NOT NULL,
    used            BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (tenant_id, metric, window_start)
);

CREATE TABLE IF NOT EXISTS work_queue (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    available_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    locked_until    TIMESTAMPTZ,
    attempts        INT NOT NULL DEFAULT 0,
    last_error      TEXT,
    done_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_work_queue_claim
    ON work_queue (available_at)
    WHERE done_at IS NULL;

CREATE TABLE IF NOT EXISTS object_blobs (
    hash            TEXT PRIMARY KEY,
    provider        TEXT NOT NULL,
    bytes           INT NOT NULL,
    path            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    subject         TEXT NOT NULL,
    email           TEXT,
    role            TEXT NOT NULL DEFAULT 'viewer' REFERENCES roles(id),
    last_login      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, subject)
);

CREATE TABLE IF NOT EXISTS sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      TEXT NOT NULL UNIQUE,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS oidc_states (
    state           TEXT PRIMARY KEY,
    nonce           TEXT NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_market_snapshots_asof_brin
    ON market_snapshots USING BRIN (as_of);
CREATE INDEX IF NOT EXISTS idx_liquidity_snapshots_asof_brin
    ON liquidity_snapshots USING BRIN (as_of);

-- RLS: isolate when app.tenant_id is set; local operator (unset) sees all.
ALTER TABLE usage_meters ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS usage_meters_tenant ON usage_meters;
CREATE POLICY usage_meters_tenant ON usage_meters
    USING (
        current_setting('app.tenant_id', true) IS NULL
        OR current_setting('app.tenant_id', true) = ''
        OR tenant_id::text = current_setting('app.tenant_id', true)
    );

ALTER TABLE work_queue ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS work_queue_tenant ON work_queue;
CREATE POLICY work_queue_tenant ON work_queue
    USING (
        current_setting('app.tenant_id', true) IS NULL
        OR current_setting('app.tenant_id', true) = ''
        OR tenant_id::text = current_setting('app.tenant_id', true)
    );

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('jobs.queue', TRUE, 'Postgres work queue + SKIP LOCKED'),
    ('jobs.raw_store', FALSE, 'Content-hash snapshot objects — off by default'),
    ('plane.billing', TRUE, 'Metering; enforce only if METERING_ENFORCE=true'),
    ('plane.sso', TRUE, 'OIDC login when OIDC_ISSUER is set')
ON CONFLICT (key) DO NOTHING;
