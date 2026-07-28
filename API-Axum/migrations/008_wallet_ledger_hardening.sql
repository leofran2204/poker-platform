-- 008_wallet_ledger_hardening.sql
-- The payment module stores every monetary value as integer cents and never
-- accepts a provider callback without a persisted, pending ledger entry.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM wallet_transactions
        WHERE amount <> trunc(amount)
    ) THEN
        RAISE EXCEPTION
            'wallet_transactions.amount contains fractional values; migrate those records to cents before applying migration 008';
    END IF;
END $$;

ALTER TABLE wallet_transactions
    ALTER COLUMN amount TYPE BIGINT USING amount::BIGINT;

ALTER TABLE wallet_transactions
    ADD COLUMN IF NOT EXISTS idempotency_key VARCHAR(128),
    ADD COLUMN IF NOT EXISTS provider VARCHAR(32) NOT NULL DEFAULT 'mock',
    ADD COLUMN IF NOT EXISTS provider_status VARCHAR(64),
    ADD COLUMN IF NOT EXISTS pix_key_fingerprint VARCHAR(64);

CREATE UNIQUE INDEX IF NOT EXISTS uq_wallet_transactions_idempotency_key
    ON wallet_transactions(idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uq_wallet_transactions_external_tx_id
    ON wallet_transactions(external_tx_id)
    WHERE external_tx_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_wallet_transactions_user_type_status
    ON wallet_transactions(user_id, transaction_type, status, created_at);
