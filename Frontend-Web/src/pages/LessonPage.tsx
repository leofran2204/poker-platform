import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { fetchCourseProgress, saveCourseProgress, type CourseProgressItem } from "@/api/client";
import { CourseQuiz } from "@/components/CourseQuiz";
import { CourseVideoPlayer } from "@/components/CourseVideoPlayer";
import { TipRichText } from "@/components/TipRichText";
import { allLessons, findLesson, isLessonUnlocked, moduleOfLesson, PASS_SCORE } from "@/lib/course";
import { isAuthenticated } from "@/lib/auth";

export function LessonPage() {
  const { lessonId } = useParams();
  const lesson = lessonId ? findLesson(lessonId) : undefined;
  const [msg, setMsg] = useState<string | null>(null);
  const [lastScore, setLastScore] = useState<number | null>(null);
  const [progress, setProgress] = useState<Record<string, CourseProgressItem>>({});
  const [sessionPassed, setSessionPassed] = useState(false);
  const authed = isAuthenticated();

  const loadProgress = useCallback(async () => {
    if (!authed) return;
    try {
      const items = await fetchCourseProgress();
      const map: Record<string, CourseProgressItem> = {};
      for (const it of items) map[it.lesson_id] = it;
      setProgress(map);
    } catch {
      // Offline ou visitante
    }
  }, [authed]);

  useEffect(() => {
    void loadProgress();
  }, [loadProgress]);

  const persist = useCallback(
    async (completed: boolean, score: number) => {
      if (!authed || !lesson) return;
      try {
        const saved = await saveCourseProgress(lesson.id, completed, score);
        setProgress((prev) => ({ ...prev, [lesson.id]: saved }));
        setMsg(
          saved.status === "completed"
            ? `Progresso salvo: concluída (melhor nota ${saved.best_score}%). Próxima aula desbloqueada!`
            : `Progresso salvo: nota ${saved.best_score}%. Requer ${PASS_SCORE}% para liberar a próxima aula.`,
        );
      } catch {
        setMsg("Não foi possível salvar agora — tente de novo logado.");
      }
    },
    [authed, lesson],
  );

  useEffect(() => {
    setMsg(null);
    setLastScore(null);
    setSessionPassed(false);
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

  const currentProg = progress[lesson.id];
  const isPassed =
    sessionPassed ||
    currentProg?.status === "completed" ||
    (currentProg?.best_score ?? 0) >= PASS_SCORE;
  const unlocked = isLessonUnlocked(lesson.id, progress);

  return (
    <div className="mx-auto w-full max-w-3xl space-y-4">
      <Link to="/curso" className="text-xs text-gold-soft">
        ← {mod?.title ?? "Curso"}
      </Link>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h1 className="text-xl font-bold text-gold-bright">{lesson.title}</h1>
          <p className="text-xs text-felt-400">~{lesson.minutes} min • Vídeo com Coach IA + Leitura + Quiz</p>
        </div>
        {isPassed && (
          <span className="rounded bg-emerald-500/20 border border-emerald-500/40 px-2.5 py-1 text-xs font-semibold text-emerald-300">
            ✓ Aula Concluída ({currentProg?.best_score ?? lastScore ?? 100}%)
          </span>
        )}
      </div>

      {msg && <p className="text-sm text-emerald-200 rounded bg-felt-900/80 p-2 border border-felt-700">{msg}</p>}

      {!unlocked ? (
        <div className="zt-panel space-y-3 p-4 text-center">
          <p className="text-3xl" aria-hidden>
            🔒
          </p>
          <p className="text-sm font-semibold text-cream">Aula bloqueada</p>
          <p className="text-xs text-felt-300">
            {!authed
              ? "Entre na sua conta e conclua as aulas anteriores para liberar esta."
              : `Atinja ${PASS_SCORE}% na aula anterior (${prev?.title ?? "—"}) para liberar esta.`}
          </p>
          {prev ? (
            <Link to={`/curso/${prev.id}`} className="zt-btn-primary !text-xs inline-block">
              Ir para {prev.title} →
            </Link>
          ) : (
            <Link to="/curso" className="zt-btn-secondary !text-xs inline-block">
              Voltar ao curso
            </Link>
          )}
        </div>
      ) : (
        <>
          {!authed && (
            <p className="rounded border border-felt-700 bg-felt-800 px-3 py-2 text-xs text-felt-200">
              Você está lendo como visitante. <Link to="/login" className="text-gold-soft">Entre</Link> para
              salvar progresso e valer nota para desbloquear as próximas aulas.
            </p>
          )}

          {/* Player de Vídeo IA e Mão Prática */}
          <CourseVideoPlayer
            video={lesson.video}
            handExample={lesson.handExample}
            lessonTitle={lesson.title}
          />

          {/* Conteúdo Textual Estruturado */}
          <div className="zt-panel space-y-3 p-4">
            <h2 className="text-xs font-mono uppercase tracking-wider text-gold-soft">
              Material Didático de Apoio
            </h2>
            <TipRichText text={lesson.body} className="space-y-3 text-sm text-felt-100" />
          </div>

          {/* Quiz de Fixação Bloqueante */}
          <CourseQuiz
            questions={lesson.quiz}
            onFinish={(score, passed) => {
              setLastScore(score);
              if (passed) setSessionPassed(true);
              void persist(passed, score);
            }}
          />
        </>
      )}

      {/* Navegação entre Aulas com Trava de Fixação */}
      <div className="flex flex-wrap items-center justify-between gap-3 pt-4 border-t border-felt-800">
        {prev ? (
          <Link to={`/curso/${prev.id}`} className="zt-btn-secondary !text-xs">
            ← {prev.title}
          </Link>
        ) : (
          <span />
        )}

        {next ? (
          isPassed ? (
            <Link
              to={`/curso/${next.id}`}
              className="zt-btn-primary !text-xs animate-pulse shadow-lg shadow-gold-bright/10"
            >
              Próxima Aula: {next.title} →
            </Link>
          ) : (
            <button
              type="button"
              disabled
              title={`Atinja pelo menos ${PASS_SCORE}% no teste de fixação acima para desbloquear`}
              className="zt-btn-secondary !text-xs opacity-50 cursor-not-allowed flex items-center gap-1.5"
            >
              <span>🔒</span> {next.title} (Requer {PASS_SCORE}%)
            </button>
          )
        ) : (
          <Link to="/curso" className="zt-btn-primary !text-xs">
            🎉 Concluir Trilha
          </Link>
        )}
      </div>
    </div>
  );
}
