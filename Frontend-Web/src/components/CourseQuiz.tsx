import { useState } from "react";
import {
  calculateQuizScores,
  isQuestionCorrect,
  PASS_SCORE,
  type CourseQuizQuestion,
} from "@/lib/course";
import { PlayingCard } from "./PlayingCard";

export function CourseQuiz({
  questions,
  onFinish,
}: {
  questions: CourseQuizQuestion[];
  onFinish: (score: number, passed: boolean) => void;
}) {
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [submitted, setSubmitted] = useState(false);

  const { totalScore, engineScore, hasEngineQuestions, passed } = calculateQuizScores(
    questions,
    answers,
  );

  if (questions.length === 0) return null;
  const allAnswered = questions.every((q) => answers[q.id] !== undefined);

  function submit() {
    if (!allAnswered || submitted) return;
    setSubmitted(true);
    // Para persistência e nota do aluno: prioriza nota prática se houver, ou total
    const effectiveScore = hasEngineQuestions ? engineScore : totalScore;
    onFinish(effectiveScore, passed);
  }

  function retry() {
    setAnswers({});
    setSubmitted(false);
  }

  return (
    <div className="space-y-4">
      <div className="zt-panel-title">Quiz — responda todas ({questions.length})</div>
      {questions.map((q, idx) => {
        const picked = answers[q.id];
        const showResult = submitted && picked !== undefined;
        const correct = showResult && isQuestionCorrect(q, picked);
        return (
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
              {q.options.map((opt) => {
                const active = picked === opt;
                const good = showResult && isCorrectOption(q, opt);
                const bad = showResult && active && !good;
                return (
                  <button
                    key={opt}
                    type="button"
                    disabled={submitted}
                    onClick={() => setAnswers((a) => ({ ...a, [q.id]: opt }))}
                    className={
                      good
                        ? "zt-btn-primary !border-emerald-400 !text-xs"
                        : bad
                          ? "zt-btn-danger !text-xs"
                          : active
                            ? "zt-btn-primary !text-xs"
                            : "zt-btn-secondary !text-xs"
                    }
                  >
                    {opt}
                  </button>
                );
              })}
            </div>
            {showResult && (
              <p className={`text-xs ${correct ? "text-emerald-200" : "text-red-200"}`}>
                {correct ? "Certo. " : "Não foi dessa vez. "}
                {q.kind === "theory" ? q.explanation : `Motor: ${q.engine}. ${q.why}`}
              </p>
            )}
          </div>
        );
      })}
      {!submitted ? (
        <button type="button" className="zt-btn-primary" disabled={!allAnswered} onClick={submit}>
          Ver resultado do teste
        </button>
      ) : (
        <div className="space-y-2 rounded-lg border border-felt-700 bg-felt-950 p-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div>
              <p className="text-sm font-bold text-cream">
                {hasEngineQuestions ? (
                  <>
                    Nota Situações Práticas:{" "}
                    <span className={engineScore >= PASS_SCORE ? "text-emerald-400" : "text-amber-400"}>
                      {engineScore}%
                    </span>{" "}
                    <span className="text-xs text-felt-400">({totalScore}% geral)</span>
                  </>
                ) : (
                  <>
                    Nota do Teste:{" "}
                    <span className={totalScore >= PASS_SCORE ? "text-emerald-400" : "text-amber-400"}>
                      {totalScore}%
                    </span>
                  </>
                )}
              </p>
              <p className="text-xs text-felt-300">
                {passed
                  ? `🎉 Parabéns! Você atingiu a nota mínima (${PASS_SCORE}%) e desbloqueou a próxima aula!`
                  : `⚠️ Você obteve nota abaixo da nota de corte (${PASS_SCORE}%). Revise o vídeo e tente novamente para avançar.`}
              </p>
            </div>
            {!passed && (
              <button type="button" onClick={retry} className="zt-btn-secondary !text-xs">
                🔄 Tentar novamente
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function isCorrectOption(q: CourseQuizQuestion, opt: string): boolean {
  if (q.kind === "theory") return q.options.indexOf(opt) === q.expectedIndex;
  return opt === q.expected;
}
