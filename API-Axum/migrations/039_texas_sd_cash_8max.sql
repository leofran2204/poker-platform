-- 039: Cash Texas Hold’em Short Deck 6 -> 8-max (PM e Real)
-- Mantém Omaha 5-max e Pineapple 6-max (037). Rake 5+ cap já cobre 8 (500).

UPDATE tables
SET max_players = 8
WHERE poker_variant = 'short_deck'
  AND name IN ('PM · SD 0,25/0,50', 'Real · SD 0,25/0,50')
  AND max_players != 8;

-- Garante SDS 0,25/0,50 caso tenha sido recriado como 6
UPDATE tables
SET max_players = 8
WHERE poker_variant = 'short_deck'
  AND small_blind = 50 AND big_blind = 50
  AND money_mode IN ('play','real')
  AND max_players = 6;

INSERT INTO audit_logs (user_id, action, metadata)
VALUES ('system','CASH_SD_8MAX_ENABLED', jsonb_build_object('migration',39,'tables', jsonb_build_array('PM · SD 0,25/0,50','Real · SD 0,25/0,50'),'max_players',8));
