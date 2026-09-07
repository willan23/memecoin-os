CREATE INDEX IF NOT EXISTS idx_market_snapshots_token_time
    ON market_snapshots (token_id, as_of DESC);

CREATE INDEX IF NOT EXISTS idx_liquidity_snapshots_token_time
    ON liquidity_snapshots (token_id, as_of DESC);

CREATE INDEX IF NOT EXISTS idx_onchain_snapshots_token_time
    ON onchain_snapshots (token_id, as_of DESC);

CREATE INDEX IF NOT EXISTS idx_social_snapshots_token_time
    ON social_snapshots (token_id, as_of DESC);

CREATE INDEX IF NOT EXISTS idx_development_snapshots_token_time
    ON development_snapshots (token_id, as_of DESC);

CREATE INDEX IF NOT EXISTS idx_scores_token_time
    ON scores (token_id, as_of DESC);

CREATE INDEX IF NOT EXISTS idx_risk_reports_token_time
    ON risk_reports (token_id, as_of DESC);

CREATE INDEX IF NOT EXISTS idx_alerts_token_time
    ON alerts (token_id, fired_at DESC);
