-- 038: Torneios com horário agendado (auto-start com 5 players) + assentos separados
-- America/Sao_Paulo fixo para todos. Q2 B: tournament_seats separado de cash_game_seats.

ALTER TABLE tournaments
    ADD COLUMN IF NOT EXISTS scheduled_start_at BIGINT,
    ADD COLUMN IF NOT EXISTS auto_start_min_players INT DEFAULT 5;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_tournaments_auto_start_min') THEN
        ALTER TABLE tournaments
            ADD CONSTRAINT chk_tournaments_auto_start_min
            CHECK (auto_start_min_players BETWEEN 2 AND 100);
    END IF;
END $$;

-- Define horário padrão agendado para torneios existentes: próximo dia 20:00 America/Sao_Paulo
-- 20:00 SP = 23:00 UTC. Usa timezone do servidor para converter.
UPDATE tournaments
SET scheduled_start_at = COALESCE(
        scheduled_start_at,
        EXTRACT(EPOCH FROM ( (CURRENT_DATE + INTERVAL '1 day' + TIME '20:00') AT TIME ZONE 'America/Sao_Paulo' ))::BIGINT
    ),
    auto_start_min_players = 5
WHERE scheduled_start_at IS NULL;

-- Assentos de torneio: 1 mesa por modalidade hoje, mas preparado para multi-mesa 9/5/6
CREATE TABLE IF NOT EXISTS tournament_seats (
    tournament_id UUID NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    table_id UUID NOT NULL REFERENCES tables(id) ON DELETE CASCADE,
    seat SMALLINT NOT NULL CHECK (seat >= 0),
    player_id TEXT NOT NULL,
    player_name VARCHAR(30) NOT NULL,
    stack BIGINT NOT NULL CHECK (stack >= 0),
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE','ELIMINATED')),
    PRIMARY KEY (tournament_id, table_id, seat)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_tournament_seats_player ON tournament_seats(tournament_id, player_id) WHERE status='ACTIVE';
CREATE UNIQUE INDEX IF NOT EXISTS uq_tournament_seats_table_seat ON tournament_seats(tournament_id, table_id, seat) WHERE status='ACTIVE';
CREATE INDEX IF NOT EXISTS idx_tournament_seats_tournament ON tournament_seats(tournament_id, status);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES ('system','TOURNAMENT_SCHEDULED_START_ENABLED', jsonb_build_object('migration',38,'auto_start_min',5,'timezone','America/Sao_Paulo','default_time','20:00'));
