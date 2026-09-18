-- 056: valor líquido creditado no depósito (taxa do provedor).
-- A cobrança guarda o valor de face; o crédito segue o líquido recebido
-- (`amount_received` do webhook). Backfill: liquidados antigos = face.

ALTER TABLE wallet_transactions
    ADD COLUMN IF NOT EXISTS credited_amount_cents BIGINT;

UPDATE wallet_transactions
    SET credited_amount_cents = amount
    WHERE status = 'COMPLETED' AND credited_amount_cents IS NULL;
