-- 037: Omaha Short Deck 4 -> 5-max, Pineapple garante 6-max (cash + torneio)
-- Valores monetários em centavos. Idempotente.

UPDATE tables
SET max_players = 5
WHERE poker_variant = 'short_deck_omaha'
  AND max_players != 5;

UPDATE tournaments
SET table_max_players = 5
WHERE poker_variant = 'short_deck_omaha'
  AND table_max_players != 5;

-- Pineapple já é 6-max, garante
UPDATE tables
SET max_players = 6
WHERE poker_variant = 'ultimate_pineapple'
  AND max_players != 6;

UPDATE tournaments
SET table_max_players = 6
WHERE poker_variant = 'ultimate_pineapple'
  AND table_max_players != 6;

INSERT INTO audit_logs (user_id, action, metadata)
VALUES (
    'system',
    'CASH_TOURNAMENT_MAX_PLAYERS_ADJUSTED',
    jsonb_build_object('migration', 37, 'omaha_sd', 5, 'pineapple', 6)
);
