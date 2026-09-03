-- 036: Texas Hold’em Freeroll FT Short Deck 6 -> 8 jogadores
-- Atualiza constraint para permitir 8 e altera os 2 torneios Texas Freeroll (play/real)

ALTER TABLE tournaments DROP CONSTRAINT IF EXISTS chk_tournaments_final_table_variant;

ALTER TABLE tournaments
    ADD CONSTRAINT chk_tournaments_final_table_variant
    CHECK (
        (final_table_variant IS NULL AND final_table_max_players IS NULL)
        OR (
            final_table_variant = 'short_deck'
            AND final_table_max_players BETWEEN 2 AND 8
        )
    );

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM tournaments
        WHERE id IN (
            'c3000001-0001-4000-8000-000000000032'::uuid,
            'c3000001-0001-4000-8000-000000000035'::uuid
        ) AND status IN ('running', 'paused')
    ) THEN
        RAISE EXCEPTION 'migration 036: Texas Freeroll FT 8 não pode alterar torneio em andamento';
    END IF;
END $$;

UPDATE tournaments
SET final_table_max_players = 8
WHERE id IN (
    'c3000001-0001-4000-8000-000000000032'::uuid,
    'c3000001-0001-4000-8000-000000000035'::uuid
);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_FT_8_ENABLED',
    jsonb_build_object('migration', 36, 'tournaments', jsonb_build_array('c3000001-0001-4000-8000-000000000032','c3000001-0001-4000-8000-000000000035'), 'final_table_max_players', 8)
);
