import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { fetchCourseProgress, type CourseProgressItem } from "@/api/client";
import { COURSE, isLessonUnlocked, PASS_SCORE } from "@/lib/course";
import { isAuthenticated } from "@/lib/auth";

export function CoursePage() {
  const [progress, setProgress] = useState<Record<string, CourseProgressItem>>({});
  const authed = isAuthenticated();

  useEffect(() => {
    if (!authed) return;
    fetchCourseProgress()
      .then((items) => {
        const map: Record<string, CourseProgressItem> = {};
        for (const it of items) map[it.lesson_id] = it;
        setProgress(map);
      })
      .catch(() => {});
  }, [authed]);

  const total = COURSE.modules.flatMap((m) => m.lessons).length;
  const done = COURSE.modules
    .flatMap((m) => m.lessons)
    .filter((l) => progress[l.id]?.status === "completed" || (progress[l.id]?.best_score ?? 0) >= PASS_SCORE).length;
  const pct = total === 0 ? 0 : Math.round((done / total) * 100);

  return (
    <div className="mx-auto w-full max-w-3xl space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h1 className="text-xl font-bold text-gold-bright">Zero Tilt Academy</h1>
          <p className="text-sm text-felt-200">
            Aprenda poker do zero com vídeos de 2 minutos, jogador virtual de IA e testes práticos na mesa.
          </p>
        </div>
        <span className="rounded border border-gold-soft/30 bg-felt-900 px-3 py-1 text-xs font-mono text-gold-bright">
          Nota de Corte: {PASS_SCORE}%
        </span>
      </div>

      {!authed && (
        <p className="rounded border border-felt-700 bg-felt-800 px-3 py-2 text-xs text-felt-200">
          Você está navegando como visitante. <Link to="/login" className="text-gold-soft underline">Entre na sua conta</Link> para salvar
          seu progresso e desbloquear os módulos sequencialmente.
        </p>
      )}

      {authed && total > 0 && (
        <div className="zt-panel p-3">
          <div className="flex justify-between text-xs text-felt-300">
            <span>Progresso da Trilha Oficial</span>
            <span className="font-mono text-gold-bright">
              {done}/{total} aulas concluídas • {pct}%
            </span>
          </div>
          <div className="mt-2 h-2 overflow-hidden rounded bg-felt-800">
            <div className="h-full rounded bg-gold-bright transition-all duration-500" style={{ width: `${pct}%` }} />
          </div>
        </div>
      )}

      {COURSE.modules.map((m) => (
        <div key={m.id} className="space-y-2 pt-2">
          <div>
            <h2 className="text-base font-bold text-cream flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-gold-bright inline-block" />
              {m.title}
            </h2>
            <p className="text-xs text-felt-400 pl-4">{m.subtitle}</p>
          </div>
          <div className="grid gap-2 pl-2">
            {m.lessons.map((l, i) => {
              const p = progress[l.id];
              const isPassed = p?.status === "completed" || (p?.best_score ?? 0) >= PASS_SCORE;
              const unlocked = isLessonUnlocked(l.id, progress);

              if (!unlocked) {
                return (
                  <div
                    key={l.id}
                    className="zt-panel flex items-center justify-between gap-3 p-3 opacity-50 cursor-not-allowed border-felt-800 bg-felt-950/40"
                    title={`Complete a aula anterior com nota mínima de ${PASS_SCORE}% para liberar`}
                  >
                    <span>
                      <span className="font-mono text-[11px] text-felt-500">{String(i + 1).padStart(2, "0")}</span>{" "}
                      <span className="text-sm font-semibold text-felt-400">{l.title}</span>
                      <span className="block text-[11px] text-felt-500">
                        🎬 Vídeo IA (2 min) • Mão na Mesa • Quiz ({l.quiz.length} questões)
                      </span>
                    </span>
                    <span className="shrink-0 font-mono text-xs text-felt-500 flex items-center gap-1">
                      🔒 Bloqueada
                    </span>
                  </div>
                );
              }

              return (
                <Link
                  key={l.id}
                  to={`/curso/${l.id}`}
                  className="zt-panel flex items-center justify-between gap-3 p-3 transition-all hover:border-gold-soft hover:bg-felt-900/90"
                >
                  <span>
                    <span className="font-mono text-[11px] text-felt-400">{String(i + 1).padStart(2, "0")}</span>{" "}
                    <span className="text-sm font-semibold text-cream">{l.title}</span>
                    <span className="block text-[11px] text-felt-400">
                      🎬 Vídeo IA (2 min) • Mão na Mesa • Quiz ({l.quiz.length} questões)
                    </span>
                  </span>
                  <span className="shrink-0 font-mono text-xs">
                    {isPassed ? (
                      <span className="text-emerald-400 font-bold flex items-center gap-1">
                        ✓ {p?.best_score ?? 100}%
                      </span>
                    ) : p ? (
                      <span className="text-amber-400 font-bold">
                        {p.best_score}% (Refazer)
                      </span>
                    ) : (
                      <span className="text-gold-soft">Disponível →</span>
                    )}
                  </span>
                </Link>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}
