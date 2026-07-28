-- 004: Assentos de cash game, escrow de fichas e ACL de histórico.
-- Valores monetários são sempre inteiros em centavos.

CREATE TABLE IF NOT EXISTS cash_game_seats (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id        UUID NOT NULL REFERENCES tables(id) ON DELETE RESTRICT,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    seat            SMALLINT NOT NULL CHECK (seat >= 0),
    chips           BIGINT NOT NULL CHECK (chips >= 0),
    buy_in          BIGINT NOT NULL CHECK (buy_in > 0),
    status          VARCHAR(20) NOT NULL DEFAULT 'ACTIVE'
                    CHECK (status IN ('ACTIVE', 'CASHED_OUT')),
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cashed_out_at   TIMESTAMPTZ
);

-- A partial uniqueness constraint allows a player to reenter after cash-out,
-- while guaranteeing at most one active seat and one active occupant per seat.
CREATE UNIQUE INDEX IF NOT EXISTS uq_cash_game_active_user
    ON cash_game_seats(table_id, user_id) WHERE status = 'ACTIVE';
CREATE UNIQUE INDEX IF NOT EXISTS uq_cash_game_active_seat
    ON cash_game_seats(table_id, seat) WHERE status = 'ACTIVE';
CREATE INDEX IF NOT EXISTS idx_cash_game_seats_active_table
    ON cash_game_seats(table_id, status);

-- Immutable audit trail for transfers between the wallet and table escrow.
CREATE TABLE IF NOT EXISTS cash_game_ledger (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    table_id        UUID NOT NULL REFERENCES tables(id) ON DELETE RESTRICT,
    seat_id         UUID NOT NULL REFERENCES cash_game_seats(id) ON DELETE RESTRICT,
    entry_type      VARCHAR(20) NOT NULL
                    CHECK (entry_type IN ('BUY_IN', 'CASH_OUT')),
    amount          BIGINT NOT NULL CHECK (amount > 0),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cash_game_ledger_user_created
    ON cash_game_ledger(user_id, created_at);

-- Hand visibility is opt-in by participation, never by knowledge of an ID.
CREATE TABLE IF NOT EXISTS hand_participants (
    hand_id         UUID NOT NULL REFERENCES hand_history(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    PRIMARY KEY (hand_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_hand_participants_user
    ON hand_participants(user_id, hand_id);
