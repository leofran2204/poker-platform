-- 010: durably marks an in-progress hand before cards or chips can change.
-- A remaining guard after a process crash means the hand was never committed
-- atomically with its final stacks; startup pauses the table for an explicit
-- administrator abort/review instead of silently resuming it.

CREATE TABLE IF NOT EXISTS table_hand_recovery_guards (
    table_id UUID PRIMARY KEY REFERENCES tables(id) ON DELETE RESTRICT,
    hand_id UUID NOT NULL UNIQUE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_table_hand_recovery_guards_started
    ON table_hand_recovery_guards(started_at);

CREATE OR REPLACE FUNCTION prevent_opening_table_with_recovery_guard()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'OPEN' AND EXISTS (
        SELECT 1
        FROM table_hand_recovery_guards
        WHERE table_id = NEW.id
    ) THEN
        RAISE EXCEPTION 'table % has an unrecovered hand', NEW.id
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


DROP TRIGGER IF EXISTS table_hand_recovery_guard_blocks_open ON tables;
CREATE TRIGGER table_hand_recovery_guard_blocks_open
    BEFORE UPDATE OF status ON tables
    FOR EACH ROW
    EXECUTE FUNCTION prevent_opening_table_with_recovery_guard();