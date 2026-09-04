-- 043: Restaura o catálogo canônico de torneios (031–038) para registering com as alterações.
-- Os sims deixaram os 4 play (031/032/033/037) em running/finished, sumindo do lobby
-- (o catálogo carrega só registering/running/paused). Volta os 8 para registering,
-- preservando inscrições de contas existentes e limpando apenas órfãos (contas excluídas).
-- Aplica as alterações: nomes Texas, FT Short Deck 8-max, Omaha 5-max, Pineapple 6-max,
-- scheduled 21:30 SP (1788481800), auto-start 5. Idempotente.

-- 1. Limpa inscrições órfãs (player_id sem conta em users) — preserva humanos inscritos
DELETE FROM tournament_players AS tp
WHERE NOT EXISTS (
    SELECT 1 FROM users AS u WHERE u.id::text = tp.player_id
);

-- 2. Limpa assentos órfãos de torneio (contas excluídas) — preserva humanos sentados
DELETE FROM tournament_seats AS ts
WHERE NOT EXISTS (
    SELECT 1 FROM users AS u WHERE u.id::text = ts.player_id
);

-- 3. Reaplica nomes/config do catálogo com as alterações
UPDATE tournaments SET name = 'Texas Hold’em — Torneio'
WHERE id IN (
    'c3000001-0001-4000-8000-000000000031'::uuid,
    'c3000001-0001-4000-8000-000000000034'::uuid
) AND name IS DISTINCT FROM 'Texas Hold’em — Torneio';

UPDATE tournaments SET name = 'Texas Hold’em — Torneio Freeroll'
WHERE id IN (
    'c3000001-0001-4000-8000-000000000032'::uuid,
    'c3000001-0001-4000-8000-000000000035'::uuid
) AND name IS DISTINCT FROM 'Texas Hold’em — Torneio Freeroll';

UPDATE tournaments SET final_table_max_players = 8
WHERE id IN (
    'c3000001-0001-4000-8000-000000000032'::uuid,
    'c3000001-0001-4000-8000-000000000035'::uuid
) AND final_table_max_players IS DISTINCT FROM 8;

UPDATE tournaments SET table_max_players = 5
WHERE id IN (
    'c3000001-0001-4000-8000-000000000033'::uuid,
    'c3000001-0001-4000-8000-000000000036'::uuid
) AND table_max_players IS DISTINCT FROM 5;

UPDATE tournaments SET table_max_players = 6
WHERE id IN (
    'c3000001-0001-4000-8000-000000000037'::uuid,
    'c3000001-0001-4000-8000-000000000038'::uuid
) AND table_max_players IS DISTINCT FROM 6;

UPDATE tournaments SET table_max_players = 9
WHERE id IN (
    'c3000001-0001-4000-8000-000000000031'::uuid,
    'c3000001-0001-4000-8000-000000000032'::uuid,
    'c3000001-0001-4000-8000-000000000034'::uuid,
    'c3000001-0001-4000-8000-000000000035'::uuid
) AND table_max_players IS DISTINCT FROM 9;

-- 4. Volta os 8 para registering, zerando relógio/placar do sim e reagendando
UPDATE tournaments AS t SET
    status = 'registering',
    current_level = 0,
    started_at = NULL,
    finished_at = NULL,
    scheduled_start_at = 1788481800,
    auto_start_min_players = 5,
    players_remaining = COALESCE((
        SELECT COUNT(*) FROM tournament_players AS tp WHERE tp.tournament_id = t.id
    ), 0),
    total_buyins = COALESCE((
        SELECT COUNT(*) FROM tournament_players AS tp WHERE tp.tournament_id = t.id
    ), 0),
    prize_pool = t.guaranteed_prize
WHERE t.id IN (
    'c3000001-0001-4000-8000-000000000031'::uuid,
    'c3000001-0001-4000-8000-000000000032'::uuid,
    'c3000001-0001-4000-8000-000000000033'::uuid,
    'c3000001-0001-4000-8000-000000000034'::uuid,
    'c3000001-0001-4000-8000-000000000035'::uuid,
    'c3000001-0001-4000-8000-000000000036'::uuid,
    'c3000001-0001-4000-8000-000000000037'::uuid,
    'c3000001-0001-4000-8000-000000000038'::uuid
);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_CATALOG_RESTORED',
    jsonb_build_object(
        'migration', 43,
        'tournaments', 8,
        'status', 'registering',
        'scheduled_start_at', 1788481800,
        'note', 'retorna catálogo com Texas/FT8/Omaha5/Pineapple6; órfãos limpos, humanos preservados'
    )
);
