CREATE TABLE IF NOT EXISTS feature_flags (
    key         TEXT PRIMARY KEY,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    reason      TEXT,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('provider.coingecko', TRUE, 'Public market API'),
    ('provider.dexscreener', TRUE, 'Public DEX pairs'),
    ('provider.github', TRUE, 'Public GitHub metadata'),
    ('jobs.snapshot_refresh', TRUE, 'Periodic live refresh'),
    ('jobs.briefing', TRUE, 'Daily briefing persist'),
    ('jobs.webhooks', TRUE, 'Outbound Discord/Telegram'),
    ('jobs.digest', TRUE, 'Optional daily digest')
ON CONFLICT (key) DO NOTHING;
