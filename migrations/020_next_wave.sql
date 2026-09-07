-- NEXT_WAVE: white-label, watchlist isolation, temporal graph edges.
-- Token snapshots stay shared. No Stripe rows. No Neo4j.

ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS branding JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE TABLE IF NOT EXISTS tenant_watchlist (
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    token_id    TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, token_id)
);

CREATE INDEX IF NOT EXISTS idx_watchlist_token ON tenant_watchlist (token_id);

ALTER TABLE entity_relationships
    ADD COLUMN IF NOT EXISTS valid_from TIMESTAMPTZ NOT NULL DEFAULT now();
ALTER TABLE entity_relationships
    ADD COLUMN IF NOT EXISTS valid_to TIMESTAMPTZ;
ALTER TABLE entity_relationships
    ADD COLUMN IF NOT EXISTS evidence_ids JSONB NOT NULL DEFAULT '[]'::jsonb;

CREATE INDEX IF NOT EXISTS idx_rel_valid
    ON entity_relationships (ecosystem_id, valid_from, valid_to);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('plane.white_label', TRUE, 'Tenant branding JSON — https logos only'),
    ('plane.watchlist', TRUE, 'Watchlist isolation, not forked Twins'),
    ('digital_twin_webgl', TRUE, 'WebGL projection of Twin graph'),
    ('provider.solana', TRUE, 'SolanaAdapter — live only if SOLANA_RPC_URL')
ON CONFLICT (key) DO NOTHING;
