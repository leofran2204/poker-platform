-- 046: ledger Minha Estrutura — 18% nível 1, 12% nível 2, resto casa.
-- Valores em centavos inteiros. Clube não recebe fatia nesta rede.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS estrutura_points BIGINT NOT NULL DEFAULT 0
        CHECK (estrutura_points >= 0);

CREATE TABLE IF NOT EXISTS estrutura_ledger (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hand_id              UUID NOT NULL REFERENCES hand_history(id) ON DELETE CASCADE,
    source_user_id       UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    beneficiary_user_id  UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    level                SMALLINT NOT NULL CHECK (level IN (1, 2)),
    source_rake_cents    BIGINT NOT NULL CHECK (source_rake_cents >= 0),
    commission_cents     BIGINT NOT NULL CHECK (commission_cents >= 0),
    eligible             BOOLEAN NOT NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_estrutura_ledger_beneficiary
    ON estrutura_ledger(beneficiary_user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_estrutura_ledger_source
    ON estrutura_ledger(source_user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_estrutura_ledger_hand
    ON estrutura_ledger(hand_id);
