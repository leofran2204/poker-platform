-- 069: prova final e certificado da Zero Tilt Academy (etapa 6b do plano de 24/11).
-- O certificado sai só pelo servidor, com todas as aulas cumpridas pelas regras
-- da 068 e a prova final aprovada; o código é público para verificação.

-- Quiz aprovado com o vídeo cumprido (90%, perguntas surpresa). Aulas aprovadas
-- antes da 068 ficam sem data até o vídeo ser assistido.
ALTER TABLE course_progress ADD COLUMN IF NOT EXISTS attended_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS course_exam_attempts (
    id            UUID PRIMARY KEY,
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deadline      TIMESTAMPTZ NOT NULL,
    -- Perguntas sorteadas (ids do courseContent.json); o gabarito fica no servidor.
    question_ids  TEXT[] NOT NULL,
    holder_name   VARCHAR(160) NOT NULL,
    name_source   VARCHAR(12) NOT NULL CHECK (name_source IN ('kyc', 'declared')),
    finished_at   TIMESTAMPTZ,
    score         SMALLINT,
    passed        BOOLEAN,
    timed_out     BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS course_exam_attempts_user_idx
    ON course_exam_attempts (user_id, started_at DESC);
-- No máximo uma prova aberta por aluno.
CREATE UNIQUE INDEX IF NOT EXISTS course_exam_attempts_open_idx
    ON course_exam_attempts (user_id) WHERE finished_at IS NULL;

CREATE TABLE IF NOT EXISTS course_certificates (
    code            VARCHAR(20) PRIMARY KEY,
    user_id         UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    attempt_id      UUID NOT NULL REFERENCES course_exam_attempts(id) ON DELETE CASCADE,
    holder_name     VARCHAR(160) NOT NULL,
    name_source     VARCHAR(12) NOT NULL CHECK (name_source IN ('kyc', 'declared')),
    score           SMALLINT NOT NULL,
    lessons         SMALLINT NOT NULL,
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ,
    revoked_reason  TEXT
);
