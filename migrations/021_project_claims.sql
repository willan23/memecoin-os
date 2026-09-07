-- Project claim + official profile. Never auto-VERIFIED. Paying never changes a score.

CREATE TABLE IF NOT EXISTS project_profiles (
    token_id        TEXT PRIMARY KEY REFERENCES tokens(id) ON DELETE CASCADE,
    claim_status    TEXT NOT NULL DEFAULT 'unclaimed',
    claimant_label  TEXT,
    claimed_at      TIMESTAMPTZ,
    website         TEXT,
    twitter         TEXT,
    telegram        TEXT,
    discord         TEXT,
    github          TEXT,
    docs            TEXT,
    roadmap         TEXT,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_project_profiles_status
    ON project_profiles (claim_status);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('plane.project_claim', TRUE, 'Operator claim + official links. Not VERIFIED. Not a score.')
ON CONFLICT (key) DO NOTHING;
