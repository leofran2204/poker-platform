import courseData from "@/data/courseContent.json";

export interface CourseQuizQuestionBase {
  id: string;
  prompt: string;
}

export interface CourseTheoryQuestion extends CourseQuizQuestionBase {
  kind: "theory";
  options: string[];
}

export interface CourseEngineQuestion extends CourseQuizQuestionBase {
  kind: "engine";
  hole: string[];
  board: string[];
  options: string[];
}

/** Pergunta como o site a recebe: sem gabarito (a API corrige). */
export type CourseQuizQuestion = CourseTheoryQuestion | CourseEngineQuestion;

/**
 * Pergunta com o gabarito editorial. Só existe no arquivo `courseContent.json`
 * (testes e API); o site publicado não recebe estes campos (`vite.config.ts`).
 */
export type EditorialQuizQuestion =
  | (CourseTheoryQuestion & { expectedIndex: number; explanation: string })
  | (CourseEngineQuestion & { expected: string; engine: string; why: string });

export interface CourseVideoInfo {
  contentHash?: string;
  rendererVersion?: number;
  /** "remotion": produzido em ZeroTiltCurso/editorial/motion (manifesto com roteiro e narração). */
  renderer?: "remotion";
  pronunciationVersion?: string;
  narrationHash?: string;
  mediaRevision?: string;
  publicationStatus?: "published" | "review" | "prior_rules";
  bettingRuleVersion?: string;
  url?: string;
  durationSeconds?: number;
  hostName?: string;
  hostRole?: string;
  hostAvatarUrl?: string;
  placeholderScript?: string;
  /** Narração real do MP4 (não o gancho do coach). */
  transcript?: string;
  captionsUrl?: string;
  posterUrl?: string;
  chapters?: { start: number; title: string }[];
}

export { formatLessonDuration } from "./courseDuration";

export interface CourseHandExample {
  title: string;
  heroPosition: string;
  villainPosition: string;
  heroCards: string[];
  boardCards: string[];
  potBb: string;
  actionText: string;
  heroDecision: string;
  keyTakeaway: string;
}

export interface CourseLesson {
  id: string;
  prerequisites?: string[];
  variant?: "holdem" | "short_deck" | "omaha" | "pineapple_draw";
  title: string;
  minutes: number;
  body: string;
  /** Chaves bibliográficas de courseSources.json. */
  sources?: string[];
  video?: CourseVideoInfo;
  handExample?: CourseHandExample;
  quiz: CourseQuizQuestion[];
}

export interface CourseModule {
  id: string;
  title: string;
  subtitle: string;
  lessons: CourseLesson[];
}

export interface CourseContent {
  version: number;
  modules: CourseModule[];
}

export const COURSE: CourseContent = courseData as CourseContent;

export const PASS_SCORE = 70;

export function allLessons(): CourseLesson[] {
  return COURSE.modules.flatMap((m) => m.lessons);
}

export function findLesson(lessonId: string): CourseLesson | undefined {
  return allLessons().find((l) => l.id === lessonId);
}

export function moduleOfLesson(lessonId: string): CourseModule | undefined {
  return COURSE.modules.find((m) => m.lessons.some((l) => l.id === lessonId));
}

export function isLessonUnlocked(
  lessonId: string,
  progress: Record<string, { status: string; best_score: number }>,
): boolean {
  if (!findLesson(lessonId)) return false;
  const passed = (id: string) => progress[id]?.status === "completed" || (progress[id]?.best_score ?? 0) >= PASS_SCORE;
  // Uma reorganização editorial nunca retira acesso a uma aula já concluída.
  return passed(lessonId) || lessonPrerequisites(lessonId).every(passed);
}

export function lessonPrerequisites(lessonId: string): string[] {
  const lessons = allLessons();
  const idx = lessons.findIndex(l => l.id === lessonId);
  if (idx < 0) return [];
  return lessons[idx].prerequisites ?? (idx > 0 ? [lessons[idx - 1].id] : []);
}

export function lessonFormat(lesson: CourseLesson): string {
  return lesson.video?.url && lesson.video.publicationStatus !== "review"
    ? "Vídeo • leitura • quiz"
    : "Leitura • quiz";
}
