import { useState } from "react";
import { PlayingCard } from "./PlayingCard";
import { handNamePt } from "./PokerTable";
import {
  clearHands,
  downloadFile,
  handToText,
  loadHands,
  streetPt,
  type JournalHand,
} from "@/lib/handJournal";
import { formatChips } from "@/lib/money";

interface Props {
  tableId: string;
  tableName: string;
  /** Abre direto no replay desta mão (botão "Ver replay" do resultado). */
  initialHand?: JournalHand | null;
  onClose: () => void;
}

function fmtDate(ts: number): string {
  return new Date(ts).toLocaleString("pt-BR", {
    day: "2-digit",
    month: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function HandJournal({ tableId, tableName, initialHand = null, onClose }: Props) {
  const [hands, setHands] = useState<JournalHand[]>(() => loadHands(tableId));
  const [view, setView] = useState<JournalHand | null>(initialHand);
  const [step, setStep] = useState(0);

  function openReplay(h: JournalHand) {
    setView(h);
    setStep(h.snapshots.length > 0 ? h.snapshots.length - 1 : 0);
  }

  function downloadAllJson() {
    downloadFile(
      `maos-${tableId.slice(0, 8)}.json`,
      JSON.stringify(hands, null, 2),
      "application/json",
    );
  }

  function downloadHandTxt(h: JournalHand) {
    downloadFile(`mao-${h.key}.txt`, handToText(h));
  }

  const snap = view?.snapshots[Math.min(step, (view?.snapshots.length ?? 1) - 1)] ?? null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
      role="dialog"
      aria-label="Minhas mãos"
      onClick={onClose}
    >
      <div
        className="max-h-[85vh] w-full max-w-2xl overflow-y-auto rounded border-2 border-gold bg-felt-950 p-4"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-3 flex items-center justify-between">
          <h2 className="font-bold text-gold-bright">
            {view ? "↺ Replay da mão" : `📥 Minhas mãos — ${tableName}`}
          </h2>
          <button type="button" className="zt-btn-secondary" onClick={onClose}>
            Fechar
          </button>
        </div>

        {!view && (
          <>
            {hands.length === 0 && (
              <p className="text-sm text-felt-300">
                Nenhuma mão registrada aqui ainda. Jogue uma mão até o fim que ela aparece aqui
                com suas cartas, board e resultado.
              </p>
            )}
            <ul className="space-y-2">
              {hands.map((h) => (
                <li
                  key={h.key}
                  className="flex flex-wrap items-center justify-between gap-2 rounded border border-rail bg-felt-900 px-3 py-2 text-sm"
                >
                  <span className="text-cream">
                    {fmtDate(h.endedAt)} — {h.winners.join(" + ")}
                    {h.winningHand ? ` (${handNamePt(h.winningHand)})` : ""}
                  </span>
                  <span className="flex gap-2">
                    <button type="button" className="zt-btn-secondary" onClick={() => openReplay(h)}>
                      Ver
                    </button>
                    <button
                      type="button"
                      className="zt-btn-secondary"
                      onClick={() => downloadHandTxt(h)}
                    >
                      TXT
                    </button>
                  </span>
                </li>
              ))}
            </ul>
            {hands.length > 0 && (
              <div className="mt-3 flex gap-2">
                <button type="button" className="zt-btn-secondary" onClick={downloadAllJson}>
                  Baixar tudo (JSON)
                </button>
                <button
                  type="button"
                  className="zt-btn-danger"
                  onClick={() => {
                    clearHands(tableId);
                    setHands([]);
                  }}
                >
                  Apagar diário
                </button>
              </div>
            )}
          </>
        )}

        {view && snap && (
          <>
            <div className="mb-2 text-sm text-felt-300">
              {fmtDate(view.endedAt)} — passo {step + 1} de {view.snapshots.length} —{" "}
              <strong className="text-cream">{streetPt(snap.street)}</strong>
            </div>
            <div className="mb-2 flex items-center gap-2 text-sm">
              <span className="text-gold-soft">Board:</span>
              {snap.board.length === 0 ? (
                <span className="italic text-felt-400">—</span>
              ) : (
                snap.board.map((c, i) => <PlayingCard key={i} code={c} size="sm" />)
              )}
            </div>
            <div className="mb-2 flex items-center gap-2 text-sm">
              <span className="text-gold-soft">Suas cartas:</span>
              {snap.heroCards.length === 0 ? (
                <span className="italic text-felt-400">—</span>
              ) : (
                snap.heroCards.map((c, i) => <PlayingCard key={i} code={c} size="sm" />)
              )}
              <span className="ml-2 text-cream">Pote {formatChips(snap.pot)}</span>
            </div>
            <ul className="mb-3 space-y-1 text-xs text-felt-200">
              {snap.bets.map((b) => (
                <li key={b.name}>
                  {b.name}: aposta {formatChips(b.bet)}
                  {b.folded ? " (fold)" : ""} — stack {formatChips(b.chips)}
                </li>
              ))}
            </ul>
            {step === view.snapshots.length - 1 && (
              <p className="mb-3 text-sm font-bold text-gold-bright">
                🏆 {view.winners.join(" + ")}
                {view.winningHand ? ` com ${handNamePt(view.winningHand)}` : ""}
              </p>
            )}
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                className="zt-btn-secondary"
                disabled={step <= 0}
                onClick={() => setStep((v) => Math.max(0, v - 1))}
              >
                ← Anterior
              </button>
              <button
                type="button"
                className="zt-btn-secondary"
                disabled={step >= view.snapshots.length - 1}
                onClick={() => setStep((v) => Math.min(view.snapshots.length - 1, v + 1))}
              >
                Próximo →
              </button>
              <button
                type="button"
                className="zt-btn-secondary"
                onClick={() => downloadHandTxt(view)}
              >
                Baixar TXT
              </button>
              <button type="button" className="zt-btn-secondary" onClick={() => setView(null)}>
                Voltar à lista
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
