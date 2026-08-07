-- Migration 016: durable, single-use MFA login challenges.
-- PostgreSQL is authoritative so a challenge works across API replicas and
-- cannot be replayed after successful verification or too many failed attempts.

CREATE TABLE IF NOT EXISTS auth_mfa_challenges (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  VARCHAR(64) NOT NULL UNIQUE,
    expires_at  BIGINT      NOT NULL,
    consumed_at BIGINT,
    attempts    SMALLINT    NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 5),
    created_at  BIGINT      NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    CHECK (expires_at > created_at)
);

CREATE INDEX IF NOT EXISTS idx_auth_mfa_challenges_active
    ON auth_mfa_challenges (user_id, expires_at)
    WHERE consumed_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_auth_mfa_challenges_retention
    ON auth_mfa_challenges (expires_at);
