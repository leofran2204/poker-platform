/**
 * Campos do gabarito em `courseContent.json`. A API corrige o quiz com eles
 * (o arquivo é embutido na compilação dela); o site publicado não os recebe:
 * `vite.config.ts` passa o arquivo por `stripCourseAnswerKey` antes do pacote.
 */
export const ANSWER_KEY_FIELDS = ["expectedIndex", "explanation", "expected", "engine", "why"] as const;

type QuizLike = Record<string, unknown>;
type ContentLike = { modules?: { lessons?: { quiz?: QuizLike[] }[] }[] };

export function stripCourseAnswerKey<T>(content: T): T {
  const copy = JSON.parse(JSON.stringify(content)) as ContentLike;
  for (const courseModule of copy.modules ?? []) {
    for (const lesson of courseModule.lessons ?? []) {
      for (const question of lesson.quiz ?? []) {
        for (const field of ANSWER_KEY_FIELDS) delete question[field];
      }
    }
  }
  return copy as T;
}
