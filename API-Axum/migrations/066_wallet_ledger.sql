-- 066: livro único do dinheiro (decisão do proprietário, etapa 6 do plano de 23/11/2026).
--
-- Toda mudança de saldo das três carteiras (`balance_pm_cash`, `balance_pm_mtt`,
-- `balance_real`) grava uma linha em `wallet_ledger`, na mesma transação, com o
-- valor com sinal, o saldo resultante, o motivo e a referência (mesa, torneio, PIX).
-- O registro é feito por gatilho em `users`, então nenhuma mudança escapa, nem SQL
-- manual da operação. Quem altera o saldo informa o motivo na mesma instrução, em
-- `users.ledger_reason` (e `ledger_ref`); o gatilho grava e limpa as duas colunas.
-- Sem motivo, a linha fica `unspecified` (o CI recusa).
--
-- O livro é só de inclusão: UPDATE e DELETE são recusados. Na virada, uma linha de
-- saldo inicial por conta e carteira com saldo diferente de zero; daí em diante
-- vale saldo = soma do livro (`wallet_ledger_mismatches`).
-- Os registros atuais (`wallet_transactions`, `cash_game_ledger`, `estrutura_ledger`,
-- `audit_logs`) continuam como estão. Idempotente.

ALTER TABLE users ADD COLUMN IF NOT EXISTS ledger_reason TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS ledger_ref TEXT;

CREATE TABLE IF NOT EXISTS wallet_ledger (
    id BIGSERIAL PRIMARY KEY,
    -- Sem FK: o histórico do dinheiro fica mesmo se a conta for apagada.
    user_id UUID NOT NULL,
    wallet TEXT NOT NULL CHECK (wallet IN ('pm_cash', 'pm_mtt', 'real')),
    amount_cents BIGINT NOT NULL CHECK (amount_cents <> 0),
    balance_after_cents BIGINT NOT NULL,
    reason TEXT NOT NULL,
    reference TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_wallet_ledger_user_wallet
    ON wallet_ledger (user_id, wallet, id DESC);
CREATE INDEX IF NOT EXISTS idx_wallet_ledger_reason ON wallet_ledger (reason);

-- Só de inclusão.
CREATE OR REPLACE FUNCTION wallet_ledger_append_only() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'wallet_ledger é só de inclusão (% recusado)', TG_OP;
END;
$$;

DROP TRIGGER IF EXISTS wallet_ledger_no_change ON wallet_ledger;
CREATE TRIGGER wallet_ledger_no_change
    BEFORE UPDATE OR DELETE ON wallet_ledger
    FOR EACH ROW EXECUTE FUNCTION wallet_ledger_append_only();

-- Saldo inicial: uma linha por conta e carteira com saldo, só na primeira aplicação.
INSERT INTO wallet_ledger (user_id, wallet, amount_cents, balance_after_cents, reason)
SELECT u.id, w.wallet, w.balance, w.balance, 'opening_balance'
  FROM users u
 CROSS JOIN LATERAL (VALUES
        ('pm_cash', u.balance_pm_cash),
        ('pm_mtt', u.balance_pm_mtt),
        ('real', u.balance_real)) AS w(wallet, balance)
 WHERE w.balance <> 0
   AND NOT EXISTS (SELECT 1 FROM wallet_ledger WHERE reason = 'opening_balance');

-- Registro de cada mudança de saldo, com o motivo dado na mesma instrução.
CREATE OR REPLACE FUNCTION users_wallet_ledger() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    why TEXT := COALESCE(NULLIF(NEW.ledger_reason, ''),
                         CASE WHEN TG_OP = 'INSERT' THEN 'account_opening' ELSE 'unspecified' END);
    old_cash BIGINT := CASE WHEN TG_OP = 'INSERT' THEN 0 ELSE OLD.balance_pm_cash END;
    old_mtt BIGINT := CASE WHEN TG_OP = 'INSERT' THEN 0 ELSE OLD.balance_pm_mtt END;
    old_real BIGINT := CASE WHEN TG_OP = 'INSERT' THEN 0 ELSE OLD.balance_real END;
BEGIN
    IF NEW.balance_pm_cash IS DISTINCT FROM old_cash THEN
        INSERT INTO wallet_ledger (user_id, wallet, amount_cents, balance_after_cents, reason, reference)
        VALUES (NEW.id, 'pm_cash', NEW.balance_pm_cash - old_cash, NEW.balance_pm_cash, why, NEW.ledger_ref);
    END IF;
    IF NEW.balance_pm_mtt IS DISTINCT FROM old_mtt THEN
        INSERT INTO wallet_ledger (user_id, wallet, amount_cents, balance_after_cents, reason, reference)
        VALUES (NEW.id, 'pm_mtt', NEW.balance_pm_mtt - old_mtt, NEW.balance_pm_mtt, why, NEW.ledger_ref);
    END IF;
    IF NEW.balance_real IS DISTINCT FROM old_real THEN
        INSERT INTO wallet_ledger (user_id, wallet, amount_cents, balance_after_cents, reason, reference)
        VALUES (NEW.id, 'real', NEW.balance_real - old_real, NEW.balance_real, why, NEW.ledger_ref);
    END IF;
    NEW.ledger_reason := NULL;
    NEW.ledger_ref := NULL;
    RETURN NEW;
END;
$$;

-- AFTER INSERT para a conta nova já existir na referência do livro; o UPDATE usa
-- BEFORE para limpar as colunas de motivo na mesma linha.
CREATE OR REPLACE FUNCTION users_wallet_ledger_after_insert() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    why TEXT := COALESCE(NULLIF(NEW.ledger_reason, ''), 'account_opening');
BEGIN
    IF NEW.balance_pm_cash <> 0 THEN
        INSERT INTO wallet_ledger (user_id, wallet, amount_cents, balance_after_cents, reason, reference)
        VALUES (NEW.id, 'pm_cash', NEW.balance_pm_cash, NEW.balance_pm_cash, why, NEW.ledger_ref);
    END IF;
    IF NEW.balance_pm_mtt <> 0 THEN
        INSERT INTO wallet_ledger (user_id, wallet, amount_cents, balance_after_cents, reason, reference)
        VALUES (NEW.id, 'pm_mtt', NEW.balance_pm_mtt, NEW.balance_pm_mtt, why, NEW.ledger_ref);
    END IF;
    IF NEW.balance_real <> 0 THEN
        INSERT INTO wallet_ledger (user_id, wallet, amount_cents, balance_after_cents, reason, reference)
        VALUES (NEW.id, 'real', NEW.balance_real, NEW.balance_real, why, NEW.ledger_ref);
    END IF;
    IF NEW.ledger_reason IS NOT NULL OR NEW.ledger_ref IS NOT NULL THEN
        UPDATE users SET ledger_reason = NULL, ledger_ref = NULL WHERE id = NEW.id;
    END IF;
    RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS users_wallet_ledger_update ON users;
CREATE TRIGGER users_wallet_ledger_update
    BEFORE UPDATE OF balance_pm_cash, balance_pm_mtt, balance_real, ledger_reason, ledger_ref ON users
    FOR EACH ROW EXECUTE FUNCTION users_wallet_ledger();

DROP TRIGGER IF EXISTS users_wallet_ledger_insert ON users;
CREATE TRIGGER users_wallet_ledger_insert
    AFTER INSERT ON users
    FOR EACH ROW EXECUTE FUNCTION users_wallet_ledger_after_insert();

-- Conferência: contas e carteiras em que o saldo não bate com a soma do livro.
CREATE OR REPLACE VIEW wallet_ledger_mismatches AS
SELECT u.id AS user_id, w.wallet, w.balance AS balance_cents,
       COALESCE(l.total, 0) AS ledger_cents
  FROM users u
 CROSS JOIN LATERAL (VALUES
        ('pm_cash', u.balance_pm_cash),
        ('pm_mtt', u.balance_pm_mtt),
        ('real', u.balance_real)) AS w(wallet, balance)
  LEFT JOIN LATERAL (
        SELECT SUM(amount_cents) AS total FROM wallet_ledger
         WHERE user_id = u.id AND wallet = w.wallet) l ON TRUE
 WHERE w.balance <> COALESCE(l.total, 0);

INSERT INTO audit_logs (user_id, action, metadata)
SELECT 'system', 'WALLET_LEDGER_066',
       jsonb_build_object('migration', 66,
                          'opening_rows', (SELECT count(*) FROM wallet_ledger WHERE reason = 'opening_balance'))
WHERE NOT EXISTS (SELECT 1 FROM audit_logs WHERE action = 'WALLET_LEDGER_066');
