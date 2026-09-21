-- 057: catálogo em três modalidades.
-- Hold’em 52 permanece. Texas Short Deck sai. Omaha vira 4 cartas / 52 / 6-max.
-- Ultimate Pineapple vira Brazilian Pineapple (Short Deck, 2+1+1+1, 5-max).
-- Freeroll Texas deixa de trocar para Short Deck na mesa final.
-- Valores em centavos. Idempotente.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM tables AS target
        WHERE target.game_type = 'cash'
          AND COALESCE(target.poker_variant, 'holdem') = 'short_deck'
          AND target.status = 'OPEN'
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
        RAISE EXCEPTION 'migration 057: mesa Short Deck ainda tem jogador ativo';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM tournaments
        WHERE status IN ('running', 'paused')
          AND (
              COALESCE(poker_variant, 'holdem') IN (
                  'short_deck', 'short_deck_omaha', 'ultimate_pineapple'
              )
              OR final_table_variant = 'short_deck'
          )
    ) THEN
        RAISE EXCEPTION 'migration 057: torneio da reforma ainda está em andamento';
    END IF;
END $$;

ALTER TABLE tournaments DROP CONSTRAINT IF EXISTS chk_tournaments_final_table_variant;

UPDATE tables
SET status = 'CLOSED',
    visibility = 'private'
WHERE game_type = 'cash'
  AND COALESCE(poker_variant, 'holdem') = 'short_deck'
  AND (status IS DISTINCT FROM 'CLOSED' OR visibility IS DISTINCT FROM 'private');

UPDATE tables
SET name = CASE
        WHEN money_mode = 'real' THEN 'Real · Omaha 0,50/0,50'
        ELSE 'PM · Omaha 0,50/0,50'
    END,
    poker_variant = 'omaha',
    max_players = 6,
    visibility = 'public',
    status = 'OPEN'
WHERE COALESCE(poker_variant, '') IN ('short_deck_omaha', 'omaha')
  AND game_type = 'cash';

UPDATE tables
SET name = CASE
        WHEN money_mode = 'real' THEN 'Real · Brazilian Pineapple 0,50'
        ELSE 'PM · Brazilian Pineapple 0,50'
    END,
    poker_variant = 'brazilian_pineapple',
    max_players = 5,
    visibility = 'public',
    status = 'OPEN'
WHERE COALESCE(poker_variant, '') IN ('ultimate_pineapple', 'brazilian_pineapple')
  AND game_type = 'cash';

UPDATE tournaments
SET name = CASE
        WHEN name ILIKE '%freeroll%' THEN name
        ELSE 'Omaha 4 Cartas — Torneio'
    END,
    poker_variant = 'omaha',
    game_type = 'Omaha',
    table_max_players = 6,
    max_players = 18
WHERE COALESCE(poker_variant, '') IN ('short_deck_omaha', 'omaha')
  AND status IN ('registering', 'cancelled');

UPDATE tournaments
SET name = 'Brazilian Pineapple — Torneio',
    poker_variant = 'brazilian_pineapple',
    game_type = 'BrazilianPineapple',
    table_max_players = 5,
    max_players = 15
WHERE COALESCE(poker_variant, '') IN ('ultimate_pineapple', 'brazilian_pineapple')
  AND status IN ('registering', 'cancelled');

UPDATE tournaments
SET final_table_variant = NULL,
    final_table_max_players = NULL
WHERE final_table_variant IS NOT NULL
   OR final_table_max_players IS NOT NULL;

ALTER TABLE tournaments
    ADD CONSTRAINT chk_tournaments_final_table_variant
    CHECK (
        (final_table_variant IS NULL AND final_table_max_players IS NULL)
        OR (
            final_table_variant IN ('holdem', 'omaha', 'brazilian_pineapple')
            AND final_table_max_players BETWEEN 2 AND 9
        )
    );

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'CATALOG_THREE_VARIANTS',
    jsonb_build_object(
        'migration', 57,
        'cash', jsonb_build_array('holdem', 'omaha', 'brazilian_pineapple'),
        'omaha_max', 6,
        'pineapple_max', 5
    )
);
