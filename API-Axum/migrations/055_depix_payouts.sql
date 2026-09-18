-- 055: payout worker reconciliado DePix (saques automáticos).
-- Chave PIX cifrada (AES-256-GCM, segredo só via env) + id externo do
-- provedor. A chave em claro nunca é persistida nem logada.

ALTER TABLE wallet_transactions
    ADD COLUMN IF NOT EXISTS pix_key_ciphertext TEXT;
ALTER TABLE wallet_transactions
    ADD COLUMN IF NOT EXISTS provider_tx_id VARCHAR(128);

CREATE INDEX IF NOT EXISTS idx_wallet_tx_payout_queue
    ON wallet_transactions(provider, provider_status)
    WHERE transaction_type = 'WITHDRAW' AND status = 'PENDING';
