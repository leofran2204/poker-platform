-- 045: convite de rede, fila da mesa, mesa viva de torneio.
-- Valores e IDs seguem o padrão UUID / VARCHAR já usado no schema.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS referral_code VARCHAR(12) UNIQUE,
    ADD COLUMN IF NOT EXISTS sponsored_by UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_users_sponsored_by ON users(sponsored_by);
CREATE INDEX IF NOT EXISTS idx_users_referral_code ON users(referral_code);

UPDATE users
SET referral_code = UPPER(SUBSTRING(REPLACE(id::text, '-', '') FROM 1 FOR 8))
WHERE referral_code IS NULL;

CREATE TABLE IF NOT EXISTS table_waitlist (
    table_id   UUID NOT NULL REFERENCES tables(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (table_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_table_waitlist_created
    ON table_waitlist(table_id, created_at);

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS live_table_id UUID REFERENCES tables(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_tournaments_live_table ON tournaments(live_table_id);
