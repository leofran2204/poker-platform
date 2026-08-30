-- 029: Catálogo canônico de Short Deck para Cash e Torneios.
-- Cash: remove Hold'em 0,25/0,50; SD Hold'em -> 0,25/0,50;
--       SD Omaha -> 0,50/0,50. Valores monetários em centavos.
-- Torneios: remove Hold'em tradicional; SD Hold'em abre em 25/50;
--           SD Omaha abre em 50/50, preservando a estrutura de 26 níveis.

-- Nunca interromper uma mesa ocupada. Se surgir ocupação entre a auditoria e o
-- deploy, a migration falha de forma segura e exige esvaziar a mesa primeiro.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM tables AS target
        WHERE target.game_type = 'cash'
          AND COALESCE(target.poker_variant, 'holdem') = 'holdem'
          AND target.small_blind = 25
          AND target.big_blind = 50
          AND (
              target.current_players > 0
              OR EXISTS (
                  SELECT 1
                  FROM cash_game_seats AS seat
                  WHERE seat.table_id = target.id
                    AND seat.status = 'ACTIVE'
              )
          )
    ) THEN
        RAISE EXCEPTION 'migration 029: Holdem 0,25/0,50 possui jogadores ativos';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM tournaments
        WHERE COALESCE(poker_variant, 'holdem') = 'holdem'
          AND status IN ('running', 'paused')
    ) THEN
        RAISE EXCEPTION 'migration 029: torneio Holdem em andamento não pode ser cancelado';
    END IF;
END $$;

-- Preserva os IDs exatos afetados e os reembolsos para auditoria financeira.
CREATE TEMP TABLE migration_029_holdem_tournaments ON COMMIT DROP AS
SELECT id, name, money_mode, buy_in, rebuy_cost
FROM tournaments
WHERE COALESCE(poker_variant, 'holdem') = 'holdem'
  AND status = 'registering';

CREATE TEMP TABLE migration_029_tournament_refunds ON COMMIT DROP AS
SELECT
    tp.player_id::uuid AS user_id,
    t.id AS tournament_id,
    t.name AS tournament_name,
    COALESCE(t.money_mode, 'play') AS money_mode,
    (
        t.buy_in
        + tp.rebuys::bigint * CASE
            WHEN t.rebuy_cost > 0 THEN t.rebuy_cost
            ELSE t.buy_in
          END
    )::bigint AS amount
FROM migration_029_holdem_tournaments AS t
JOIN tournament_players AS tp ON tp.tournament_id = t.id;

WITH refunds AS (
    SELECT
        user_id,
        SUM(amount) FILTER (WHERE money_mode = 'play') AS play_amount,
        SUM(amount) FILTER (WHERE money_mode = 'real') AS real_amount
    FROM migration_029_tournament_refunds
    GROUP BY user_id
)
UPDATE users AS account
SET balance_pm_mtt = account.balance_pm_mtt + COALESCE(refunds.play_amount, 0),
    balance_real = account.balance_real + COALESCE(refunds.real_amount, 0)
FROM refunds
WHERE account.id = refunds.user_id;

INSERT INTO audit_logs (user_id, action, metadata)
SELECT
    refund.user_id::text,
    'TOURNAMENT_CANCELLATION_REFUND',
    jsonb_build_object(
        'migration', 29,
        'tournament_id', refund.tournament_id,
        'tournament_name', refund.tournament_name,
        'wallet_mode', refund.money_mode,
        'amount_cents', refund.amount
    )
FROM migration_029_tournament_refunds AS refund
WHERE refund.amount > 0;

UPDATE tournaments AS tournament
SET status = 'cancelled',
    finished_at = COALESCE(tournament.finished_at, EXTRACT(EPOCH FROM NOW())::bigint)
FROM migration_029_holdem_tournaments AS removed
WHERE tournament.id = removed.id;

-- Fecha e oculta Hold'em tradicional 0,25/0,50 sem apagar históricos de mãos.
UPDATE tables
SET status = 'CLOSED',
    visibility = 'private'
WHERE game_type = 'cash'
  AND COALESCE(poker_variant, 'holdem') = 'holdem'
  AND small_blind = 25
  AND big_blind = 50;

-- Hold'em Short Deck: 0,25/0,50, 6-max, frente fixa R$75.
UPDATE tables
SET name = CASE
        WHEN money_mode = 'real' THEN 'Real · SD 0,25/0,50'
        ELSE 'PM · SD 0,25/0,50'
    END,
    small_blind = 25,
    big_blind = 50,
    min_buy_in = 7500,
    max_buy_in = 7500,
    max_players = 6,
    visibility = 'public',
    status = 'OPEN'
WHERE game_type = 'cash'
  AND poker_variant = 'short_deck'
  AND visibility = 'public'
  AND status = 'OPEN';

-- Omaha 4 Cartas Short Deck: 0,50/0,50, 4-max, frente fixa R$100.
UPDATE tables
SET name = CASE
        WHEN money_mode = 'real' THEN 'Real · SD Omaha 0,50/0,50'
        ELSE 'PM · SD Omaha 0,50/0,50'
    END,
    small_blind = 50,
    big_blind = 50,
    min_buy_in = 10000,
    max_buy_in = 10000,
    max_players = 4,
    visibility = 'public',
    status = 'OPEN'
WHERE game_type = 'cash'
  AND poker_variant = 'short_deck_omaha'
  AND visibility = 'public'
  AND status = 'OPEN';

-- Hold'em Short Deck mantém abertura 25/50.
UPDATE tournaments
SET blind_levels = jsonb_set(
        jsonb_set(blind_levels, '{0,small_blind}', '25'::jsonb, false),
        '{0,big_blind}',
        '50'::jsonb,
        false
    )
WHERE poker_variant = 'short_deck'
  AND status IN ('registering', 'running', 'paused');

-- Omaha Short Deck passa a abrir 50/50; níveis 2–26 permanecem inalterados.
UPDATE tournaments
SET blind_levels = jsonb_set(
        jsonb_set(blind_levels, '{0,small_blind}', '50'::jsonb, false),
        '{0,big_blind}',
        '50'::jsonb,
        false
    )
WHERE poker_variant = 'short_deck_omaha'
  AND status IN ('registering', 'running', 'paused');

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'SHORT_DECK_CATALOG_REBALANCED',
    jsonb_build_object(
        'migration', 29,
        'holdem_cash_025_050', 'closed_private',
        'holdem_tournaments_cancelled', COALESCE(
            (SELECT jsonb_agg(id) FROM migration_029_holdem_tournaments),
            '[]'::jsonb
        ),
        'short_deck_cash', jsonb_build_object('small_blind', 25, 'big_blind', 50),
        'short_deck_tournament_level_1', jsonb_build_object('small_blind', 25, 'big_blind', 50),
        'omaha_short_deck_cash', jsonb_build_object('small_blind', 50, 'big_blind', 50),
        'omaha_short_deck_tournament_level_1', jsonb_build_object('small_blind', 50, 'big_blind', 50),
        'tournament_levels', 26
    )
);
