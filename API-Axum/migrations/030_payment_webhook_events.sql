-- 030: Durable, provider-scoped webhook deduplication.
--
-- Only event metadata and a SHA-256 digest of the signed body are retained.
-- Payer documents and raw provider payloads are deliberately not stored.

ALTER TABLE wallet_transactions
    ADD COLUMN IF NOT EXISTS provider_payment_url TEXT,
    ADD COLUMN IF NOT EXISTS provider_expires_at VARCHAR(64);
CREATE TABLE IF NOT EXISTS payment_webhook_events (
    provider VARCHAR(32) NOT NULL,
    event_id VARCHAR(128) NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    payload_sha256 CHAR(64) NOT NULL
        CHECK (payload_sha256 ~ '^[0-9a-f]{64}$'),
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (provider, event_id)
);

CREATE INDEX IF NOT EXISTS idx_payment_webhook_events_received_at
    ON payment_webhook_events (received_at DESC);
