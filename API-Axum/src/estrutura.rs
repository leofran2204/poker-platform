//! Agente ZT Poker: um nível direto. O ledger registra receita bruta atribuível
//! durante o mês; a comissão de 30% ou 35% do NGR é creditada no fechamento.

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub const BASE_PERCENT: i64 = 30;
pub const BONUS_PERCENT: i64 = 5;

/// Centavos inteiros; intermediário largo evita overflow antes da divisão.
pub fn commission(ngr: i64, percent: i64) -> i64 {
    ((i128::from(ngr.max(0)) * i128::from(percent)) / 100) as i64
}

#[derive(Debug, PartialEq, Eq)]
pub struct CycleCalculation {
    pub ngr: i64,
    pub percent: i64,
    pub commission: i64,
    pub carry_forward: i64,
}

pub fn calculate_cycle(gross: i64, deductions: i64, target: i64) -> CycleCalculation {
    let ngr = gross.saturating_sub(deductions).max(0);
    let percent = BASE_PERCENT
        + if target > 0 && ngr >= target {
            BONUS_PERCENT
        } else {
            0
        };
    CycleCalculation {
        ngr,
        percent,
        commission: commission(ngr, percent),
        carry_forward: deductions.saturating_sub(gross).max(0),
    }
}

/// Todas as mutações do ciclo usam a mesma trava, inclusive o fechamento.
pub async fn lock_cycle(
    tx: &mut Transaction<'_, Postgres>,
    agent: Uuid,
    cycle: chrono::NaiveDate,
    mode: &str,
) -> Result<String, sqlx::Error> {
    // Serializa todos os ciclos do mesmo agente. Isto impede que um ciclo antigo
    // seja aberto ao mesmo tempo em que um ciclo posterior está sendo fechado.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 43))")
        .bind(agent.to_string())
        .execute(&mut **tx)
        .await?;
    sqlx::query("INSERT INTO agent_monthly_cycles (agent_user_id, cycle_start, money_mode) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(agent).bind(cycle).bind(mode).execute(&mut **tx).await?;
    sqlx::query_scalar("SELECT status FROM agent_monthly_cycles WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3 FOR UPDATE")
        .bind(agent).bind(cycle).bind(mode).fetch_one(&mut **tx).await
}

pub async fn distribute_hand_rake(
    tx: &mut Transaction<'_, Postgres>,
    hand_id: Uuid,
    participants: &[Uuid],
    rake: i64,
) -> Result<(), sqlx::Error> {
    if rake <= 0 || participants.is_empty() {
        return Ok(());
    }
    let n = participants.len() as i64;
    let (money_mode, private_table): (String, bool) = sqlx::query_as(
        "SELECT t.money_mode, t.club_id IS NOT NULL FROM hand_history hh \
         JOIN tables t ON t.id = hh.table_id WHERE hh.id = $1",
    )
    .bind(hand_id)
    .fetch_one(&mut **tx)
    .await?;
    // Mesas B2B já repassam 85% ao clube: apenas a receita retida é atribuível.
    let attributable_rake = if private_table {
        commission(rake, 15)
    } else {
        rake
    };
    let share = attributable_rake / n;

    let rows: Vec<(Uuid, Option<Uuid>)> = sqlx::query_as(
        "SELECT source.id, source.sponsored_by \
         FROM users source \
         JOIN users agent ON agent.id = source.sponsored_by \
         WHERE source.id = ANY($1) AND agent.agent_status = 'active' \
           AND NOT source.is_bot AND NOT agent.is_bot ORDER BY agent.id, source.id",
    )
    .bind(participants)
    .fetch_all(&mut **tx)
    .await?;

    for (source, sponsor) in rows {
        let Some(agent) = sponsor else {
            continue;
        };
        if agent == source {
            continue;
        }
        insert_line(
            tx,
            LedgerLine {
                hand_id: Some(hand_id),
                source,
                beneficiary: agent,
                source_rake: share,
                source_type: "hand",
                source_reference_id: Some(hand_id),
                money_mode: money_mode.clone(),
            },
        )
        .await?;
    }
    Ok(())
}

/// Registra o fee de torneio como receita bruta do agente direto ativo.
/// O retorno informa a projeção base de 30%; nenhum ponto é creditado aqui.
pub async fn distribute_fee(
    tx: &mut Transaction<'_, Postgres>,
    payer: Uuid,
    fee_cents: i64,
    source_reference_id: Uuid,
    money_mode: &str,
) -> Result<(i64, i64), sqlx::Error> {
    if fee_cents <= 0 {
        return Ok((0, 0));
    }
    let sponsor: Option<Uuid> = sqlx::query_scalar(
        "SELECT source.sponsored_by \
         FROM users source \
         JOIN users agent ON agent.id = source.sponsored_by \
         WHERE source.id = $1 AND agent.agent_status = 'active' \
           AND NOT source.is_bot AND NOT agent.is_bot",
    )
    .bind(payer)
    .fetch_optional(&mut **tx)
    .await?
    .flatten();
    let Some(agent) = sponsor else {
        return Ok((0, 0));
    };
    if agent == payer {
        return Ok((0, 0));
    }
    insert_line(
        tx,
        LedgerLine {
            hand_id: None,
            source: payer,
            beneficiary: agent,
            source_rake: fee_cents,
            source_type: "fee",
            source_reference_id: Some(source_reference_id),
            money_mode: money_mode.to_string(),
        },
    )
    .await?;
    Ok((commission(fee_cents, BASE_PERCENT), 0))
}

/// Uma linha do ledger Minha Estrutura (mão de cash ou fee de MTT).
struct LedgerLine {
    hand_id: Option<Uuid>,
    source: Uuid,
    beneficiary: Uuid,
    source_rake: i64,
    source_type: &'static str,
    source_reference_id: Option<Uuid>,
    money_mode: String,
}

async fn insert_line(
    tx: &mut Transaction<'_, Postgres>,
    line: LedgerLine,
) -> Result<(), sqlx::Error> {
    if line.source_rake <= 0 {
        return Ok(());
    }
    let cycle: chrono::NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month', timezone('America/Sao_Paulo', clock_timestamp()))::date",
    )
    .fetch_one(&mut **tx)
    .await?;
    if lock_cycle(tx, line.beneficiary, cycle, &line.money_mode).await? != "open" {
        return Err(sqlx::Error::Protocol("Agent cycle already closed".into()));
    }
    sqlx::query(
        "INSERT INTO estrutura_ledger \
            (hand_id, source_user_id, beneficiary_user_id, level, \
              source_rake_cents, commission_cents, eligible, source_type, \
              program_version, cycle_start, source_reference_id, money_mode) \
          SELECT $1, $2, $3, 1, $4, 0, TRUE, $5, 2, $8, $6, $7 \
          WHERE $1::uuid IS NULL OR NOT EXISTS ( \
              SELECT 1 FROM estrutura_ledger WHERE hand_id = $1 AND source_user_id = $2 \
                AND program_version = 2)",
    )
    .bind(line.hand_id)
    .bind(line.source)
    .bind(line.beneficiary)
    .bind(line.source_rake)
    .bind(line.source_type)
    .bind(line.source_reference_id)
    .bind(line.money_mode)
    .bind(cycle)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Reverte somente fees desta inscrição, preservando receitas e fechamentos.
/// A chave do ajuste é a própria linha de receita: repetição não duplica estorno.
pub async fn refund_fee(
    tx: &mut Transaction<'_, Postgres>,
    payer: Uuid,
    tournament: Uuid,
) -> Result<(), sqlx::Error> {
    let lines: Vec<(Uuid, Uuid, chrono::NaiveDate, String, i64)> = sqlx::query_as(
        "SELECT el.id, el.beneficiary_user_id, el.cycle_start, el.money_mode, el.source_rake_cents \
         FROM estrutura_ledger el WHERE el.source_type = 'fee' AND el.source_user_id = $1 \
           AND el.source_reference_id = $2 AND el.program_version = 2 \
           AND NOT EXISTS (SELECT 1 FROM agent_ngr_adjustments a WHERE a.id = el.id) \
         ORDER BY el.beneficiary_user_id, el.cycle_start, el.money_mode, el.id",
    ).bind(payer).bind(tournament).fetch_all(&mut **tx).await?;
    for (id, agent, mut cycle, mode, amount) in lines {
        if lock_cycle(tx, agent, cycle, &mode).await? == "closed" {
            cycle = sqlx::query_scalar("SELECT date_trunc('month', timezone('America/Sao_Paulo', clock_timestamp()))::date")
                .fetch_one(&mut **tx).await?;
            if lock_cycle(tx, agent, cycle, &mode).await? != "open" {
                return Err(sqlx::Error::Protocol("Refund cycle already closed".into()));
            }
        }
        sqlx::query("INSERT INTO agent_ngr_adjustments (id, agent_user_id, cycle_start, money_mode, category, amount_cents, note) VALUES ($1, $2, $3, $4, 'refund', $5, $6) ON CONFLICT (id) DO NOTHING")
            .bind(id).bind(agent).bind(cycle).bind(mode).bind(amount)
            .bind("Estorno de taxa por cancelamento de inscrição em torneio")
            .execute(&mut **tx).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monthly_calculation_respects_target_and_deficit() {
        assert_eq!(calculate_cycle(100_000, 0, 100_001).commission, 30_000);
        let reached = calculate_cycle(100_000, 0, 100_000);
        assert_eq!(
            (reached.ngr, reached.percent, reached.commission),
            (100_000, 35, 35_000)
        );
        assert_eq!(calculate_cycle(100_000, 0, 0).percent, 30);
        assert_eq!(calculate_cycle(100_000, 25_000, 75_000).commission, 26_250);
        assert_eq!(calculate_cycle(100_000, 25_001, 75_000).commission, 22_499);
        let negative = calculate_cycle(100, 150, 1);
        assert_eq!(
            (negative.ngr, negative.commission, negative.carry_forward),
            (0, 0, 50)
        );
        assert_eq!(
            commission(i64::MAX, 35),
            (i128::from(i64::MAX) * 35 / 100) as i64
        );
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL with migration 060 — set DATABASE_URL to run"]
    async fn refund_is_scoped_to_one_tournament_and_idempotent() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for refund contract");
        let pool = sqlx::PgPool::connect(&url)
            .await
            .expect("local test database");
        let mut tx = pool.begin().await.expect("transaction");
        let agent = Uuid::new_v4();
        let player = Uuid::new_v4();
        let tournament_one = Uuid::new_v4();
        let tournament_two = Uuid::new_v4();
        let tournament_real = Uuid::new_v4();
        let agent_name = format!("ag{}", &agent.simple().to_string()[..14]);
        let player_name = format!("pl{}", &player.simple().to_string()[..14]);
        sqlx::query("INSERT INTO users (id, username, email, password_hash, agent_status) VALUES ($1, $2, $3, 'test', 'active')")
            .bind(agent).bind(&agent_name).bind(format!("{agent_name}@test.invalid"))
            .execute(&mut *tx).await.expect("agent fixture");
        sqlx::query("INSERT INTO users (id, username, email, password_hash, sponsored_by) VALUES ($1, $2, $3, 'test', $4)")
            .bind(player).bind(&player_name).bind(format!("{player_name}@test.invalid")).bind(agent)
            .execute(&mut *tx).await.expect("player fixture");

        distribute_fee(&mut tx, player, 1_500, tournament_one, "play")
            .await
            .expect("first fee");
        distribute_fee(&mut tx, player, 2_000, tournament_two, "play")
            .await
            .expect("second fee");
        distribute_fee(&mut tx, player, 700, tournament_real, "real")
            .await
            .expect("real fee");
        refund_fee(&mut tx, player, tournament_one)
            .await
            .expect("first refund");
        refund_fee(&mut tx, player, tournament_one)
            .await
            .expect("retry refund");

        let previous_cycle: chrono::NaiveDate = sqlx::query_scalar(
            "SELECT (date_trunc('month', timezone('America/Sao_Paulo', now())) - INTERVAL '1 month')::date",
        ).fetch_one(&mut *tx).await.expect("previous month");
        sqlx::query("INSERT INTO agent_monthly_cycles (agent_user_id, cycle_start, money_mode, status) VALUES ($1, $2, 'play', 'closed')")
            .bind(agent).bind(previous_cycle).execute(&mut *tx).await.expect("closed fixture");
        sqlx::query("UPDATE estrutura_ledger SET cycle_start = $3 WHERE source_user_id = $1 AND source_reference_id = $2")
            .bind(player).bind(tournament_two).bind(previous_cycle)
            .execute(&mut *tx).await.expect("closed revenue fixture");
        refund_fee(&mut tx, player, tournament_two)
            .await
            .expect("refund after close");

        let (revenue_count, revenue_total): (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT, SUM(source_rake_cents)::BIGINT FROM estrutura_ledger WHERE source_user_id = $1 AND program_version = 2 AND money_mode = 'play'",
        ).bind(player).fetch_one(&mut *tx).await.expect("revenues");
        let (refund_count, refund_total): (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT, SUM(amount_cents)::BIGINT FROM agent_ngr_adjustments WHERE agent_user_id = $1 AND category = 'refund' AND money_mode = 'play' AND cycle_start <> $2",
        ).bind(agent).bind(previous_cycle).fetch_one(&mut *tx).await.expect("refunds");
        let real_revenue: i64 = sqlx::query_scalar(
            "SELECT SUM(source_rake_cents)::BIGINT FROM estrutura_ledger WHERE source_user_id = $1 AND program_version = 2 AND money_mode = 'real'",
        ).bind(player).fetch_one(&mut *tx).await.expect("real revenue");
        let old_revenue: i64 = sqlx::query_scalar(
            "SELECT SUM(source_rake_cents)::BIGINT FROM estrutura_ledger WHERE source_user_id = $1 AND cycle_start = $2 AND program_version = 2",
        ).bind(player).bind(previous_cycle).fetch_one(&mut *tx).await.expect("closed revenue");
        assert_eq!((revenue_count, revenue_total), (2, 3_500));
        assert_eq!((refund_count, refund_total), (2, 3_500));
        assert_eq!(real_revenue, 700);
        assert_eq!(old_revenue, 2_000);
        tx.rollback().await.expect("fixtures rolled back");
    }
}
