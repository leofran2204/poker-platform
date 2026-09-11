//! Course progress — aulas concluídas e melhor nota de quiz por usuário.

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CourseProgressItem {
    pub lesson_id: String,
    pub status: String,
    pub best_score: i16,
    pub attempts: i32,
    pub updated_at: String,
}

type ProgressRow = (String, String, i16, i32, String);

/// GET /api/course/progress — progresso do usuário autenticado.
pub async fn get_progress(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
) -> Result<Json<Vec<CourseProgressItem>>, ApiError> {
    let rows: Vec<ProgressRow> = sqlx::query_as(
        "SELECT lesson_id, status, best_score, attempts, updated_at::text \
         FROM course_progress WHERE user_id = $1::uuid ORDER BY lesson_id",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(
                |(lesson_id, status, best_score, attempts, updated_at)| CourseProgressItem {
                    lesson_id,
                    status,
                    best_score,
                    attempts,
                    updated_at,
                },
            )
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct SaveProgressBody {
    pub lesson_id: String,
    pub completed: bool,
    pub score: i16,
}

/// POST /api/course/progress — registra conclusão e melhor nota da aula.
pub async fn save_progress(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<SaveProgressBody>,
) -> Result<Json<CourseProgressItem>, ApiError> {
    let lesson_id = body.lesson_id.trim();
    if lesson_id.is_empty() || lesson_id.len() > 32 {
        return Err(ApiError::BadRequest(
            "lesson_id deve ter entre 1 e 32 caracteres".to_string(),
        ));
    }
    if !(0..=100).contains(&body.score) {
        return Err(ApiError::BadRequest(
            "score deve estar entre 0 e 100".to_string(),
        ));
    }
    let status = if body.completed { "completed" } else { "started" };
    let row: ProgressRow = sqlx::query_as(
        "INSERT INTO course_progress (user_id, lesson_id, status, best_score, attempts, updated_at) \
         VALUES ($1::uuid, $2, $3, $4, 1, NOW()) \
         ON CONFLICT (user_id, lesson_id) DO UPDATE SET \
           status = CASE WHEN EXCLUDED.status = 'completed' THEN 'completed' ELSE course_progress.status END, \
           best_score = GREATEST(course_progress.best_score, EXCLUDED.best_score), \
           attempts = course_progress.attempts + 1, \
           updated_at = NOW() \
         RETURNING lesson_id, status, best_score, attempts, updated_at::text",
    )
    .bind(&auth_user.user_id)
    .bind(lesson_id)
    .bind(status)
    .bind(body.score)
    .fetch_one(&state.db)
    .await?;
    let (lesson_id, status, best_score, attempts, updated_at) = row;
    Ok(Json(CourseProgressItem {
        lesson_id,
        status,
        best_score,
        attempts,
        updated_at,
    }))
}
