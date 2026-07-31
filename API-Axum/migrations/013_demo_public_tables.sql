-- 013: Mesas públicas de demo para feedback (play-money).
-- Idempotente: só insere se o nome ainda não existir.
-- Capacidade total ~72 assentos (8 mesas × 9) para dezenas de amigos.

-- NL2 (SB 1 / BB 2 centavos? No: blinds em centavos de real)
-- Padrão da plataforma: blinds e buy-in em CENTAVOS.
-- NL2 ≈ SB R$0,01 BB R$0,02 → 1 / 2 centavos — muito micro.
-- Usamos stakes legíveis para demo:
--   NL2:  SB 100 / BB 200   (R$1 / R$2)     buy-in 20–200 BB
--   NL5:  SB 250 / BB 500
--   NL10: SB 500 / BB 1000
--   NL25: SB 1250 / BB 2500

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'a1000001-0001-4000-8000-000000000001'::uuid,
    'Demo NL2 #1',
    'cash',
    100, 200, 4000, 40000,
    9, 0, 'public', 'OPEN',
    500, 10000,
    300, 600, 1000
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Demo NL2 #1');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'a1000001-0001-4000-8000-000000000002'::uuid,
    'Demo NL2 #2',
    'cash',
    100, 200, 4000, 40000,
    9, 0, 'public', 'OPEN',
    500, 10000,
    300, 600, 1000
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Demo NL2 #2');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'a1000001-0001-4000-8000-000000000003'::uuid,
    'Demo NL5 #1',
    'cash',
    250, 500, 10000, 100000,
    9, 0, 'public', 'OPEN',
    500, 10000,
    300, 600, 1000
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Demo NL5 #1');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'a1000001-0001-4000-8000-000000000004'::uuid,
    'Demo NL5 #2',
    'cash',
    250, 500, 10000, 100000,
    9, 0, 'public', 'OPEN',
    500, 10000,
    300, 600, 1000
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Demo NL5 #2');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'a1000001-0001-4000-8000-000000000005'::uuid,
    'Demo NL10 #1',
    'cash',
    500, 1000, 20000, 100000,
    9, 0, 'public', 'OPEN',
    500, 10000,
    300, 600, 1000
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Demo NL10 #1');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'a1000001-0001-4000-8000-000000000006'::uuid,
    'Demo NL10 #2',
    'cash',
    500, 1000, 20000, 100000,
    9, 0, 'public', 'OPEN',
    500, 10000,
    300, 600, 1000
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Demo NL10 #2');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'a1000001-0001-4000-8000-000000000007'::uuid,
    'Demo NL25 #1',
    'cash',
    1250, 2500, 50000, 250000,
    9, 0, 'public', 'OPEN',
    500, 10000,
    300, 600, 1000
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Demo NL25 #1');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'a1000001-0001-4000-8000-000000000008'::uuid,
    'Demo NL25 #2',
    'cash',
    1250, 2500, 50000, 250000,
    9, 0, 'public', 'OPEN',
    500, 10000,
    300, 600, 1000
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Demo NL25 #2');
