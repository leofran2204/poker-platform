-- 020: Play Money (cash + MTT) × Jogo Real wallets

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS balance_pm_cash BIGINT NOT NULL DEFAULT 100000,
    ADD COLUMN IF NOT EXISTS balance_pm_mtt BIGINT NOT NULL DEFAULT 1500000,
    ADD COLUMN IF NOT EXISTS balance_real BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_pm_reset_date DATE,
    ADD COLUMN IF NOT EXISTS pm_cash_rebuy_used_on DATE,
    ADD COLUMN IF NOT EXISTS pm_mtt_rebuy_used_on DATE,
    ADD COLUMN IF NOT EXISTS preferred_wallet_mode VARCHAR(16) NOT NULL DEFAULT 'play';

-- Backfill from legacy balance → PM cash; MTT full grant; real zero
UPDATE users SET
    balance_pm_cash = CASE WHEN balance > 0 THEN balance ELSE 100000 END,
    balance_pm_mtt = 1500000,
    balance_real = 0,
    last_pm_reset_date = (timezone('America/Sao_Paulo', now()))::date,
    preferred_wallet_mode = 'play'
WHERE last_pm_reset_date IS NULL;

-- Keep legacy balance in sync with PM cash for old admin queries
UPDATE users SET balance = balance_pm_cash;

ALTER TABLE cash_game_seats
    ADD COLUMN IF NOT EXISTS wallet_kind VARCHAR(16) NOT NULL DEFAULT 'pm_cash'
        CHECK (wallet_kind IN ('pm_cash', 'real'));
