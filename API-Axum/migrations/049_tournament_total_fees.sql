-- 049: total de fees 15% coletadas por torneio (espelho de total_buyins).
-- O split 18/12/70 de cada fee vive no estrutura_ledger (source_type='fee').

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS total_fees BIGINT NOT NULL DEFAULT 0;
