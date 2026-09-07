-- Click-to-connect Discord/Telegram + first-party community chat.
-- Chat is user-typed only. No seeded messages. Not a social firehose.

CREATE TABLE IF NOT EXISTS connect_states (
    state       TEXT PRIMARY KEY,
    kind        TEXT NOT NULL,
    token_id    TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS community_sessions (
    id          UUID PRIMARY KEY,
    handle      TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS community_messages (
    id          UUID PRIMARY KEY,
    room_id     TEXT NOT NULL,
    session_id  UUID NOT NULL REFERENCES community_sessions(id) ON DELETE CASCADE,
    handle      TEXT NOT NULL,
    body        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_community_messages_room
    ON community_messages (room_id, created_at DESC);

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('plane.community_chat', TRUE, 'First-party rooms. Empty until someone types. Not firehose.'),
    ('plane.connect_oauth', TRUE, 'Discord webhook.incoming + Telegram bot start when env is set')
ON CONFLICT (key) DO NOTHING;
