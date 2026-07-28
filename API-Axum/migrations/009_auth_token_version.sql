-- 009_auth_token_version.sql
-- Incrementing token_version invalidates every previously issued access and
-- refresh token for an account across all API replicas.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS token_version BIGINT NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_users_token_version ON users(id, token_version);

CREATE OR REPLACE FUNCTION bump_user_token_version_on_security_change()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.token_version = OLD.token_version THEN
        NEW.token_version := OLD.token_version + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS users_security_change_revokes_tokens ON users;
CREATE TRIGGER users_security_change_revokes_tokens
BEFORE UPDATE OF status, role, mfa_enabled, mfa_secret, password_hash ON users
FOR EACH ROW
WHEN (
    OLD.status IS DISTINCT FROM NEW.status
    OR OLD.role IS DISTINCT FROM NEW.role
    OR OLD.mfa_enabled IS DISTINCT FROM NEW.mfa_enabled
    OR OLD.mfa_secret IS DISTINCT FROM NEW.mfa_secret
    OR OLD.password_hash IS DISTINCT FROM NEW.password_hash
)
EXECUTE FUNCTION bump_user_token_version_on_security_change();
