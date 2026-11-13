import { useEffect, useState } from "react";
import { Link } from "react-router";
import { motion } from "motion/react";
import { fetchCourseProgress, type CourseProgressItem } from "@/api/client";
import { BrandMark } from "@/components/BrandMark";
import { PineappleDrawFilm } from "@/components/PineappleDrawFilm";
import { COURSE, isLessonUnlocked, lessonFormat, PASS_SCORE } from "@/lib/course";
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

  const tone = (id: string) =>
    id.includes("pineapple") ? "pineapple" : id.includes("short") ? "shortdeck" : id.includes("omaha") ? "omaha" : "holdem";
  const minutes = (seconds?: number) => (seconds ? `${Math.max(1, Math.round(seconds / 60))} min` : null);

  return (
    <div className="zt-academy">
      <header className="zt-academy-head">
        <div>
          <p className="zt-lobby-eyebrow">
            <BrandMark /> Zero Tilt Academy
          </p>
          <h1>
            Aprenda o jogo. <em>Depois jogue.</em>
          </h1>
          <p className="zt-lobby-sub">
            Assista às aulas, resolva os exercícios e pratique na mesa de cada módulo. Nota mínima para avançar: {PASS_SCORE}%.
          </p>
          <p className="mt-3 flex flex-wrap gap-x-4 gap-y-1 text-xs">
            <Link to="/lobby" className="text-gold-soft hover:underline">
              Ir ao lobby →
            </Link>
            <Link to="/noticias" className="text-gold-soft hover:underline">
              Notícias
            </Link>
            <Link to="/dicas" className="text-gold-soft hover:underline">
              Dica do Pró
            </Link>
          </p>
        </div>
        <div className="zt-academy-progress" aria-label={`${done} de ${total} aulas concluídas`}>
          <svg viewBox="0 0 120 120" aria-hidden="true">
            <circle cx="60" cy="60" r="52" className="track" />
            <motion.circle
              cx="60"
              cy="60"
              r="52"
              className="bar"
              strokeDasharray={2 * Math.PI * 52}
              initial={{ strokeDashoffset: 2 * Math.PI * 52 }}
              animate={{ strokeDashoffset: 2 * Math.PI * 52 * (1 - pct / 100) }}
              transition={{ duration: 0.9, ease: [0.16, 1, 0.3, 1] }}
            />
          </svg>
          <div>
            <strong>{authed ? `${pct}%` : total}</strong>
            <span>{authed ? `${done}/${total} aulas` : "aulas"}</span>
          </div>
        </div>
      </header>

      <PineappleDrawFilm academy />

      {!authed && (
        <p className="rounded-xl border border-felt-700 bg-felt-800/70 px-4 py-3 text-xs text-felt-200">
          Você está navegando como visitante. <Link to="/login" className="text-gold-soft underline">Entre na sua conta</Link> para salvar
          seu progresso. As trilhas por modalidade ficam disponíveis após a aula de regras; história é opcional.
        </p>
      )}

      {COURSE.modules.map((m) => (
        <section key={m.id} className={`zt-track ${tone(m.id)}`} aria-labelledby={`track-${m.id}`}>
          <div className="zt-track-head">
            <div>
              <h2 id={`track-${m.id}`}>{m.title}</h2>
              <p>{m.subtitle}</p>
            </div>
            <Link to={`/curso/mesa/${m.id}`} className="zt-track-practice">
              Praticar na mesa →
            </Link>
          </div>
          <div className="zt-lesson-grid">
            {m.lessons.map((l, i) => {
              const item = progress[l.id];
              const isPassed = item?.status === "completed" || (item?.best_score ?? 0) >= PASS_SCORE;
              const unlocked = isLessonUnlocked(l.id, progress);
              const poster = l.video?.posterUrl;
              const length = minutes(l.video?.durationSeconds);
              const body = (
                <>
                  <div className="zt-lesson-cover">
                    {poster ? <img src={poster} alt="" loading="lazy" width={640} height={360} /> : <span className="zt-lesson-nocover">♠</span>}
                    <span className="zt-lesson-number">{String(i + 1).padStart(2, "0")}</span>
                    {length && <span className="zt-lesson-length">{length}</span>}
                    {unlocked ? <span className="zt-lesson-play" aria-hidden="true">▶</span> : <span className="zt-lesson-lock" aria-hidden="true">🔒</span>}
                  </div>
                  <div className="zt-lesson-info">
                    <h3>{l.title}</h3>
                    <p>
                      {lessonFormat(l)} · {l.quiz.length} {l.quiz.length === 1 ? "questão" : "questões"}
                    </p>
                    <span className={`zt-lesson-status ${!unlocked ? "locked" : isPassed ? "done" : item ? "retry" : ""}`}>
                      {!unlocked ? "Bloqueada" : isPassed ? `✓ Concluída · ${item?.best_score ?? 100}%` : item ? `${item.best_score}% · refazer` : "Disponível"}
                    </span>
                  </div>
                </>
              );
              return unlocked ? (
                <motion.div key={l.id} initial={{ opacity: 0, y: 14 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true }} transition={{ duration: 0.3, delay: Math.min(i, 6) * 0.04 }} whileHover={{ y: -4 }}>
                  <Link to={`/curso/${l.id}`} className="zt-lesson-card">
                    {body}
                  </Link>
                </motion.div>
              ) : (
                <div key={l.id} className="zt-lesson-card locked" title={`Complete os pré-requisitos com nota mínima de ${PASS_SCORE}% para liberar`}>
                  {body}
                </div>
              );
            })}
          </div>
        </section>
      ))}
    </div>
  );
}
