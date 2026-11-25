//! Tempo de vídeo das aulas contado pelo servidor e perguntas surpresa.
//!
//! O navegador manda trechos contínuos (sem pulo, aba visível, até 2x). O
//! servidor só aceita o trecho que caiba no tempo de relógio desde o último
//! aviso (no máximo 2x) e marca um bit por segundo assistido. Em 1 a 3 momentos
//! sorteados por aula, abre uma pergunta sobre o capítulo que acabou de passar;
//! errar ou deixar 60 s sem resposta apaga o trecho desde a pergunta anterior.
//! O quiz abre com 90% do vídeo marcado e todas as perguntas acertadas.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

use super::course::{
    quiz_key, retry_at_sql, LessonKey, ReviewItem, VideoKey, PASS_SCORE, RETRY_WAIT_SECONDS,
};
use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

pub(crate) const REQUIRED_PERCENT: i16 = 90;
const MAX_RATE: f64 = 2.0;
/// O site manda um trecho a cada 10 s de vídeo; 30 s cobre atraso de rede.
const MAX_SEGMENT_SECONDS: f64 = 30.0;
const CLOCK_SLACK_SECONDS: f64 = 1.5;
const CHECK_ANSWER_SECONDS: f64 = 60.0;
/// Folga de rede para a resposta enviada no último segundo.
const CHECK_GRACE_SECONDS: f64 = 5.0;
const CHECK_MIN_GAP_SECONDS: f64 = 75.0;
/// Capítulos a até 90 s antes (ou 15 s depois) da pergunta não viram alternativa errada.
const CHECK_NEAR_BEFORE: f64 = 90.0;
const CHECK_NEAR_AFTER: f64 = 15.0;
const CHECK_OPTIONS: usize = 4;
const CHECK_PROMPT: &str = "Pergunta surpresa: qual destes temas acabou de passar no vídeo?";

// ─── Estado guardado por (aluno, aula) ───

struct OpenCheck {
    id: Uuid,
    at: f64,
    answer: String,
    prompt: String,
    options: Vec<String>,
    /// Segundos desde que a pergunta abriu.
    age: f64,
}

struct WatchRow {
    watched: Vec<u8>,
    last_position: f64,
    /// Segundos desde o último aviso do navegador (`None` = nenhum ainda).
    beat_age: Option<f64>,
    checkpoints: Vec<f64>,
    stretch_from: f64,
    check: Option<OpenCheck>,
}

type RawRow = (
    Vec<u8>,
    f64,
    Option<f64>,
    Vec<f64>,
    f64,
    Option<Uuid>,
    Option<f64>,
    Option<String>,
    Option<String>,
    Option<Vec<String>>,
    Option<f64>,
);

macro_rules! select_row {
    () => {
        "SELECT watched, last_position, \
                EXTRACT(EPOCH FROM (NOW() - last_beat_at))::float8, \
                checkpoints, stretch_from, check_id, check_at, check_answer, \
                check_prompt, check_options, \
                EXTRACT(EPOCH FROM (NOW() - check_issued_at))::float8 \
         FROM course_watch WHERE user_id = $1::uuid AND lesson_id = $2"
    };
}

impl From<RawRow> for WatchRow {
    fn from(raw: RawRow) -> Self {
        let (
            watched,
            last_position,
            beat_age,
            checkpoints,
            stretch_from,
            id,
            at,
            answer,
            prompt,
            options,
            age,
        ) = raw;
        let check = match (id, at, answer, prompt, options) {
            (Some(id), Some(at), Some(answer), Some(prompt), Some(options)) => Some(OpenCheck {
                id,
                at,
                answer,
                prompt,
                options,
                age: age.unwrap_or(0.0),
            }),
            _ => None,
        };
        WatchRow {
            watched,
            last_position,
            beat_age,
            checkpoints,
            stretch_from,
            check,
        }
    }
}

async fn load(
    conn: &mut PgConnection,
    user_id: &str,
    lesson_id: &str,
) -> Result<Option<WatchRow>, ApiError> {
    let raw: Option<RawRow> = sqlx::query_as(select_row!())
        .bind(user_id)
        .bind(lesson_id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(raw.map(WatchRow::from))
}

/// Cria a linha na primeira vez (sorteando os momentos das perguntas) e a trava.
async fn load_for_update(
    conn: &mut PgConnection,
    user_id: &str,
    lesson_id: &str,
    video: &VideoKey,
) -> Result<WatchRow, ApiError> {
    let checkpoints = plan_checkpoints(video, &mut rand_below);
    sqlx::query(
        "INSERT INTO course_watch (user_id, lesson_id, checkpoints) VALUES ($1::uuid, $2, $3) \
         ON CONFLICT (user_id, lesson_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(lesson_id)
    .bind(&checkpoints)
    .execute(&mut *conn)
    .await?;
    let raw: RawRow = sqlx::query_as(concat!(select_row!(), " FOR UPDATE"))
        .bind(user_id)
        .bind(lesson_id)
        .fetch_one(&mut *conn)
        .await?;
    Ok(raw.into())
}

async fn save(
    conn: &mut PgConnection,
    user_id: &str,
    lesson_id: &str,
    row: &WatchRow,
    beat: bool,
) -> Result<(), ApiError> {
    let check = row.check.as_ref();
    sqlx::query(
        "UPDATE course_watch SET watched = $3, last_position = $4, checkpoints = $5, \
           stretch_from = $6, check_id = $7, check_at = $8, check_answer = $9, \
           check_prompt = $10, check_options = $11, \
           check_issued_at = CASE WHEN $7::uuid IS NULL THEN NULL \
                                  WHEN check_id IS DISTINCT FROM $7::uuid THEN NOW() \
                                  ELSE check_issued_at END, \
           last_beat_at = CASE WHEN $12 THEN NOW() ELSE last_beat_at END, \
           updated_at = NOW() \
         WHERE user_id = $1::uuid AND lesson_id = $2",
    )
    .bind(user_id)
    .bind(lesson_id)
    .bind(&row.watched)
    .bind(row.last_position)
    .bind(&row.checkpoints)
    .bind(row.stretch_from)
    .bind(check.map(|c| c.id))
    .bind(check.map(|c| c.at))
    .bind(check.map(|c| c.answer.as_str()))
    .bind(check.map(|c| c.prompt.as_str()))
    .bind(check.map(|c| c.options.clone()))
    .bind(beat)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

// ─── Regras (puras, testadas abaixo) ───

pub(crate) fn rand_below(n: u32) -> u32 {
    (Uuid::new_v4().as_u128() % u128::from(n.max(1))) as u32
}

pub(crate) fn shuffle<T>(items: &mut [T], rand_below: &mut impl FnMut(u32) -> u32) {
    for i in (1..items.len()).rev() {
        let j = rand_below(i as u32 + 1) as usize;
        items.swap(i, j);
    }
}

fn total_seconds(video: &VideoKey) -> usize {
    video.duration.ceil().max(1.0) as usize
}

fn set_range(bits: &mut Vec<u8>, total: usize, start: usize, end: usize, on: bool) {
    let bytes = total.div_ceil(8);
    if bits.len() < bytes {
        bits.resize(bytes, 0);
    }
    for second in start..end.min(total) {
        let mask = 1u8 << (second % 8);
        if on {
            bits[second / 8] |= mask;
        } else {
            bits[second / 8] &= !mask;
        }
    }
}

fn watched_percent(bits: &[u8], total: usize) -> i16 {
    let seen = (0..total)
        .filter(|s| bits.get(s / 8).is_some_and(|b| b & (1 << (s % 8)) != 0))
        .count();
    (seen * 100 / total.max(1)) as i16
}

/// Trecho contínuo que o servidor aceita: até 2x, dentro do vídeo e sem passar
/// do que o relógio permite desde o aviso anterior.
fn segment_accepted(from: f64, to: f64, rate: f64, beat_age: Option<f64>, duration: f64) -> bool {
    if !(from.is_finite() && to.is_finite() && rate.is_finite()) {
        return false;
    }
    if rate <= 0.0 || rate > MAX_RATE + 1e-9 || from < 0.0 || to <= from || to > duration + 1.0 {
        return false;
    }
    let span = to - from;
    span <= MAX_SEGMENT_SECONDS
        && beat_age.is_some_and(|age| span <= MAX_RATE * age + CLOCK_SLACK_SECONDS)
}

/// 3 perguntas em vídeos de 4 min ou mais, 2 de 100 s ou mais, senão 1; entre
/// 15% e 90% do vídeo e com 75 s entre elas.
fn plan_checkpoints(video: &VideoKey, rand_below: &mut impl FnMut(u32) -> u32) -> Vec<f64> {
    if video.chapters.len() < 2 {
        return Vec::new();
    }
    let duration = video.duration;
    let (lo, hi) = (duration * 0.15, duration * 0.9);
    let mut count: usize = if duration >= 240.0 {
        3
    } else if duration >= 100.0 {
        2
    } else {
        1
    };
    while count > 1 && hi - lo < CHECK_MIN_GAP_SECONDS * (count - 1) as f64 {
        count -= 1;
    }
    let free = (hi - lo - CHECK_MIN_GAP_SECONDS * (count - 1) as f64).max(0.0);
    let mut picks: Vec<f64> = (0..count)
        .map(|_| free * f64::from(rand_below(10_000)) / 10_000.0)
        .collect();
    picks.sort_by(f64::total_cmp);
    picks
        .iter()
        .enumerate()
        .map(|(i, p)| ((lo + p + CHECK_MIN_GAP_SECONDS * i as f64) * 10.0).round() / 10.0)
        .collect()
}

fn chapter_at(chapters: &[(f64, String)], t: f64) -> Option<usize> {
    chapters.iter().rposition(|(start, _)| *start <= t)
}

/// Pergunta sobre o capítulo que acabou de passar. Alternativas erradas: temas
/// da mesma aula longe deste trecho e, se faltarem, temas de outras aulas.
fn build_check(
    video: &VideoKey,
    at: f64,
    other_titles: &[&str],
    rand_below: &mut impl FnMut(u32) -> u32,
) -> Option<OpenCheck> {
    let chapters = &video.chapters;
    let mut index = chapter_at(chapters, (at - 5.0).max(0.0))?;
    if index == 0 {
        // A abertura repete o título da aula, que está na tela: vale o capítulo seguinte.
        index = chapter_at(chapters, at)?;
    }
    let answer = chapters[index].1.clone();
    let end_of = |i: usize| chapters.get(i + 1).map_or(video.duration, |c| c.0);
    let near =
        |i: usize| chapters[i].0 <= at + CHECK_NEAR_AFTER && end_of(i) >= at - CHECK_NEAR_BEFORE;
    let mut pool: Vec<String> = Vec::new();
    for (i, (_, title)) in chapters.iter().enumerate() {
        let excluded = near(i) || *title == answer || pool.contains(title);
        if !excluded
            && !chapters
                .iter()
                .enumerate()
                .any(|(j, c)| near(j) && c.1 == *title)
        {
            pool.push(title.clone());
        }
    }
    shuffle(&mut pool, rand_below);
    pool.truncate(CHECK_OPTIONS - 1);
    if pool.len() < CHECK_OPTIONS - 1 {
        let mut others: Vec<&str> = other_titles
            .iter()
            .copied()
            .filter(|t| !chapters.iter().any(|c| c.1 == *t))
            .collect();
        shuffle(&mut others, rand_below);
        for title in others {
            if pool.len() == CHECK_OPTIONS - 1 {
                break;
            }
            if !pool.iter().any(|p| p == title) {
                pool.push(title.to_string());
            }
        }
    }
    pool.push(answer.clone());
    shuffle(&mut pool, rand_below);
    Some(OpenCheck {
        id: Uuid::new_v4(),
        at,
        answer,
        prompt: CHECK_PROMPT.to_string(),
        options: pool,
        age: 0.0,
    })
}

/// Títulos de capítulo de todas as aulas, sem repetição (alternativas de reserva).
fn all_titles() -> &'static [&'static str] {
    static TITLES: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    TITLES.get_or_init(|| {
        let mut titles: Vec<&'static str> = Vec::new();
        for video in quiz_key().values().filter_map(|l| l.video.as_ref()) {
            for (_, title) in &video.chapters {
                if !titles.contains(&title.as_str()) {
                    titles.push(title.as_str());
                }
            }
        }
        titles
    })
}

/// Errou ou deixou passar o tempo: o trecho desde a pergunta anterior deixa de
/// contar e a pergunta volta quando o aluno passar de novo por aquele momento.
/// Devolve o ponto para onde o vídeo volta.
fn fail_check(row: &mut WatchRow, total: usize) -> f64 {
    let from = row.stretch_from;
    let to = row.last_position.max(from);
    set_range(
        &mut row.watched,
        total,
        from.floor() as usize,
        to.ceil() as usize,
        false,
    );
    row.last_position = from;
    row.check = None;
    from
}

fn check_expired(check: &OpenCheck) -> bool {
    check.age > CHECK_ANSWER_SECONDS + CHECK_GRACE_SECONDS
}

fn quiz_ready(row: &WatchRow, video: &VideoKey) -> bool {
    watched_percent(&row.watched, total_seconds(video)) >= REQUIRED_PERCENT
        && row.checkpoints.is_empty()
        && row.check.is_none()
}

/// O quiz só abre com o vídeo assistido e as perguntas surpresa acertadas.
pub(crate) async fn quiz_unlocked(
    conn: &mut PgConnection,
    user_id: &str,
    lesson_id: &str,
    video: &VideoKey,
) -> Result<bool, ApiError> {
    Ok(load(conn, user_id, lesson_id)
        .await?
        .is_some_and(|row| quiz_ready(&row, video)))
}

// ─── Rotas ───

#[derive(Debug, Serialize)]
pub struct CheckOut {
    pub id: Uuid,
    pub prompt: String,
    pub options: Vec<String>,
    pub seconds_left: i64,
}

/// Estado da aula para o site: vídeo, perguntas, quiz e espera.
#[derive(Debug, Serialize)]
pub struct LessonState {
    pub lesson_id: String,
    pub status: String,
    pub best_score: i16,
    pub attempts: i32,
    pub has_video: bool,
    pub watched_percent: i16,
    pub required_percent: i16,
    pub checks_left: usize,
    /// Com o vídeo já em 90%, momento (s) de uma pergunta ainda não alcançada.
    pub missing_check_near: Option<f64>,
    pub pending_check: Option<CheckOut>,
    pub quiz_open: bool,
    /// Próxima tentativa do quiz (UTC), enquanto a espera corre.
    pub retry_at: Option<String>,
    pub review: Option<Vec<ReviewItem>>,
    /// Só nas respostas de `watch`/`check`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_result: Option<&'static str>,
    /// Ponto do vídeo para onde voltar depois de uma pergunta perdida.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewind_to: Option<f64>,
}

fn find_lesson(lesson_id: &str) -> Result<&'static LessonKey, ApiError> {
    quiz_key()
        .get(lesson_id.trim())
        .ok_or_else(|| ApiError::BadRequest("Aula inexistente.".to_string()))
}

async fn lesson_state(
    conn: &mut PgConnection,
    user_id: &str,
    lesson_id: &str,
    lesson: &LessonKey,
) -> Result<LessonState, ApiError> {
    let progress: Option<(String, i16, i32, Option<String>)> = sqlx::query_as(concat!(
        "SELECT status, best_score, attempts, ",
        retry_at_sql!(),
        " FROM course_progress WHERE user_id = $1::uuid AND lesson_id = $2"
    ))
    .bind(user_id)
    .bind(lesson_id)
    .bind(RETRY_WAIT_SECONDS)
    .fetch_optional(&mut *conn)
    .await?;
    let (status, best_score, attempts, retry_at) =
        progress.unwrap_or_else(|| ("new".to_string(), 0, 0, None));
    let completed = status == "completed" || best_score >= PASS_SCORE;
    let mut state = LessonState {
        lesson_id: lesson_id.to_string(),
        status,
        best_score,
        attempts,
        has_video: lesson.video.is_some(),
        watched_percent: 0,
        required_percent: REQUIRED_PERCENT,
        checks_left: 0,
        missing_check_near: None,
        pending_check: None,
        quiz_open: lesson.video.is_none() || completed,
        retry_at: if completed { None } else { retry_at },
        review: completed.then(|| lesson.review.clone()),
        accepted: None,
        check_result: None,
        rewind_to: None,
    };
    let Some(video) = &lesson.video else {
        return Ok(state);
    };
    if let Some(row) = load(conn, user_id, lesson_id).await? {
        state.watched_percent = watched_percent(&row.watched, total_seconds(video));
        state.checks_left = row.checkpoints.len();
        state.quiz_open = completed || quiz_ready(&row, video);
        if state.watched_percent >= REQUIRED_PERCENT && row.check.is_none() {
            state.missing_check_near = row.checkpoints.first().map(|t| (t - 5.0).max(0.0));
        }
        state.pending_check = row.check.map(|c| CheckOut {
            id: c.id,
            prompt: c.prompt,
            options: c.options,
            seconds_left: (CHECK_ANSWER_SECONDS - c.age).ceil().max(0.0) as i64,
        });
    } else if !completed {
        state.checks_left = plan_checkpoints(video, &mut |_| 0).len();
    }
    Ok(state)
}

#[derive(Debug, Deserialize)]
pub struct LessonQuery {
    pub lesson_id: String,
}

/// GET /api/course/lesson?lesson_id= — estado da aula do aluno.
pub async fn get_lesson_state(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Query(query): Query<LessonQuery>,
) -> Result<Json<LessonState>, ApiError> {
    let lesson = find_lesson(&query.lesson_id)?;
    let mut conn = state.db.acquire().await?;
    Ok(Json(
        lesson_state(
            &mut conn,
            &auth_user.user_id,
            query.lesson_id.trim(),
            lesson,
        )
        .await?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct WatchBody {
    pub lesson_id: String,
    pub from: f64,
    pub to: f64,
    pub rate: f64,
}

/// POST /api/course/watch — trecho assistido (`from == to` só marca o início).
pub async fn post_watch(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<WatchBody>,
) -> Result<Json<LessonState>, ApiError> {
    let lesson_id = body.lesson_id.trim();
    let lesson = find_lesson(lesson_id)?;
    let Some(video) = &lesson.video else {
        return Err(ApiError::BadRequest("Aula sem vídeo.".to_string()));
    };
    let user_id = auth_user.user_id.as_str();
    let mut tx = state.db.begin().await?;
    let mut out = lesson_state(&mut tx, user_id, lesson_id, lesson).await?;
    if out.status == "completed" || out.best_score >= PASS_SCORE {
        // Aula já concluída: rever o vídeo não conta nem abre pergunta.
        out.accepted = Some(false);
        return Ok(Json(out));
    }
    let total = total_seconds(video);
    let mut row = load_for_update(&mut tx, user_id, lesson_id, video).await?;
    let mut rewind_to = None;
    let mut check_result = None;
    if row.check.as_ref().is_some_and(check_expired) {
        rewind_to = Some(fail_check(&mut row, total));
        check_result = Some("expired");
    }
    let accepted = segment_accepted(body.from, body.to, body.rate, row.beat_age, video.duration);
    if accepted && rewind_to.is_none() {
        set_range(
            &mut row.watched,
            total,
            body.from.floor() as usize,
            body.to.floor() as usize,
            true,
        );
        row.last_position = body.to;
        if row.check.is_none() {
            if let Some(&at) = row
                .checkpoints
                .iter()
                .find(|t| body.from < **t && **t <= body.to)
            {
                row.check = build_check(video, at, all_titles(), &mut rand_below);
            }
        }
    }
    save(&mut tx, user_id, lesson_id, &row, true).await?;
    let mut out = lesson_state(&mut tx, user_id, lesson_id, lesson).await?;
    tx.commit().await?;
    out.accepted = Some(accepted && rewind_to.is_none());
    out.check_result = check_result;
    out.rewind_to = rewind_to;
    Ok(Json(out))
}

#[derive(Debug, Deserialize)]
pub struct CheckBody {
    pub lesson_id: String,
    pub check_id: Uuid,
    /// `None` = o tempo acabou no navegador.
    pub answer: Option<String>,
}

/// POST /api/course/check — resposta da pergunta surpresa.
pub async fn post_check(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<CheckBody>,
) -> Result<Json<LessonState>, ApiError> {
    let lesson_id = body.lesson_id.trim();
    let lesson = find_lesson(lesson_id)?;
    let Some(video) = &lesson.video else {
        return Err(ApiError::BadRequest("Aula sem vídeo.".to_string()));
    };
    let user_id = auth_user.user_id.as_str();
    let mut tx = state.db.begin().await?;
    let mut row = load_for_update(&mut tx, user_id, lesson_id, video).await?;
    let Some(check) = row.check.as_ref().filter(|c| c.id == body.check_id) else {
        let mut out = lesson_state(&mut tx, user_id, lesson_id, lesson).await?;
        out.check_result = Some("stale");
        return Ok(Json(out));
    };
    let expired = check_expired(check) || body.answer.is_none();
    let right = !expired && body.answer.as_deref() == Some(check.answer.as_str());
    let at = check.at;
    let mut rewind_to = None;
    let result = if right {
        row.checkpoints.retain(|t| (t - at).abs() > 1e-6);
        row.stretch_from = at;
        row.check = None;
        // Se o aluno já passou da próxima pergunta enquanto respondia, ela abre agora.
        if let Some(&next) = row.checkpoints.first().filter(|t| **t <= row.last_position) {
            row.check = build_check(video, next, all_titles(), &mut rand_below);
        }
        "right"
    } else {
        rewind_to = Some(fail_check(&mut row, total_seconds(video)));
        if expired {
            "expired"
        } else {
            "wrong"
        }
    };
    save(&mut tx, user_id, lesson_id, &row, false).await?;
    let mut out = lesson_state(&mut tx, user_id, lesson_id, lesson).await?;
    tx.commit().await?;
    out.check_result = Some(result);
    out.rewind_to = rewind_to;
    Ok(Json(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn videos() -> Vec<(&'static String, &'static VideoKey)> {
        let mut all: Vec<_> = quiz_key()
            .iter()
            .filter_map(|(id, l)| l.video.as_ref().map(|v| (id, v)))
            .collect();
        all.sort_by_key(|(id, _)| id.as_str());
        all
    }

    #[test]
    fn trecho_so_conta_ate_2x_e_dentro_do_relogio() {
        // 10 s de vídeo depois de 5 s de relógio = 2x: aceito.
        assert!(segment_accepted(0.0, 10.0, 2.0, Some(5.0), 300.0));
        // Velocidade acima de 2x, sem aviso anterior, ou rápido demais para o relógio.
        assert!(!segment_accepted(0.0, 10.0, 2.5, Some(60.0), 300.0));
        assert!(!segment_accepted(0.0, 10.0, 1.0, None, 300.0));
        assert!(!segment_accepted(0.0, 10.0, 1.0, Some(4.0), 300.0));
        // Fora do vídeo, de trás para a frente, longo demais ou número inválido.
        assert!(!segment_accepted(295.0, 305.0, 1.0, Some(60.0), 300.0));
        assert!(!segment_accepted(10.0, 5.0, 1.0, Some(60.0), 300.0));
        assert!(!segment_accepted(0.0, 31.0, 1.0, Some(60.0), 300.0));
        assert!(!segment_accepted(f64::NAN, 5.0, 1.0, Some(60.0), 300.0));
    }

    #[test]
    fn segundos_marcados_e_apagados() {
        let mut bits = Vec::new();
        set_range(&mut bits, 100, 0, 89, true);
        assert_eq!(watched_percent(&bits, 100), 89);
        set_range(&mut bits, 100, 89, 95, true);
        assert_eq!(watched_percent(&bits, 100), 95);
        set_range(&mut bits, 100, 50, 200, false);
        assert_eq!(watched_percent(&bits, 100), 50);
    }

    #[test]
    fn perguntas_sorteadas_em_todas_as_aulas_com_video() {
        assert!(!videos().is_empty());
        for (id, video) in videos() {
            let expected = if video.duration >= 240.0 {
                3
            } else if video.duration >= 100.0 {
                2
            } else {
                1
            };
            for extreme in [0u32, 9_999] {
                let points = plan_checkpoints(video, &mut |n| extreme.min(n - 1));
                assert_eq!(points.len(), expected, "{id}");
                for (i, t) in points.iter().enumerate() {
                    assert!(*t >= video.duration * 0.15 - 0.1, "{id}: {t}");
                    assert!(*t <= video.duration * 0.9 + 0.1, "{id}: {t}");
                    if i > 0 {
                        assert!(t - points[i - 1] >= CHECK_MIN_GAP_SECONDS - 0.1, "{id}");
                    }
                    let check = build_check(video, *t, all_titles(), &mut |n| extreme.min(n - 1))
                        .unwrap_or_else(|| panic!("{id}: sem pergunta em {t}"));
                    assert_eq!(
                        check.options.len(),
                        CHECK_OPTIONS,
                        "{id}: {:?}",
                        check.options
                    );
                    let mut unique = check.options.clone();
                    unique.sort();
                    unique.dedup();
                    assert_eq!(unique.len(), CHECK_OPTIONS, "{id}: {:?}", check.options);
                    assert!(check.options.contains(&check.answer), "{id}");
                    // A resposta é um capítulo que acabou de passar, e nenhuma
                    // alternativa errada é tema do mesmo trecho.
                    let start = video
                        .chapters
                        .iter()
                        .find(|c| c.1 == check.answer)
                        .map(|c| c.0)
                        .unwrap();
                    assert!(start <= *t, "{id}");
                    for wrong in check.options.iter().filter(|o| **o != check.answer) {
                        let near = video.chapters.iter().enumerate().any(|(i, c)| {
                            let end = video.chapters.get(i + 1).map_or(video.duration, |n| n.0);
                            c.1 == *wrong
                                && c.0 <= t + CHECK_NEAR_AFTER
                                && end >= t - CHECK_NEAR_BEFORE
                        });
                        assert!(!near, "{id}: {wrong} é do mesmo trecho");
                    }
                }
            }
        }
    }

    #[test]
    fn pergunta_perdida_apaga_o_trecho_desde_a_anterior() {
        let mut row = WatchRow {
            watched: Vec::new(),
            last_position: 130.0,
            beat_age: Some(1.0),
            checkpoints: vec![120.0, 200.0],
            stretch_from: 40.0,
            check: None,
        };
        set_range(&mut row.watched, 300, 0, 130, true);
        assert_eq!(fail_check(&mut row, 300), 40.0);
        assert_eq!(watched_percent(&row.watched, 300), 13);
        assert_eq!(row.last_position, 40.0);
        assert_eq!(row.checkpoints, vec![120.0, 200.0]);
    }
}
