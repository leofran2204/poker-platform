-- 060: Agente ZT Poker — um nivel, 30% do NGR direto + 5 p.p. por meta mensal.
-- O ledger 18/12 anterior permanece preservado com program_version = 1.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS agent_status VARCHAR(16) NOT NULL DEFAULT 'inactive'
        CHECK (agent_status IN ('inactive', 'active', 'suspended'));

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS agent_commission_balance_cents BIGINT NOT NULL DEFAULT 0
        CHECK (agent_commission_balance_cents >= 0);

ALTER TABLE estrutura_ledger
    ADD COLUMN IF NOT EXISTS program_version SMALLINT NOT NULL DEFAULT 1
        CHECK (program_version IN (1, 2));

ALTER TABLE estrutura_ledger
    ADD COLUMN IF NOT EXISTS cycle_start DATE;

ALTER TABLE estrutura_ledger
    ADD COLUMN IF NOT EXISTS source_reference_id UUID;

ALTER TABLE estrutura_ledger
    ADD COLUMN IF NOT EXISTS money_mode VARCHAR(8) NOT NULL DEFAULT 'play'
        CHECK (money_mode IN ('play', 'real'));

UPDATE estrutura_ledger
SET cycle_start = date_trunc('month', created_at AT TIME ZONE 'America/Sao_Paulo')::date
WHERE cycle_start IS NULL;

CREATE INDEX IF NOT EXISTS idx_estrutura_ledger_agent_cycle_v2
    ON estrutura_ledger(beneficiary_user_id, cycle_start, money_mode, source_user_id)
    WHERE program_version = 2;

CREATE INDEX IF NOT EXISTS idx_estrutura_ledger_source_reference
    ON estrutura_ledger(source_reference_id)
    WHERE source_reference_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS agent_monthly_cycles (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_user_id          UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    cycle_start            DATE NOT NULL CHECK (EXTRACT(DAY FROM cycle_start) = 1),
    money_mode             VARCHAR(8) NOT NULL CHECK (money_mode IN ('play', 'real')),
    target_ngr_cents       BIGINT NOT NULL DEFAULT 0 CHECK (target_ngr_cents >= 0),
    status                 VARCHAR(12) NOT NULL DEFAULT 'open'
        CHECK (status IN ('open', 'closed')),
    gross_revenue_cents    BIGINT NOT NULL DEFAULT 0 CHECK (gross_revenue_cents >= 0),
    deductions_cents       BIGINT NOT NULL DEFAULT 0 CHECK (deductions_cents >= 0),
    ngr_cents              BIGINT NOT NULL DEFAULT 0 CHECK (ngr_cents >= 0),
    commission_percent     SMALLINT CHECK (commission_percent IN (30, 35)),
    commission_cents       BIGINT NOT NULL DEFAULT 0 CHECK (commission_cents >= 0),
    closed_at              TIMESTAMPTZ,
    closed_by              UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (agent_user_id, cycle_start, money_mode)
);

CREATE INDEX IF NOT EXISTS idx_agent_monthly_cycles_status
    ON agent_monthly_cycles(status, cycle_start, money_mode);

CREATE TABLE IF NOT EXISTS agent_ngr_adjustments (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_user_id          UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    cycle_start            DATE NOT NULL CHECK (EXTRACT(DAY FROM cycle_start) = 1),
    money_mode             VARCHAR(8) NOT NULL CHECK (money_mode IN ('play', 'real')),
    category               VARCHAR(24) NOT NULL
        CHECK (category IN ('reward', 'refund', 'chargeback', 'tax', 'payment_cost', 'other')),
    amount_cents           BIGINT NOT NULL CHECK (amount_cents > 0),
    note                   VARCHAR(240) NOT NULL,
    created_by             UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agent_ngr_adjustments_cycle
    ON agent_ngr_adjustments(agent_user_id, cycle_start, money_mode);
