-- 027: Replace the Short Deck freerolls with paid Short Deck Omaha R$100 GTD.
-- Existing freeroll registrations are preserved under cancelled tournaments for auditability.

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS table_max_players SMALLINT NOT NULL DEFAULT 9;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_tournaments_table_max_players'
    ) THEN
        ALTER TABLE tournaments
            ADD CONSTRAINT chk_tournaments_table_max_players
            CHECK (table_max_players BETWEEN 2 AND 9);
    END IF;
END $$;

-- Backfill the physical table size independently from tournament capacity.
UPDATE tournaments
SET table_max_players = CASE
    WHEN poker_variant = 'short_deck_omaha' THEN 4
    WHEN poker_variant = 'short_deck' THEN 6
    ELSE 9
END;

-- Remove the old freerolls from the active catalog without deleting their registrations.
UPDATE tournaments
SET status = 'cancelled',
    finished_at = COALESCE(finished_at, EXTRACT(EPOCH FROM NOW())::BIGINT)
WHERE id IN (
    'c3000001-0001-4000-8000-000000000021'::uuid,
    'c3000001-0001-4000-8000-000000000023'::uuid
)
  AND status <> 'cancelled';

INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode, poker_variant
)
SELECT
    'c3000001-0001-4000-8000-000000000027'::uuid,
    'PM · Omaha Short Deck R$100 GTD',
    1000, 10000, 100, 4,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, FALSE,
    0, 0, 0, 0,
    0, FALSE,
    COALESCE(
        (SELECT blind_levels FROM tournaments
         WHERE id = 'c3000001-0001-4000-8000-000000000021'::uuid),
        '[]'::jsonb
    ),
    'ShortDeckOmaha', 'play', 'short_deck_omaha'
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments
    WHERE id = 'c3000001-0001-4000-8000-000000000027'::uuid
);

INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode, poker_variant
)
SELECT
    'c3000001-0001-4000-8000-000000000028'::uuid,
    'Real · Omaha Short Deck R$100 GTD',
    1000, 10000, 100, 4,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, FALSE,
    0, 0, 0, 0,
    0, FALSE,
    COALESCE(
        (SELECT blind_levels FROM tournaments
         WHERE id = 'c3000001-0001-4000-8000-000000000023'::uuid),
        '[]'::jsonb
    ),
    'ShortDeckOmaha', 'real', 'short_deck_omaha'
WHERE NOT EXISTS (
    SELECT 1 FROM tournaments
    WHERE id = 'c3000001-0001-4000-8000-000000000028'::uuid
);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_CATALOG_REPLACED',
    jsonb_build_object(
        'cancelled_tournament_ids', jsonb_build_array(
            'c3000001-0001-4000-8000-000000000021',
            'c3000001-0001-4000-8000-000000000023'
        ),
        'created_tournament_ids', jsonb_build_array(
            'c3000001-0001-4000-8000-000000000027',
            'c3000001-0001-4000-8000-000000000028'
        ),
        'variant', 'short_deck_omaha',
        'guaranteed_prize', 10000,
        'buy_in', 1000,
        'table_max_players', 4
    )
);