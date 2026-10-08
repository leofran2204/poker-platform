import courseData from "@/data/courseContent.json";

export interface CourseQuizQuestionBase {
  id: string;
  prompt: string;
}

export interface CourseTheoryQuestion extends CourseQuizQuestionBase {
  kind: "theory";
  options: string[];
  expectedIndex: number;
  explanation: string;
}

export interface CourseEngineQuestion extends CourseQuizQuestionBase {
  kind: "engine";
  hole: string[];
  board: string[];
  options: string[];
  expected: string;
  engine: string;
  why: string;
}

export type CourseQuizQuestion = CourseTheoryQuestion | CourseEngineQuestion;

export interface CourseVideoInfo {
  contentHash?: string;
  rendererVersion?: number;
  pronunciationVersion?: string;
  narrationHash?: string;
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
  variant?: "holdem" | "short_deck" | "omaha" | "brazilian_pineapple";
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

export function isQuestionCorrect(q: CourseQuizQuestion, answer: string): boolean {
  if (q.kind === "theory") return q.options.indexOf(answer) === q.expectedIndex;
  return answer === q.expected;
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

export function calculateQuizScores(
  questions: CourseQuizQuestion[],
  answers: Record<string, string>,
): { totalScore: number; engineScore: number; hasEngineQuestions: boolean; passed: boolean } {
  if (questions.length === 0) {
    return { totalScore: 100, engineScore: 100, hasEngineQuestions: false, passed: true };
  }

  const answeredQuestions = questions.filter((q) => answers[q.id] !== undefined);
  const totalCorrect = answeredQuestions.filter((q) => isQuestionCorrect(q, answers[q.id])).length;
  const totalScore = Math.round((totalCorrect / questions.length) * 100);

  const engineQuestions = questions.filter((q) => q.kind === "engine");
  const hasEngineQuestions = engineQuestions.length > 0;
  let engineScore = 100;

  if (hasEngineQuestions) {
    const engineCorrect = engineQuestions.filter(
      (q) => answers[q.id] !== undefined && isQuestionCorrect(q, answers[q.id]),
    ).length;
    engineScore = Math.round((engineCorrect / engineQuestions.length) * 100);
  }

  // Regra aprovada: 70% nos exercícios práticos (gabarito editorial) ou 70% geral.
  const passed = hasEngineQuestions ? engineScore >= PASS_SCORE : totalScore >= PASS_SCORE;

  return { totalScore, engineScore, hasEngineQuestions, passed };
}
