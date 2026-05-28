-- migrations/0003_create_posts.sql

CREATE TABLE posts (
    id         BIGSERIAL PRIMARY KEY,
    author_id  BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content    TEXT NOT NULL,            -- stored XSS yüzeyi
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
