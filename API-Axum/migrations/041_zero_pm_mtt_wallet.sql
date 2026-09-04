-- 041: Zera a carteira Play Money de torneio (freerolls são grátis) e ajusta o grant de cadastro.
-- Novo cadastro ganha apenas R$ 150,00 (15000 cents) para cash games; torneio fica zerado.
-- Valores monetários em centavos. Idempotente.

-- Grants padrão das colunas passam a refletir a regra
ALTER TABLE users ALTER COLUMN balance_pm_cash SET DEFAULT 15000;
ALTER TABLE users ALTER COLUMN balance_pm_mtt SET DEFAULT 0;

-- Zera a carteira de torneio de todas as contas existentes agora
-- (o reset diário já manteria zerado, mas a regra vale imediatamente).
UPDATE users
SET balance_pm_mtt = 0
WHERE balance_pm_mtt != 0;

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'PM_MTT_WALLET_ZEROED',
    jsonb_build_object(
        'migration', 41,
        'pm_cash_daily_cents', 15000,
        'pm_mtt_daily_cents', 0,
        'reason', 'freerolls gratuitos; cadastro ganha apenas R$ 150 para cash'
    )
);
