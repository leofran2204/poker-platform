import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { fetchCourseProgress, type CourseProgressItem } from "@/api/client";
import { COURSE } from "@/lib/course";
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
    .filter((l) => progress[l.id]?.status === "completed").length;
  const pct = total === 0 ? 0 : Math.round((done / total) * 100);

  return (
    <div className="mx-auto w-full max-w-3xl space-y-4">
      <h1 className="text-xl font-bold text-gold-bright">Curso Zero Tilt</h1>
      <p className="text-sm text-felt-200">
        Do zero absoluto ao jogo pensante — com quiz avaliado pelo motor de estratégia da plataforma.
      </p>
      {!authed && (
        <p className="rounded border border-felt-700 bg-felt-800 px-3 py-2 text-xs text-felt-200">
          Lendo como visitante. <Link to="/login" className="text-gold-soft">Entre</Link> para salvar
          seu progresso e valer nota.
        </p>
      )}
      {authed && total > 0 && (
        <div className="zt-panel p-3">
          <div className="flex justify-between text-xs text-felt-300">
            <span>Seu progresso</span>
            <span className="font-mono">
              {done}/{total} · {pct}%
            </span>
          </div>
          <div className="mt-2 h-2 overflow-hidden rounded bg-felt-800">
            <div className="h-full rounded bg-gold-bright" style={{ width: `${pct}%` }} />
          </div>
        </div>
      )}

      {COURSE.modules.map((m) => (
        <div key={m.id} className="space-y-2">
          <div>
            <h2 className="text-base font-bold text-cream">{m.title}</h2>
            <p className="text-xs text-felt-400">{m.subtitle}</p>
          </div>
          <div className="grid gap-2">
            {m.lessons.map((l, i) => {
              const p = progress[l.id];
              return (
                <Link
                  key={l.id}
                  to={`/curso/${l.id}`}
                  className="zt-panel flex items-center justify-between gap-3 p-3 transition-colors hover:border-gold-soft"
                >
                  <span>
                    <span className="font-mono text-[11px] text-felt-400">{String(i + 1).padStart(2, "0")}</span>{" "}
                    <span className="text-sm font-semibold text-cream">{l.title}</span>
                    <span className="block text-[11px] text-felt-400">
                      ~{l.minutes} min · {l.quiz.length} questões
                    </span>
                  </span>
                  <span className="shrink-0 font-mono text-xs text-gold-soft">
                    {p?.status === "completed" ? `✓ ${p.best_score}%` : p ? `${p.best_score}%` : "→"}
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
