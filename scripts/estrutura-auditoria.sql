-- estrutura-auditoria.sql — Gates da Fase D (rodar no banco da demo/VPS).
-- Prefixo das contas de teste: t18_ / smoke_mtt_. Bots: is_bot.

-- G1: matemática 18/12 exata em 100% das linhas (divisão inteira).
SELECT COUNT(*) AS linhas_erradas
FROM estrutura_ledger
WHERE (level = 1 AND commission_cents <> (source_rake_cents * 18) / 100)
   OR (level = 2 AND commission_cents <> (source_rake_cents * 12) / 100);

-- G1b: níveis válidos e tipos válidos.
SELECT COUNT(*) AS linhas_invalidas FROM estrutura_ledger
WHERE level NOT IN (1, 2) OR source_type NOT IN ('hand', 'fee');

-- G1c: fee sem mão e mão sem fee nulo.
SELECT COUNT(*) AS fee_sem_regra FROM estrutura_ledger
WHERE (source_type = 'fee' AND hand_id IS NOT NULL)
   OR (source_type = 'hand' AND hand_id IS NULL);

-- G2: casa >= 70% do rake+fee gerado pela rede de teste.
WITH base AS (
    SELECT COALESCE(SUM(source_rake_cents), 0) AS total,
           COALESCE(SUM(commission_cents), 0) AS rede
    FROM estrutura_ledger
)
SELECT total, rede, total - rede AS casa,
       CASE WHEN total > 0 THEN round(100.0 * (total - rede) / total, 2) END AS casa_pct
FROM base;

-- G2b: bots não geram nem recebem rede (sponsored_by NULL).
SELECT COUNT(*) AS linhas_com_bot FROM estrutura_ledger el
JOIN users s ON s.id = el.source_user_id
JOIN users b ON b.id = el.beneficiary_user_id
WHERE s.is_bot OR b.is_bot;

-- G3: fantasmas retêm sem creditar (withheld > 0 e pontos intactos por conta).
SELECT u.username,
       COALESCE(SUM(el.commission_cents) FILTER (WHERE NOT el.eligible), 0) AS retido,
       u.estrutura_points AS pontos
FROM users u
LEFT JOIN estrutura_ledger el ON el.beneficiary_user_id = u.id
WHERE u.username LIKE 't18_%' OR u.username LIKE 'smoke_mtt_%'
GROUP BY u.username, u.estrutura_points
ORDER BY u.username;

-- G3b: sem crédito retroativo (linhas antigas de quem virou elegível seguem inelegíveis).
SELECT beneficiary_user_id, COUNT(*) AS linhas_antigas_inelegiveis
FROM estrutura_ledger WHERE NOT eligible GROUP BY 1;

-- G4: erros/halts de mesa MTT no período.
SELECT action, COUNT(*) FROM audit_logs
WHERE action IN ('MTT_TABLE_HALTED', 'MTT_FINISHED', 'FT_CONSOLIDATED')
GROUP BY 1;

-- Panorama: pontos por beneficiário da rede de teste.
SELECT u.username,
       COALESCE(SUM(el.commission_cents) FILTER (WHERE el.eligible AND el.level = 1), 0) AS l1,
       COALESCE(SUM(el.commission_cents) FILTER (WHERE el.eligible AND el.level = 2), 0) AS l2,
       COALESCE(SUM(el.commission_cents) FILTER (WHERE NOT el.eligible), 0) AS retido
FROM users u
LEFT JOIN estrutura_ledger el ON el.beneficiary_user_id = u.id
WHERE u.username LIKE 't18_%'
GROUP BY u.username
ORDER BY u.username;
