-- 006: Invariantes de mesas cash aplicadas no PostgreSQL.
-- `current_players` é uma projeção dos assentos ativos, nunca uma fonte de verdade.

ALTER TABLE tables
    ADD CONSTRAINT chk_tables_blind_structure
        CHECK (small_blind > 0 AND big_blind > 0 AND big_blind % 2 = 0 AND big_blind / 2 = small_blind),
    ADD CONSTRAINT chk_tables_buy_in_range
        CHECK (min_buy_in > 0 AND max_buy_in >= min_buy_in),
    ADD CONSTRAINT chk_tables_max_players
        CHECK (max_players BETWEEN 2 AND 9),
    ADD CONSTRAINT chk_tables_current_players
        CHECK (current_players BETWEEN 0 AND max_players);

CREATE OR REPLACE FUNCTION refresh_cash_game_table_player_count(p_table_id UUID)
RETURNS VOID
LANGUAGE sql
AS $$
    UPDATE tables
    SET current_players = (
        SELECT COUNT(*)::SMALLINT
        FROM cash_game_seats
        WHERE table_id = p_table_id AND status = 'ACTIVE'
    )
    WHERE id = p_table_id;
$$;

CREATE OR REPLACE FUNCTION validate_cash_game_seat()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    table_max_players SMALLINT;
    table_status VARCHAR(20);
BEGIN
    IF NEW.status = 'ACTIVE' THEN
        SELECT max_players, status
        INTO table_max_players, table_status
        FROM tables
        WHERE id = NEW.table_id
        FOR KEY SHARE;

        IF NOT FOUND THEN
            RAISE EXCEPTION 'cash-game table does not exist'
                USING ERRCODE = 'foreign_key_violation';
        END IF;
        IF table_status <> 'OPEN' THEN
            RAISE EXCEPTION 'cash-game table is not accepting new seats'
                USING ERRCODE = 'check_violation';
        END IF;
        IF NEW.seat >= table_max_players THEN
            RAISE EXCEPTION 'seat % exceeds table capacity %', NEW.seat, table_max_players
                USING ERRCODE = 'check_violation';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION sync_cash_game_table_player_count()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        PERFORM refresh_cash_game_table_player_count(OLD.table_id);
    ELSIF TG_OP = 'UPDATE' THEN
        PERFORM refresh_cash_game_table_player_count(NEW.table_id);
        IF OLD.table_id <> NEW.table_id THEN
            PERFORM refresh_cash_game_table_player_count(OLD.table_id);
        END IF;
    ELSE
        PERFORM refresh_cash_game_table_player_count(NEW.table_id);
    END IF;
    RETURN NULL;
END;
$$;

CREATE TRIGGER trg_cash_game_seat_validate
    BEFORE INSERT OR UPDATE OF table_id, seat, status ON cash_game_seats
    FOR EACH ROW
    EXECUTE FUNCTION validate_cash_game_seat();

CREATE TRIGGER trg_cash_game_seat_sync_count
    AFTER INSERT OR DELETE OR UPDATE OF table_id, status ON cash_game_seats
    FOR EACH ROW
    EXECUTE FUNCTION sync_cash_game_table_player_count();

UPDATE tables AS table_row
SET current_players = (
    SELECT COUNT(*)::SMALLINT
    FROM cash_game_seats
    WHERE table_id = table_row.id AND status = 'ACTIVE'
);
