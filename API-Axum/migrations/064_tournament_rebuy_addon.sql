-- 064: rebuy e add-on nos torneios, configurados por evento no admin.
-- - Add-on: ligar ou não, preço (centavos), fichas e último nível. Uma compra por
--   jogador. Preço 0 = buy-in; fichas 0 = stack inicial; nível 0 = mesmo limite do rebuy.
-- - Totais em centavos de rebuys e add-ons por torneio e valores pagos por jogador,
--   para reidratar o motor depois de um reinício e calcular a perda do jogo responsável.
-- - `pending_chips`: fichas compradas por quem ainda está na mesa (add-on ou rebuy
--   com stack baixo). Entram no começo da mão seguinte, nunca no meio de uma mão.
-- - A chave primária (torneio, mesa, assento) incluía assentos eliminados e travava o
--   número da cadeira para sempre (rebalanceamento e mesa final podiam colidir). A
--   unicidade passa a valer só para assentos ativos, pelos índices parciais da 038.
-- Sem taxa em rebuy e add-on (BUSINESS_RULES). Idempotente.

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS allow_addon BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS addon_cost BIGINT NOT NULL DEFAULT 0 CHECK (addon_cost >= 0),
    ADD COLUMN IF NOT EXISTS addon_chips BIGINT NOT NULL DEFAULT 0 CHECK (addon_chips >= 0),
    ADD COLUMN IF NOT EXISTS addon_max_level INTEGER NOT NULL DEFAULT 0 CHECK (addon_max_level >= 0),
    ADD COLUMN IF NOT EXISTS total_rebuys BIGINT NOT NULL DEFAULT 0 CHECK (total_rebuys >= 0),
    ADD COLUMN IF NOT EXISTS total_addons BIGINT NOT NULL DEFAULT 0 CHECK (total_addons >= 0);

ALTER TABLE tournament_players
    ADD COLUMN IF NOT EXISTS rebuy_paid BIGINT NOT NULL DEFAULT 0 CHECK (rebuy_paid >= 0),
    ADD COLUMN IF NOT EXISTS addon_paid BIGINT NOT NULL DEFAULT 0 CHECK (addon_paid >= 0);

ALTER TABLE tournament_seats
    ADD COLUMN IF NOT EXISTS pending_chips BIGINT NOT NULL DEFAULT 0 CHECK (pending_chips >= 0);

ALTER TABLE tournament_seats DROP CONSTRAINT IF EXISTS tournament_seats_pkey;

CREATE UNIQUE INDEX IF NOT EXISTS uq_tournament_seats_player
    ON tournament_seats (tournament_id, player_id) WHERE status = 'ACTIVE';
CREATE UNIQUE INDEX IF NOT EXISTS uq_tournament_seats_table_seat
    ON tournament_seats (tournament_id, table_id, seat) WHERE status = 'ACTIVE';

INSERT INTO audit_logs (user_id, action, metadata)
SELECT 'system', 'TOURNAMENT_REBUY_ADDON_064',
       jsonb_build_object('migration', 64, 'addon_default', false, 'fee_on_rebuy_addon', 0)
WHERE NOT EXISTS (
    SELECT 1 FROM audit_logs WHERE action = 'TOURNAMENT_REBUY_ADDON_064'
);
