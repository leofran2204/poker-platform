-- wipe-all-test-accounts.sql — Zera TODAS as contas locais (uso lab)
-- Preserva schema/tables/tournaments/hand_history. Apaga users + dependências.
-- Backup obrigatório: pg_dump -U user -d poker_db > backups/pre_wipe_all_$(date +%Y%m%d_%H%M%S).sql
BEGIN;
CREATE TEMP TABLE _wipe_ids ON COMMIT DROP AS SELECT id FROM users;
CREATE TEMP TABLE wipe_seats ON COMMIT DROP AS SELECT s.id AS seat_id, s.user_id, s.table_id, s.chips, s.wallet_kind FROM cash_game_seats s JOIN _wipe_ids w ON w.id=s.user_id WHERE s.status='ACTIVE';
UPDATE users SET balance_pm_cash=balance_pm_cash+c.pm_cash, balance=balance_pm_cash+c.pm_cash, balance_real=balance_real+c.real FROM (SELECT user_id, COALESCE(SUM(chips) FILTER(WHERE wallet_kind='pm_cash'),0)::bigint pm_cash, COALESCE(SUM(chips) FILTER(WHERE wallet_kind='real'),0)::bigint real FROM wipe_seats GROUP BY user_id) c WHERE users.id=c.user_id;
INSERT INTO cash_game_ledger(user_id,table_id,seat_id,entry_type,amount) SELECT user_id,table_id,seat_id,'CASH_OUT',chips FROM wipe_seats WHERE chips>0;
UPDATE cash_game_seats SET status='CASHED_OUT', cashed_out_at=NOW() WHERE id IN (SELECT seat_id FROM wipe_seats);
DELETE FROM hand_participants WHERE user_id IN (SELECT id FROM _wipe_ids);
DELETE FROM cash_game_ledger WHERE user_id IN (SELECT id FROM _wipe_ids);
DELETE FROM cash_game_seats WHERE user_id IN (SELECT id FROM _wipe_ids);
DELETE FROM tournament_players WHERE player_id IN (SELECT id::text FROM _wipe_ids);
DELETE FROM users WHERE id IN (SELECT id FROM _wipe_ids);
COMMIT;
-- Redis: docker exec poker_redis redis-cli FLUSHDB
