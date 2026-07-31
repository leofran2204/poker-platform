-- 011: Caps de rake por quantidade de jogadores que receberam cartas.
-- NULL mantém compatibilidade e usa o rake_cap legado da mesa.

ALTER TABLE tables
    ADD COLUMN rake_cap_heads_up BIGINT NULL
        CHECK (rake_cap_heads_up >= 0),
    ADD COLUMN rake_cap_three_to_four BIGINT NULL
        CHECK (rake_cap_three_to_four >= 0),
    ADD COLUMN rake_cap_five_plus BIGINT NULL
        CHECK (rake_cap_five_plus >= 0),
    ADD CONSTRAINT tables_rake_cap_schedule_complete
        CHECK (
            (
                rake_cap_heads_up IS NULL
                AND rake_cap_three_to_four IS NULL
                AND rake_cap_five_plus IS NULL
            )
            OR
            (
                rake_cap_heads_up IS NOT NULL
                AND rake_cap_three_to_four IS NOT NULL
                AND rake_cap_five_plus IS NOT NULL
            )
        );
