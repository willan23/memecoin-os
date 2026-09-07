-- Default tenant, API key labels, alert rules, extra flags.

INSERT INTO tenants (slug, name)
VALUES ('default', 'Local operator')
ON CONFLICT (slug) DO NOTHING;

ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS name TEXT NOT NULL DEFAULT 'operator';
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS alert_rules (
    kind        TEXT PRIMARY KEY,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    reason      TEXT,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO alert_rules (kind, enabled, reason) VALUES
    ('price_conflict', TRUE, 'Provider price disagreement'),
    ('data_freshness', TRUE, 'Stale market or liquidity'),
    ('liquidity_anomaly', TRUE, 'Liquidity drop'),
    ('score_change', TRUE, 'Health delta'),
    ('risk_change', TRUE, 'Risk band change'),
    ('development_spike', TRUE, 'Commit spike'),
    ('development_silence', TRUE, 'Quiet repository')
ON CONFLICT (kind) DO NOTHING;

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('provider.holders', TRUE, 'Ethplorer / Etherscan holder counts'),
    ('provider.social', FALSE, 'Social firehose — blocked until licence'),
    ('jobs.digest', TRUE, 'Optional daily digest')
ON CONFLICT (key) DO NOTHING;
