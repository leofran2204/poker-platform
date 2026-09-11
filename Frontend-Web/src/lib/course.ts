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

export interface CourseLesson {
  id: string;
  title: string;
  minutes: number;
  body: string;
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
