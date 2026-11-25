import { useEffect, useRef, useState } from "react";
import type { CourseSurpriseCheck as Check } from "@/api/client";

/**
 * Pergunta surpresa durante o vídeo. O vídeo segue tocando; sem resposta no
 * prazo, `onTimeout` pausa o vídeo e avisa o servidor. Use com `key={check.id}`.
 */
export function CourseSurpriseCheck({
  check,
  onAnswer,
  onTimeout,
}: {
  check: Check;
  onAnswer: (option: string) => void;
  onTimeout: () => void;
}) {
  const [left, setLeft] = useState(check.seconds_left);
  const [sent, setSent] = useState(false);
  const timeout = useRef(onTimeout);
  useEffect(() => {
    timeout.current = onTimeout;
  }, [onTimeout]);
  useEffect(() => {
    const deadline = Date.now() + check.seconds_left * 1000;
    const timer = window.setInterval(() => {
      const remaining = Math.max(0, Math.ceil((deadline - Date.now()) / 1000));
      setLeft(remaining);
      if (remaining === 0) {
        window.clearInterval(timer);
        timeout.current();
      }
    }, 500);
    return () => window.clearInterval(timer);
  }, [check.seconds_left]);

  return (
    <div
      role="alertdialog"
      aria-labelledby={`check-${check.id}`}
      className="space-y-3 border-t-2 border-gold-bright bg-felt-900 px-4 py-3"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p id={`check-${check.id}`} className="text-sm font-semibold text-gold-bright">
          {check.prompt}
        </p>
        <span aria-live="polite" className="rounded-sm bg-felt-800 px-2 py-1 font-mono text-xs text-cream">
          {left} s
        </span>
      </div>
      <div className="flex flex-wrap gap-2">
        {check.options.map((option) => (
          <button
            key={option}
            type="button"
            disabled={sent || left === 0}
            onClick={() => {
              setSent(true);
              onAnswer(option);
            }}
            className="zt-btn-secondary min-h-11 text-left text-xs!"
          >
            {option}
          </button>
        ))}
      </div>
      <p className="text-[11px] text-felt-400">
        Sem resposta em 60 s, o vídeo pausa e o trecho desde a pergunta anterior volta a contar do zero.
      </p>
    </div>
  );
}
