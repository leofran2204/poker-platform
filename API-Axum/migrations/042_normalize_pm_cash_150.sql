-- 042: Normaliza o cash Play Money existente para o novo grant de R$ 150,00 (15000 cents).
-- A 041 zerou o torneio mas manteve o cash antigo (ex.: R$ 1.000); o reset diário
-- só normalizaria no dia seguinte. Esta migration aplica a regra imediatamente,
-- descontando o escrow de assentos ACTIVE (mesma lógica do reset diário).
-- Valores monetários em centavos. Idempotente.

UPDATE users AS u SET
    balance_pm_cash = GREATEST(
        0,
        15000 - COALESCE((
            SELECT SUM(s.chips)
            FROM cash_game_seats AS s
            WHERE s.user_id = u.id
              AND s.status = 'ACTIVE'
              AND s.wallet_kind = 'pm_cash'
        ), 0)
    ),
    balance = GREATEST(
        0,
        15000 - COALESCE((
            SELECT SUM(s.chips)
            FROM cash_game_seats AS s
            WHERE s.user_id = u.id
              AND s.status = 'ACTIVE'
              AND s.wallet_kind = 'pm_cash'
        ), 0)
    )
WHERE u.balance_pm_cash IS DISTINCT FROM GREATEST(
    0,
    15000 - COALESCE((
        SELECT SUM(s.chips)
        FROM cash_game_seats AS s
        WHERE s.user_id = u.id
          AND s.status = 'ACTIVE'
          AND s.wallet_kind = 'pm_cash'
    ), 0)
);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'PM_CASH_NORMALIZED_150',
    jsonb_build_object(
        'migration', 42,
        'pm_cash_cents', 15000,
        'reason', 'novo grant de cadastro R$ 150 aplicado às contas existentes'
    )
);
