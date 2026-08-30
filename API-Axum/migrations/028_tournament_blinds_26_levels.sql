-- 028: Canonical tournament structure with 26 five-minute blind levels.
-- Active tournaments are updated; finished/cancelled history remains immutable.

ALTER TABLE tournaments
    ALTER COLUMN blind_levels SET DEFAULT '[
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
      {"level":12,"small_blind":1000,"big_blind":2000,"ante":300,"duration_minutes":5},
      {"level":13,"small_blind":1200,"big_blind":2400,"ante":400,"duration_minutes":5},
      {"level":14,"small_blind":1500,"big_blind":3000,"ante":500,"duration_minutes":5},
      {"level":15,"small_blind":2000,"big_blind":4000,"ante":500,"duration_minutes":5},
      {"level":16,"small_blind":2500,"big_blind":5000,"ante":700,"duration_minutes":5},
      {"level":17,"small_blind":3000,"big_blind":6000,"ante":800,"duration_minutes":5},
      {"level":18,"small_blind":4000,"big_blind":8000,"ante":1000,"duration_minutes":5},
      {"level":19,"small_blind":5000,"big_blind":10000,"ante":1500,"duration_minutes":5},
      {"level":20,"small_blind":6000,"big_blind":12000,"ante":2000,"duration_minutes":5},
      {"level":21,"small_blind":8000,"big_blind":16000,"ante":2500,"duration_minutes":5},
      {"level":22,"small_blind":10000,"big_blind":20000,"ante":3000,"duration_minutes":5},
      {"level":23,"small_blind":12000,"big_blind":24000,"ante":4000,"duration_minutes":5},
      {"level":24,"small_blind":15000,"big_blind":30000,"ante":5000,"duration_minutes":5},
      {"level":25,"small_blind":20000,"big_blind":40000,"ante":6000,"duration_minutes":5},
      {"level":26,"small_blind":25000,"big_blind":50000,"ante":8000,"duration_minutes":5}
    ]'::jsonb;

UPDATE tournaments
SET blind_levels = DEFAULT
WHERE status IN ('registering', 'running', 'paused');

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_active_tournaments_have_26_blind_levels'
    ) THEN
        ALTER TABLE tournaments
            ADD CONSTRAINT chk_active_tournaments_have_26_blind_levels
            CHECK (
                status NOT IN ('registering', 'running', 'paused')
                OR (
                    jsonb_typeof(blind_levels) = 'array'
                    AND jsonb_array_length(blind_levels) = 26
                )
            ) NOT VALID;
    END IF;
END $$;

ALTER TABLE tournaments
    VALIDATE CONSTRAINT chk_active_tournaments_have_26_blind_levels;

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_BLINDS_EXPANDED',
    jsonb_build_object(
        'levels', 26,
        'duration_minutes', 5,
        'first_level', jsonb_build_object('small_blind', 25, 'big_blind', 50, 'ante', 0),
        'last_level', jsonb_build_object('small_blind', 25000, 'big_blind', 50000, 'ante', 8000)
    )
);
