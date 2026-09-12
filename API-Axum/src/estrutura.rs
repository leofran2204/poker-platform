//! Bonificação Minha Estrutura: 18% (1º nível) + 12% (2º nível) do rake
//! individual de quem sentou. Resto à casa. Clube não recebe.

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub const L1_PERCENT: i64 = 18;
pub const L2_PERCENT: i64 = 12;
pub const VP_HANDS_WEEK: i64 = 100;
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
        insert_line(
            tx,
            LedgerLine {
                hand_id: Some(hand_id),
                source: *source,
                beneficiary: l1,
                level: 1,
                source_rake: share,
                commission: l1_cut,
                eligible: l1_ok,
                source_type: "hand",
            },
        )
        .await?;

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
        insert_line(
            tx,
            LedgerLine {
                hand_id: Some(hand_id),
                source: *source,
                beneficiary: l2,
                level: 2,
                source_rake: share,
                commission: l2_cut,
                eligible: l2_ok,
                source_type: "hand",
            },
        )
        .await?;
    }
    Ok(())
}

/// Divide o fee de inscrição do torneio na rede: 18% ao patrocinador direto,
/// 12% ao avô, resto à casa. Sem patrocinador: 100% casa (sem linhas).
/// Linhas `source_type='fee'` com `hand_id` NULL (migration 048).
/// Retorna (corte_l1, corte_l2) para auditoria.
pub async fn distribute_fee(
    tx: &mut Transaction<'_, Postgres>,
    payer: Uuid,
    fee_cents: i64,
    week_start_epoch: i64,
) -> Result<(i64, i64), sqlx::Error> {
    if fee_cents <= 0 {
        return Ok((0, 0));
    }
    let sponsor: Option<Uuid> = sqlx::query_scalar("SELECT sponsored_by FROM users WHERE id = $1")
        .bind(payer)
        .fetch_one(&mut **tx)
        .await?;
    let Some(l1) = sponsor else {
        return Ok((0, 0));
    };
    let l1_cut = (fee_cents * L1_PERCENT) / 100;
    let l1_ok = beneficiary_eligible(tx, l1, week_start_epoch).await?;
    insert_line(
        tx,
        LedgerLine {
            hand_id: None,
            source: payer,
            beneficiary: l1,
            level: 1,
            source_rake: fee_cents,
            commission: l1_cut,
            eligible: l1_ok,
            source_type: "fee",
        },
    )
    .await?;

    let l2: Option<Uuid> = sqlx::query_scalar("SELECT sponsored_by FROM users WHERE id = $1")
        .bind(l1)
        .fetch_one(&mut **tx)
        .await?;
    let Some(l2) = l2 else {
        return Ok((l1_cut, 0));
    };
    if l2 == payer || l2 == l1 {
        return Ok((l1_cut, 0));
    }
    let l2_cut = (fee_cents * L2_PERCENT) / 100;
    let l2_ok = beneficiary_eligible(tx, l2, week_start_epoch).await?;
    insert_line(
        tx,
        LedgerLine {
            hand_id: None,
            source: payer,
            beneficiary: l2,
            level: 2,
            source_rake: fee_cents,
            commission: l2_cut,
            eligible: l2_ok,
            source_type: "fee",
        },
    )
    .await?;
    Ok((l1_cut, l2_cut))
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
            SELECT DISTINCT COALESCE(hand_id::text, id::text), source_rake_cents \
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

/// Uma linha do ledger Minha Estrutura (mão de cash ou fee de MTT).
struct LedgerLine {
    hand_id: Option<Uuid>,
    source: Uuid,
    beneficiary: Uuid,
    level: i16,
    source_rake: i64,
    commission: i64,
    eligible: bool,
    source_type: &'static str,
}

async fn insert_line(
    tx: &mut Transaction<'_, Postgres>,
    line: LedgerLine,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO estrutura_ledger \
            (hand_id, source_user_id, beneficiary_user_id, level, \
              source_rake_cents, commission_cents, eligible, source_type) \
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(line.hand_id)
    .bind(line.source)
    .bind(line.beneficiary)
    .bind(line.level)
    .bind(line.source_rake)
    .bind(line.commission)
    .bind(line.eligible)
    .bind(line.source_type)
    .execute(&mut **tx)
    .await?;
    if line.eligible && line.commission > 0 {
        sqlx::query("UPDATE users SET estrutura_points = estrutura_points + $2 WHERE id = $1")
            .bind(line.beneficiary)
            .bind(line.commission)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
