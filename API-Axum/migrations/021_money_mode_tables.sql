-- 021: Mesas/torneios exclusivos Play Money vs Jogo Real (sem misturar saldos)

ALTER TABLE tables
    ADD COLUMN IF NOT EXISTS money_mode VARCHAR(16) NOT NULL DEFAULT 'play'
        CHECK (money_mode IN ('play', 'real'));

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS money_mode VARCHAR(16) NOT NULL DEFAULT 'play'
        CHECK (money_mode IN ('play', 'real'));

-- Existentes = Play Money
UPDATE tables SET money_mode = 'play' WHERE money_mode IS NULL OR money_mode = 'play';
UPDATE tournaments SET money_mode = 'play';

-- Renomear mesas play para deixar explícito
UPDATE tables SET name = 'PM · NL 0,50' WHERE name IN ('NL 0,50', 'PM · NL 0,50');
UPDATE tables SET name = 'PM · NL 1' WHERE name IN ('NL 1', 'PM · NL 1');

-- Mesas Jogo Real (mesmos stakes; saldo real separado)
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode
)
SELECT
    'b2000001-0001-4000-8000-000000000011'::uuid,
    'Real · NL 0,50',
    'cash',
    25, 50, 2500, 5000,
    9, 0, 'public', 'OPEN',
    500, 250,
    75, 150, 250,
    'real'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Real · NL 0,50');

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    money_mode
)
SELECT
    'b2000001-0001-4000-8000-000000000012'::uuid,
    'Real · NL 1',
    'cash',
    50, 100, 5000, 10000,
    9, 0, 'public', 'OPEN',
    500, 500,
    150, 300, 500,
    'real'
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'Real · NL 1');

UPDATE tables SET
    money_mode = 'real',
    visibility = 'public',
    status = 'OPEN'
WHERE name IN ('Real · NL 0,50', 'Real · NL 1');

UPDATE tables SET money_mode = 'play'
WHERE name LIKE 'PM ·%';

-- Torneios oficiais = Play Money (inscrição com saldo PM MTT)
UPDATE tournaments SET
    money_mode = 'play',
    name = CASE
        WHEN name LIKE 'PM ·%' THEN name
        WHEN id = 'c3000001-0001-4000-8000-000000000001'::uuid THEN 'PM · Freeroll R$100 GTD'
        WHEN id = 'c3000001-0001-4000-8000-000000000002'::uuid THEN 'PM · MTT R$200 GTD'
        ELSE name
    END
WHERE id IN (
    'c3000001-0001-4000-8000-000000000001'::uuid,
    'c3000001-0001-4000-8000-000000000002'::uuid
);

-- Torneios Jogo Real (espelho; buy-in real)
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode
)
SELECT
    'c3000001-0001-4000-8000-000000000011'::uuid,
    'Real · Freeroll R$100 GTD',
    0, 10000, 100,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, TRUE,
    0, 0, 0, 0,
    0, FALSE,
    (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000001'::uuid),
    'Holdem',
    'real'
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000011'::uuid
);

INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode
)
SELECT
    'c3000001-0001-4000-8000-000000000012'::uuid,
    'Real · MTT R$200 GTD',
    2000, 10000, 100,
    TRUE, 4, 'normal', 'registering',
    20000, 0, 0, 0,
    20000, FALSE,
    3000, 25000, 1, 5000,
    6, TRUE,
    (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000002'::uuid),
    'Holdem',
    'real'
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000012'::uuid
);
