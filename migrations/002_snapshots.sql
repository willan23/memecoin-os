CREATE TABLE IF NOT EXISTS token_snapshots (
    token_id    TEXT PRIMARY KEY REFERENCES tokens(id) ON DELETE CASCADE,
    as_of       TIMESTAMPTZ NOT NULL,
    data_state  TEXT NOT NULL,
    payload     JSONB NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market_snapshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    as_of           TIMESTAMPTZ NOT NULL,
    price_usd       DOUBLE PRECISION NOT NULL,
    market_cap_usd  DOUBLE PRECISION NOT NULL,
    volume_24h_usd  DOUBLE PRECISION NOT NULL,
    change_24h_pct  DOUBLE PRECISION NOT NULL,
    data_state      TEXT NOT NULL,
    payload         JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS liquidity_snapshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    as_of           TIMESTAMPTZ NOT NULL,
    liquidity_usd   DOUBLE PRECISION NOT NULL,
    pool_count      INT NOT NULL,
    spread_bps      DOUBLE PRECISION NOT NULL,
    data_state      TEXT NOT NULL,
    payload         JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS onchain_snapshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    as_of           TIMESTAMPTZ NOT NULL,
    holders         BIGINT NOT NULL,
    data_state      TEXT NOT NULL,
    payload         JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS social_snapshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    as_of           TIMESTAMPTZ NOT NULL,
    mentions_24h    BIGINT NOT NULL,
    data_state      TEXT NOT NULL,
    payload         JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS development_snapshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id        TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    as_of           TIMESTAMPTZ NOT NULL,
    commits_30d     BIGINT NOT NULL,
    data_state      TEXT NOT NULL,
    payload         JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS risk_reports (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id            TEXT NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    algorithm           TEXT NOT NULL,
    algorithm_version   TEXT NOT NULL,
    as_of               TIMESTAMPTZ NOT NULL,
    score               DOUBLE PRECISION NOT NULL,
    level               TEXT NOT NULL,
    payload             JSONB NOT NULL,
    UNIQUE (token_id, algorithm, algorithm_version, as_of)
);
