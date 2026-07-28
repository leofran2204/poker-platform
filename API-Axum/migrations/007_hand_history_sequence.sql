-- 007: Sequência auditável de mãos por mesa e normalização do legado.

ALTER TABLE hand_history
    ALTER COLUMN hand_number TYPE BIGINT;

WITH numbered_hands AS (
    SELECT
        id,
        ROW_NUMBER() OVER (PARTITION BY table_id ORDER BY created_at, id) AS hand_number
    FROM hand_history
    WHERE table_id IS NOT NULL
)
UPDATE hand_history AS history
SET hand_number = numbered_hands.hand_number
FROM numbered_hands
WHERE history.id = numbered_hands.id;

ALTER TABLE hand_history
    ADD CONSTRAINT chk_hand_history_number_positive CHECK (hand_number > 0);

ALTER TABLE tables
    ADD COLUMN hand_sequence BIGINT NOT NULL DEFAULT 0
        CHECK (hand_sequence >= 0);

UPDATE tables AS table_row
SET hand_sequence = COALESCE(
    (
        SELECT MAX(hand_number)
        FROM hand_history
        WHERE table_id = table_row.id
    ),
    0
);

CREATE UNIQUE INDEX uq_hand_history_table_number
    ON hand_history(table_id, hand_number)
    WHERE table_id IS NOT NULL;
