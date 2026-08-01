-- Migration 014: B2B Organizations (Clubs) Schema
-- Transforma a plataforma monolítica em um SaaS Multi-Tenant White-Label.

-- ═══════════════════════════════════════════════════════════════
-- CLUBS (Organizações B2B)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS clubs (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name              VARCHAR(100) NOT NULL,
    subdomain         VARCHAR(100) NOT NULL UNIQUE,
    custom_theme_json JSONB        NOT NULL DEFAULT '{}'::JSONB,
    status            VARCHAR(20)  NOT NULL DEFAULT 'active',
    balance           BIGINT       NOT NULL DEFAULT 0,
    created_at        BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);

-- ═══════════════════════════════════════════════════════════════
-- CLUB MEMBERSHIPS (Liquidez Compartilhada)
-- ═══════════════════════════════════════════════════════════════
-- Permite que um usuário jogue em múltiplos clubes da rede (Liquidez global).
-- Cada membro pode ter um cargo específico (ex: player, admin, agent).
CREATE TABLE IF NOT EXISTS club_memberships (
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    club_id     UUID        NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    role        VARCHAR(20) NOT NULL DEFAULT 'player',
    status      VARCHAR(20) NOT NULL DEFAULT 'active',
    joined_at   BIGINT      NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    PRIMARY KEY (user_id, club_id)
);

-- ═══════════════════════════════════════════════════════════════
-- MULTI-TENANT ISOLATION EM MESAS E TORNEIOS
-- ═══════════════════════════════════════════════════════════════
-- Se club_id for NULL, a mesa é "Global". 
-- Se tiver club_id, a mesa/torneio pertence e gera rake exclusivo para o Clube.
ALTER TABLE tables 
    ADD COLUMN club_id UUID REFERENCES clubs(id) ON DELETE CASCADE;

CREATE INDEX idx_tables_club_id ON tables(club_id);

ALTER TABLE tournaments 
    ADD COLUMN club_id UUID REFERENCES clubs(id) ON DELETE CASCADE;

CREATE INDEX idx_tournaments_club_id ON tournaments(club_id);

-- ═══════════════════════════════════════════════════════════════
-- CLUB AGENTS (Afiliados / Rakeback sobre a fatia do clube)
-- ═══════════════════════════════════════════════════════════════
-- Comissões em centavos inteiros. rakeback_percentage: 0–50.
CREATE TABLE IF NOT EXISTS club_agents (
    id                       UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    club_id                  UUID         NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    name                     VARCHAR(100) NOT NULL,
    rakeback_percentage      SMALLINT     NOT NULL DEFAULT 0
        CHECK (rakeback_percentage >= 0 AND rakeback_percentage <= 50),
    total_players_referred   INT          NOT NULL DEFAULT 0
        CHECK (total_players_referred >= 0),
    total_commission_earned  BIGINT       NOT NULL DEFAULT 0
        CHECK (total_commission_earned >= 0),
    status                   VARCHAR(20)  NOT NULL DEFAULT 'active',
    created_at               BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);

CREATE INDEX IF NOT EXISTS idx_club_agents_club_id ON club_agents(club_id);
CREATE INDEX IF NOT EXISTS idx_club_agents_status ON club_agents(club_id, status);
