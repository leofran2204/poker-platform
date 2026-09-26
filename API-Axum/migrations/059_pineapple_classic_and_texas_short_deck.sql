-- 059: Brazilian Pineapple passa a 52 cartas/ranking clássico e 6-max.
-- Texas Hold'em Short Deck volta como variante separada, cash 0,50/0,50 8-max.
-- A mudança de baralho/avaliador vive no motor; esta migration altera o catálogo.
-- Antes de criar: maior migration em arquivos, PostgreSQL local e VPS = 058.
-- Valores em centavos; idempotente. Não modifica migrations já aplicadas.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM tournaments
        WHERE COALESCE(poker_variant, '') IN ('brazilian_pineapple', 'ultimate_pineapple')
          AND status IN ('running', 'paused')
    ) THEN
        RAISE EXCEPTION 'migration 059: aguarde terminar torneio Pineapple antes de trocar o baralho';
    END IF;

    IF EXISTS (
        SELECT 1 FROM tables AS target
        WHERE target.game_type = 'cash'
          AND COALESCE(target.poker_variant, '') IN ('brazilian_pineapple', 'ultimate_pineapple')
          AND (target.current_players > 0 OR EXISTS (
              SELECT 1 FROM cash_game_seats AS seat
              WHERE seat.table_id = target.id AND seat.status = 'ACTIVE'
          ))
    ) THEN
        RAISE EXCEPTION 'migration 059: esvazie as mesas Pineapple antes de trocar o baralho';
    END IF;
END $$;

UPDATE tables
SET max_players = 6
WHERE game_type = 'cash'
  AND poker_variant = 'brazilian_pineapple'
  AND status = 'OPEN'
  AND max_players IS DISTINCT FROM 6;

UPDATE tournaments
SET table_max_players = 6,
    max_players = 18
WHERE poker_variant = 'brazilian_pineapple'
  AND status = 'registering'
  AND (table_max_players IS DISTINCT FROM 6 OR max_players IS DISTINCT FROM 18);

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    poker_variant, money_mode
)
SELECT
    'b2000059-0001-4000-8000-000000000001'::uuid,
    'PM · Texas Short Deck 0,50/0,50', 'cash',
    50, 50, 10000, 10000,
    8, 0, 'public', 'OPEN',
    500, 500, 150, 300, 500,
    'short_deck', 'play'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE game_type = 'cash' AND poker_variant = 'short_deck'
      AND money_mode = 'play' AND status = 'OPEN' AND visibility = 'public'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    poker_variant, money_mode
)
SELECT
    'b2000059-0001-4000-8000-000000000002'::uuid,
    'Real · Texas Short Deck 0,50/0,50', 'cash',
    50, 50, 10000, 10000,
    8, 0, 'public', 'OPEN',
    500, 500, 150, 300, 500,
    'short_deck', 'real'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE game_type = 'cash' AND poker_variant = 'short_deck'
      AND money_mode = 'real' AND status = 'OPEN' AND visibility = 'public'
)
ON CONFLICT (id) DO NOTHING;

UPDATE tables
SET max_players = 8
WHERE game_type = 'cash' AND poker_variant = 'short_deck'
  AND status = 'OPEN' AND max_players IS DISTINCT FROM 8;

UPDATE tournaments
SET table_max_players = 8, max_players = 24
WHERE poker_variant = 'short_deck' AND status = 'registering'
  AND (table_max_players IS DISTINCT FROM 8 OR max_players IS DISTINCT FROM 24);

ALTER TABLE tournaments DROP CONSTRAINT IF EXISTS chk_tournaments_final_table_variant;
ALTER TABLE tournaments ADD CONSTRAINT chk_tournaments_final_table_variant
    CHECK (
        (final_table_variant IS NULL AND final_table_max_players IS NULL)
        OR (
            final_table_variant IN ('holdem', 'short_deck', 'omaha', 'brazilian_pineapple')
            AND final_table_max_players BETWEEN 2 AND
                CASE final_table_variant WHEN 'holdem' THEN 9 WHEN 'short_deck' THEN 8 ELSE 6 END
        )
    );

INSERT INTO audit_logs (user_id, action, metadata)
SELECT 'system', 'CATALOG_FOUR_VARIANTS_059', jsonb_build_object(
    'migration', 59,
    'pineapple', jsonb_build_object('deck', 52, 'ranking', 'classic', 'max_players', 6),
    'short_deck', jsonb_build_object('deck', 36, 'cash_blinds', '50/50', 'buy_in_cents', 10000, 'max_players', 8)
)
WHERE NOT EXISTS (
    SELECT 1 FROM audit_logs
    WHERE action = 'CATALOG_FOUR_VARIANTS_059' AND metadata->>'migration' = '59'
);
