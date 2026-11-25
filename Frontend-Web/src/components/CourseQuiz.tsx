import { useEffect, useState, type ReactNode } from "react";
import type { CourseQuizResult, CourseReviewItem } from "@/api/client";
import { PASS_SCORE, type CourseQuizQuestion } from "@/lib/course";
import { clock } from "@/lib/useLessonWatch";
import { PlayingCard } from "./PlayingCard";

function shuffled<T>(items: T[]): T[] {
  const copy = [...items];
  for (let i = copy.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [copy[i], copy[j]] = [copy[j], copy[i]];
  }
  return copy;
}

function localTime(iso: string): string {
  return new Date(iso).toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" });
}

/**
 * Quiz da aula. A nota vem do servidor; na reprovação aparecem só a nota e o
 * tema a revisar, e a nova tentativa abre depois da espera, com as
 * alternativas em outra ordem.
 */
export function CourseQuiz({
  questions,
  lessonTitle,
  retryAt: initialRetryAt,
  onSubmit,
}: {
  questions: CourseQuizQuestion[];
  lessonTitle: string;
  retryAt: string | null;
  onSubmit: (answers: Record<string, string>) => Promise<CourseQuizResult>;
}) {
  const [attempt, setAttempt] = useState(0);
  const [result, setResult] = useState<CourseQuizResult | null>(null);
  const [retryAt, setRetryAt] = useState(initialRetryAt);
  const [now, setNow] = useState(() => Date.now());
  const waitUntil = retryAt ? Date.parse(retryAt) : 0;
  const waiting = waitUntil > now;

  useEffect(() => {
    if (!waitUntil) return;
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, [waitUntil]);

  if (questions.length === 0) return null;

  const review = (
    <p className="text-xs text-felt-300">
      Tema a revisar: <span className="font-semibold text-cream">{lessonTitle}</span>. Reveja o vídeo e o
      material de apoio.{" "}
      {waiting && retryAt
        ? `Nova tentativa às ${localTime(retryAt)} (faltam ${clock((waitUntil - now) / 1000)}).`
        : "A nova tentativa já está liberada."}
    </p>
  );

  if (waiting && !result) {
    return (
      <div className="zt-panel space-y-2 p-4">
        <div className="zt-panel-title">Quiz — aguarde a nova tentativa</div>
        <p className="text-sm text-cream">A última tentativa ficou abaixo de {PASS_SCORE}%.</p>
        {review}
      </div>
    );
  }

  return (
    <QuizAttempt
      key={attempt}
      questions={questions}
      locked={result !== null}
      onSubmit={async (answers) => {
        const saved = await onSubmit(answers);
        setResult(saved);
        setRetryAt(saved.retry_at ?? null);
        setNow(Date.now());
        return saved;
      }}
      footer={
        result &&
        !result.passed && (
          <div className="space-y-2 rounded-lg border border-felt-700 bg-felt-950 p-4">
            <p className="text-sm font-bold text-cream">
              Nota do teste: <span className="text-amber-400">{result.score}%</span>{" "}
              <span className="text-xs font-normal text-felt-400">(mínimo {PASS_SCORE}%)</span>
            </p>
            {review}
            <button
              type="button"
              disabled={waiting}
              onClick={() => {
                setResult(null);
                setAttempt((a) => a + 1);
              }}
              className="zt-btn-secondary text-xs! disabled:cursor-not-allowed disabled:opacity-50"
            >
              Tentar de novo
            </button>
          </div>
        )
      }
    />
  );
}

function QuizAttempt({
  questions,
  locked,
  onSubmit,
  footer,
}: {
  questions: CourseQuizQuestion[];
  locked: boolean;
  onSubmit: (answers: Record<string, string>) => Promise<CourseQuizResult>;
  footer: ReactNode;
}) {
  // Nova ordem das alternativas a cada tentativa.
  const [ordered] = useState(() => questions.map((q) => ({ ...q, options: shuffled(q.options) })));
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const allAnswered = ordered.every((q) => answers[q.id] !== undefined);

  async function submit() {
    if (!allAnswered || sending || locked) return;
    setSending(true);
    setError(null);
    try {
      await onSubmit(answers);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Não foi possível enviar agora. Tente de novo.");
    } finally {
      setSending(false);
    }
  }

  return (
    <div className="space-y-4">
      <div className="zt-panel-title">Quiz — responda todas ({ordered.length})</div>
      {ordered.map((q, idx) => (
        <div key={q.id} className="zt-panel space-y-2 p-4">
          <p className="text-sm font-semibold text-cream">
            {idx + 1}. {q.prompt}
          </p>
          {q.kind === "engine" && (
            <div className="flex flex-wrap items-center gap-4 text-xs text-felt-300">
              <span className="flex items-center gap-1">
                Mão:
                {q.hole.map((c) => (
                  <PlayingCard key={c} code={c} size="xs" inline />
                ))}
              </span>
              {q.board.length > 0 && (
                <span className="flex items-center gap-1">
                  Mesa:
                  {q.board.map((c) => (
                    <PlayingCard key={c} code={c} size="xs" inline />
                  ))}
                </span>
              )}
            </div>
          )}
          <div className="flex flex-wrap gap-2">
            {q.options.map((opt) => (
              <button
                key={opt}
                type="button"
                aria-pressed={answers[q.id] === opt}
                disabled={locked || sending}
                onClick={() => setAnswers((a) => ({ ...a, [q.id]: opt }))}
                className={answers[q.id] === opt ? "zt-btn-primary text-xs!" : "zt-btn-secondary text-xs!"}
              >
                {opt}
              </button>
            ))}
          </div>
        </div>
      ))}
      {!locked && (
        <button type="button" className="zt-btn-primary" disabled={!allAnswered || sending} onClick={() => void submit()}>
          {sending ? "Enviando…" : "Ver resultado do teste"}
        </button>
      )}
      {error && <p className="text-xs text-red-200">{error}</p>}
      {footer}
    </div>
  );
}

/** Gabarito comentado, mostrado só depois da aprovação. */
export function CourseReview({ items, score }: { items: CourseReviewItem[]; score: number }) {
  return (
    <div className="zt-panel space-y-3 p-4">
      <div className="zt-panel-title">Quiz concluído ({score}%) — gabarito comentado</div>
      {items.map((item, idx) => (
        <div key={item.id} className="space-y-1 border-t border-felt-800 pt-2 first:border-t-0 first:pt-0">
          <p className="text-sm font-semibold text-cream">
            {idx + 1}. {item.prompt}
          </p>
          <p className="text-xs text-emerald-300">Resposta: {item.answer}</p>
          <p className="text-xs text-felt-300">{item.explanation}</p>
        </div>
      ))}
    </div>
  );
}
