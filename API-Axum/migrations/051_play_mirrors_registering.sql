-- 051: Play Money com os mesmos torneios do Jogo Real.
-- Os 4 espelhos play do catálogo (031 Texas, 032 Freeroll, 033 Omaha, 037 Pineapple)
-- ficaram cancelled/finished e sumiram do lobby (só registering/running/paused carregam).
-- Volta os 4 para registering, zerando relógio/placar e reagendando 21:30 SP,
-- preservando inscrições humanas existentes. Idempotente (mesmo padrão da 043).

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
    'c3000001-0001-4000-8000-000000000037'::uuid
);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_PLAY_MIRRORS_RESTORED',
    jsonb_build_object(
        'migration', 51,
        'tournaments', 4,
        'status', 'registering',
        'scheduled_start_at', 1788481800,
        'note', 'play com os mesmos 4 do real: Texas/FT8 Freeroll/Omaha5/Pineapple6'
    )
);
