-- Migration 015: e-mail verification codes for registration
-- Status canônico do usuário: pending_email_verification | active | ...

CREATE TABLE IF NOT EXISTS email_verification_codes (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash    VARCHAR(64)  NOT NULL,
    expires_at   BIGINT       NOT NULL,
    consumed_at  BIGINT,
    created_at   BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);

CREATE INDEX IF NOT EXISTS idx_email_verification_user
    ON email_verification_codes (user_id);

CREATE INDEX IF NOT EXISTS idx_email_verification_active
    ON email_verification_codes (user_id, expires_at)
    WHERE consumed_at IS NULL;

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS email_verified_at BIGINT;
