-- 048: torneios em 3 mesas + fee 15% na rede 18/12.
-- 3 mesas fisicas por torneio: max_players = 3 x table_max_players.
-- live_table_ids: mesas vivas do torneio (a legado live_table_id segue primeira).
-- estrutura_ledger: fee de inscricao nao tem mao (hand_id NULL, source_type='fee').

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS live_table_ids UUID[] NOT NULL DEFAULT '{}';

UPDATE tournaments
SET max_players = 3 * table_max_players
WHERE table_max_players BETWEEN 2 AND 9
  AND (max_players IS NULL OR max_players <> 3 * table_max_players);

ALTER TABLE estrutura_ledger
    ALTER COLUMN hand_id DROP NOT NULL;

ALTER TABLE estrutura_ledger
    ADD COLUMN IF NOT EXISTS source_type VARCHAR(8) NOT NULL DEFAULT 'hand'
        CHECK (source_type IN ('hand', 'fee'));

ALTER TABLE estrutura_ledger
    ADD CONSTRAINT chk_estrutura_fee_no_hand
        CHECK (source_type <> 'fee' OR hand_id IS NULL);

ALTER TABLE estrutura_ledger
    ADD CONSTRAINT chk_estrutura_hand_needs_hand
        CHECK (source_type <> 'hand' OR hand_id IS NOT NULL);
