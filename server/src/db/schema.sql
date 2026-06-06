-- Poker Platform Database Schema

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(30) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    balance DECIMAL(15,2) NOT NULL DEFAULT 1000.00 CHECK (balance >= 0),
    avatar VARCHAR(50) DEFAULT 'default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ
);

-- Game tables
CREATE TABLE game_tables (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    game_type VARCHAR(20) NOT NULL DEFAULT 'cash' CHECK (game_type IN ('cash', 'tournament')),
    small_blind DECIMAL(10,2) NOT NULL,
    big_blind DECIMAL(10,2) NOT NULL,
    min_buy_in DECIMAL(15,2) NOT NULL,
    max_buy_in DECIMAL(15,2) NOT NULL,
    max_players INT NOT NULL DEFAULT 6 CHECK (max_players BETWEEN 2 AND 9),
    speed VARCHAR(20) NOT NULL DEFAULT 'normal' CHECK (speed IN ('normal', 'turbo', 'hyper')),
    ante DECIMAL(10,2) DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table sessions (active games)
CREATE TABLE table_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    table_id UUID NOT NULL REFERENCES game_tables(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    hand_count INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed'))
);

-- Hand history
CREATE TABLE hand_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES table_sessions(id),
    hand_number INT NOT NULL,
    community_cards JSONB,
    pot DECIMAL(15,2) NOT NULL,
    rake DECIMAL(15,2) NOT NULL DEFAULT 0,
    played_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Player hands in history
CREATE TABLE hand_players (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    hand_id UUID NOT NULL REFERENCES hand_history(id),
    user_id UUID NOT NULL REFERENCES users(id),
    seat_index INT NOT NULL,
    hole_cards JSONB,
    hand_rank VARCHAR(30),
    action_sequence JSONB,
    net_result DECIMAL(15,2) NOT NULL DEFAULT 0,
    is_winner BOOLEAN NOT NULL DEFAULT false
);

-- Transactions
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    type VARCHAR(30) NOT NULL CHECK (type IN ('deposit', 'withdraw', 'buy_in', 'cash_out', 'win', 'loss', 'rake')),
    amount DECIMAL(15,2) NOT NULL,
    table_id UUID REFERENCES game_tables(id),
    hand_id UUID REFERENCES hand_history(id),
    balance_after DECIMAL(15,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_game_tables_active ON game_tables(is_active) WHERE is_active = true;
CREATE INDEX idx_transactions_user ON transactions(user_id, created_at DESC);
CREATE INDEX idx_hand_history_session ON hand_history(session_id);
CREATE INDEX idx_hand_players_user ON hand_players(user_id);
CREATE INDEX idx_hand_players_hand ON hand_players(hand_id);

-- Updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();