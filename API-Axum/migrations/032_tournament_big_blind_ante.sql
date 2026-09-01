-- 032: Big Blind Ante em todos os torneios, desde o nivel 1.
-- O jogador no Big Blind paga primeiro o blind. O ante so e cobrado depois
-- do Big completo e entra como dinheiro morto no pote principal.

ALTER TABLE tournaments
    ALTER COLUMN blind_levels SET DEFAULT '[
      {"level":1,"small_blind":25,"big_blind":50,"ante":50,"duration_minutes":5},
      {"level":2,"small_blind":50,"big_blind":100,"ante":100,"duration_minutes":5},
      {"level":3,"small_blind":75,"big_blind":150,"ante":150,"duration_minutes":5},
      {"level":4,"small_blind":100,"big_blind":200,"ante":200,"duration_minutes":5},
      {"level":5,"small_blind":150,"big_blind":300,"ante":300,"duration_minutes":5},
      {"level":6,"small_blind":200,"big_blind":400,"ante":400,"duration_minutes":5},
      {"level":7,"small_blind":300,"big_blind":600,"ante":600,"duration_minutes":5},
      {"level":8,"small_blind":400,"big_blind":800,"ante":800,"duration_minutes":5},
      {"level":9,"small_blind":500,"big_blind":1000,"ante":1000,"duration_minutes":5},
      {"level":10,"small_blind":600,"big_blind":1200,"ante":1200,"duration_minutes":5},
      {"level":11,"small_blind":800,"big_blind":1600,"ante":1600,"duration_minutes":5},
      {"level":12,"small_blind":1000,"big_blind":2000,"ante":2000,"duration_minutes":5},
      {"level":13,"small_blind":1200,"big_blind":2400,"ante":2400,"duration_minutes":5},
      {"level":14,"small_blind":1500,"big_blind":3000,"ante":3000,"duration_minutes":5},
      {"level":15,"small_blind":2000,"big_blind":4000,"ante":4000,"duration_minutes":5},
      {"level":16,"small_blind":2500,"big_blind":5000,"ante":5000,"duration_minutes":5},
      {"level":17,"small_blind":3000,"big_blind":6000,"ante":6000,"duration_minutes":5},
      {"level":18,"small_blind":4000,"big_blind":8000,"ante":8000,"duration_minutes":5},
      {"level":19,"small_blind":5000,"big_blind":10000,"ante":10000,"duration_minutes":5},
      {"level":20,"small_blind":6000,"big_blind":12000,"ante":12000,"duration_minutes":5},
      {"level":21,"small_blind":8000,"big_blind":16000,"ante":16000,"duration_minutes":5},
      {"level":22,"small_blind":10000,"big_blind":20000,"ante":20000,"duration_minutes":5},
      {"level":23,"small_blind":12000,"big_blind":24000,"ante":24000,"duration_minutes":5},
      {"level":24,"small_blind":15000,"big_blind":30000,"ante":30000,"duration_minutes":5},
      {"level":25,"small_blind":20000,"big_blind":40000,"ante":40000,"duration_minutes":5},
      {"level":26,"small_blind":25000,"big_blind":50000,"ante":50000,"duration_minutes":5}
    ]'::jsonb;

WITH rewritten AS (
    SELECT
        tournament.id,
        jsonb_agg(
            jsonb_set(
                level.value,
                '{ante}',
                to_jsonb((level.value ->> 'big_blind')::bigint),
                true
            )
            ORDER BY level.ordinality
        ) AS blind_levels
    FROM tournaments AS tournament
    CROSS JOIN LATERAL jsonb_array_elements(tournament.blind_levels)
        WITH ORDINALITY AS level(value, ordinality)
    WHERE tournament.status IN ('registering', 'running', 'paused')
    GROUP BY tournament.id
)
UPDATE tournaments AS tournament
SET blind_levels = rewritten.blind_levels
FROM rewritten
WHERE tournament.id = rewritten.id;

-- Remove a nomenclatura tecnica Long/Short tambem do campo legado exposto pela API.
-- Valor mantido dentro de VARCHAR(30) do schema (20 chars).
UPDATE tournaments
SET game_type = 'HoldemFTShortDeck'
WHERE id IN (
    'c3000001-0001-4000-8000-000000000032'::uuid,
    'c3000001-0001-4000-8000-000000000035'::uuid
);

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM tournaments AS tournament
        WHERE tournament.status IN ('registering', 'running', 'paused')
          AND (
              jsonb_typeof(tournament.blind_levels) <> 'array'
              OR jsonb_array_length(tournament.blind_levels) <> 26
              OR EXISTS (
                  SELECT 1
                  FROM jsonb_array_elements(tournament.blind_levels) AS level(value)
                  WHERE (level.value ->> 'ante')::bigint
                      <> (level.value ->> 'big_blind')::bigint
              )
          )
    ) THEN
        RAISE EXCEPTION 'migration 032: estrutura Big Blind Ante invalida';
    END IF;
END $$;

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_BIG_BLIND_ANTE_ENABLED',
    jsonb_build_object(
        'migration', 32,
        'starts_at_level', 1,
        'levels', 26,
        'ante_equals_big_blind', true,
        'blind_has_priority', true,
        'short_big_blind_skips_ante', true
    )
);
