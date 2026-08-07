-- 017: Persist a complete, tamper-evident settlement without exposing hole cards.

ALTER TABLE hand_history
    ADD COLUMN IF NOT EXISTS settlement_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS settlement_signature VARCHAR(64);

ALTER TABLE hand_history
    ADD CONSTRAINT chk_hand_history_settlement_object
        CHECK (jsonb_typeof(settlement_json) = 'object');

ALTER TABLE hand_history
    ADD CONSTRAINT chk_hand_history_settlement_signature
        CHECK (
            settlement_signature IS NULL
            OR settlement_signature ~ '^[0-9a-f]{64}$'
        );
