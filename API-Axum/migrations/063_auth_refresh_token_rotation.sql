-- Migration 063: rotação do refresh token com detecção de reuso.
--
-- Cada refresh token vale uma única renovação. O JWT nunca é guardado: só o
-- SHA-256 dele. Os tokens de um mesmo login formam uma família (family_id);
-- reapresentar um token já trocado, fora da tolerância de corrida entre abas,
-- revoga a família inteira. PostgreSQL é a fonte de verdade entre réplicas.

CREATE TABLE IF NOT EXISTS auth_refresh_tokens (
    token_hash  VARCHAR(64) PRIMARY KEY,
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    family_id   UUID        NOT NULL,
    created_at  BIGINT      NOT NULL,
    expires_at  BIGINT      NOT NULL,
    consumed_at BIGINT,
    revoked_at  BIGINT,
    CHECK (expires_at > created_at)
);

CREATE INDEX IF NOT EXISTS idx_auth_refresh_tokens_family
    ON auth_refresh_tokens (family_id);

CREATE INDEX IF NOT EXISTS idx_auth_refresh_tokens_user
    ON auth_refresh_tokens (user_id);

CREATE INDEX IF NOT EXISTS idx_auth_refresh_tokens_retention
    ON auth_refresh_tokens (expires_at);
