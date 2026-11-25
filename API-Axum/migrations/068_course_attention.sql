-- 068: aula cumprida de verdade (etapa 6a do plano de 24/11).
-- Espera depois de reprovar no quiz e tempo de vídeo contado pelo servidor,
-- com perguntas surpresa em momentos sorteados do vídeo.

ALTER TABLE course_progress ADD COLUMN IF NOT EXISTS last_failed_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS course_watch (
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lesson_id       VARCHAR(32) NOT NULL,
    -- Um bit por segundo de vídeo assistido sem pular, com a aba visível e até 2x.
    watched         BYTEA NOT NULL DEFAULT ''::bytea,
    last_position   DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_beat_at    TIMESTAMPTZ,
    -- Momentos (segundo do vídeo) das perguntas surpresa ainda não acertadas.
    checkpoints     DOUBLE PRECISION[] NOT NULL DEFAULT '{}',
    -- Início do trecho coberto pela próxima pergunta (momento da última acertada).
    stretch_from    DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Pergunta surpresa aberta (no máximo uma por aula).
    check_id        UUID,
    check_at        DOUBLE PRECISION,
    check_answer    TEXT,
    check_prompt    TEXT,
    check_options   TEXT[],
    check_issued_at TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, lesson_id)
);
