-- 062: o Brazilian Pineapple atual (`pineapple_draw`) substitui o formato antigo
-- (`brazilian_pineapple`: 2 privadas + 1 por street). Decisões do proprietário:
-- 09/11/2026 (troca de formato) e 11/11/2026 (o jogo mantém o nome Brazilian Pineapple
-- para o jogador; a regra nova troca depois das apostas do flop e do turn e descarta
-- uma carta no river).
-- Antes de criar: maior migration em arquivos e no PostgreSQL local = 061 (ainda não
-- publicada); VPS = 060 (STATUS `migrations_latest`). Publicar 061 e 062 juntas.
--
-- - Mesas cash do formato antigo fecham mesmo com jogadores sentados. As fichas
--   desses assentos não somem: no início da API, `reconcile_closed_table_seats`
--   (main.rs) devolve à carteira de origem todo assento ativo sem mão interrompida,
--   com ledger CASH_OUT e auditoria. Mesa com mão interrompida fica para revisão.
-- - Torneios do formato antigo ainda não iniciados: o de Play Money passa ao formato
--   atual e mantém os inscritos; o de Jogo Real é cancelado SEM reembolso das
--   inscrições (decisão do proprietário, 09/11/2026), cada uma registrada em
--   audit_logs. Torneio em andamento continua impedindo a troca.
-- - O formato atual abre primeiro só no Play Money: uma mesa cash 6-max 0,50/0,50
--   (buy-in R$ 75 em fichas) e o torneio Play Money. O Jogo Real fica para depois da
--   validação, com nova migration. O histórico não é alterado.
-- - A regra do jogo vive no motor (`pineapple_draw_v1`).
-- Valores em centavos. Idempotente.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM tournaments
        WHERE COALESCE(poker_variant, '') IN ('brazilian_pineapple', 'ultimate_pineapple')
          AND status IN ('running', 'paused')
    ) THEN
        RAISE EXCEPTION 'migration 062: aguarde terminar o torneio Brazilian Pineapple (formato antigo) em andamento';
    END IF;
END $$;

-- Mesas cash do formato antigo (Play Money e Jogo Real) saem do lobby.
UPDATE tables
SET status = 'CLOSED',
    visibility = 'private'
WHERE game_type = 'cash'
  AND COALESCE(poker_variant, '') IN ('brazilian_pineapple', 'ultimate_pineapple')
  AND (status IS DISTINCT FROM 'CLOSED' OR visibility IS DISTINCT FROM 'private');

-- Mesa cash do formato atual: só Play Money nesta fase.
INSERT INTO tables (
    id, name, game_type,
    small_blind, big_blind, min_buy_in, max_buy_in,
    max_players, current_players, visibility, status,
    rake_basis_points, rake_cap,
    rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus,
    poker_variant, money_mode
)
SELECT
    'b2000062-0001-4000-8000-000000000001'::uuid,
    'PM · Brazilian Pineapple 0,50', 'cash',
    50, 50, 7500, 7500,
    6, 0, 'public', 'OPEN',
    500, 500, 150, 300, 500,
    'pineapple_draw', 'play'
WHERE NOT EXISTS (
    SELECT 1 FROM tables
    WHERE game_type = 'cash' AND poker_variant = 'pineapple_draw'
      AND money_mode = 'play' AND status = 'OPEN' AND visibility = 'public'
)
ON CONFLICT (id) DO NOTHING;

-- Jogo Real: inscrições canceladas sem reembolso ficam registradas, uma por linha.
INSERT INTO audit_logs (user_id, action, metadata)
SELECT
    players.player_id,
    'TOURNAMENT_REGISTRATION_CANCELLED_NO_REFUND_062',
    jsonb_build_object(
        'migration', 62,
        'tournament_id', t.id,
        'tournament_name', t.name,
        'buy_in_cents', t.buy_in,
        'rebuys', players.rebuys
    )
FROM tournaments AS t
JOIN tournament_players AS players ON players.tournament_id = t.id
WHERE COALESCE(t.poker_variant, '') IN ('brazilian_pineapple', 'ultimate_pineapple')
  AND COALESCE(t.money_mode, 'real') = 'real'
  AND t.status = 'registering';

UPDATE tournaments
SET status = 'cancelled'
WHERE COALESCE(poker_variant, '') IN ('brazilian_pineapple', 'ultimate_pineapple')
  AND COALESCE(money_mode, 'real') = 'real'
  AND status = 'registering';

-- Play Money: o torneio ainda não iniciado passa ao formato atual, com os inscritos.
UPDATE tournaments
SET name = 'Brazilian Pineapple — Torneio',
    poker_variant = 'pineapple_draw',
    game_type = 'PineappleDraw',
    table_max_players = 6,
    max_players = 18
WHERE COALESCE(poker_variant, '') IN ('brazilian_pineapple', 'ultimate_pineapple')
  AND money_mode = 'play'
  AND status = 'registering';

-- Mesa final: a modalidade nova entra; o nome antigo fica só para linhas históricas.
ALTER TABLE tournaments DROP CONSTRAINT IF EXISTS chk_tournaments_final_table_variant;
ALTER TABLE tournaments ADD CONSTRAINT chk_tournaments_final_table_variant
    CHECK (
        (final_table_variant IS NULL AND final_table_max_players IS NULL)
        OR (
            final_table_variant IN ('holdem', 'short_deck', 'omaha', 'pineapple_draw', 'brazilian_pineapple')
            AND final_table_max_players BETWEEN 2 AND
                CASE final_table_variant WHEN 'holdem' THEN 9 WHEN 'short_deck' THEN 8 ELSE 6 END
        )
    );

INSERT INTO audit_logs (user_id, action, metadata)
SELECT 'system', 'CATALOG_PINEAPPLE_DRAW_062', jsonb_build_object(
    'migration', 62,
    'removed', 'brazilian_pineapple',
    'added', 'pineapple_draw',
    'money_modes', jsonb_build_array('play'),
    'cash', jsonb_build_object('blinds', '50/50', 'buy_in_cents', 7500, 'max_players', 6),
    'real_registrations_refunded', false,
    'rule', 'pineapple_draw_v1'
)
WHERE NOT EXISTS (
    SELECT 1 FROM audit_logs
    WHERE action = 'CATALOG_PINEAPPLE_DRAW_062' AND metadata->>'migration' = '62'
);
