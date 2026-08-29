-- 025: Remove SD 1/1; SD Omaha → 0,50/1,00 4-max frente R$100
-- Valores em CENTAVOS. Idempotente.

-- Fecha / esconde SD 1/1 (PM e Real)
UPDATE tables SET
    status = 'CLOSED',
    visibility = 'private'
WHERE name IN ('PM · SD 1/1', 'Real · SD 1/1', 'PM · SD 1/2', 'Real · SD 1/2');

-- SD Omaha: blinds 0,50/1,00 · frente R$100 · 4-max
UPDATE tables SET
    name = CASE
        WHEN money_mode = 'real' THEN 'Real · SD Omaha 0,50/1'
        ELSE 'PM · SD Omaha 0,50/1'
    END,
    small_blind = 50,
    big_blind = 100,
    min_buy_in = 10000,
    max_buy_in = 10000,
    max_players = 4,
    poker_variant = 'short_deck_omaha',
    visibility = 'public',
    status = 'OPEN'
WHERE name IN (
    'PM · SD Omaha 1/2',
    'Real · SD Omaha 1/2',
    'PM · SD Omaha 0,50/1',
    'Real · SD Omaha 0,50/1'
);

-- Garantir existência se rename/re-run
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
    'PM · SD Omaha 0,50/1',
    'cash',
    50, 100, 10000, 10000,
    4, 0, 'public', 'OPEN',
    500, 1000,
    300, 600, 1000,
    'play', 'short_deck_omaha'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE name = 'PM · SD Omaha 0,50/1'
       OR id = 'b2000001-0001-4000-8000-000000000041'::uuid
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
    'b2000001-0001-4000-8000-000000000042'::uuid,
    'Real · SD Omaha 0,50/1',
    'cash',
    50, 100, 10000, 10000,
    4, 0, 'public', 'OPEN',
    500, 1000,
    300, 600, 1000,
    'real', 'short_deck_omaha'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE name = 'Real · SD Omaha 0,50/1'
       OR id = 'b2000001-0001-4000-8000-000000000042'::uuid
);

UPDATE tables SET
    small_blind = 50,
    big_blind = 100,
    min_buy_in = 10000,
    max_buy_in = 10000,
    max_players = 4,
    poker_variant = 'short_deck_omaha',
    visibility = 'public',
    status = 'OPEN'
WHERE name IN ('PM · SD Omaha 0,50/1', 'Real · SD Omaha 0,50/1');
