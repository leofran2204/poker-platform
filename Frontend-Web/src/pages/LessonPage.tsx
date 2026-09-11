import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { saveCourseProgress } from "@/api/client";
import { CourseQuiz } from "@/components/CourseQuiz";
import { TipRichText } from "@/components/TipRichText";
import { allLessons, findLesson, moduleOfLesson } from "@/lib/course";
import { isAuthenticated } from "@/lib/auth";

export function LessonPage() {
  const { lessonId } = useParams();
  const lesson = lessonId ? findLesson(lessonId) : undefined;
  const [msg, setMsg] = useState<string | null>(null);
  const [lastScore, setLastScore] = useState<number | null>(null);
  const authed = isAuthenticated();

  const persist = useCallback(
    async (completed: boolean, score: number) => {
      if (!authed || !lesson) return;
      try {
        const saved = await saveCourseProgress(lesson.id, completed, score);
        setMsg(
          saved.status === "completed"
            ? `Progresso salvo: concluída (melhor nota ${saved.best_score}%).`
            : `Progresso salvo: melhor nota ${saved.best_score}%.`,
        );
      } catch {
        setMsg("Não foi preciso salvar agora — tente de novo logado.");
      }
    },
    [authed, lesson],
  );

  useEffect(() => {
    setMsg(null);
    setLastScore(null);
  }, [lessonId]);

  if (!lesson) {
    return (
      <div className="space-y-4">
        <p className="text-sm text-red-200">Aula não encontrada.</p>
        <Link to="/curso" className="zt-btn-secondary !text-xs">
          Voltar ao curso
        </Link>
      </div>
    );
  }

  const lessons = allLessons();
  const idx = lessons.findIndex((l) => l.id === lesson.id);
  const prev = idx > 0 ? lessons[idx - 1] : undefined;
  const next = idx >= 0 && idx < lessons.length - 1 ? lessons[idx + 1] : undefined;
  const mod = moduleOfLesson(lesson.id);

  return (
    <div className="mx-auto w-full max-w-3xl space-y-4">
      <Link to="/curso" className="text-xs text-gold-soft">
        ← {mod?.title ?? "Curso"}
      </Link>
      <h1 className="text-xl font-bold text-gold-bright">{lesson.title}</h1>
      <p className="text-xs text-felt-400">~{lesson.minutes} min de leitura + quiz</p>
      {msg && <p className="text-sm text-emerald-200">{msg}</p>}
      {!authed && (
        <p className="rounded border border-felt-700 bg-felt-800 px-3 py-2 text-xs text-felt-200">
          Você está lendo como visitante. <Link to="/login" className="text-gold-soft">Entre</Link> para
          salvar progresso e valer nota no quiz.
        </p>
      )}

      <div className="zt-panel space-y-3 p-4">
        <TipRichText text={lesson.body} className="space-y-3 text-sm text-felt-100" />
      </div>

      <CourseQuiz
        questions={lesson.quiz}
        onFinish={(score) => {
          setLastScore(score);
          void persist(score >= 60, score);
        }}
      />

      {authed && (
        <button
          type="button"
          className="zt-btn-secondary !text-xs"
          onClick={() => void persist(true, lastScore ?? 0)}
        >
          Marcar como concluída
        </button>
      )}

      <div className="flex justify-between gap-2 pt-2">
        {prev ? (
          <Link to={`/curso/${prev.id}`} className="zt-btn-secondary !text-xs">
            ← {prev.title}
          </Link>
        ) : (
          <span />
        )}
        {next ? (
          <Link to={`/curso/${next.id}`} className="zt-btn-secondary !text-xs">
            {next.title} →
          </Link>
        ) : (
          <Link to="/curso" className="zt-btn-primary !text-xs">
            Concluir trilha
          </Link>
        )}
      </div>
    </div>
  );
}
