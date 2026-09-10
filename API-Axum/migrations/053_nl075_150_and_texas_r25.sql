-- 053: Mesa cash Texas Hold'em NL 0,75/1,50 (PM + Real) e torneio Texas R$25
-- (PM + Real, buy-in 2500, stack 15000, 1 reentrada 2500 p/ 25000).
-- Idempotente (WHERE NOT EXISTS). Cash 9-max frente 15000; rake espelha
-- a convenção das demais mesas 9-max (500bp, cap 250/75/150/250).

-- Cash PM 0,75/1,50
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    poker_variant, money_mode
)
SELECT
    'b2000001-0001-4000-8000-000000000033'::uuid,
    'PM · NL 0,75/1,50',
    'cash',
    75, 150, 15000, 15000,
    9, 0, 'public', 'OPEN',
    500, 250,
    75, 150, 250,
    'holdem', 'play'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE id = 'b2000001-0001-4000-8000-000000000033'::uuid);

-- Cash Real 0,75/1,50
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    poker_variant, money_mode
)
SELECT
    'b2000001-0001-4000-8000-000000000034'::uuid,
    'Real · NL 0,75/1,50',
    'cash',
    75, 150, 15000, 15000,
    9, 0, 'public', 'OPEN',
    500, 250,
    75, 150, 250,
    'holdem', 'real'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE id = 'b2000001-0001-4000-8000-000000000034'::uuid);

-- Torneio PM Texas R$25 (agenda 2026-09-10 21:30 SP, auto-start 5+)
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode,
    poker_variant, scheduled_start_at, auto_start_min_players,
    prize_pool, current_level, players_remaining, total_buyins
)
SELECT
    'c3000001-0001-4000-8000-000000000039'::uuid,
    'PM · Texas R$25',
    2500, 15000, 27, 9,
    TRUE, 4, 'normal', 'registering',
    0, FALSE,
    2500, 25000, 1, 15000,
    6, TRUE,
    (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000031'::uuid),
    'Holdem', 'play',
    'holdem', 1789086600, 5,
    0, 0, 0, 0
WHERE NOT EXISTS (SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000039'::uuid);

-- Torneio Real Texas R$25 (mesma estrutura, saldo real)
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode,
    poker_variant, scheduled_start_at, auto_start_min_players,
    prize_pool, current_level, players_remaining, total_buyins
)
SELECT
    'c3000001-0001-4000-8000-000000000040'::uuid,
    'Real · Texas R$25',
    2500, 15000, 27, 9,
    TRUE, 4, 'normal', 'registering',
    0, FALSE,
    2500, 25000, 1, 15000,
    6, TRUE,
    (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000031'::uuid),
    'Holdem', 'real',
    'holdem', 1789086600, 5,
    0, 0, 0, 0
WHERE NOT EXISTS (SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000040'::uuid);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'CASH_AND_MTT_R25_075_150',
    jsonb_build_object(
        'migration', 53,
        'cash_tables', jsonb_build_array('PM · NL 0,75/1,50', 'Real · NL 0,75/1,50'),
        'tournaments', jsonb_build_array('PM · Texas R$25', 'Real · Texas R$25'),
        'note', 'cash 9-max frente 15000; MTT buy-in 2500 stack 15000 + 1 reentrada 2500/25000'
    )
);
