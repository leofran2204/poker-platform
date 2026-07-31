-- 012: Persist Loss Deflator audit (equity, tier, cashback) per hand.

ALTER TABLE hand_history
    ADD COLUMN IF NOT EXISTS loss_deflators_json JSONB NOT NULL DEFAULT '[]'::jsonb;
