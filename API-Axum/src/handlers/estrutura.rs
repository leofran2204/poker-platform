use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct EstruturaMember {
    pub username: String,
    pub level: i16,
    pub sponsor_username: Option<String>,
    pub rake_generated_week: i64,
    pub commission_paid_week: i64,
}

#[derive(Debug, Serialize)]
pub struct EstruturaResponse {
    pub referral_code: Option<String>,
    pub eligible: bool,
    pub hands_this_week: i64,
    pub personal_rake_cents_week: i64,
    pub vp_hands_needed: i64,
    pub vp_rake_cents_needed: i64,
    pub estrutura_points: i64,
    pub points_week_l1: i64,
    pub points_week_l2: i64,
    pub withheld_week: i64,
    pub level1: Vec<EstruturaMember>,
    pub level2: Vec<EstruturaMember>,
}

/// GET /api/estrutura — Minha Estrutura (só 2 níveis).
pub async fn get_estrutura(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
) -> Result<Json<EstruturaResponse>, ApiError> {
    let uid = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| ApiError::Internal("Authenticated user id is invalid".to_string()))?;

    let week_start: i64 = sqlx::query_scalar(
        "SELECT EXTRACT(EPOCH FROM date_trunc('week', timezone('America/Sao_Paulo', now())))::BIGINT",
    )
    .fetch_one(&state.db)
    .await?;

    let (referral_code, estrutura_points): (Option<String>, i64) = sqlx::query_as(
        "SELECT referral_code, estrutura_points FROM users WHERE id = $1",
    )
    .bind(uid)
    .fetch_one(&state.db)
    .await?;

    let hands_this_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hand_participants hp \
         JOIN hand_history hh ON hh.id = hp.hand_id \
         WHERE hp.user_id = $1 AND hh.created_at >= $2",
    )
    .bind(uid)
    .bind(week_start)
    .fetch_one(&state.db)
    .await?;

    let personal_rake_cents_week: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(source_rake_cents)::BIGINT, 0) FROM ( \
            SELECT DISTINCT hand_id, source_rake_cents \
            FROM estrutura_ledger \
            WHERE source_user_id = $1 AND created_at >= to_timestamp($2) \
         ) q",
    )
    .bind(uid)
    .bind(week_start)
    .fetch_one(&state.db)
    .await?;

    let eligible = hands_this_week >= crate::estrutura::VP_HANDS_WEEK
        || personal_rake_cents_week >= crate::estrutura::VP_RAKE_CENTS_WEEK;

    let (points_week_l1, points_week_l2, withheld_week): (i64, i64, i64) = sqlx::query_as(
        "SELECT \
            COALESCE(SUM(commission_cents) FILTER (WHERE eligible AND level = 1), 0)::BIGINT, \
            COALESCE(SUM(commission_cents) FILTER (WHERE eligible AND level = 2), 0)::BIGINT, \
            COALESCE(SUM(commission_cents) FILTER (WHERE NOT eligible), 0)::BIGINT \
         FROM estrutura_ledger \
         WHERE beneficiary_user_id = $1 AND created_at >= to_timestamp($2)",
    )
    .bind(uid)
    .bind(week_start)
    .fetch_one(&state.db)
    .await?;

    let l1_rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT u.username, \
                COALESCE((SELECT SUM(x.source_rake_cents)::BIGINT FROM ( \
                    SELECT DISTINCT el.hand_id, el.source_rake_cents \
                    FROM estrutura_ledger el \
                    WHERE el.source_user_id = u.id AND el.created_at >= to_timestamp($2) \
                ) x), 0), \
                COALESCE((SELECT SUM(el.commission_cents)::BIGINT FROM estrutura_ledger el \
                    WHERE el.source_user_id = u.id AND el.beneficiary_user_id = $1 \
                      AND el.level = 1 AND el.eligible AND el.created_at >= to_timestamp($2)), 0) \
         FROM users u \
         WHERE u.sponsored_by = $1 \
         ORDER BY u.username",
    )
    .bind(uid)
    .bind(week_start)
    .fetch_all(&state.db)
    .await?;

    let level1 = l1_rows
        .into_iter()
        .map(|(username, rake_generated_week, commission_paid_week)| EstruturaMember {
            username,
            level: 1,
            sponsor_username: None,
            rake_generated_week,
            commission_paid_week,
        })
        .collect();

    let l2_rows: Vec<(String, String, i64, i64)> = sqlx::query_as(
        "SELECT u.username, p.username, \
                COALESCE((SELECT SUM(x.source_rake_cents)::BIGINT FROM ( \
                    SELECT DISTINCT el.hand_id, el.source_rake_cents \
                    FROM estrutura_ledger el \
                    WHERE el.source_user_id = u.id AND el.created_at >= to_timestamp($2) \
                ) x), 0), \
                COALESCE((SELECT SUM(el.commission_cents)::BIGINT FROM estrutura_ledger el \
                    WHERE el.source_user_id = u.id AND el.beneficiary_user_id = $1 \
                      AND el.level = 2 AND el.eligible AND el.created_at >= to_timestamp($2)), 0) \
         FROM users u \
         JOIN users p ON p.id = u.sponsored_by \
         WHERE p.sponsored_by = $1 \
         ORDER BY u.username",
    )
    .bind(uid)
    .bind(week_start)
    .fetch_all(&state.db)
    .await?;

    let level2 = l2_rows
        .into_iter()
        .map(
            |(username, sponsor_username, rake_generated_week, commission_paid_week)| {
                EstruturaMember {
                    username,
                    level: 2,
                    sponsor_username: Some(sponsor_username),
                    rake_generated_week,
                    commission_paid_week,
                }
            },
        )
        .collect();

    Ok(Json(EstruturaResponse {
        referral_code,
        eligible,
        hands_this_week,
        personal_rake_cents_week,
        vp_hands_needed: crate::estrutura::VP_HANDS_WEEK,
        vp_rake_cents_needed: crate::estrutura::VP_RAKE_CENTS_WEEK,
        estrutura_points,
        points_week_l1,
        points_week_l2,
        withheld_week,
        level1,
        level2,
    }))
}
