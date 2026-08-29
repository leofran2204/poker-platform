-- 022: Short Deck (6-max 1/2 frente 100) + frentes fixas NLHE 25/75 (PM + Real)

ALTER TABLE tables
    ADD COLUMN IF NOT EXISTS poker_variant VARCHAR(16) NOT NULL DEFAULT 'holdem';

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS poker_variant VARCHAR(16) NOT NULL DEFAULT 'holdem';

-- NLHE: frente fixa (0,25/0,50 → R$25 · 0,50/1,00 → R$75) em PM e Real
UPDATE tables SET
    min_buy_in = 2500,
    max_buy_in = 2500,
    poker_variant = 'holdem',
    max_players = 9
WHERE big_blind = 50
  AND COALESCE(poker_variant, 'holdem') <> 'short_deck'
  AND name IN ('PM · NL 0,50', 'Real · NL 0,50', 'NL 0,50');

UPDATE tables SET
    min_buy_in = 7500,
    max_buy_in = 7500,
    poker_variant = 'holdem',
    max_players = 9
WHERE big_blind = 100
  AND COALESCE(poker_variant, 'holdem') <> 'short_deck'
  AND name IN ('PM · NL 1', 'Real · NL 1', 'NL 1');

UPDATE tables SET poker_variant = 'holdem'
WHERE poker_variant IS NULL OR poker_variant = '';

-- Short Deck cash PM
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000021'::uuid,
    'PM · SD 1/2',
    'cash',
    100, 200, 10000, 10000,
    6, 0, 'public', 'OPEN',
    500, 1000,
    300, 600, 1000,
    'play', 'short_deck'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'PM · SD 1/2');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000022'::uuid,
    'Real · SD 1/2',
    'cash',
    100, 200, 10000, 10000,
    6, 0, 'public', 'OPEN',
    500, 1000,
    300, 600, 1000,
    'real', 'short_deck'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Real · SD 1/2');

UPDATE tables SET
    small_blind = 100, big_blind = 200,
    min_buy_in = 10000, max_buy_in = 10000,
    max_players = 6, visibility = 'public', status = 'OPEN',
    poker_variant = 'short_deck'
WHERE name IN ('PM · SD 1/2', 'Real · SD 1/2');

-- Torneios Short Deck (espelho blinds do Holdem)
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode, poker_variant
)
SELECT
    'c3000001-0001-4000-8000-000000000021'::uuid,
    'PM · SD Freeroll R$100 GTD',
    0, 10000, 100,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, TRUE,
    0, 0, 0, 0,
    0, FALSE,
    COALESCE(
        (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000001'::uuid),
        '[]'::jsonb
    ),
    'ShortDeck', 'play', 'short_deck'
WHERE NOT EXISTS (SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000021'::uuid);

INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode, poker_variant
)
SELECT
    'c3000001-0001-4000-8000-000000000022'::uuid,
    'PM · SD MTT R$200 GTD',
    2000, 10000, 100,
    TRUE, 4, 'normal', 'registering',
    20000, 0, 0, 0,
    20000, FALSE,
    3000, 25000, 1, 5000,
    6, TRUE,
    COALESCE(
        (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000002'::uuid),
        '[]'::jsonb
    ),
    'ShortDeck', 'play', 'short_deck'
WHERE NOT EXISTS (SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000022'::uuid);

INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode, poker_variant
)
SELECT
    'c3000001-0001-4000-8000-000000000023'::uuid,
    'Real · SD Freeroll R$100 GTD',
    0, 10000, 100,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, TRUE,
    0, 0, 0, 0,
    0, FALSE,
    COALESCE(
        (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000001'::uuid),
        '[]'::jsonb
    ),
    'ShortDeck', 'real', 'short_deck'
WHERE NOT EXISTS (SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000023'::uuid);

INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode, poker_variant
)
SELECT
    'c3000001-0001-4000-8000-000000000024'::uuid,
    'Real · SD MTT R$200 GTD',
    2000, 10000, 100,
    TRUE, 4, 'normal', 'registering',
    20000, 0, 0, 0,
    20000, FALSE,
    3000, 25000, 1, 5000,
    6, TRUE,
    COALESCE(
        (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000002'::uuid),
        '[]'::jsonb
    ),
    'ShortDeck', 'real', 'short_deck'
WHERE NOT EXISTS (SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000024'::uuid);

UPDATE tournaments SET poker_variant = 'holdem'
WHERE poker_variant IS NULL OR poker_variant = '' OR name LIKE '%NL%' OR (name LIKE 'PM · %' AND name NOT LIKE '%SD%');

UPDATE tournaments SET poker_variant = 'short_deck', game_type = 'ShortDeck'
WHERE name LIKE '%SD %';
