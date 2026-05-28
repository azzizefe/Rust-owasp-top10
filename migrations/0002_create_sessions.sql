-- migrations/0002_create_sessions.sql

CREATE TABLE sessions (
    token      TEXT PRIMARY KEY,         -- kriptografik rastgele
    user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
