-- Phase 4: grounded research reports. RAG does not replace this table.

CREATE TABLE IF NOT EXISTS research_reports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT REFERENCES tokens(id) ON DELETE SET NULL,
    question        TEXT NOT NULL,
    markdown        TEXT NOT NULL,
    payload         JSONB NOT NULL,
    confidence      DOUBLE PRECISION NOT NULL,
    model_id        TEXT NOT NULL,
    generated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_research_reports_token_time
    ON research_reports (token_id, generated_at DESC);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('plane.ai', TRUE, 'Grounded research agents — no live LLM invention'),
    ('jobs.research', TRUE, 'Persist reports on POST /v1/research')
ON CONFLICT (key) DO NOTHING;
