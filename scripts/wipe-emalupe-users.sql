-- wipe-emalupe-users.sql — Remove ONLY @emalupe.com accounts (human-safe)
-- Usage: psql -U user -d poker_db -f wipe-emalupe-users.sql
-- Backup first: pg_dump -U user -d poker_db > backups/pre_wipe_emalupe_YYYYMMDD_HHMMSS.sql
-- Idempotente e transacional. Preserva humanos (não-emalupe).

BEGIN;

-- 1. Isola candidatos (case-insensitive, cobre subdomínios)
CREATE TEMP TABLE _wipe_ids ON COMMIT DROP AS
  SELECT id FROM users WHERE email ILIKE '%@%emalupe.com';

-- Dry-run: descomente para inspecionar antes do DELETE
-- SELECT id::text, username, email, role, status FROM users WHERE id IN (SELECT id FROM _wipe_ids);

-- 2. Se houver assentos ACTIVE, devolve escrow (evita FK RESTRICT e double-spend bug 026)
CREATE TEMP TABLE wipe_seats ON COMMIT DROP AS
  SELECT s.id AS seat_id, s.user_id, s.table_id, s.chips, s.wallet_kind
  FROM cash_game_seats s JOIN _wipe_ids w ON w.id = s.user_id
  WHERE s.status = 'ACTIVE';

UPDATE users SET
  balance_pm_cash = balance_pm_cash + c.pm_cash,
  balance = balance_pm_cash + c.pm_cash,
  balance_real = balance_real + c.real
FROM (
  SELECT user_id,
         COALESCE(SUM(chips) FILTER (WHERE wallet_kind='pm_cash'),0)::bigint AS pm_cash,
         COALESCE(SUM(chips) FILTER (WHERE wallet_kind='real'),0)::bigint AS real
  FROM wipe_seats GROUP BY user_id
) c WHERE users.id = c.user_id;

INSERT INTO cash_game_ledger(user_id, table_id, seat_id, entry_type, amount)
  SELECT user_id, table_id, seat_id, 'CASH_OUT', chips FROM wipe_seats WHERE chips > 0;

UPDATE cash_game_seats SET status='CASHED_OUT', cashed_out_at=NOW()
WHERE id IN (SELECT seat_id FROM wipe_seats);

-- 3. FK RESTRICT tables (precisam vir antes de users)
DELETE FROM hand_participants WHERE user_id IN (SELECT id FROM _wipe_ids);
DELETE FROM cash_game_ledger WHERE user_id IN (SELECT id FROM _wipe_ids);
DELETE FROM cash_game_seats WHERE user_id IN (SELECT id FROM _wipe_ids);
DELETE FROM tournament_players WHERE player_id IN (SELECT id::text FROM _wipe_ids);
-- audit_logs preservado para compliance; descomente se quiser apagar:
-- DELETE FROM audit_logs WHERE user_id IN (SELECT id::text FROM _wipe_ids);

-- 4. Users (CASCADE apaga sessions, wallet_transactions, club_memberships, email_verification_codes, auth_mfa_challenges, deposit_requests)
DELETE FROM users WHERE id IN (SELECT id FROM _wipe_ids);

-- 5. Validação dentro da transação
-- SELECT COUNT(*) AS emalupe_restante FROM users WHERE email ILIKE '%@%emalupe.com'; -- deve ser 0

COMMIT;

-- Pós-wipe Redis (executar fora do psql):
-- docker exec poker_redis redis-cli --scan --pattern 'poker:presence:*' | xargs -r docker exec poker_redis redis-cli DEL
-- docker exec poker_redis redis-cli --scan --pattern 'poker:ws-ticket:*' | xargs -r docker exec poker_redis redis-cli DEL
