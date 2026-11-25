import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router";
import { fetchCourseMaterial, fetchCourseProgress, saveCourseProgress, type CourseLessonState, type CourseProgressItem } from "@/api/client";
import { CourseQuiz, CourseReview } from "@/components/CourseQuiz";
import { CourseSources } from "@/components/CourseSources";
import { CourseSurpriseCheck } from "@/components/CourseSurpriseCheck";
import { CourseVideoPlayer } from "@/components/CourseVideoPlayer";
import { TipRichText } from "@/components/TipRichText";
import { allLessons, findLesson, isLessonUnlocked, isMembersOnlyVideo, lessonFormat, lessonPrerequisites, moduleOfLesson, PASS_SCORE } from "@/lib/course";
import { isAuthenticated } from "@/lib/auth";
import { TRAINING } from "@/lib/academy";
import { clock, MAX_COUNTED_RATE, useLessonWatch } from "@/lib/useLessonWatch";

/** Quiz fechado até o vídeo ser assistido: quanto falta e como conta. */
function WatchGate({ state }: { state: CourseLessonState }) {
  const percent = Math.min(100, Math.round((state.watched_percent / state.required_percent) * 100));
  return (
    <div className="zt-panel space-y-2 p-4">
      <div className="zt-panel-title">Quiz liberado depois do vídeo</div>
      <div className="h-2 w-full overflow-hidden rounded-full bg-felt-800" role="progressbar" aria-valuemin={0} aria-valuemax={state.required_percent} aria-valuenow={state.watched_percent} aria-label="Vídeo assistido">
        <div className="h-full bg-gold-bright" style={{ width: `${percent}%` }} />
      </div>
      <p className="text-sm text-cream">
        Vídeo assistido: {state.watched_percent}% de {state.required_percent}% necessários.
      </p>
      <p className="text-xs text-felt-300">
        Conta o vídeo tocando com esta aba visível, sem pular trechos e em até {MAX_COUNTED_RATE}x.
        {state.checks_left > 0 &&
          ` Durante o vídeo ${state.checks_left === 1 ? "aparece 1 pergunta surpresa" : `aparecem ${state.checks_left} perguntas surpresa`} sobre o trecho que acabou de passar.`}
      </p>
      {state.missing_check_near !== null && (
        <p className="text-xs text-amber-300">Falta assistir o trecho perto de {clock(state.missing_check_near)}.</p>
      )}
    </div>
  );
}

export function LessonPage() {
  const { lessonId } = useParams();
  const lesson = lessonId ? findLesson(lessonId) : undefined;
  const [lastScore, setLastScore] = useState<number | null>(null);
  const [progress, setProgress] = useState<Record<string, CourseProgressItem>>({});
  const [sessionPassed, setSessionPassed] = useState(false);
  const authed = isAuthenticated();
  const watch = useLessonWatch(lesson?.id ?? "", authed && lesson !== undefined);

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

  // Texto das aulas de aluno: vem da API (o pacote do site não o traz).
  const [material, setMaterial] = useState<{ lessonId: string; body: string; transcript: string } | null>(null);
  const [materialFailed, setMaterialFailed] = useState(false);
  const needsMaterial = authed && lesson !== undefined && lesson.body === undefined;
  useEffect(() => {
    if (!needsMaterial || !lesson) return;
    let alive = true;
    setMaterialFailed(false);
    fetchCourseMaterial(lesson.id)
      .then((loaded) => alive && setMaterial({ lessonId: lesson.id, ...loaded }))
      .catch(() => alive && setMaterialFailed(true));
    return () => {
      alive = false;
    };
  }, [needsMaterial, lesson]);
  const lessonMaterial = material && material.lessonId === lesson?.id ? material : null;
  const lessonBody = lesson?.body ?? lessonMaterial?.body;

  const submitQuiz = useCallback(
    async (answers: Record<string, string>) => {
      if (!lesson) throw new Error("Aula não encontrada.");
      const saved = await saveCourseProgress(lesson.id, answers);
      setProgress((prev) => ({ ...prev, [lesson.id]: saved }));
      if (saved.passed) {
        setLastScore(saved.score);
        setSessionPassed(true);
        void watch.refresh();
      }
      return saved;
    },
    [lesson, watch],
  );

  useEffect(() => {
    setLastScore(null);
    setSessionPassed(false);
  }, [lessonId]);

  if (!lesson) {
    return (
      <div className="space-y-4">
        <p className="text-sm text-red-200">Aula não encontrada.</p>
        <Link to="/curso" className="zt-btn-secondary text-xs!">
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
  const missingPrerequisite = lessonPrerequisites(lesson.id).map(findLesson).find(p => p && progress[p.id]?.status !== "completed" && (progress[p.id]?.best_score ?? 0) < PASS_SCORE);

  const currentProg = progress[lesson.id];
  const isPassed =
    sessionPassed ||
    currentProg?.status === "completed" ||
    (currentProg?.best_score ?? 0) >= PASS_SCORE;
  const unlocked = isLessonUnlocked(lesson.id, progress);

  return (
    <div className="mx-auto w-full max-w-4xl space-y-4">
      <Link to="/curso" className="block text-xs leading-6 text-gold-soft">
        ← {mod?.title ?? "Curso"}
      </Link>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h1 className="zt-lesson-title">{lesson.title}</h1>
          <p className="text-xs text-felt-400">{lessonFormat(lesson)}</p>
        </div>
        {isPassed && (
          <span className="rounded-sm bg-emerald-500/20 border border-emerald-500/40 px-2.5 py-1 text-xs font-semibold text-emerald-300">
            ✓ Aula Concluída ({currentProg?.best_score ?? lastScore ?? 100}%)
          </span>
        )}
      </div>

      {lesson.video?.publicationStatus === "prior_rules" && <div className="zt-panel p-4 text-sm">Vídeo da regra anterior de apostas. Para jogar hoje: todas as rodadas limitam o aumento ao pote antes do call; sem ante; no Brazilian Pineapple, a troca e o descarte vêm na vez de cada jogador, antes da aposta. As regras de cartas continuam válidas. A leitura e a mesa de treino abaixo já seguem a estrutura vigente.</div>}

      {!unlocked ? (
        <div className="zt-panel space-y-3 p-4 text-center">
          <p className="text-3xl" aria-hidden>
            🔒
          </p>
          <p className="text-sm font-semibold text-cream">Aula bloqueada</p>
          <p className="text-xs text-felt-300">
            {!authed
              ? "Entre na sua conta e conclua as aulas anteriores para liberar esta."
              : `Atinja ${PASS_SCORE}% no pré-requisito: ${missingPrerequisite?.title ?? "aula de fundamentos"}.`}
          </p>
          {missingPrerequisite ? (
            <Link to={`/curso/${missingPrerequisite.id}`} className="zt-btn-primary text-xs! inline-block">
              Ir para {missingPrerequisite.title} →
            </Link>
          ) : (
            <Link to="/curso" className="zt-btn-secondary text-xs! inline-block">
              Voltar ao curso
            </Link>
          )}
        </div>
      ) : !authed && isMembersOnlyVideo(lesson) ? (
        <div className="zt-panel space-y-3 border border-gold-soft/30 p-5 text-center">
          <p className="text-sm font-semibold text-cream">Aula para alunos com conta</p>
          <p className="text-xs text-felt-300">
            Sem conta, dá para assistir aos vídeos de apresentação. Crie sua conta grátis para ver as aulas em vídeo,
            ler o material, fazer o quiz e acompanhar o progresso.
          </p>
          <div className="flex flex-wrap justify-center gap-2">
            <Link to="/register" className="zt-btn-primary text-xs!">Criar conta</Link>
            <Link to="/login" className="zt-btn-secondary text-xs!">Entrar</Link>
          </div>
        </div>
      ) : (
        <>
          {!authed && (
            <p className="rounded-sm border border-felt-700 bg-felt-800 px-3 py-2 text-xs text-felt-200">
              Você está lendo como visitante. <Link to="/login" className="text-gold-soft">Entre</Link> para
              fazer o quiz, salvar o progresso e desbloquear as próximas aulas.
            </p>
          )}

          {/* Player de Vídeo da Aula */}
          <CourseVideoPlayer
            key={`video-${lesson.id}`}
            video={lesson.video}
            lessonTitle={lesson.title}
            playerRef={watch.videoRef}
            hideChapters={watch.state?.pending_check != null}
            transcript={lessonMaterial?.transcript}
          >
            {watch.state?.pending_check && (
              <CourseSurpriseCheck
                key={watch.state.pending_check.id}
                check={watch.state.pending_check}
                onAnswer={(option) => void watch.answer(watch.state!.pending_check!.id, option)}
                onTimeout={() => {
                  watch.videoRef.current?.pause();
                  void watch.answer(watch.state!.pending_check!.id, null);
                }}
              />
            )}
            {watch.notice && (
              <p role="status" className="flex items-start justify-between gap-2 border-t border-felt-800 bg-felt-900 px-4 py-2 text-xs text-cream">
                {watch.notice}
                <button type="button" className="text-gold-soft" onClick={watch.clearNotice} aria-label="Fechar aviso">
                  ✕
                </button>
              </p>
            )}
          </CourseVideoPlayer>

          {/* Conteúdo Textual Estruturado */}
          <div className="zt-panel space-y-3 p-4">
            <h2 className="text-xs font-mono uppercase tracking-wider text-gold-soft">
              Material Didático de Apoio
            </h2>
            {lessonBody !== undefined ? (
              <TipRichText text={lessonBody} className="space-y-3 text-sm text-felt-100" />
            ) : (
              <p className="text-xs text-felt-400">{materialFailed ? "Não foi possível carregar o material agora. Recarregue a página." : "Carregando o material da aula…"}</p>
            )}
            <CourseSources sourceIds={lesson.sources} />
          </div>

          {/* Quiz de Fixação Bloqueante */}
          {!authed ? (
            <p className="zt-panel p-4 text-xs text-felt-300">
              <Link to="/login" className="text-gold-soft">Entre</Link> para fazer o quiz desta aula.
            </p>
          ) : !watch.state ? (
            <p className="text-xs text-felt-400">Carregando o quiz…</p>
          ) : watch.state.review ? (
            <CourseReview items={watch.state.review} score={watch.state.best_score} />
          ) : !watch.state.quiz_open ? (
            <WatchGate state={watch.state} />
          ) : (
            <CourseQuiz
              key={`quiz-${lesson.id}`}
              questions={lesson.quiz}
              lessonTitle={lesson.title}
              retryAt={watch.state.retry_at}
              onSubmit={submitQuiz}
            />
          )}
          {mod && <div className="zt-panel p-5 space-y-3 border-gold-soft/30">
            <h2 className="font-semibold text-gold-bright">Leve esta ideia para a mesa</h2>
            <p className="text-sm text-felt-200">Bots, ações reais e replay. Pratique com fichas sem valor e compare suas decisões.</p>
            <Link className="zt-btn-primary inline-flex text-xs!" to={`/curso/mesa/${mod.id}?cenario=${TRAINING.scenarios.find(s => s.lesson === lesson.id)?.id ?? ""}`}>Abrir simulador desta aula →</Link>
          </div>}
        </>
      )}

      {/* Navegação entre Aulas com Trava de Fixação */}
      <div className="flex flex-wrap items-center justify-between gap-3 pt-4 border-t border-felt-800">
        {prev ? (
          <Link to={`/curso/${prev.id}`} className="zt-btn-secondary text-xs!">
            ← {prev.title}
          </Link>
        ) : (
          <span />
        )}

        {next ? (
          isLessonUnlocked(next.id, { ...progress, ...(isPassed ? { [lesson.id]: { status: "completed", best_score: lastScore ?? PASS_SCORE } } : {}) }) ? (
            <Link
              to={`/curso/${next.id}`}
              className="zt-btn-primary text-xs! animate-pulse shadow-lg shadow-gold-bright/10"
            >
              Próxima Aula: {next.title} →
            </Link>
          ) : (
            <button
              type="button"
              disabled
              title={`Atinja pelo menos ${PASS_SCORE}% no teste de fixação acima para desbloquear`}
              className="zt-btn-secondary text-xs! opacity-50 cursor-not-allowed flex items-center gap-1.5"
            >
              <span>🔒</span> {next.title} (Requer {PASS_SCORE}%)
            </button>
          )
        ) : (
          <Link to="/curso" className="zt-btn-primary text-xs!">
            🎉 Concluir Trilha
          </Link>
        )}
      </div>
    </div>
  );
}
