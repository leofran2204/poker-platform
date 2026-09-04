-- 040: Torneios agendados para hoje 21:30 America/Sao_Paulo (1788481800)
-- Reflete o estado atual: todos os registering passam a 21:30 SP, auto-start 5 fixo.
-- Idempotente. Não mexe em running/paused/finished/cancelled.

UPDATE tournaments
SET scheduled_start_at = 1788481800,
    auto_start_min_players = 5
WHERE status = 'registering'
  AND (scheduled_start_at IS DISTINCT FROM 1788481800 OR auto_start_min_players IS DISTINCT FROM 5);

INSERT INTO audit_logs (user_id, action, metadata)
VALUES ('system','TOURNAMENT_SCHEDULED_2130_SP', jsonb_build_object('migration',40,'scheduled_start_at',1788481800,'sp_time','2026-09-03 21:30:00 America/Sao_Paulo','auto_start_min',5));
