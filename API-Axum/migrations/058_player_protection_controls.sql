-- 058: proteção do jogador, KYC básico, recuperação de senha e suporte.
-- Valores monetários permanecem em centavos inteiros.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS date_of_birth DATE,
    ADD COLUMN IF NOT EXISTS over_18_declared_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS kyc_status VARCHAR(20) NOT NULL DEFAULT 'not_submitted',
    ADD COLUMN IF NOT EXISTS kyc_legal_name VARCHAR(160),
    ADD COLUMN IF NOT EXISTS kyc_tax_id_hash VARCHAR(64),
    ADD COLUMN IF NOT EXISTS kyc_tax_id_last4 VARCHAR(4),
    ADD COLUMN IF NOT EXISTS kyc_submitted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS kyc_reviewed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS kyc_reviewed_by UUID REFERENCES users(id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'users_kyc_status_check'
    ) THEN
        ALTER TABLE users ADD CONSTRAINT users_kyc_status_check
            CHECK (kyc_status IN ('not_submitted', 'pending', 'verified', 'rejected'));
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS uq_users_kyc_tax_id_hash
    ON users(kyc_tax_id_hash) WHERE kyc_tax_id_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_kyc_status ON users(kyc_status);

CREATE TABLE IF NOT EXISTS responsible_gaming_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    deposit_limit_daily_cents BIGINT CHECK (deposit_limit_daily_cents IS NULL OR deposit_limit_daily_cents >= 0),
    deposit_limit_weekly_cents BIGINT CHECK (deposit_limit_weekly_cents IS NULL OR deposit_limit_weekly_cents >= 0),
    deposit_limit_monthly_cents BIGINT CHECK (deposit_limit_monthly_cents IS NULL OR deposit_limit_monthly_cents >= 0),
    loss_limit_daily_cents BIGINT CHECK (loss_limit_daily_cents IS NULL OR loss_limit_daily_cents >= 0),
    loss_limit_weekly_cents BIGINT CHECK (loss_limit_weekly_cents IS NULL OR loss_limit_weekly_cents >= 0),
    loss_limit_monthly_cents BIGINT CHECK (loss_limit_monthly_cents IS NULL OR loss_limit_monthly_cents >= 0),
    play_time_limit_daily_minutes INTEGER CHECK (play_time_limit_daily_minutes IS NULL OR play_time_limit_daily_minutes BETWEEN 15 AND 1440),
    pending_limits JSONB,
    pending_limits_effective_at TIMESTAMPTZ,
    self_excluded_until TIMESTAMPTZ,
    self_excluded_permanently BOOLEAN NOT NULL DEFAULT FALSE,
    self_exclusion_started_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS responsible_gaming_daily_activity (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_date DATE NOT NULL,
    real_play_seconds INTEGER NOT NULL DEFAULT 0 CHECK (real_play_seconds >= 0),
    last_heartbeat_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, activity_date)
);

CREATE TABLE IF NOT EXISTS password_reset_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash VARCHAR(64) NOT NULL,
    expires_at BIGINT NOT NULL,
    attempts SMALLINT NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    consumed_at BIGINT,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);
CREATE INDEX IF NOT EXISTS idx_password_reset_active
    ON password_reset_codes(user_id, expires_at) WHERE consumed_at IS NULL;

CREATE TABLE IF NOT EXISTS support_tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    category VARCHAR(30) NOT NULL CHECK (category IN ('account', 'payments', 'responsible_gaming', 'technical', 'other')),
    subject VARCHAR(120) NOT NULL,
    message TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'resolved', 'closed')),
    admin_response TEXT,
    responded_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_support_tickets_user_created
    ON support_tickets(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_support_tickets_status_created
    ON support_tickets(status, created_at DESC);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES ('system', 'MIGRATION_058_PLAYER_PROTECTION', '{"controls":"limits,self_exclusion,kyc,password_reset,support"}'::jsonb);
