-- 024: Catálogo cash — frentes + SD 0,50 + SD 1/1 + SD Omaha 1/2
-- NL 0,25/0,25 R$25 · NL 0,25/0,50 R$50 · SD 0,50/0,50 R$75 6-max
-- SD 1/1 R$100 · SD Omaha 1/2 R$150 4-max. Sem NLHE 0,50/1.
-- Valores em CENTAVOS. Idempotente.

-- NL 0,50 Hold'em: frente R$50
UPDATE tables SET
    min_buy_in = 5000,
    max_buy_in = 5000,
    small_blind = 25,
    big_blind = 50,
    max_players = 9,
    poker_variant = 'holdem',
    visibility = 'public',
    status = 'OPEN'
WHERE name IN ('PM · NL 0,50', 'Real · NL 0,50');

-- Converter NL 1 Hold'em → SD 0,50 (0,50/0,50 · 6-max · frente R$75)
UPDATE tables SET
    name = CASE
        WHEN money_mode = 'real' THEN 'Real · SD 0,50'
        ELSE 'PM · SD 0,50'
    END,
    small_blind = 50,
    big_blind = 50,
    min_buy_in = 7500,
    max_buy_in = 7500,
    max_players = 6,
    poker_variant = 'short_deck',
    visibility = 'public',
    status = 'OPEN'
WHERE name IN ('PM · NL 1', 'Real · NL 1', 'PM · SD 0,50', 'Real · SD 0,50');

-- Garantir mesas SD 0,50 se rename já ocorreu / re-run
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000002'::uuid,
    'PM · SD 0,50',
    'cash',
    50, 50, 7500, 7500,
    6, 0, 'public', 'OPEN',
    500, 500,
    150, 300, 500,
    'play', 'short_deck'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE name = 'PM · SD 0,50'
       OR id = 'b2000001-0001-4000-8000-000000000002'::uuid
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
    'b2000001-0001-4000-8000-000000000012'::uuid,
    'Real · SD 0,50',
    'cash',
    50, 50, 7500, 7500,
    6, 0, 'public', 'OPEN',
    500, 500,
    150, 300, 500,
    'real', 'short_deck'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE name = 'Real · SD 0,50'
       OR id = 'b2000001-0001-4000-8000-000000000012'::uuid
);

UPDATE tables SET
    small_blind = 50, big_blind = 50,
    min_buy_in = 7500, max_buy_in = 7500,
    max_players = 6, poker_variant = 'short_deck',
    visibility = 'public', status = 'OPEN'
WHERE name IN ('PM · SD 0,50', 'Real · SD 0,50');

-- SD 1/2 → SD 1/1 (blinds 100/100, frente R$100)
UPDATE tables SET
    name = CASE
        WHEN money_mode = 'real' THEN 'Real · SD 1/1'
        ELSE 'PM · SD 1/1'
    END,
    small_blind = 100,
    big_blind = 100,
    min_buy_in = 10000,
    max_buy_in = 10000,
    max_players = 6,
    poker_variant = 'short_deck',
    visibility = 'public',
    status = 'OPEN'
WHERE name IN ('PM · SD 1/2', 'Real · SD 1/2', 'PM · SD 1/1', 'Real · SD 1/1');

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
    'PM · SD 1/1',
    'cash',
    100, 100, 10000, 10000,
    6, 0, 'public', 'OPEN',
    500, 1000,
    300, 600, 1000,
    'play', 'short_deck'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE name = 'PM · SD 1/1'
       OR id = 'b2000001-0001-4000-8000-000000000021'::uuid
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
    'b2000001-0001-4000-8000-000000000022'::uuid,
    'Real · SD 1/1',
    'cash',
    100, 100, 10000, 10000,
    6, 0, 'public', 'OPEN',
    500, 1000,
    300, 600, 1000,
    'real', 'short_deck'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE name = 'Real · SD 1/1'
       OR id = 'b2000001-0001-4000-8000-000000000022'::uuid
);

UPDATE tables SET
    small_blind = 100, big_blind = 100,
    min_buy_in = 10000, max_buy_in = 10000,
    max_players = 6, poker_variant = 'short_deck',
    visibility = 'public', status = 'OPEN'
WHERE name IN ('PM · SD 1/1', 'Real · SD 1/1');

-- SD Omaha 1/2 4-max frente R$150
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000041'::uuid,
    'PM · SD Omaha 1/2',
    'cash',
    100, 200, 15000, 15000,
    4, 0, 'public', 'OPEN',
    500, 1000,
    300, 600, 1000,
    'play', 'short_deck_omaha'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'PM · SD Omaha 1/2');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000042'::uuid,
    'Real · SD Omaha 1/2',
    'cash',
    100, 200, 15000, 15000,
    4, 0, 'public', 'OPEN',
    500, 1000,
    300, 600, 1000,
    'real', 'short_deck_omaha'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Real · SD Omaha 1/2');

UPDATE tables SET
    small_blind = 100, big_blind = 200,
    min_buy_in = 15000, max_buy_in = 15000,
    max_players = 4, poker_variant = 'short_deck_omaha',
    visibility = 'public', status = 'OPEN'
WHERE name IN ('PM · SD Omaha 1/2', 'Real · SD Omaha 1/2');

-- Fechar qualquer residual NL 1 Hold'em
UPDATE tables SET status = 'CLOSED', visibility = 'private'
WHERE name IN ('NL 1', 'PM · NL 1', 'Real · NL 1')
  AND COALESCE(poker_variant, 'holdem') = 'holdem';
