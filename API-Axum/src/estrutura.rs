//! Bonificação Minha Estrutura: 18% (1º nível) + 12% (2º nível) do rake
//! individual de quem sentou. Resto à casa. Clube não recebe.

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub const L1_PERCENT: i64 = 18;
pub const L2_PERCENT: i64 = 12;
pub const VP_HANDS_WEEK: i64 = 50;
pub const VP_RAKE_CENTS_WEEK: i64 = 2000;

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
    let share = rake / n;

    let rows: Vec<(Uuid, Option<Uuid>)> = sqlx::query_as(
        "SELECT id, sponsored_by FROM users WHERE id = ANY($1)",
    )
    .bind(participants)
    .fetch_all(&mut **tx)
    .await?;

    let mut sponsor_of = std::collections::HashMap::new();
    for (id, sp) in rows {
        sponsor_of.insert(id, sp);
    }

    let week_start: i64 = sqlx::query_scalar(
        "SELECT EXTRACT(EPOCH FROM date_trunc('week', timezone('America/Sao_Paulo', now())))::BIGINT",
    )
    .fetch_one(&mut **tx)
    .await?;

    for source in participants {
        let Some(l1) = sponsor_of.get(source).copied().flatten() else {
            continue;
        };
        let l1_cut = (share * L1_PERCENT) / 100;
        let l1_ok = beneficiary_eligible(tx, l1, week_start).await?;
        insert_line(tx, hand_id, *source, l1, 1, share, l1_cut, l1_ok).await?;

        let l2: Option<Uuid> =
            sqlx::query_scalar("SELECT sponsored_by FROM users WHERE id = $1")
                .bind(l1)
                .fetch_one(&mut **tx)
                .await?;
        let Some(l2) = l2 else {
            continue;
        };
        if l2 == *source || l2 == l1 {
            continue;
        }
        let l2_cut = (share * L2_PERCENT) / 100;
        let l2_ok = beneficiary_eligible(tx, l2, week_start).await?;
        insert_line(tx, hand_id, *source, l2, 2, share, l2_cut, l2_ok).await?;
    }
    Ok(())
}

async fn beneficiary_eligible(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    week_start_epoch: i64,
) -> Result<bool, sqlx::Error> {
    let hands: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hand_participants hp \
         JOIN hand_history hh ON hh.id = hp.hand_id \
         WHERE hp.user_id = $1 AND hh.created_at >= $2",
    )
    .bind(user_id)
    .bind(week_start_epoch)
    .fetch_one(&mut **tx)
    .await?;
    if hands >= VP_HANDS_WEEK {
        return Ok(true);
    }
    let personal_rake: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(source_rake_cents)::BIGINT, 0) FROM ( \
            SELECT DISTINCT hand_id, source_rake_cents \
            FROM estrutura_ledger \
            WHERE source_user_id = $1 AND created_at >= to_timestamp($2) \
         ) q",
    )
    .bind(user_id)
    .bind(week_start_epoch)
    .fetch_one(&mut **tx)
    .await?;
    Ok(personal_rake >= VP_RAKE_CENTS_WEEK)
}

async fn insert_line(
    tx: &mut Transaction<'_, Postgres>,
    hand_id: Uuid,
    source: Uuid,
    beneficiary: Uuid,
    level: i16,
    source_rake: i64,
    commission: i64,
    eligible: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO estrutura_ledger \
            (hand_id, source_user_id, beneficiary_user_id, level, \
             source_rake_cents, commission_cents, eligible) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(hand_id)
    .bind(source)
    .bind(beneficiary)
    .bind(level)
    .bind(source_rake)
    .bind(commission)
    .bind(eligible)
    .execute(&mut **tx)
    .await?;
    if eligible && commission > 0 {
        sqlx::query("UPDATE users SET estrutura_points = estrutura_points + $2 WHERE id = $1")
            .bind(beneficiary)
            .bind(commission)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
