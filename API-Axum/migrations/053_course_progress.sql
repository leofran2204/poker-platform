-- 053: progresso do curso por usuário (Módulo 0 em diante).
-- Uma linha por (usuário, aula): status, melhor nota de quiz (0–100) e tentativas.

CREATE TABLE IF NOT EXISTS course_progress (
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lesson_id  VARCHAR(32) NOT NULL,
    status     VARCHAR(16) NOT NULL DEFAULT 'started'
        CHECK (status IN ('started', 'completed')),
    best_score SMALLINT NOT NULL DEFAULT 0
        CHECK (best_score BETWEEN 0 AND 100),
    attempts   INTEGER NOT NULL DEFAULT 0
        CHECK (attempts >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, lesson_id)
);

CREATE INDEX IF NOT EXISTS idx_course_progress_user
    ON course_progress(user_id);
