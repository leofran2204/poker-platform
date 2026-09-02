-- 033: Ultimate Pineapple Short Deck — 6-max, 3 hole usa 2+3, sem descarte, ranking Short Deck (flush > full house)
-- PM e Real: Ultimate Pineapple 0,50/0,50 frente R$75 (mesmo stake do Short Deck 6-max para liquidez)
-- Valores em CENTAVOS. Idempotente.
-- `ultimate_pineapple` tem 18 chars; 022 criou VARCHAR(16) (cabe `short_deck_omaha`).

ALTER TABLE tables
    ALTER COLUMN poker_variant TYPE VARCHAR(32);
ALTER TABLE tournaments
    ALTER COLUMN poker_variant TYPE VARCHAR(32);

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000050'::uuid,
    'PM · Pineapple 0,50',
    'cash',
    50, 50, 7500, 7500,
    6, 0, 'public', 'OPEN',
    500, 500,
    150, 300, 500,
    'play', 'ultimate_pineapple'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE name = 'PM · Pineapple 0,50'
       OR id = 'b2000001-0001-4000-8000-000000000050'::uuid
);

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000051'::uuid,
    'Real · Pineapple 0,50',
    'cash',
    50, 50, 7500, 7500,
    6, 0, 'public', 'OPEN',
    500, 500,
    150, 300, 500,
    'real', 'ultimate_pineapple'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE name = 'Real · Pineapple 0,50'
       OR id = 'b2000001-0001-4000-8000-000000000051'::uuid
);

UPDATE tables SET
    small_blind = 50, big_blind = 50,
    min_buy_in = 7500, max_buy_in = 7500,
    max_players = 6, poker_variant = 'ultimate_pineapple',
    visibility = 'public', status = 'OPEN'
WHERE name IN ('PM · Pineapple 0,50', 'Real · Pineapple 0,50');
