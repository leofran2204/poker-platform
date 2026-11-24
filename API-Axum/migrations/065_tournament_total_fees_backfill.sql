-- 065: `tournaments.total_fees` = entradas × fee de inscrição (15% do buy-in).
-- A inscrição pelo site não gravava o fee (painel do admin com zero) e outros
-- caminhos podiam deixar a coluna defasada (achado na etapa 2 de 23/11/2026).
-- `total_buyins` é a contagem de entradas; o fee segue `entry_fee_cents` do motor
-- (buy_in * 1500 / 10000, em centavos inteiros). Torneio cancelado fica como está
-- (o reembolso devolve buy-in e fee). Idempotente: só muda onde está diferente.

WITH fixed AS (
    UPDATE tournaments
       SET total_fees = GREATEST(total_buyins, 0)::bigint * ((buy_in::bigint * 1500) / 10000)
     WHERE status <> 'cancelled'
       AND total_fees IS DISTINCT FROM GREATEST(total_buyins, 0)::bigint * ((buy_in::bigint * 1500) / 10000)
    RETURNING id
)
INSERT INTO audit_logs (user_id, action, metadata)
SELECT 'system', 'TOURNAMENT_TOTAL_FEES_065',
       jsonb_build_object('migration', 65, 'tournaments_fixed', (SELECT count(*) FROM fixed))
WHERE NOT EXISTS (
    SELECT 1 FROM audit_logs WHERE action = 'TOURNAMENT_TOTAL_FEES_065'
);
