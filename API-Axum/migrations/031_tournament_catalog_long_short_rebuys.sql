-- 031: Catálogo canônico de torneios (Hold'em, freeroll Long/Short e Omaha).
-- Valores monetários em centavos. Os seis torneios são espelhados entre
-- Play Money e Jogo Real, sem prefixos de modo ou "GTD" no nome exibido.

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS final_table_variant VARCHAR(30),
    ADD COLUMN IF NOT EXISTS final_table_max_players SMALLINT;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'chk_tournaments_final_table_variant'
    ) THEN
        ALTER TABLE tournaments
            ADD CONSTRAINT chk_tournaments_final_table_variant
            CHECK (
                (final_table_variant IS NULL AND final_table_max_players IS NULL)
                OR (
                    final_table_variant = 'short_deck'
                    AND final_table_max_players BETWEEN 2 AND 6
                )
            );
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM tournaments
        WHERE id IN (
            'c3000001-0001-4000-8000-000000000022'::uuid,
            'c3000001-0001-4000-8000-000000000024'::uuid,
            'c3000001-0001-4000-8000-000000000027'::uuid,
            'c3000001-0001-4000-8000-000000000028'::uuid
        )
          AND status IN ('running', 'paused')
    ) THEN
        RAISE EXCEPTION 'migration 031: torneio substituído está em andamento';
    END IF;
END $$;

CREATE TEMP TABLE migration_031_replaced_tournaments ON COMMIT DROP AS
SELECT id, name, money_mode, buy_in, rebuy_cost
FROM tournaments
WHERE id IN (
    'c3000001-0001-4000-8000-000000000022'::uuid,
    'c3000001-0001-4000-8000-000000000024'::uuid,
    'c3000001-0001-4000-8000-000000000027'::uuid,
    'c3000001-0001-4000-8000-000000000028'::uuid
)
  AND status = 'registering';

CREATE TEMP TABLE migration_031_refunds ON COMMIT DROP AS
SELECT
    tp.player_id::uuid AS user_id,
    tournament.id AS tournament_id,
    tournament.name AS tournament_name,
    COALESCE(tournament.money_mode, 'play') AS money_mode,
    (
        tournament.buy_in
        + tp.rebuys::bigint * CASE
            WHEN tournament.rebuy_cost > 0 THEN tournament.rebuy_cost
            ELSE tournament.buy_in
          END
    )::bigint AS amount
FROM migration_031_replaced_tournaments AS tournament
JOIN tournament_players AS tp ON tp.tournament_id = tournament.id;

WITH refunds AS (
    SELECT
        user_id,
        SUM(amount) FILTER (WHERE money_mode = 'play') AS play_amount,
        SUM(amount) FILTER (WHERE money_mode = 'real') AS real_amount
    FROM migration_031_refunds
    GROUP BY user_id
)
UPDATE users AS account
SET balance_pm_mtt = account.balance_pm_mtt + COALESCE(refunds.play_amount, 0),
    balance_real = account.balance_real + COALESCE(refunds.real_amount, 0)
FROM refunds
WHERE account.id = refunds.user_id;

INSERT INTO audit_logs (user_id, action, metadata)
SELECT
    refund.user_id::text,
    'TOURNAMENT_CANCELLATION_REFUND',
    jsonb_build_object(
        'migration', 31,
        'tournament_id', refund.tournament_id,
        'tournament_name', refund.tournament_name,
        'wallet_mode', refund.money_mode,
        'amount_cents', refund.amount
    )
FROM migration_031_refunds AS refund
WHERE refund.amount > 0;

UPDATE tournaments AS tournament
SET status = 'cancelled',
    finished_at = COALESCE(tournament.finished_at, EXTRACT(EPOCH FROM NOW())::bigint)
FROM migration_031_replaced_tournaments AS replaced
WHERE tournament.id = replaced.id;

-- Play Money: Hold'em tradicional R$150 garantidos.
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode,
    poker_variant, final_table_variant, final_table_max_players
) VALUES (
    'c3000001-0001-4000-8000-000000000031'::uuid,
    'Hold’em — Torneio',
    1500, 10000, 100, 9,
    TRUE, 4, 'normal', 'registering',
    15000, 0, 0, 0,
    15000, FALSE,
    1500, 15000, 1, 0,
    6, TRUE, DEFAULT, 'Holdem', 'play',
    'holdem', NULL, NULL
);

-- Play Money: freeroll Long Deck que muda para Short Deck na mesa final (6 jogadores).
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode,
    poker_variant, final_table_variant, final_table_max_players
) VALUES (
    'c3000001-0001-4000-8000-000000000032'::uuid,
    'Hold’em — Torneio Freeroll',
    0, 5000, 100, 9,
    TRUE, 4, 'normal', 'registering',
    7500, 0, 0, 0,
    7500, TRUE,
    1000, 10000, 1, 0,
    6, TRUE, DEFAULT, 'HoldemLongShort', 'play',
    'holdem', 'short_deck', 6
);

-- Play Money: Omaha 4 cartas Short Deck R$100 garantidos.
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode,
    poker_variant, final_table_variant, final_table_max_players
) VALUES (
    'c3000001-0001-4000-8000-000000000033'::uuid,
    'Omaha 4 Cartas — Torneio',
    1000, 10000, 100, 4,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, FALSE,
    2000, 20000, 1, 0,
    6, TRUE, DEFAULT, 'ShortDeckOmaha', 'play',
    'short_deck_omaha', NULL, NULL
);

-- Jogo Real: espelho exato do catálogo Play Money.
INSERT INTO tournaments (
    id, name, buy_in, starting_stack, max_players, table_max_players,
    late_registration, late_reg_max_level, speed, status,
    prize_pool, current_level, players_remaining, total_buyins,
    guaranteed_prize, is_freeroll,
    rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
    rebuy_max_level, allow_rebuy, blind_levels, game_type, money_mode,
    poker_variant, final_table_variant, final_table_max_players
) VALUES
(
    'c3000001-0001-4000-8000-000000000034'::uuid,
    'Hold’em — Torneio',
    1500, 10000, 100, 9,
    TRUE, 4, 'normal', 'registering',
    15000, 0, 0, 0,
    15000, FALSE,
    1500, 15000, 1, 0,
    6, TRUE, DEFAULT, 'Holdem', 'real',
    'holdem', NULL, NULL
),
(
    'c3000001-0001-4000-8000-000000000035'::uuid,
    'Hold’em — Torneio Freeroll',
    0, 5000, 100, 9,
    TRUE, 4, 'normal', 'registering',
    7500, 0, 0, 0,
    7500, TRUE,
    1000, 10000, 1, 0,
    6, TRUE, DEFAULT, 'HoldemLongShort', 'real',
    'holdem', 'short_deck', 6
),
(
    'c3000001-0001-4000-8000-000000000036'::uuid,
    'Omaha 4 Cartas — Torneio',
    1000, 10000, 100, 4,
    TRUE, 4, 'normal', 'registering',
    10000, 0, 0, 0,
    10000, FALSE,
    2000, 20000, 1, 0,
    6, TRUE, DEFAULT, 'ShortDeckOmaha', 'real',
    'short_deck_omaha', NULL, NULL
);

-- Omaha abre em 50/50; os demais níveis e o total de 26 permanecem canônicos.
UPDATE tournaments
SET blind_levels = jsonb_set(
        jsonb_set(blind_levels, '{0,small_blind}', '50'::jsonb, false),
        '{0,big_blind}',
        '50'::jsonb,
        false
    )
WHERE id IN (
    'c3000001-0001-4000-8000-000000000033'::uuid,
    'c3000001-0001-4000-8000-000000000036'::uuid
);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'TOURNAMENT_CATALOG_LONG_SHORT_CREATED',
    jsonb_build_object(
        'migration', 31,
        'modes', jsonb_build_array('play', 'real'),
        'holdem', jsonb_build_object(
            'guaranteed_prize', 15000,
            'buy_in', 1500,
            'starting_stack', 10000,
            'rebuy_cost', 1500,
            'rebuy_chips', 15000,
            'rebuy_max_level', 6
        ),
        'freeroll_long_short', jsonb_build_object(
            'guaranteed_prize', 7500,
            'starting_stack', 5000,
            'rebuy_cost', 1000,
            'rebuy_chips', 10000,
            'rebuy_max_level', 6,
            'final_table_variant', 'short_deck',
            'final_table_max_players', 6
        ),
        'omaha_short_deck', jsonb_build_object(
            'guaranteed_prize', 10000,
            'buy_in', 1000,
            'starting_stack', 10000,
            'rebuy_cost', 2000,
            'rebuy_chips', 20000,
            'rebuy_max_level', 6
        ),
        'blind_levels', 26
    )
);
