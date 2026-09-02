BEGIN;

CREATE TEMP TABLE zombie_seats ON COMMIT DROP AS
SELECT
    seats.id AS seat_id,
    seats.user_id,
    seats.table_id,
    seats.chips,
    seats.wallet_kind
FROM cash_game_seats AS seats
JOIN tables ON tables.id = seats.table_id
WHERE seats.status = 'ACTIVE'
  AND tables.status = 'OPEN'
  AND tables.visibility = 'public'
  AND tables.game_type = 'cash'
  AND COALESCE(tables.money_mode, 'play') = 'play';

UPDATE users AS account
SET
    balance_pm_cash = account.balance_pm_cash + credits.pm_cash,
    balance = account.balance_pm_cash + credits.pm_cash,
    balance_real = account.balance_real + credits.real
FROM (
    SELECT
        user_id,
        COALESCE(SUM(chips) FILTER (WHERE wallet_kind = 'pm_cash'), 0)::bigint AS pm_cash,
        COALESCE(SUM(chips) FILTER (WHERE wallet_kind = 'real'), 0)::bigint AS real
    FROM zombie_seats
    GROUP BY user_id
) AS credits
WHERE account.id = credits.user_id;

INSERT INTO cash_game_ledger (user_id, table_id, seat_id, entry_type, amount)
SELECT user_id, table_id, seat_id, 'CASH_OUT', chips
FROM zombie_seats
WHERE chips > 0;

UPDATE cash_game_seats
SET status = 'CASHED_OUT', cashed_out_at = NOW()
WHERE id IN (SELECT seat_id FROM zombie_seats);

COMMIT;

SELECT name, current_players, max_players, status
FROM tables
WHERE COALESCE(money_mode, 'play') = 'play'
  AND game_type = 'cash'
  AND visibility = 'public'
  AND status = 'OPEN'
ORDER BY name;
