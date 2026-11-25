/**
 * O que o site publicado recebe de `courseContent.json`. O arquivo completo
 * fica com a API (embutido na compilação dela) e com os testes; `vite.config.ts`
 * passa o arquivo por `publicCourseContent` antes de montar o pacote.
 */

/** Campos do gabarito: a API corrige o quiz com eles. */
export const ANSWER_KEY_FIELDS = ["expectedIndex", "explanation", "expected", "engine", "why"] as const;

/**
 * Aulas só para quem entrou na conta: vídeo, legenda e texto. Visitante assiste
 * só aos vídeos de apresentação (episódios de história e filmes do Brazilian
 * Pineapple). O Caddy aplica a mesma regra aos arquivos
 * (`Infraestrutura-Docker/Caddyfile`, `@course_media`).
 */
export const MEMBERS_ONLY_VIDEO = /^\/videos\/zt-academy-[^/]+\.(mp4|vtt)$/;

/** Texto das aulas só para alunos; vem de `GET /api/course/material`. */
export const MEMBERS_ONLY_LESSON_FIELDS = ["body", "handExample"] as const;
export const MEMBERS_ONLY_VIDEO_FIELDS = ["transcript", "placeholderScript"] as const;

type Loose = Record<string, unknown>;
type ContentLike = {
  modules?: { lessons?: (Loose & { quiz?: Loose[]; video?: Loose & { url?: string } })[] }[];
};

export function publicCourseContent<T>(content: T): T {
  const copy = JSON.parse(JSON.stringify(content)) as ContentLike;
  for (const courseModule of copy.modules ?? []) {
    for (const lesson of courseModule.lessons ?? []) {
      for (const question of lesson.quiz ?? []) {
        for (const field of ANSWER_KEY_FIELDS) delete question[field];
      }
      if (!MEMBERS_ONLY_VIDEO.test(lesson.video?.url ?? "")) continue;
      for (const field of MEMBERS_ONLY_LESSON_FIELDS) delete lesson[field];
      for (const field of MEMBERS_ONLY_VIDEO_FIELDS) delete lesson.video?.[field];
    }
  }
  return copy as T;
}
