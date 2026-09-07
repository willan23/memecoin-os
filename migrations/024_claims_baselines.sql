-- Persist latest derived evidence claims. Extend baselines with MAD / percentiles.

ALTER TABLE metric_baselines ADD COLUMN IF NOT EXISTS mad DOUBLE PRECISION NOT NULL DEFAULT 0;
ALTER TABLE metric_baselines ADD COLUMN IF NOT EXISTS percentile_25 DOUBLE PRECISION;
ALTER TABLE metric_baselines ADD COLUMN IF NOT EXISTS percentile_75 DOUBLE PRECISION;

CREATE TABLE IF NOT EXISTS evidence_claims (
    token_id     TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    claim_id     TEXT NOT NULL,
    claim        TEXT NOT NULL,
    relation     TEXT NOT NULL,
    evidence     JSONB NOT NULL DEFAULT '[]'::jsonb,
    data_state   TEXT NOT NULL,
    confidence   DOUBLE PRECISION NOT NULL,
    as_of        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (token_id, claim_id)
);

CREATE INDEX IF NOT EXISTS idx_evidence_claims_relation
    ON evidence_claims (token_id, relation);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('jobs.claims_persist', TRUE, 'Persist derived SUPPORTS/CONTRADICTS/INSUFFICIENT on snapshot refresh')
ON CONFLICT (key) DO NOTHING;
