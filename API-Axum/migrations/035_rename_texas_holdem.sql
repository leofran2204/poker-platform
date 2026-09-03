-- 035: Renomeia "Hold’em — Torneio" -> "Texas Hold’em — Torneio" (FT label já é frontend)
-- Frontend deckTypeLabel agora é "Tradicional/Short Deck na FT" (gameLabels.ts:30)

UPDATE tournaments
SET name = 'Texas Hold’em — Torneio'
WHERE name = 'Hold’em — Torneio';

UPDATE tournaments
SET name = 'Texas Hold’em — Torneio Freeroll'
WHERE name = 'Hold’em — Torneio Freeroll';
