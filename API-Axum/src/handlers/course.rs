//! Course progress — aulas concluídas e melhor nota de quiz por usuário.
//! A nota é calculada aqui, a partir das respostas, com o gabarito do servidor.

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

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

/// POST /api/course/progress — corrige o quiz pelo gabarito do servidor e
/// registra conclusão e melhor nota da aula.
pub async fn save_progress(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<SaveProgressBody>,
) -> Result<Json<CourseProgressItem>, ApiError> {
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
    .fetch_all(&state.db)
    .await?;
    let passed = |id: &str| passed_rows.iter().any(|p| p == id);
    // Uma reorganização editorial nunca retira acesso a uma aula já concluída.
    if !passed(lesson_id) && !lesson.prerequisites.iter().all(|p| passed(p)) {
        return Err(ApiError::Forbidden(
            "Conclua o pré-requisito antes desta aula.".to_string(),
        ));
    }

    let grade = grade(lesson, &answers);
    let status = if grade.passed { "completed" } else { "started" };
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
    .bind(grade.score)
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

// ─── Gabarito do quiz ───
//
// O conteúdo das aulas mora no site (`courseContent.json`); a API embute o
// mesmo arquivo na compilação, como já faz com `courseTraining.json`, para que
// gabarito e pré-requisitos sejam sempre os da versão publicada do site.

const COURSE_CONTENT: &str = include_str!("../../../Frontend-Web/src/data/courseContent.json");
/// Mesmo corte de `PASS_SCORE` em `Frontend-Web/src/lib/course.ts`.
const PASS_SCORE: i16 = 70;
const MAX_ANSWERS: usize = 64;
const MAX_ANSWER_LEN: usize = 200;

#[derive(Deserialize)]
struct RawContent {
    modules: Vec<RawModule>,
}

#[derive(Deserialize)]
struct RawModule {
    lessons: Vec<RawLesson>,
}

#[derive(Deserialize)]
struct RawLesson {
    id: String,
    prerequisites: Option<Vec<String>>,
    quiz: Vec<RawQuestion>,
}

#[derive(Deserialize)]
struct RawQuestion {
    id: String,
    kind: String,
    options: Vec<String>,
    #[serde(rename = "expectedIndex")]
    expected_index: Option<usize>,
    expected: Option<String>,
}

struct QuestionKey {
    id: String,
    engine: bool,
    correct: String,
}

struct LessonKey {
    prerequisites: Vec<String>,
    questions: Vec<QuestionKey>,
}

#[derive(Debug, PartialEq, Eq)]
struct Grade {
    score: i16,
    passed: bool,
}

fn quiz_key() -> &'static HashMap<String, LessonKey> {
    static KEY: OnceLock<HashMap<String, LessonKey>> = OnceLock::new();
    KEY.get_or_init(|| parse_key(COURSE_CONTENT))
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
                QuestionKey {
                    id: q.id,
                    engine,
                    correct,
                }
            })
            .collect();
        previous = Some(lesson.id.clone());
        key.insert(
            lesson.id,
            LessonKey {
                prerequisites,
                questions,
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
        };
        let pick = |ids: &[&str]| -> HashMap<String, String> {
            ids.iter()
                .map(|id| (id.to_string(), "c".to_string()))
                .collect()
        };
        let mixed = LessonKey {
            prerequisites: vec![],
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
            questions: vec![q("t1", false), q("t2", false)],
        };
        assert_eq!(grade(&theory, &pick(&["t1"])).score, 50);
        // Meio para cima, como Math.round: 5 de 8 = 62,5 → 63; 7 de 8 = 87,5 → 88.
        let eight = LessonKey {
            prerequisites: vec![],
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
