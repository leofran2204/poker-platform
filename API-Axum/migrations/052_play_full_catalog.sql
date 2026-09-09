-- 052: Play Money com os mesmos 4 torneios do Jogo Real (reforço da 051).
-- Freeroll (032) e Pineapple (037) play foram cancelados no wipe de contas
-- artificiais; volta ambos para registering com relógio/placar zerados e
-- agenda 21:30 SP. Sem inscritos a preservar. Idempotente.

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
    'c3000001-0001-4000-8000-000000000032'::uuid,
    'c3000001-0001-4000-8000-000000000037'::uuid
);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_PLAY_FULL_CATALOG',
    jsonb_build_object(
        'migration', 52,
        'tournaments', 2,
        'status', 'registering',
        'note', 'play 4x4 igual ao real: Freeroll 032 + Pineapple 037 reativados'
    )
);
