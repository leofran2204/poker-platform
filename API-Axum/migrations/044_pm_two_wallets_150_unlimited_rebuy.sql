-- 044: Play Money em duas carteiras de R$ 150 (cash + torneio), rebuy ilimitado com saldo.
-- Sem rebuy diário grátis (só renovação 00:00 America/Sao_Paulo). Reentradas em
-- mesa/torneio são ilimitadas enquanto houver saldo (cada join/register debita).
-- Torneio play: rebuy ilimitado até o fim do nível 6 (rebuy_max_count 0 = ilimitado).
-- Vale só para play money; Jogo Real intocado. Idempotente.

-- Grant de torneio volta a R$ 150
ALTER TABLE users ALTER COLUMN balance_pm_mtt SET DEFAULT 15000;

UPDATE users
SET balance_pm_mtt = 15000
WHERE balance_pm_mtt IS DISTINCT FROM 15000;

-- Rebuy ilimitado (com saldo) em torneios play até o nível 6
UPDATE tournaments
SET rebuy_max_count = 0
WHERE money_mode = 'play'
  AND allow_rebuy = TRUE
  AND rebuy_max_count IS DISTINCT FROM 0;

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'PM_WALLETS_150_UNLIMITED_REBUY',
    jsonb_build_object(
        'migration', 44,
        'pm_cash_cents', 15000,
        'pm_mtt_cents', 15000,
        'rebuy_max_count_play', 0,
        'rebuy_max_level', 6,
        'note', 'duas carteiras R$150; sem rebuy diario; ilimitado com saldo (play money)'
    )
);
