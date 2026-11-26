-- 071: o Brazilian Pineapple (`pineapple_draw`) abre no Jogo Real.
-- Preparada em 25/11/2026 (como 070, fora do master) e aprovada pelo proprietário
-- em 26/11/2026: "tá aprovado o jogo real de Brazilian Pineapple". Renumerada
-- para 071 porque a 070 virou `070_course_watch_video.sql`.
--
-- Mesmos valores do Play Money (062), que é o que já foi testado:
-- - mesa cash 6-max 0,50/0,50, buy-in R$ 75, rake 5% com teto de R$ 1,50 (2
--   jogadores), R$ 3,00 (3 ou 4) e R$ 5,00 (5 ou mais), igual à Short Deck real;
-- - torneio R$ 10, 10.000 fichas, até 18 jogadores em mesas de 6, garantido
--   R$ 100, inscrição tardia até o nível 4, rebuy de R$ 20 por 20.000 fichas
--   até o nível 6, sem add-on, mesmos níveis do Omaha real. Diferença: no Jogo
--   Real, 1 rebuy por jogador, como todos os torneios reais do catálogo.
-- O Brazilian Pineapple não tem ante (a API zera o ante dos níveis), o Loss
-- Deflator e o cashback já valem em todas as modalidades e nos dois modos, e
-- a guarda de rede vale para cash e torneio. Não há outra trava no código.
-- Valores em centavos. Idempotente.

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM tournaments
        WHERE id = 'c3000001-0001-4000-8000-000000000036'::uuid
          AND jsonb_typeof(blind_levels) = 'array'
          AND jsonb_array_length(blind_levels) > 0
    ) THEN
        RAISE EXCEPTION 'migration 071: níveis de blinds do torneio Omaha real (c3000001-...-036) não encontrados';
    END IF;
END $$;

-- Mesa cash do Jogo Real.
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    poker_variant, money_mode
)
SELECT
    'b2000070-0001-4000-8000-000000000001'::uuid,
    'Real · Brazilian Pineapple 0,50', 'cash',
    50, 50, 7500, 7500,
    6, 0, 'public', 'OPEN',
    500, 500, 150, 300, 500,
    'pineapple_draw', 'real'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE game_type = 'cash' AND poker_variant = 'pineapple_draw'
      AND money_mode = 'real' AND status = 'OPEN' AND visibility = 'public'
)
ON CONFLICT (id) DO NOTHING;

-- Torneio do Jogo Real.
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy,
    allow_addon, addon_cost, addon_chips, addon_max_level,
    blind_levels, game_type, money_mode,
    poker_variant, scheduled_start_at, auto_start_min_players,
    prize_pool, current_level, players_remaining, total_buyins
)
SELECT
    'c3000001-0001-4000-8000-000000000041'::uuid,
    'Brazilian Pineapple — Torneio',
    1000, 10000, 18, 6,
    TRUE, 4, 'normal', 'registering',
    10000, FALSE,
    2000, 20000, 1, 0,
    6, TRUE,
    FALSE, 0, 0, 0,
    (SELECT blind_levels FROM tournaments WHERE id = 'c3000001-0001-4000-8000-000000000036'::uuid),
    'PineappleDraw', 'real',
    'pineapple_draw', 1788481800, 5,
    0, 0, 0, 0
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments
    WHERE poker_variant = 'pineapple_draw' AND money_mode = 'real'
      AND status IN ('registering', 'running', 'paused')
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO audit_logs (user_id, action, metadata)
SELECT 'system', 'CATALOG_PINEAPPLE_DRAW_REAL_071', jsonb_build_object(
    'migration', 71,
    'variant', 'pineapple_draw',
    'money_mode', 'real',
    'cash', jsonb_build_object('blinds', '50/50', 'buy_in_cents', 7500, 'max_players', 6),
    'tournament', jsonb_build_object(
        'buy_in_cents', 1000, 'guaranteed_cents', 10000,
        'max_players', 18, 'rebuy_max_count', 1
    )
)
WHERE NOT EXISTS (
    SELECT 1 FROM audit_logs
    WHERE action = 'CATALOG_PINEAPPLE_DRAW_REAL_071' AND metadata->>'migration' = '71'
);
