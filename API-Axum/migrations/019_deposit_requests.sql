-- 019: Pedidos de crédito (PIX manual fora do site + aprovação admin)

CREATE TABLE IF NOT EXISTS deposit_requests (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount_cents    BIGINT NOT NULL CHECK (amount_cents > 0),
    status          VARCHAR(20) NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'approved', 'rejected', 'cancelled')),
    player_note     TEXT,
    proof_text      TEXT NOT NULL,
    proof_url       TEXT,
    admin_note      TEXT,
    reviewed_by     UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_deposit_requests_user_created
    ON deposit_requests (user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_deposit_requests_status_created
    ON deposit_requests (status, created_at DESC);
