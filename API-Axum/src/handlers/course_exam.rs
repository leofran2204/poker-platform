//! Prova final e certificado da Zero Tilt Academy (etapa 6b do plano de 24/11).
//!
//! O certificado sai só pelo servidor: todas as aulas exigidas cumpridas pelas
//! regras da 6a (quiz aprovado com o vídeo assistido) e prova final com tempo,
//! uma pergunta sorteada de cada uma de 20 aulas sorteadas, alternativas
//! embaralhadas e uma tentativa a cada 24 h. O código do certificado é público:
//! `GET /api/certificate/verify` confirma nome, nota e data.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use std::collections::HashMap;
use uuid::Uuid;

use super::course::{certificate_lessons, quiz_key, QuestionKey, PASS_SCORE};
use super::course_watch::{quiz_unlocked, rand_below, shuffle};
use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

pub(crate) const EXAM_QUESTIONS: usize = 20;
pub(crate) const EXAM_SECONDS: f64 = 30.0 * 60.0;
/// Folga de rede depois do fim do tempo; depois dela a prova conta como reprovada.
const SUBMIT_GRACE_SECONDS: f64 = 30.0;
/// Uma tentativa a cada 24 h, contadas do início da anterior.
pub(crate) const EXAM_COOLDOWN_SECONDS: f64 = 24.0 * 3600.0;
const NAME_MIN: usize = 5;
const NAME_MAX: usize = 80;
const MAX_ANSWERS: usize = 64;
const MAX_ANSWER_LEN: usize = 200;
/// Sem 0, 1, I e O, para o código ser lido e digitado sem confusão.
const CODE_ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const CODE_BODY_LEN: usize = 12;

/// Texto UTC (`2026-11-25T18:00:00Z`) de uma expressão de data do SQL.
macro_rules! utc_text {
    ($e:literal) => {
        concat!(
            "to_char((",
            $e,
            ") AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')"
        )
    };
}

// ─── Regras puras ───

/// Nome para o certificado: nome e sobrenome, só letras, espaço, apóstrofo,
/// hífen e ponto.
pub(crate) fn clean_name(raw: &str) -> Option<String> {
    let name = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let length = name.chars().count();
    let allowed = name
        .chars()
        .all(|c| c.is_alphabetic() || matches!(c, ' ' | '\'' | '’' | '-' | '.'));
    let words = name
        .split(' ')
        .filter(|w| w.chars().any(char::is_alphabetic))
        .count();
    (allowed && words >= 2 && (NAME_MIN..=NAME_MAX).contains(&length)).then_some(name)
}

pub(crate) fn new_code(rand: &mut impl FnMut(u32) -> u32) -> String {
    let body: String = (0..CODE_BODY_LEN)
        .map(|_| CODE_ALPHABET[rand(CODE_ALPHABET.len() as u32) as usize] as char)
        .collect();
    format!("ZT-{}-{}-{}", &body[0..4], &body[4..8], &body[8..12])
}

/// Aceita o código com ou sem `ZT-`, hífens, espaços e minúsculas.
pub(crate) fn normalize_code(raw: &str) -> Option<String> {
    let chars: String = raw
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let body = match chars.len() {
        CODE_BODY_LEN => chars.as_str(),
        n if n == CODE_BODY_LEN + 2 && chars.starts_with("ZT") => &chars[2..],
        _ => return None,
    };
    body.bytes()
        .all(|b| CODE_ALPHABET.contains(&b))
        .then(|| format!("ZT-{}-{}-{}", &body[0..4], &body[4..8], &body[8..12]))
}

/// Uma pergunta de cada uma de até 20 aulas exigidas, sorteadas.
pub(crate) fn draw_questions(rand: &mut impl FnMut(u32) -> u32) -> Vec<String> {
    let key = quiz_key();
    let mut lessons: Vec<&String> = certificate_lessons()
        .iter()
        .filter(|id| key.get(*id).is_some_and(|l| !l.questions.is_empty()))
        .collect();
    shuffle(&mut lessons, rand);
    lessons
        .into_iter()
        .take(EXAM_QUESTIONS)
        .map(|id| {
            let questions = &key[id].questions;
            questions[rand(questions.len() as u32) as usize].id.clone()
        })
        .collect()
}

fn find_question(id: &str) -> Option<(&'static str, &'static QuestionKey)> {
    quiz_key().iter().find_map(|(lesson_id, lesson)| {
        lesson
            .questions
            .iter()
            .find(|q| q.id == id)
            .map(|q| (lesson_id.as_str(), q))
    })
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ExamGrade {
    pub(crate) score: i16,
    /// Aulas das perguntas erradas, na ordem da prova, sem repetir.
    pub(crate) review_lessons: Vec<String>,
}

/// Pergunta sem resposta conta como erro; arredonda como o quiz das aulas.
pub(crate) fn grade_exam(question_ids: &[String], answers: &HashMap<String, String>) -> ExamGrade {
    let mut right = 0usize;
    let mut review_lessons: Vec<String> = Vec::new();
    for id in question_ids {
        match find_question(id) {
            Some((_, q)) if answers.get(id) == Some(&q.correct) => right += 1,
            Some((lesson_id, _)) if !review_lessons.iter().any(|l| l == lesson_id) => {
                review_lessons.push(lesson_id.to_string());
            }
            _ => {}
        }
    }
    let total = question_ids.len().max(1);
    ExamGrade {
        score: ((right * 200 + total) / (total * 2)) as i16,
        review_lessons,
    }
}

// ─── Saídas ───

#[derive(Debug, Serialize)]
pub struct ExamQuestion {
    pub id: String,
    pub kind: &'static str,
    pub prompt: String,
    pub options: Vec<String>,
    pub hole: Vec<String>,
    pub board: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExamOut {
    pub attempt_id: Uuid,
    pub holder_name: String,
    pub deadline: String,
    pub seconds_left: i64,
    pub questions: Vec<ExamQuestion>,
}

fn exam_out(
    attempt_id: Uuid,
    holder_name: String,
    deadline: String,
    seconds_left: f64,
    question_ids: &[String],
) -> ExamOut {
    let questions = question_ids
        .iter()
        .filter_map(|id| find_question(id))
        .map(|(_, q)| {
            let mut options = q.options.clone();
            shuffle(&mut options, &mut rand_below);
            ExamQuestion {
                id: q.id.clone(),
                kind: if q.engine { "engine" } else { "theory" },
                prompt: q.prompt.clone(),
                options,
                hole: q.hole.clone(),
                board: q.board.clone(),
            }
        })
        .collect();
    ExamOut {
        attempt_id,
        holder_name,
        deadline,
        seconds_left: seconds_left.max(0.0).ceil() as i64,
        questions,
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct MissingLesson {
    pub lesson_id: String,
    /// `quiz` = falta passar no quiz; `video` = quiz aprovado antes da contagem
    /// do vídeo (25/11), falta assistir ao vídeo com as perguntas surpresa.
    pub reason: &'static str,
}

#[derive(Debug, Serialize)]
pub struct CertificateOut {
    pub code: String,
    pub holder_name: String,
    /// `kyc` = nome conferido na verificação de identidade; `declared` = informado pelo aluno.
    pub name_source: String,
    pub score: i16,
    pub lessons: i16,
    pub issued_at: String,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LastAttempt {
    pub score: i16,
    pub passed: bool,
    pub timed_out: bool,
    pub finished_at: String,
}

#[derive(Debug, Serialize)]
pub struct CertificateStatus {
    pub total_lessons: usize,
    pub missing: Vec<MissingLesson>,
    pub eligible: bool,
    /// Nome da verificação de identidade aprovada, usado no certificado.
    pub verified_name: Option<String>,
    pub exam_questions: usize,
    pub exam_minutes: i64,
    pub pass_score: i16,
    pub open_exam: Option<ExamOut>,
    pub next_attempt_at: Option<String>,
    pub last_attempt: Option<LastAttempt>,
    pub certificate: Option<CertificateOut>,
}

#[derive(Debug, Serialize)]
pub struct ExamResult {
    pub score: i16,
    pub passed: bool,
    pub timed_out: bool,
    /// Só na reprovação: aulas a rever (sem gabarito).
    pub review_lessons: Vec<String>,
    pub next_attempt_at: Option<String>,
    pub certificate: Option<CertificateOut>,
}

// ─── Banco ───

/// Aulas exigidas que ainda faltam. Quiz aprovado com o vídeo cumprido fica
/// gravado em `attended_at`; quem passou antes da contagem do vídeo precisa
/// assisti-lo (90%, perguntas surpresa) para a aula valer no certificado.
async fn missing_lessons(
    conn: &mut PgConnection,
    user_id: &str,
) -> Result<Vec<MissingLesson>, ApiError> {
    let required = certificate_lessons();
    let rows: Vec<(String, bool, bool)> = sqlx::query_as(
        "SELECT lesson_id, (status = 'completed' OR best_score >= $3), attended_at IS NOT NULL \
         FROM course_progress WHERE user_id = $1::uuid AND lesson_id = ANY($2)",
    )
    .bind(user_id)
    .bind(required)
    .bind(PASS_SCORE)
    .fetch_all(&mut *conn)
    .await?;
    let by_id: HashMap<&str, (bool, bool)> = rows
        .iter()
        .map(|(id, passed, attended)| (id.as_str(), (*passed, *attended)))
        .collect();
    let mut missing = Vec::new();
    let mut attended_now: Vec<&str> = Vec::new();
    for lesson_id in required {
        let (passed, attended) = by_id
            .get(lesson_id.as_str())
            .copied()
            .unwrap_or((false, false));
        if !passed {
            missing.push(MissingLesson {
                lesson_id: lesson_id.clone(),
                reason: "quiz",
            });
            continue;
        }
        if attended {
            continue;
        }
        let video_ok = match quiz_key().get(lesson_id).and_then(|l| l.video.as_ref()) {
            None => true,
            Some(video) => quiz_unlocked(conn, user_id, lesson_id, video).await?,
        };
        if video_ok {
            attended_now.push(lesson_id);
        } else {
            missing.push(MissingLesson {
                lesson_id: lesson_id.clone(),
                reason: "video",
            });
        }
    }
    if !attended_now.is_empty() {
        sqlx::query(
            "UPDATE course_progress SET attended_at = NOW() \
             WHERE user_id = $1::uuid AND lesson_id = ANY($2) AND attended_at IS NULL",
        )
        .bind(user_id)
        .bind(&attended_now)
        .execute(&mut *conn)
        .await?;
    }
    Ok(missing)
}

/// Prova aberta e já fora do tempo (com a folga) = reprovada por tempo.
async fn close_expired(conn: &mut PgConnection, user_id: &str) -> Result<(), ApiError> {
    sqlx::query(
        "UPDATE course_exam_attempts \
         SET finished_at = NOW(), score = 0, passed = FALSE, timed_out = TRUE \
         WHERE user_id = $1::uuid AND finished_at IS NULL \
           AND NOW() > deadline + make_interval(secs => $2)",
    )
    .bind(user_id)
    .bind(SUBMIT_GRACE_SECONDS)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

type CertificateRow = (String, String, String, i16, i16, String, Option<String>);

fn certificate_out(row: CertificateRow) -> CertificateOut {
    let (code, holder_name, name_source, score, lessons, issued_at, revoked_at) = row;
    CertificateOut {
        code,
        holder_name,
        name_source,
        score,
        lessons,
        issued_at,
        revoked_at,
    }
}

macro_rules! certificate_select {
    () => {
        concat!(
            "SELECT code, holder_name, name_source, score, lessons, ",
            utc_text!("issued_at"),
            ", ",
            utc_text!("revoked_at"),
            " FROM course_certificates"
        )
    };
}

async fn user_certificate(
    conn: &mut PgConnection,
    user_id: &str,
) -> Result<Option<CertificateOut>, ApiError> {
    let row: Option<CertificateRow> =
        sqlx::query_as(concat!(certificate_select!(), " WHERE user_id = $1::uuid"))
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await?;
    Ok(row.map(certificate_out))
}

async fn open_exam(conn: &mut PgConnection, user_id: &str) -> Result<Option<ExamOut>, ApiError> {
    let row: Option<(Uuid, String, String, f64, Vec<String>)> = sqlx::query_as(concat!(
        "SELECT id, holder_name, ",
        utc_text!("deadline"),
        ", EXTRACT(EPOCH FROM deadline - NOW())::float8, question_ids \
         FROM course_exam_attempts WHERE user_id = $1::uuid AND finished_at IS NULL"
    ))
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(row.map(|(id, name, deadline, left, ids)| exam_out(id, name, deadline, left, &ids)))
}

/// Próxima tentativa (UTC) enquanto corre a espera de 24 h depois de reprovar.
async fn next_attempt_at(
    conn: &mut PgConnection,
    user_id: &str,
) -> Result<Option<String>, ApiError> {
    Ok(sqlx::query_scalar(concat!(
        "SELECT ",
        utc_text!("started_at + make_interval(secs => $2)"),
        " FROM course_exam_attempts \
         WHERE user_id = $1::uuid AND finished_at IS NOT NULL AND NOT passed \
           AND started_at + make_interval(secs => $2) > NOW() \
         ORDER BY started_at DESC LIMIT 1"
    ))
    .bind(user_id)
    .bind(EXAM_COOLDOWN_SECONDS)
    .fetch_optional(&mut *conn)
    .await?)
}

async fn verified_name(conn: &mut PgConnection, user_id: &str) -> Result<Option<String>, ApiError> {
    let name: Option<Option<String>> = sqlx::query_scalar(
        "SELECT kyc_legal_name FROM users WHERE id = $1::uuid AND kyc_status = 'verified'",
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(name
        .flatten()
        .map(|n| n.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|n| !n.is_empty()))
}

// ─── Rotas ───

/// GET /api/course/certificate — aulas que faltam, prova e certificado.
pub async fn get_status(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
) -> Result<Json<CertificateStatus>, ApiError> {
    let user_id = auth_user.user_id.as_str();
    let mut tx = state.db.begin().await?;
    close_expired(&mut tx, user_id).await?;
    let missing = missing_lessons(&mut tx, user_id).await?;
    let certificate = user_certificate(&mut tx, user_id).await?;
    let open_exam = open_exam(&mut tx, user_id).await?;
    let next_attempt_at = next_attempt_at(&mut tx, user_id).await?;
    let verified_name = verified_name(&mut tx, user_id).await?;
    let last: Option<(i16, bool, bool, String)> = sqlx::query_as(concat!(
        "SELECT score, passed, timed_out, ",
        utc_text!("finished_at"),
        " FROM course_exam_attempts WHERE user_id = $1::uuid AND finished_at IS NOT NULL \
         ORDER BY started_at DESC LIMIT 1"
    ))
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(CertificateStatus {
        total_lessons: certificate_lessons().len(),
        eligible: missing.is_empty(),
        missing,
        verified_name,
        exam_questions: EXAM_QUESTIONS,
        exam_minutes: (EXAM_SECONDS / 60.0) as i64,
        pass_score: PASS_SCORE,
        open_exam,
        next_attempt_at,
        last_attempt: last.map(|(score, passed, timed_out, finished_at)| LastAttempt {
            score,
            passed,
            timed_out,
            finished_at,
        }),
        certificate,
    }))
}

#[derive(Debug, Deserialize)]
pub struct StartExamBody {
    /// Nome para o certificado; ignorado quando há verificação de identidade aprovada.
    pub name: Option<String>,
}

/// POST /api/course/exam/start — abre a prova final (ou devolve a que está aberta).
pub async fn start_exam(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<StartExamBody>,
) -> Result<Json<ExamOut>, ApiError> {
    let user_id = auth_user.user_id.as_str();
    let mut tx = state.db.begin().await?;
    // Uma abertura por vez por aluno: dois cliques não abrem duas provas.
    sqlx::query("SELECT 1 FROM users WHERE id = $1::uuid FOR UPDATE")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    close_expired(&mut tx, user_id).await?;
    if user_certificate(&mut tx, user_id).await?.is_some() {
        return Err(ApiError::Conflict(
            "Seu certificado já foi emitido.".to_string(),
        ));
    }
    if let Some(open) = open_exam(&mut tx, user_id).await? {
        tx.commit().await?;
        return Ok(Json(open));
    }
    if next_attempt_at(&mut tx, user_id).await?.is_some() {
        return Err(ApiError::TooManyRequests(
            "A prova final tem uma tentativa a cada 24 horas. Use o intervalo para rever as aulas."
                .to_string(),
        ));
    }
    if !missing_lessons(&mut tx, user_id).await?.is_empty() {
        return Err(ApiError::Forbidden(
            "Conclua todas as aulas (vídeo e quiz) antes da prova final.".to_string(),
        ));
    }
    let (holder_name, name_source) = match verified_name(&mut tx, user_id).await? {
        Some(name) => (name, "kyc"),
        None => {
            let name = body.name.as_deref().and_then(clean_name).ok_or_else(|| {
                ApiError::BadRequest(
                    "Informe nome e sobrenome para o certificado (só letras).".to_string(),
                )
            })?;
            (name, "declared")
        }
    };
    let question_ids = draw_questions(&mut rand_below);
    let (id, deadline, left): (Uuid, String, f64) = sqlx::query_as(concat!(
        "INSERT INTO course_exam_attempts \
           (id, user_id, deadline, question_ids, holder_name, name_source) \
         VALUES ($1, $2::uuid, NOW() + make_interval(secs => $3), $4, $5, $6) \
         RETURNING id, ",
        utc_text!("deadline"),
        ", EXTRACT(EPOCH FROM deadline - NOW())::float8"
    ))
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(EXAM_SECONDS)
    .bind(&question_ids)
    .bind(&holder_name)
    .bind(name_source)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(exam_out(
        id,
        holder_name,
        deadline,
        left,
        &question_ids,
    )))
}

#[derive(Debug, Deserialize)]
pub struct SubmitExamBody {
    pub attempt_id: Uuid,
    pub answers: HashMap<String, String>,
}

/// POST /api/course/exam/submit — corrige a prova e emite o certificado na aprovação.
pub async fn submit_exam(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<SubmitExamBody>,
) -> Result<Json<ExamResult>, ApiError> {
    if body.answers.len() > MAX_ANSWERS || body.answers.values().any(|a| a.len() > MAX_ANSWER_LEN) {
        return Err(ApiError::BadRequest("Respostas inválidas.".to_string()));
    }
    let user_id = auth_user.user_id.as_str();
    let mut tx = state.db.begin().await?;
    let attempt: Option<(Vec<String>, String, String, bool, bool)> = sqlx::query_as(
        "SELECT question_ids, holder_name, name_source, finished_at IS NOT NULL, \
                NOW() > deadline + make_interval(secs => $3) \
         FROM course_exam_attempts WHERE id = $1 AND user_id = $2::uuid FOR UPDATE",
    )
    .bind(body.attempt_id)
    .bind(user_id)
    .bind(SUBMIT_GRACE_SECONDS)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((question_ids, holder_name, name_source, finished, late)) = attempt else {
        return Err(ApiError::NotFound("Prova não encontrada.".to_string()));
    };
    if finished {
        return Err(ApiError::Conflict(
            "Esta prova já foi entregue.".to_string(),
        ));
    }
    let grade = grade_exam(&question_ids, &body.answers);
    let passed = !late && grade.score >= PASS_SCORE;
    sqlx::query(
        "UPDATE course_exam_attempts SET finished_at = NOW(), score = $2, passed = $3, \
           timed_out = $4 WHERE id = $1",
    )
    .bind(body.attempt_id)
    .bind(grade.score)
    .bind(passed)
    .bind(late)
    .execute(&mut *tx)
    .await?;

    let mut certificate = None;
    if passed {
        // Colisão de código (60 bits) é improvável; ainda assim, tenta outro.
        for _ in 0..3 {
            sqlx::query(
                "INSERT INTO course_certificates \
                   (code, user_id, attempt_id, holder_name, name_source, score, lessons) \
                 VALUES ($1, $2::uuid, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING",
            )
            .bind(new_code(&mut rand_below))
            .bind(user_id)
            .bind(body.attempt_id)
            .bind(&holder_name)
            .bind(&name_source)
            .bind(grade.score)
            .bind(certificate_lessons().len() as i16)
            .execute(&mut *tx)
            .await?;
            certificate = user_certificate(&mut tx, user_id).await?;
            if certificate.is_some() {
                break;
            }
        }
        if certificate.is_none() {
            return Err(ApiError::Internal(
                "Não foi possível emitir o certificado agora.".to_string(),
            ));
        }
    }
    let next_attempt_at = if passed {
        None
    } else {
        next_attempt_at(&mut tx, user_id).await?
    };
    tx.commit().await?;
    Ok(Json(ExamResult {
        score: grade.score,
        passed,
        timed_out: late,
        review_lessons: if passed {
            Vec::new()
        } else {
            grade.review_lessons
        },
        next_attempt_at,
        certificate,
    }))
}

#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    pub code: String,
}

/// GET /api/certificate/verify?code=ZT-XXXX-XXXX-XXXX — página pública de verificação.
pub async fn verify_certificate(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
) -> Result<Json<CertificateOut>, ApiError> {
    let not_found = || ApiError::NotFound("Certificado não encontrado.".to_string());
    let code = normalize_code(&query.code).ok_or_else(not_found)?;
    let row: Option<CertificateRow> =
        sqlx::query_as(concat!(certificate_select!(), " WHERE code = $1"))
            .bind(&code)
            .fetch_optional(&state.db)
            .await?;
    row.map(|r| Json(certificate_out(r))).ok_or_else(not_found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certificado_exige_as_25_aulas_sem_a_historia_opcional() {
        let required = certificate_lessons();
        assert_eq!(required.len(), 25);
        for optional in ["m0l1", "m0l2", "m0l3"] {
            assert!(!required.iter().any(|id| id == optional));
        }
        assert_eq!(required.first().map(String::as_str), Some("m0l4"));
        assert!(required.iter().all(|id| quiz_key().contains_key(id)));
    }

    #[test]
    fn prova_sorteia_20_perguntas_de_aulas_diferentes() {
        for seed in 0..50u32 {
            let mut state = seed;
            let mut rand = |n: u32| {
                state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                (state >> 8) % n.max(1)
            };
            let ids = draw_questions(&mut rand);
            assert_eq!(ids.len(), EXAM_QUESTIONS);
            let mut lessons: Vec<&str> = ids
                .iter()
                .map(|id| find_question(id).expect("pergunta existe").0)
                .collect();
            assert!(lessons
                .iter()
                .all(|l| certificate_lessons().iter().any(|r| r == l)));
            lessons.sort_unstable();
            lessons.dedup();
            assert_eq!(lessons.len(), EXAM_QUESTIONS);
        }
    }

    #[test]
    fn nota_da_prova_e_aulas_a_rever() {
        let ids = draw_questions(&mut rand_below);
        let mut answers: HashMap<String, String> = ids
            .iter()
            .map(|id| (id.clone(), find_question(id).unwrap().1.correct.clone()))
            .collect();
        assert_eq!(grade_exam(&ids, &answers).score, 100);
        for id in ids.iter().take(7) {
            answers.insert(id.clone(), "errada".to_string());
        }
        let grade = grade_exam(&ids, &answers);
        assert_eq!(grade.score, 65);
        assert_eq!(grade.review_lessons.len(), 7);
        answers.remove(&ids[0]);
        answers.insert(
            ids[1].clone(),
            find_question(&ids[1]).unwrap().1.correct.clone(),
        );
        assert_eq!(grade_exam(&ids, &answers).score, 70);
        assert_eq!(grade_exam(&ids, &HashMap::new()).score, 0);
    }

    #[test]
    fn nome_do_certificado() {
        assert_eq!(
            clean_name("  maria   da  Silva "),
            Some("maria da Silva".to_string())
        );
        assert_eq!(
            clean_name("João D'Ávila-Neto"),
            Some("João D'Ávila-Neto".to_string())
        );
        assert_eq!(clean_name("Maria"), None);
        assert_eq!(clean_name("Ana B"), Some("Ana B".to_string()));
        assert_eq!(clean_name("Maria Silva 2"), None);
        assert_eq!(clean_name("<b>Maria</b> Silva"), None);
        assert_eq!(clean_name(&format!("Ana {}", "a".repeat(80))), None);
    }

    #[test]
    fn codigo_do_certificado() {
        let code = new_code(&mut rand_below);
        assert_eq!(code.len(), 17);
        assert_eq!(normalize_code(&code), Some(code.clone()));
        let loose = code.to_lowercase().replace('-', " ");
        assert_eq!(normalize_code(&loose), Some(code.clone()));
        assert_eq!(
            normalize_code(&code[3..].replace('-', "")),
            Some(code.clone())
        );
        assert_eq!(normalize_code("ZT-0000-AAAA-BBBB"), None);
        assert_eq!(normalize_code("ZT-AAAA-BBBB"), None);
        assert_eq!(normalize_code("' OR 1=1 --"), None);
    }
}
