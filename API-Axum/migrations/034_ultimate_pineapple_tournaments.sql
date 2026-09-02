-- 034: Ultimate Pineapple nos torneios (Play Money + Jogo Real).
-- 6-max, 3 hole / 2+3, sem descarte, ranking Short Deck.
-- Espelha o MTT Omaha (buy-in R$10, GTD R$100, rebuy, BBA 26 níveis).
-- Nível 1 abre 0,50/0,50 como o cash Pineapple. Idempotente.

INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode,
    poker_variant, final_table_variant, final_table_max_players
)
SELECT
    'c3000001-0001-4000-8000-000000000037'::uuid,
    'Ultimate Pineapple — Torneio',
    1000, 10000, 100, 6,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, FALSE,
    2000, 20000, 1, 0,
    6, TRUE, DEFAULT, 'UltimatePineapple', 'play',
    'ultimate_pineapple', NULL, NULL
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments
    WHERE id = 'c3000001-0001-4000-8000-000000000037'::uuid
       OR (name = 'Ultimate Pineapple — Torneio' AND money_mode = 'play')
);

INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode,
    poker_variant, final_table_variant, final_table_max_players
)
SELECT
    'c3000001-0001-4000-8000-000000000038'::uuid,
    'Ultimate Pineapple — Torneio',
    1000, 10000, 100, 6,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, FALSE,
    2000, 20000, 1, 0,
    6, TRUE, DEFAULT, 'UltimatePineapple', 'real',
    'ultimate_pineapple', NULL, NULL
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments
    WHERE id = 'c3000001-0001-4000-8000-000000000038'::uuid
       OR (name = 'Ultimate Pineapple — Torneio' AND money_mode = 'real')
);

UPDATE tournaments
SET table_max_players = 6,
    poker_variant = 'ultimate_pineapple',
    game_type = 'UltimatePineapple',
    status = 'registering'
WHERE id IN (
    'c3000001-0001-4000-8000-000000000037'::uuid,
    'c3000001-0001-4000-8000-000000000038'::uuid
)
  AND status IN ('registering', 'cancelled');

-- Nível 1: 0,50/0,50; ante = big_blind (BBA).
UPDATE tournaments
SET blind_levels = jsonb_set(
        jsonb_set(
            jsonb_set(blind_levels, '{0,small_blind}', '50'::jsonb, false),
            '{0,big_blind}',
            '50'::jsonb,
            false
        ),
        '{0,ante}',
        '50'::jsonb,
        false
    )
WHERE id IN (
    'c3000001-0001-4000-8000-000000000037'::uuid,
    'c3000001-0001-4000-8000-000000000038'::uuid
)
  AND status = 'registering';

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_CATALOG_PINEAPPLE_CREATED',
    jsonb_build_object(
        'migration', 34,
        'modes', jsonb_build_array('play', 'real'),
        'variant', 'ultimate_pineapple',
        'table_max_players', 6,
        'buy_in', 1000,
        'guaranteed_prize', 10000
    )
);
