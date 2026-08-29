-- 023: Cash NL 0,25/0,25 (frente R$25) em Play Money e Jogo Real
-- Mantém NL 0,25/0,50 (R$25), NL 0,50/1 (R$75) e SD 1/2 (R$100).
-- Valores em CENTAVOS. Idempotente.
--
-- A constraint legada exigia big_blind par e small_blind = big_blind/2,
-- o que impede stakes iguais (0,25/0,25). Relaxamos para SB <= BB.

ALTER TABLE tables DROP CONSTRAINT IF EXISTS chk_tables_blind_structure;

ALTER TABLE tables
    ADD CONSTRAINT chk_tables_blind_structure
        CHECK (small_blind > 0 AND big_blind > 0 AND small_blind <= big_blind);

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000031'::uuid,
    'PM · NL 0,25',
    'cash',
    25, 25, 2500, 2500,
    9, 0, 'public', 'OPEN',
    500, 250,
    75, 150, 250,
    'play', 'holdem'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'PM · NL 0,25');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode, poker_variant
)
SELECT
    'b2000001-0001-4000-8000-000000000032'::uuid,
    'Real · NL 0,25',
    'cash',
    25, 25, 2500, 2500,
    9, 0, 'public', 'OPEN',
    500, 250,
    75, 150, 250,
    'real', 'holdem'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Real · NL 0,25');

UPDATE tables SET
    small_blind = 25,
    big_blind = 25,
    min_buy_in = 2500,
    max_buy_in = 2500,
    max_players = 9,
    visibility = 'public',
    status = 'OPEN',
    poker_variant = 'holdem',
    rake_basis_points = 500,
    rake_cap = 250,
    rake_cap_heads_up = 75,
    rake_cap_three_to_four = 150,
    rake_cap_five_plus = 250
WHERE name IN ('PM · NL 0,25', 'Real · NL 0,25');

-- Garante os outros stakes NLHE oficiais (frente fixa) em PM/Real
UPDATE tables SET
    small_blind = 25, big_blind = 50,
    min_buy_in = 2500, max_buy_in = 2500,
    max_players = 9, visibility = 'public', status = 'OPEN',
    poker_variant = 'holdem'
WHERE name IN ('PM · NL 0,50', 'Real · NL 0,50');

UPDATE tables SET
    small_blind = 50, big_blind = 100,
    min_buy_in = 7500, max_buy_in = 7500,
    max_players = 9, visibility = 'public', status = 'OPEN',
    poker_variant = 'holdem'
WHERE name IN ('PM · NL 1', 'Real · NL 1');
