-- Server-side sessions; cookie holds opaque token, DB stores SHA-256 hash.
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id),
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS sessions_token_hash_unique
    ON sessions (token_hash)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS sessions_user_id_idx
    ON sessions (user_id)
    WHERE deleted_at IS NULL;
