-- 026: Repair PM daily resets that ignored table escrow and cash out seats
-- left active on CLOSED catalog tables. All amounts are integer cents.

-- The previous reset wrote the full daily grant to the wallet even when part
-- of the bankroll was already held by a seat from an earlier day. Subtract
-- that escrow once before enabling the corrected application-level reset.
WITH reset_escrow AS (
    SELECT
        users.id AS user_id,
        COALESCE(SUM(seats.chips), 0)::BIGINT AS escrow
    FROM users
    JOIN cash_game_seats AS seats
      ON seats.user_id = users.id
     AND seats.status = 'ACTIVE'
     AND seats.wallet_kind = 'pm_cash'
     AND (timezone('America/Sao_Paulo', seats.joined_at))::DATE
         < users.last_pm_reset_date
    WHERE users.last_pm_reset_date IS NOT NULL
    GROUP BY users.id
), repaired AS (
    UPDATE users
    SET
        balance_pm_cash = GREATEST(0, users.balance_pm_cash - reset_escrow.escrow),
        balance = GREATEST(0, users.balance_pm_cash - reset_escrow.escrow)
    FROM reset_escrow
    WHERE users.id = reset_escrow.user_id
      AND reset_escrow.escrow > 0
    RETURNING users.id, reset_escrow.escrow
)
INSERT INTO audit_logs (user_id, action, metadata)
SELECT
    repaired.id::TEXT,
    'PM_RESET_ESCROW_RECONCILED',
    jsonb_build_object('escrow_cents', repaired.escrow, 'migration', 26)
FROM repaired;

-- A CLOSED table cannot have a live actor accepting actions. With no recovery
-- guard, its persisted stack is safe to return to the originating wallet.
CREATE TEMP TABLE migration_026_closed_seats ON COMMIT DROP AS
SELECT
    seats.id AS seat_id,
    seats.user_id,
    seats.table_id,
    seats.chips,
    seats.wallet_kind
FROM cash_game_seats AS seats
JOIN tables ON tables.id = seats.table_id
WHERE seats.status = 'ACTIVE'
  AND tables.status = 'CLOSED'
  AND NOT EXISTS (
      SELECT 1
      FROM table_hand_recovery_guards AS guard
      WHERE guard.table_id = seats.table_id
  );

UPDATE users
SET
    balance_pm_cash = balance_pm_cash + credits.pm_cash,
    balance = balance_pm_cash + credits.pm_cash,
    balance_real = balance_real + credits.real
FROM (
    SELECT
        user_id,
        COALESCE(SUM(chips) FILTER (WHERE wallet_kind = 'pm_cash'), 0)::BIGINT AS pm_cash,
        COALESCE(SUM(chips) FILTER (WHERE wallet_kind = 'real'), 0)::BIGINT AS real
    FROM migration_026_closed_seats
    GROUP BY user_id
) AS credits
WHERE users.id = credits.user_id;

INSERT INTO cash_game_ledger (user_id, table_id, seat_id, entry_type, amount)
SELECT user_id, table_id, seat_id, 'CASH_OUT', chips
FROM migration_026_closed_seats
WHERE chips > 0;

INSERT INTO audit_logs (user_id, action, metadata)
SELECT
    user_id::TEXT,
    'CLOSED_TABLE_SEAT_RECONCILED',
    jsonb_build_object(
        'table_id', table_id,
        'seat_id', seat_id,
        'chips_cents', chips,
        'wallet_kind', wallet_kind,
        'migration', 26
    )
FROM migration_026_closed_seats;

UPDATE cash_game_seats AS seats
SET status = 'CASHED_OUT', cashed_out_at = NOW()
FROM migration_026_closed_seats AS stale
WHERE seats.id = stale.seat_id
  AND seats.status = 'ACTIVE';
