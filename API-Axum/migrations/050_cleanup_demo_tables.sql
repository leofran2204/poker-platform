-- 050: garante que as mesas demo (013) fiquem fora do lobby e do admin.
-- Fecha + privatiza qualquer mesa cash com nome Demo% (idempotente; reforça 018).
-- O lobby público só lista visibility='public' AND status='OPEN'.

UPDATE tables
SET status = 'CLOSED',
    visibility = 'private'
WHERE game_type = 'cash'
  AND name LIKE 'Demo%';
