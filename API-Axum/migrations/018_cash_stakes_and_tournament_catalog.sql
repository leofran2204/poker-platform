-- 018: Stakes oficiais de cash (2 mesas) + catálogo de torneios (Freeroll / MTT GTD).
-- Valores em CENTAVOS. Idempotente para re-runs.

-- ─── A) Fechar mesas demo antigas (NL2–NL25 etc.) ───
UPDATE tables
SET status = 'CLOSED',
    visibility = 'private'
WHERE game_type = 'cash'
  AND visibility = 'public'
  AND status = 'OPEN'
  AND name NOT IN ('NL 0,50', 'NL 1');

-- ─── B) Duas mesas cash oficiais ───
-- NL 0,50: SB 25 / BB 50 · frente R$25 (2500) · max 100BB = R$50 (5000)
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'b2000001-0001-4000-8000-000000000001'::uuid,
    'NL 0,50',
    'cash',
    25, 50, 2500, 5000,
    9, 0, 'public', 'OPEN',
    500, 250,
    75, 150, 250
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'NL 0,50');

UPDATE tables SET
    small_blind = 25,
    big_blind = 50,
    min_buy_in = 2500,
    max_buy_in = 5000,
    max_players = 9,
    visibility = 'public',
    status = 'OPEN',
    rake_basis_points = 500,
    rake_cap = 250,
    rake_cap_heads_up = 75,
    rake_cap_three_to_four = 150,
    rake_cap_five_plus = 250
WHERE name = 'NL 0,50';

-- NL 1: SB 50 / BB 100 · frente R$50 (5000) · max 100BB = R$100 (10000)
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus
)
SELECT
    'b2000001-0001-4000-8000-000000000002'::uuid,
    'NL 1',
    'cash',
    50, 100, 5000, 10000,
    9, 0, 'public', 'OPEN',
    500, 500,
    150, 300, 500
WHERE NOT EXISTS (SELECT 1 FROM tables WHERE name = 'NL 1');

UPDATE tables SET
    small_blind = 50,
    big_blind = 100,
    min_buy_in = 5000,
    max_buy_in = 10000,
    max_players = 9,
    visibility = 'public',
    status = 'OPEN',
    rake_basis_points = 500,
    rake_cap = 500,
    rake_cap_heads_up = 150,
    rake_cap_three_to_four = 300,
    rake_cap_five_plus = 500
WHERE name = 'NL 1';

-- ─── C) Colunas de catálogo de torneio ───
ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS guaranteed_prize BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS is_freeroll BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS rebuy_cost BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rebuy_chips BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rebuy_max_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rebuy_stack_threshold BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rebuy_max_level INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS allow_rebuy BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS blind_levels JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS game_type VARCHAR(30) NOT NULL DEFAULT 'Holdem';

-- Estrutura convencional 5 minutos (níveis 1–12)
-- Usada pelos dois torneios oficiais.
-- (JSON alinhado a BlindLevel do motor)

-- ─── D) Freeroll R$100 GTD ───
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type
)
SELECT
    'c3000001-0001-4000-8000-000000000001'::uuid,
    'Freeroll R$100 GTD',
    0, 10000, 100,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, TRUE,
    0, 0, 0, 0,
    0, FALSE,
    '[
      {"level":1,"small_blind":25,"big_blind":50,"ante":0,"duration_minutes":5},
      {"level":2,"small_blind":50,"big_blind":100,"ante":0,"duration_minutes":5},
      {"level":3,"small_blind":75,"big_blind":150,"ante":0,"duration_minutes":5},
      {"level":4,"small_blind":100,"big_blind":200,"ante":0,"duration_minutes":5},
      {"level":5,"small_blind":150,"big_blind":300,"ante":0,"duration_minutes":5},
      {"level":6,"small_blind":200,"big_blind":400,"ante":0,"duration_minutes":5},
      {"level":7,"small_blind":300,"big_blind":600,"ante":0,"duration_minutes":5},
      {"level":8,"small_blind":400,"big_blind":800,"ante":0,"duration_minutes":5},
      {"level":9,"small_blind":500,"big_blind":1000,"ante":50,"duration_minutes":5},
      {"level":10,"small_blind":600,"big_blind":1200,"ante":100,"duration_minutes":5},
      {"level":11,"small_blind":800,"big_blind":1600,"ante":200,"duration_minutes":5},
      {"level":12,"small_blind":1000,"big_blind":2000,"ante":300,"duration_minutes":5}
    ]'::jsonb,
    'Holdem'
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000001'::uuid
);

UPDATE tournaments SET
    name = 'Freeroll R$100 GTD',
    buy_in = 0,
    starting_stack = 10000,
    max_players = 100,
    late_registration = TRUE,
    late_reg_max_level = 4,
    speed = 'normal',
    status = CASE WHEN status IN ('finished', 'cancelled') THEN status ELSE 'registering' END,
    prize_pool = GREATEST(prize_pool, 10000),
    guaranteed_prize = 10000,
    is_freeroll = TRUE,
    rebuy_cost = 0,
    rebuy_chips = 0,
    rebuy_max_count = 0,
    rebuy_stack_threshold = 0,
    rebuy_max_level = 0,
    allow_rebuy = FALSE,
    blind_levels = '[
      {"level":1,"small_blind":25,"big_blind":50,"ante":0,"duration_minutes":5},
      {"level":2,"small_blind":50,"big_blind":100,"ante":0,"duration_minutes":5},
      {"level":3,"small_blind":75,"big_blind":150,"ante":0,"duration_minutes":5},
      {"level":4,"small_blind":100,"big_blind":200,"ante":0,"duration_minutes":5},
      {"level":5,"small_blind":150,"big_blind":300,"ante":0,"duration_minutes":5},
      {"level":6,"small_blind":200,"big_blind":400,"ante":0,"duration_minutes":5},
      {"level":7,"small_blind":300,"big_blind":600,"ante":0,"duration_minutes":5},
      {"level":8,"small_blind":400,"big_blind":800,"ante":0,"duration_minutes":5},
      {"level":9,"small_blind":500,"big_blind":1000,"ante":50,"duration_minutes":5},
      {"level":10,"small_blind":600,"big_blind":1200,"ante":100,"duration_minutes":5},
      {"level":11,"small_blind":800,"big_blind":1600,"ante":200,"duration_minutes":5},
      {"level":12,"small_blind":1000,"big_blind":2000,"ante":300,"duration_minutes":5}
    ]'::jsonb,
    game_type = 'Holdem'
WHERE id = 'c3000001-0001-4000-8000-000000000001'::uuid;

-- ─── E) MTT R$200 GTD · buy-in R$20 → 10k · 1 rebuy até niv.6 · R$30 → 25k se ≤5k ───
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type
)
SELECT
    'c3000001-0001-4000-8000-000000000002'::uuid,
    'MTT R$200 GTD',
    2000, 10000, 100,
    TRUE, 4, 'normal', 'registering',
    20000, 0, 0, 0,
    20000, FALSE,
    3000, 25000, 1, 5000,
    6, TRUE,
    '[
      {"level":1,"small_blind":25,"big_blind":50,"ante":0,"duration_minutes":5},
      {"level":2,"small_blind":50,"big_blind":100,"ante":0,"duration_minutes":5},
      {"level":3,"small_blind":75,"big_blind":150,"ante":0,"duration_minutes":5},
      {"level":4,"small_blind":100,"big_blind":200,"ante":0,"duration_minutes":5},
      {"level":5,"small_blind":150,"big_blind":300,"ante":0,"duration_minutes":5},
      {"level":6,"small_blind":200,"big_blind":400,"ante":0,"duration_minutes":5},
      {"level":7,"small_blind":300,"big_blind":600,"ante":0,"duration_minutes":5},
      {"level":8,"small_blind":400,"big_blind":800,"ante":0,"duration_minutes":5},
      {"level":9,"small_blind":500,"big_blind":1000,"ante":50,"duration_minutes":5},
      {"level":10,"small_blind":600,"big_blind":1200,"ante":100,"duration_minutes":5},
      {"level":11,"small_blind":800,"big_blind":1600,"ante":200,"duration_minutes":5},
      {"level":12,"small_blind":1000,"big_blind":2000,"ante":300,"duration_minutes":5}
    ]'::jsonb,
    'Holdem'
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000002'::uuid
);

UPDATE tournaments SET
    name = 'MTT R$200 GTD',
    buy_in = 2000,
    starting_stack = 10000,
    max_players = 100,
    late_registration = TRUE,
    late_reg_max_level = 4,
    speed = 'normal',
    status = CASE WHEN status IN ('finished', 'cancelled') THEN status ELSE 'registering' END,
    prize_pool = GREATEST(prize_pool, 20000),
    guaranteed_prize = 20000,
    is_freeroll = FALSE,
    rebuy_cost = 3000,
    rebuy_chips = 25000,
    rebuy_max_count = 1,
    rebuy_stack_threshold = 5000,
    rebuy_max_level = 6,
    allow_rebuy = TRUE,
    blind_levels = '[
      {"level":1,"small_blind":25,"big_blind":50,"ante":0,"duration_minutes":5},
      {"level":2,"small_blind":50,"big_blind":100,"ante":0,"duration_minutes":5},
      {"level":3,"small_blind":75,"big_blind":150,"ante":0,"duration_minutes":5},
      {"level":4,"small_blind":100,"big_blind":200,"ante":0,"duration_minutes":5},
      {"level":5,"small_blind":150,"big_blind":300,"ante":0,"duration_minutes":5},
      {"level":6,"small_blind":200,"big_blind":400,"ante":0,"duration_minutes":5},
      {"level":7,"small_blind":300,"big_blind":600,"ante":0,"duration_minutes":5},
      {"level":8,"small_blind":400,"big_blind":800,"ante":0,"duration_minutes":5},
      {"level":9,"small_blind":500,"big_blind":1000,"ante":50,"duration_minutes":5},
      {"level":10,"small_blind":600,"big_blind":1200,"ante":100,"duration_minutes":5},
      {"level":11,"small_blind":800,"big_blind":1600,"ante":200,"duration_minutes":5},
      {"level":12,"small_blind":1000,"big_blind":2000,"ante":300,"duration_minutes":5}
    ]'::jsonb,
    game_type = 'Holdem'
WHERE id = 'c3000001-0001-4000-8000-000000000002'::uuid;
