-- Migration 001: Initial schema for poker platform
-- Creates: users, tables, hand_history, tournaments, sessions

-- ═══════════════════════════════════════════════════════════════
-- USERS
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username        VARCHAR(30)  NOT NULL UNIQUE,
    email           VARCHAR(255) NOT NULL UNIQUE,
    password_hash   VARCHAR(255) NOT NULL,
    role            VARCHAR(20)  NOT NULL DEFAULT 'player',
    status          VARCHAR(30)  NOT NULL DEFAULT 'active',
    balance         BIGINT       NOT NULL DEFAULT 0,
    mfa_enabled     BOOLEAN      NOT NULL DEFAULT FALSE,
    mfa_secret      VARCHAR(255),
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until    BIGINT,
    created_at      BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    last_login      BIGINT
);

-- ═══════════════════════════════════════════════════════════════
-- SESSIONS
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    username    VARCHAR(30)  NOT NULL,
    ip_address  VARCHAR(45)  NOT NULL DEFAULT '',
    user_agent  TEXT         NOT NULL DEFAULT '',
    created_at  BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    expires_at  BIGINT       NOT NULL,
    is_active   BOOLEAN      NOT NULL DEFAULT TRUE
);

-- ═══════════════════════════════════════════════════════════════
-- TABLES (lobby)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS tables (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(100) NOT NULL,
    game_type       VARCHAR(20)  NOT NULL DEFAULT 'cash',
    small_blind     BIGINT       NOT NULL,
    big_blind       BIGINT       NOT NULL,
    min_buy_in      BIGINT       NOT NULL,
    max_buy_in      BIGINT       NOT NULL,
    max_players     SMALLINT     NOT NULL DEFAULT 9,
    current_players SMALLINT     NOT NULL DEFAULT 0,
    visibility      VARCHAR(10)  NOT NULL DEFAULT 'public',
    password_hash   VARCHAR(255),
    created_at      BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);

-- ═══════════════════════════════════════════════════════════════
-- HAND HISTORY
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS hand_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id        UUID REFERENCES tables(id) ON DELETE SET NULL,
    hand_number     INTEGER      NOT NULL,
    game_type       VARCHAR(20)  NOT NULL,
    small_blind     BIGINT       NOT NULL,
    big_blind       BIGINT       NOT NULL,
    dealer_player_id VARCHAR(255),
    actions_json    JSONB        NOT NULL DEFAULT '[]'::JSONB,
    community_cards_json JSONB   NOT NULL DEFAULT '[]'::JSONB,
    pot_total       BIGINT       NOT NULL DEFAULT 0,
    rake_collected  BIGINT       NOT NULL DEFAULT 0,
    end_reason      VARCHAR(30),
    winner_player_id VARCHAR(255),
    created_at      BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);

-- ═══════════════════════════════════════════════════════════════
-- TOURNAMENTS
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS tournaments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                VARCHAR(100) NOT NULL,
    buy_in              BIGINT       NOT NULL,
    starting_stack     BIGINT       NOT NULL,
    max_players         INTEGER      NOT NULL,
    late_registration  BOOLEAN      NOT NULL DEFAULT TRUE,
    late_reg_max_level INTEGER      NOT NULL DEFAULT 3,
    speed               VARCHAR(20)  NOT NULL DEFAULT 'normal',
    status              VARCHAR(20)  NOT NULL DEFAULT 'registering',
    prize_pool          BIGINT       NOT NULL DEFAULT 0,
    current_level       INTEGER      NOT NULL DEFAULT 0,
    players_remaining   INTEGER      NOT NULL DEFAULT 0,
    total_buyins        INTEGER      NOT NULL DEFAULT 0,
    created_at          BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    started_at          BIGINT,
    finished_at         BIGINT
);

-- ═══════════════════════════════════════════════════════════════
-- TOURNAMENT PLAYERS (junction table)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS tournament_players (
    tournament_id  UUID   NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    player_id      VARCHAR(255) NOT NULL,
    player_name    VARCHAR(30)  NOT NULL,
    stack          BIGINT       NOT NULL,
    table_id       UUID,
    seat           SMALLINT,
    rebuys         INTEGER      NOT NULL DEFAULT 0,
    addon_done     BOOLEAN      NOT NULL DEFAULT FALSE,
    final_position INTEGER,
    prize          BIGINT,
    registered_at  BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    eliminated_at  BIGINT,
    PRIMARY KEY (tournament_id, player_id)
);

-- ═══════════════════════════════════════════════════════════════
-- INDEXES
-- ═══════════════════════════════════════════════════════════════
CREATE INDEX IF NOT EXISTS idx_sessions_user_id     ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_active      ON sessions(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_hand_history_table   ON hand_history(table_id);
CREATE INDEX IF NOT EXISTS idx_tournament_players_t  ON tournament_players(tournament_id);
