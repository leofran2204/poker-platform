-- 005: Operação administrativa de mesas e configuração de rake por mesa.

ALTER TABLE tables
    ADD COLUMN IF NOT EXISTS status VARCHAR(20) NOT NULL DEFAULT 'OPEN'
        CHECK (status IN ('OPEN', 'PAUSED', 'CLOSED')),
    ADD COLUMN IF NOT EXISTS rake_basis_points SMALLINT NOT NULL DEFAULT 500
        CHECK (rake_basis_points >= 0 AND rake_basis_points <= 1000),
    ADD COLUMN IF NOT EXISTS rake_cap BIGINT NOT NULL DEFAULT 10000
        CHECK (rake_cap >= 0);

CREATE INDEX IF NOT EXISTS idx_tables_public_open
    ON tables(visibility, status, big_blind)
    WHERE visibility = 'public' AND status = 'OPEN';
