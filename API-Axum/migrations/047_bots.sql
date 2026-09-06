-- 047: marca contas da frota de bots (coach/testes).
-- Bots nunca fazem login nem recebem convite; so jogam via manager interno.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS is_bot BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_users_is_bot
    ON users(is_bot) WHERE is_bot;
