//! Course progress — aulas concluídas e melhor nota de quiz por usuário.
//! A nota é calculada aqui, a partir das respostas, com o gabarito do servidor.
//! O quiz só abre depois do vídeo assistido (`course_watch`) e, depois de uma
//! reprovação, só depois da espera; o gabarito comentado sai só para quem passou.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

use super::course_watch;
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
    /// Resposta escolhida por pergunta (`id` da pergunta → texto da alternativa).
    /// Clientes antigos mandavam `completed`/`score`; o servidor não aceita mais
    /// nota pronta do navegador.
    pub answers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct SaveProgressResponse {
    #[serde(flatten)]
    pub progress: CourseProgressItem,
    /// Nota desta tentativa (a melhor fica em `best_score`).
    pub score: i16,
    pub passed: bool,
    /// Quando abre a próxima tentativa (só depois de reprovar), em UTC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_at: Option<String>,
    /// Gabarito comentado: só para quem já passou na aula.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<Vec<ReviewItem>>,
}

/// SQL do horário da próxima tentativa (texto UTC), `NULL` se já liberada.
/// `$3` = espera em segundos.
macro_rules! retry_at_sql {
    () => {
        "CASE WHEN last_failed_at + make_interval(secs => $3) > NOW() \
         THEN to_char((last_failed_at + make_interval(secs => $3)) AT TIME ZONE 'UTC', \
                      'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') END"
    };
}
pub(crate) use retry_at_sql;

/// POST /api/course/progress — corrige o quiz pelo gabarito do servidor e
/// registra conclusão e melhor nota da aula.
pub async fn save_progress(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<SaveProgressBody>,
) -> Result<Json<SaveProgressResponse>, ApiError> {
    let lesson_id = body.lesson_id.trim();
    let Some(lesson) = quiz_key().get(lesson_id) else {
        return Err(ApiError::BadRequest("Aula inexistente.".to_string()));
    };
    let Some(answers) = body.answers else {
        return Err(ApiError::BadRequest(
            "Atualize a página: a nota do quiz agora é calculada pelo servidor.".to_string(),
        ));
    };
    if answers.len() > MAX_ANSWERS || answers.values().any(|a| a.len() > MAX_ANSWER_LEN) {
        return Err(ApiError::BadRequest("Respostas inválidas.".to_string()));
    }

    let mut tx = state.db.begin().await?;
    let mut ids: Vec<&str> = lesson.prerequisites.iter().map(String::as_str).collect();
    ids.push(lesson_id);
    let passed_rows: Vec<String> = sqlx::query_scalar(
        "SELECT lesson_id FROM course_progress \
         WHERE user_id = $1::uuid AND lesson_id = ANY($2) \
           AND (status = 'completed' OR best_score >= $3)",
    )
    .bind(&auth_user.user_id)
    .bind(&ids)
    .bind(PASS_SCORE)
    .fetch_all(&mut *tx)
    .await?;
    let passed = |id: &str| passed_rows.iter().any(|p| p == id);
    // Uma reorganização editorial nunca retira acesso a uma aula já concluída.
    if !passed(lesson_id) && !lesson.prerequisites.iter().all(|p| passed(p)) {
        return Err(ApiError::Forbidden(
            "Conclua o pré-requisito antes desta aula.".to_string(),
        ));
    }

    // Trava a linha da aula: duas tentativas ao mesmo tempo não furam a espera.
    sqlx::query(
        "INSERT INTO course_progress (user_id, lesson_id) VALUES ($1::uuid, $2) \
         ON CONFLICT (user_id, lesson_id) DO NOTHING",
    )
    .bind(&auth_user.user_id)
    .bind(lesson_id)
    .execute(&mut *tx)
    .await?;
    let (row, retry_at): (ProgressRow, Option<String>) = {
        let (id, status, best, attempts, updated, retry): (
            String,
            String,
            i16,
            i32,
            String,
            Option<String>,
        ) = sqlx::query_as(concat!(
            "SELECT lesson_id, status, best_score, attempts, updated_at::text, ",
            retry_at_sql!(),
            " FROM course_progress WHERE user_id = $1::uuid AND lesson_id = $2 FOR UPDATE"
        ))
        .bind(&auth_user.user_id)
        .bind(lesson_id)
        .bind(RETRY_WAIT_SECONDS)
        .fetch_one(&mut *tx)
        .await?;
        ((id, status, best, attempts, updated), retry)
    };

    if passed(lesson_id) {
        // Aula já concluída: nada a corrigir de novo; devolve o gabarito comentado.
        let (lesson_id, status, best_score, attempts, updated_at) = row;
        return Ok(Json(SaveProgressResponse {
            progress: CourseProgressItem {
                lesson_id,
                status,
                best_score,
                attempts,
                updated_at,
            },
            score: best_score,
            passed: true,
            retry_at: None,
            review: Some(lesson.review.clone()),
        }));
    }
    if let Some(video) = &lesson.video {
        if !course_watch::quiz_unlocked(&mut tx, &auth_user.user_id, lesson_id, video).await? {
            return Err(ApiError::Forbidden(
                "Assista ao vídeo (pelo menos 90%, sem pular) e responda às perguntas \
                 surpresa antes do quiz."
                    .to_string(),
            ));
        }
    }
    if retry_at.is_some() {
        return Err(ApiError::TooManyRequests(format!(
            "Nova tentativa liberada {} min depois da anterior. Use a espera para rever a aula.",
            RETRY_WAIT_SECONDS as i64 / 60
        )));
    }

    let grade = grade(lesson, &answers);
    let (lesson_id, status, best_score, attempts, updated_at, retry_at): (
        String,
        String,
        i16,
        i32,
        String,
        Option<String>,
    ) = sqlx::query_as(concat!(
        "UPDATE course_progress SET \
           status = CASE WHEN $4 THEN 'completed' ELSE status END, \
           best_score = GREATEST(best_score, $5), \
           attempts = attempts + 1, \
           last_failed_at = CASE WHEN $4 THEN NULL ELSE NOW() END,            attended_at = CASE WHEN $4 THEN COALESCE(attended_at, NOW()) ELSE attended_at END, \
           updated_at = NOW() \
         WHERE user_id = $1::uuid AND lesson_id = $2 \
         RETURNING lesson_id, status, best_score, attempts, updated_at::text, ",
        retry_at_sql!()
    ))
    .bind(&auth_user.user_id)
    .bind(lesson_id)
    .bind(RETRY_WAIT_SECONDS)
    .bind(grade.passed)
    .bind(grade.score)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(SaveProgressResponse {
        progress: CourseProgressItem {
            lesson_id,
            status,
            best_score,
            attempts,
            updated_at,
        },
        score: grade.score,
        passed: grade.passed,
        retry_at,
        review: grade.passed.then(|| lesson.review.clone()),
    }))
}

/// Texto da aula: leitura e transcrição da narração. O site publicado não traz
/// esse texto nas aulas só para alunos (`Frontend-Web/src/lib/coursePublicContent.ts`).
#[derive(Debug, Clone, Serialize)]
pub struct LessonMaterial {
    pub body: String,
    pub transcript: String,
}

/// GET /api/course/material?lesson_id= — texto da aula para quem entrou na conta.
pub async fn get_material(
    RequireAuth(_auth_user): RequireAuth,
    Query(query): Query<course_watch::LessonQuery>,
) -> Result<Json<LessonMaterial>, ApiError> {
    quiz_key()
        .get(query.lesson_id.trim())
        .map(|lesson| Json(lesson.material.clone()))
        .ok_or_else(|| ApiError::BadRequest("Aula inexistente.".to_string()))
}

// ─── Gabarito do quiz ───
//
// O conteúdo das aulas mora no site (`courseContent.json`); a API embute o
// mesmo arquivo na compilação, como já faz com `courseTraining.json`, para que
// gabarito, pré-requisitos e capítulos dos vídeos sejam sempre os da versão
// publicada do site. O site publica o arquivo sem o gabarito (`vite.config.ts`).

const COURSE_CONTENT: &str = include_str!("../../../Frontend-Web/src/data/courseContent.json");
/// Mesmo corte de `PASS_SCORE` em `Frontend-Web/src/lib/course.ts`.
pub(crate) const PASS_SCORE: i16 = 70;
/// Espera para nova tentativa depois de reprovar (15 min).
pub(crate) const RETRY_WAIT_SECONDS: f64 = 900.0;
const MAX_ANSWERS: usize = 64;
const MAX_ANSWER_LEN: usize = 200;

#[derive(Deserialize)]
struct RawContent {
    modules: Vec<RawModule>,
}

#[derive(Deserialize)]
struct RawModule {
    #[serde(default)]
    id: String,
    lessons: Vec<RawLesson>,
}

#[derive(Deserialize)]
struct RawLesson {
    id: String,
    #[serde(default)]
    body: String,
    prerequisites: Option<Vec<String>>,
    quiz: Vec<RawQuestion>,
    video: Option<RawVideo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawVideo {
    url: Option<String>,
    publication_status: Option<String>,
    duration_seconds: Option<f64>,
    #[serde(default)]
    chapters: Vec<RawChapter>,
    transcript: Option<String>,
    placeholder_script: Option<String>,
}

#[derive(Deserialize)]
struct RawChapter {
    start: f64,
    title: String,
}

#[derive(Deserialize)]
struct RawQuestion {
    id: String,
    kind: String,
    #[serde(default)]
    prompt: String,
    options: Vec<String>,
    #[serde(default)]
    hole: Vec<String>,
    #[serde(default)]
    board: Vec<String>,
    #[serde(rename = "expectedIndex")]
    expected_index: Option<usize>,
    expected: Option<String>,
    explanation: Option<String>,
    /// Critério do motor nas perguntas práticas.
    engine: Option<String>,
    why: Option<String>,
}

pub(crate) struct QuestionKey {
    pub(crate) id: String,
    pub(crate) engine: bool,
    pub(crate) correct: String,
    pub(crate) prompt: String,
    pub(crate) options: Vec<String>,
    pub(crate) hole: Vec<String>,
    pub(crate) board: Vec<String>,
}

/// Vídeo publicado da aula: endereço, duração e capítulos (início em segundos, título).
pub(crate) struct VideoKey {
    /// Muda a cada nova versão do vídeo; o progresso contado em outra versão não vale.
    pub(crate) url: String,
    pub(crate) duration: f64,
    pub(crate) chapters: Vec<(f64, String)>,
}

pub(crate) struct LessonKey {
    pub(crate) prerequisites: Vec<String>,
    pub(crate) questions: Vec<QuestionKey>,
    /// `None` = aula sem vídeo publicado (segue só pela leitura).
    pub(crate) video: Option<VideoKey>,
    pub(crate) review: Vec<ReviewItem>,
    pub(crate) material: LessonMaterial,
}

/// Uma linha do gabarito comentado, mostrado depois da aprovação.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewItem {
    pub id: String,
    pub prompt: String,
    pub answer: String,
    pub explanation: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Grade {
    score: i16,
    passed: bool,
}

pub(crate) fn quiz_key() -> &'static HashMap<String, LessonKey> {
    static KEY: OnceLock<HashMap<String, LessonKey>> = OnceLock::new();
    KEY.get_or_init(|| parse_key(COURSE_CONTENT))
}

/// Módulos fora do certificado (história do poker, opcional).
pub(crate) const OPTIONAL_MODULES: &[&str] = &["m0"];

/// Aulas exigidas para o certificado, na ordem do curso.
pub(crate) fn certificate_lessons() -> &'static [String] {
    static LESSONS: OnceLock<Vec<String>> = OnceLock::new();
    LESSONS.get_or_init(|| required_lessons(COURSE_CONTENT))
}

fn required_lessons(json: &str) -> Vec<String> {
    let content: RawContent =
        serde_json::from_str(json).expect("courseContent.json válido (conferido nos testes)");
    content
        .modules
        .into_iter()
        .filter(|m| !OPTIONAL_MODULES.contains(&m.id.as_str()))
        .flat_map(|m| m.lessons.into_iter().map(|l| l.id))
        .collect()
}

fn parse_key(json: &str) -> HashMap<String, LessonKey> {
    let content: RawContent =
        serde_json::from_str(json).expect("courseContent.json válido (conferido nos testes)");
    let mut previous: Option<String> = None;
    let mut key = HashMap::new();
    for lesson in content.modules.into_iter().flat_map(|m| m.lessons) {
        // Sem lista explícita, o pré-requisito é a aula anterior (como no site).
        let prerequisites = lesson
            .prerequisites
            .unwrap_or_else(|| previous.iter().cloned().collect());
        let mut review = Vec::new();
        let questions = lesson
            .quiz
            .into_iter()
            .map(|q| {
                let engine = q.kind == "engine";
                let correct = if engine {
                    q.expected.expect("pergunta engine com expected")
                } else {
                    q.options[q.expected_index.expect("pergunta theory com expectedIndex")].clone()
                };
                let explanation = if engine {
                    format!(
                        "Critério: {}. {}",
                        q.engine.unwrap_or_default(),
                        q.why.unwrap_or_default()
                    )
                } else {
                    q.explanation.unwrap_or_default()
                };
                review.push(ReviewItem {
                    id: q.id.clone(),
                    prompt: q.prompt.clone(),
                    answer: correct.clone(),
                    explanation,
                });
                QuestionKey {
                    id: q.id,
                    engine,
                    correct,
                    prompt: q.prompt,
                    options: q.options,
                    hole: q.hole,
                    board: q.board,
                }
            })
            .collect();
        let transcript = lesson
            .video
            .as_ref()
            .and_then(|v| {
                v.transcript
                    .clone()
                    .or_else(|| v.placeholder_script.clone())
            })
            .unwrap_or_default();
        let material = LessonMaterial {
            body: lesson.body,
            transcript: transcript.trim().to_string(),
        };
        let video = lesson.video.and_then(|v| {
            let url = v
                .url
                .filter(|_| v.publication_status.as_deref() != Some("review"))?;
            let duration = v.duration_seconds.filter(|d| d.is_finite() && *d > 0.0)?;
            Some(VideoKey {
                url,
                duration,
                chapters: v.chapters.into_iter().map(|c| (c.start, c.title)).collect(),
            })
        });
        previous = Some(lesson.id.clone());
        key.insert(
            lesson.id,
            LessonKey {
                prerequisites,
                questions,
                video,
                review,
                material,
            },
        );
    }
    key
}

/// Mesma conta de `calculateQuizScores` no site: com perguntas práticas
/// (engine), vale a nota delas; sem elas, a nota geral. Pergunta sem resposta
/// conta como erro.
fn grade(lesson: &LessonKey, answers: &HashMap<String, String>) -> Grade {
    let percent = |items: &[&QuestionKey]| -> i16 {
        if items.is_empty() {
            return 100;
        }
        let right = items
            .iter()
            .filter(|q| answers.get(&q.id) == Some(&q.correct))
            .count();
        // Arredondamento de Math.round para valores positivos (meio para cima).
        ((right * 200 + items.len()) / (items.len() * 2)) as i16
    };
    let all: Vec<&QuestionKey> = lesson.questions.iter().collect();
    let engine: Vec<&QuestionKey> = all.iter().copied().filter(|q| q.engine).collect();
    let score = if engine.is_empty() {
        percent(&all)
    } else {
        percent(&engine)
    };
    Grade {
        score,
        passed: score >= PASS_SCORE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answers_for(lesson: &LessonKey, right: usize) -> HashMap<String, String> {
        lesson
            .questions
            .iter()
            .enumerate()
            .map(|(i, q)| {
                let answer = if i < right {
                    q.correct.clone()
                } else {
                    "errada".to_string()
                };
                (q.id.clone(), answer)
            })
            .collect()
    }

    #[test]
    fn gabarito_cobre_todas_as_aulas_com_alternativas_validas() {
        let raw: serde_json::Value = serde_json::from_str(COURSE_CONTENT).unwrap();
        let total: usize = raw["modules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["lessons"].as_array().unwrap().len())
            .sum();
        let key = quiz_key();
        assert_eq!(key.len(), total, "id de aula repetido");
        for (id, lesson) in key {
            assert!(id.len() <= 32, "{id}: coluna lesson_id aceita 32");
            assert!(!lesson.questions.is_empty(), "{id} sem quiz");
            assert!(!lesson.material.body.is_empty(), "{id} sem texto");
            assert!(
                !lesson.material.transcript.is_empty(),
                "{id} sem transcrição"
            );
            for p in &lesson.prerequisites {
                assert!(key.contains_key(p), "{id}: pré-requisito {p} inexistente");
            }
            let all_right = answers_for(lesson, lesson.questions.len());
            let expected = Grade {
                score: 100,
                passed: true,
            };
            assert_eq!(grade(lesson, &all_right), expected, "{id}");
            let none = Grade {
                score: 0,
                passed: false,
            };
            assert_eq!(grade(lesson, &HashMap::new()), none, "{id}");
        }
    }

    #[test]
    fn nota_pratica_prevalece_e_arredonda_como_o_site() {
        let q = |id: &str, engine: bool| QuestionKey {
            id: id.to_string(),
            engine,
            correct: "c".to_string(),
            prompt: String::new(),
            options: Vec::new(),
            hole: Vec::new(),
            board: Vec::new(),
        };
        let pick = |ids: &[&str]| -> HashMap<String, String> {
            ids.iter()
                .map(|id| (id.to_string(), "c".to_string()))
                .collect()
        };
        let mixed = LessonKey {
            prerequisites: vec![],
            video: None,
            review: vec![],
            material: LessonMaterial {
                body: String::new(),
                transcript: String::new(),
            },
            questions: vec![q("t1", false), q("e1", true), q("e2", true), q("e3", true)],
        };
        // Teoria certa não compensa a prática: 1 de 3 práticas = 33.
        assert_eq!(grade(&mixed, &pick(&["t1", "e1"])).score, 33);
        // 2 de 3 = 66,67 → 67, abaixo do corte.
        let two = grade(&mixed, &pick(&["e1", "e2"]));
        assert_eq!((two.score, two.passed), (67, false));
        assert!(grade(&mixed, &pick(&["e1", "e2", "e3"])).passed);
        // Só teoria: 1 de 2 = 50.
        let theory = LessonKey {
            prerequisites: vec![],
            video: None,
            review: vec![],
            material: LessonMaterial {
                body: String::new(),
                transcript: String::new(),
            },
            questions: vec![q("t1", false), q("t2", false)],
        };
        assert_eq!(grade(&theory, &pick(&["t1"])).score, 50);
        // Meio para cima, como Math.round: 5 de 8 = 62,5 → 63; 7 de 8 = 87,5 → 88.
        let eight = LessonKey {
            prerequisites: vec![],
            video: None,
            review: vec![],
            material: LessonMaterial {
                body: String::new(),
                transcript: String::new(),
            },
            questions: (0..8).map(|i| q(&format!("t{i}"), false)).collect(),
        };
        assert_eq!(grade(&eight, &answers_for(&eight, 5)).score, 63);
        assert_eq!(grade(&eight, &answers_for(&eight, 7)).score, 88);
    }

    #[test]
    fn sem_lista_de_pre_requisito_vale_a_aula_anterior() {
        let json = r#"{"modules":[{"lessons":[
            {"id":"a","quiz":[{"id":"q","kind":"theory","options":["x","y"],"expectedIndex":1}]},
            {"id":"b","quiz":[{"id":"q","kind":"engine","options":["x","y"],"expected":"x"}]},
            {"id":"c","prerequisites":[],"quiz":[]}]}]}"#;
        let key = parse_key(json);
        assert!(key["a"].prerequisites.is_empty());
        assert_eq!(key["b"].prerequisites, vec!["a".to_string()]);
        assert!(key["c"].prerequisites.is_empty());
        assert_eq!(key["a"].questions[0].correct, "y");
        assert_eq!(key["b"].questions[0].correct, "x");
    }
}
