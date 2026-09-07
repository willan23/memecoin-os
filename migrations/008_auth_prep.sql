CREATE TABLE IF NOT EXISTS roles (
    id          TEXT PRIMARY KEY,
    description TEXT NOT NULL
);

INSERT INTO roles (id, description) VALUES
    ('platform_admin', 'Platform operator'),
    ('tenant_owner', 'Tenant owner'),
    ('admin', 'Tenant admin'),
    ('analyst', 'Read + analyze'),
    ('developer', 'API client developer'),
    ('community_manager', 'Alerts and briefing'),
    ('viewer', 'Read only'),
    ('api_client', 'Machine client')
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS tenant_members (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    subject     TEXT NOT NULL,
    role        TEXT NOT NULL REFERENCES roles(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, subject)
);

CREATE INDEX IF NOT EXISTS idx_api_keys_tenant ON api_keys (tenant_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys (prefix);
