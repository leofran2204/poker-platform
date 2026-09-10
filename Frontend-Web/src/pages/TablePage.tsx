import { useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { getTable, leaveTable } from "@/api/client";
import type { PlayerWsData, PotWsData, ServerMessage, ShowdownEntry } from "@/api/types";
import { TableSocket, type WsStatus } from "@/api/ws";
import { PokerTable, handNamePt } from "@/components/PokerTable";
import { PlayingCard } from "@/components/PlayingCard";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

export function TablePage() {
  const { id = "" } = useParams();
  const socketRef = useRef<TableSocket | null>(null);

  const [status, setStatus] = useState<WsStatus>("disconnected");
  const [statusDetail, setStatusDetail] = useState<string | null>(null);
  const [localPlayerId, setLocalPlayerId] = useState<string | null>(null);
  const [players, setPlayers] = useState<PlayerWsData[]>([]);
  const [community, setCommunity] = useState<string[]>([]);
  const [stage, setStage] = useState("waiting");
  const [pots, setPots] = useState<PotWsData[]>([]);
  const [actions, setActions] = useState<string[]>([]);
  const [raiseAmount, setRaiseAmount] = useState(200);
  const [callAmount, setCallAmount] = useState(0);
  const [minimumWager, setMinimumWager] = useState(0);
  const [maximumWager, setMaximumWager] = useState(0);
  const [deflatorMsg, setDeflatorMsg] = useState<string | null>(null);
  const [tableName, setTableName] = useState(id);
  const [moneyMode, setMoneyMode] = useState<string | null>(null);
  const [sittingOut, setSittingOut] = useState(false);
  const [winners, setWinners] = useState<string[]>([]);
  const [showdown, setShowdown] = useState<ShowdownEntry[]>([]);
  // Resultado fixo: ao chegar o showdown, fixa o painel até dispensar.
  // Sobrevive à mão seguinte para dar tempo de ler quem ganhou e com o quê.
  const [lastResult, setLastResult] = useState<{
    key: number;
    names: string[];
    hand: string | null;
    entries: { name: string; hand: string | null; cards: string[] }[];
  } | null>(null);
  const [resultOpen, setResultOpen] = useState(false);
  const winnersRef = useRef<string[]>([]);
  const resultTimer = useRef<number | null>(null);
  const [turnLeft, setTurnLeft] = useState<number | null>(null);
  const turnActiveRef = useRef(false);
  const TURN_SECONDS = 30;
  // Ritual do crupiê: detecta mão nova (era finished, agora preflop sem board)
  // e mostra "embaralhando + distribuindo" com as cartas entrando por assento.
  const [dealing, setDealing] = useState(false);
  const finishedRef = useRef(false);
  const dealTimer = useRef<number | null>(null);

  useEffect(() => {
    if (!id || !isAuthenticated()) return;

    const sock = new TableSocket(id, {
      onStatus: (s, detail) => {
        setStatus(s);
        setStatusDetail(detail ?? null);
      },
      onMessage: (msg: ServerMessage) => {
        switch (msg.type) {
          case "welcome":
            setLocalPlayerId(msg.player_id);
            break;
          case "table_state":
            setPlayers(msg.players ?? []);
            setCommunity(msg.community_cards ?? []);
            setStage(msg.stage ?? "waiting");
            // Mão nova = estava finished e agora voltou ao preflop sem board.
            {
              const fin = msg.is_finished ?? false;
              const freshDeal =
                finishedRef.current &&
                !fin &&
                (msg.community_cards ?? []).length === 0 &&
                (msg.stage ?? "") === "preflop";
              finishedRef.current = fin;
              if (freshDeal) {
                setDealing(true);
                if (dealTimer.current !== null) window.clearTimeout(dealTimer.current);
                dealTimer.current = window.setTimeout(() => setDealing(false), 2200);
              }
            }
            setPots(msg.pots ?? []);
            setActions(msg.available_actions ?? []);
            setWinners(msg.winners ?? []);
            setShowdown(msg.showdown ?? []);
            // Fixou o resultado: chegou vencedor novo, abre o painel de leitura.
            {
              const w = msg.winners ?? [];
              const prev = winnersRef.current;
              winnersRef.current = w;
              if (w.length > 0 && prev.length === 0) {
                const sd = msg.showdown ?? [];
                const nameOf = (pid: string) =>
                  sd.find((e) => e.player_id === pid)?.player_name ??
                  (msg.players ?? []).find((p) => p.id === pid)?.name ??
                  pid;
                setLastResult({
                  key: Date.now(),
                  names: w.map(nameOf),
                  hand:
                    sd.find((e) => w.includes(e.player_id))?.hand_name ??
                    sd[0]?.hand_name ??
                    null,
                  entries: sd.map((e) => ({
                    name: e.player_name ?? nameOf(e.player_id),
                    hand: e.hand_name ?? null,
                    cards: e.cards ?? [],
                  })),
                });
                setResultOpen(true);
                if (resultTimer.current !== null) window.clearTimeout(resultTimer.current);
                resultTimer.current = window.setTimeout(() => setResultOpen(false), 15000);
              }
            }
            setCallAmount(msg.call_amount ?? 0);
            setMinimumWager(msg.minimum_wager ?? 0);
            setMaximumWager(msg.maximum_wager ?? 0);
            // Countdown do turno: arma ao chegar sua vez, desarma ao agir.
            if ((msg.available_actions ?? []).length > 0 && !turnActiveRef.current) {
              turnActiveRef.current = true;
              setTurnLeft(TURN_SECONDS);
            } else if ((msg.available_actions ?? []).length === 0 && turnActiveRef.current) {
              turnActiveRef.current = false;
              setTurnLeft(null);
            }
            setRaiseAmount((current) =>
              (msg.minimum_wager ?? 0) > 0
                ? Math.min(
                    msg.maximum_wager || Number.MAX_SAFE_INTEGER,
                    Math.max(current, msg.minimum_wager),
                  )
                : current,
            );
            if (localPlayerId) {
              const me = (msg.players ?? []).find((p) => p.id === localPlayerId);
              if (me && typeof me.is_sitting === "boolean") {
                setSittingOut(!me.is_sitting);
              }
            }
            break;
          case "your_turn":
            setActions(msg.actions ?? []);
            break;
          case "table_info":
            setTableName(msg.name || id);
            break;
          case "deflator_triggered":
            setDeflatorMsg(
              `Loss Deflator: ${msg.loser_name} recebeu ${formatBrlFromCents(msg.cashback_amount)} de volta.`,
            );
            break;
          case "error":
            setStatusDetail(msg.message);
            break;
          default:
            break;
        }
      },
    });
    socketRef.current = sock;
    void sock.connect();

    return () => {
      sock.disconnect();
      socketRef.current = null;
      if (dealTimer.current !== null) window.clearTimeout(dealTimer.current);
      if (resultTimer.current !== null) window.clearTimeout(resultTimer.current);
    };
  }, [id]);

  useEffect(() => {
    if (!id || !isAuthenticated()) return;
    void getTable(id)
      .then((table) => setMoneyMode(table.money_mode ?? null))
      .catch(() => setMoneyMode(null));
  }, [id]);

  // Regressiva de 1s do turno (servidor folda aos 30s).
  useEffect(() => {
    if (turnLeft === null || turnLeft <= 0) return;
    const t = window.setTimeout(() => setTurnLeft((v) => (v === null ? v : v - 1)), 1000);
    return () => window.clearTimeout(t);
  }, [turnLeft]);

  function onAction(action: string, amount = 0) {
    socketRef.current?.sendAction(action, amount);
  }

  async function handleLeave() {
    try {
      await leaveTable(id);
    } catch {
      /* still leave UI */
    }
    socketRef.current?.disconnect();
  }

  if (!isAuthenticated()) {
    return (
      <div className="zt-panel p-8 text-center">
        <p className="text-felt-300">Login necessário para jogar.</p>
        <Link to="/login" className="zt-btn-primary mt-4 inline-flex">
          Entrar
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-bold text-gold-bright">{tableName}</h1>
          {moneyMode === "play" ? (
            <span className="zt-chip mt-1 inline-flex">Play Money</span>
          ) : moneyMode === "real" ? (
            <span className="zt-chip mt-1 inline-flex">Jogo Real</span>
          ) : null}
          <p className="text-xs text-felt-400">
            WS:{" "}
            <span
              className={
                status === "connected"
                  ? "text-felt-300"
                  : status === "error"
                    ? "text-red-300"
                    : "text-gold-soft"
              }
            >
              {status}
              {statusDetail ? ` — ${statusDetail}` : ""}
            </span>
          </p>
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            className="zt-btn-secondary"
            onClick={() => {
              if (sittingOut) {
                socketRef.current?.sendSitIn();
                setSittingOut(false);
              } else {
                socketRef.current?.sendSitOut();
                setSittingOut(true);
              }
            }}
          >
            {sittingOut ? "Voltar a jogar" : "Sit-out"}
          </button>
          <Link to="/lobby" className="zt-btn-secondary" onClick={() => void handleLeave()}>
            Sair da mesa
          </Link>
        </div>
      </div>

      {deflatorMsg && (
        <div className="rounded border-2 border-gold bg-felt-850 px-4 py-3 text-sm text-gold-soft">
          {deflatorMsg}
          <button
            type="button"
            className="ml-3 text-xs underline"
            onClick={() => setDeflatorMsg(null)}
          >
            fechar
          </button>
        </div>
      )}

      {lastResult && resultOpen && (
        <div
          key={lastResult.key}
          className="rounded border-2 border-gold-bright bg-gold/15 px-4 py-3 text-sm"
          role="status"
        >
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="font-bold text-gold-bright">
              🏆 {lastResult.names.join(" + ")} venceu
              {lastResult.hand ? <> com {handNamePt(lastResult.hand)}</> : null}
            </span>
            <button
              type="button"
              className="text-xs underline"
              onClick={() => setResultOpen(false)}
            >
              Entendi, fechar
            </button>
          </div>
          {lastResult.entries.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-3">
              {lastResult.entries.map((entry) => (
                <div key={entry.name} className="flex items-center gap-1.5">
                  <span className="text-xs text-cream">
                    {entry.name}
                    {entry.hand ? ` (${handNamePt(entry.hand)})` : ""}
                  </span>
                  <span className="flex gap-0.5">
                    {entry.cards.map((c, i) => (
                      <PlayingCard key={`${entry.name}-${i}`} code={c} size="sm" />
                    ))}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      <PokerTable
        players={players}
        communityCards={community}
        stage={stage}
        pots={pots}
        localPlayerId={localPlayerId}
        availableActions={actions}
        onAction={onAction}
        raiseAmount={raiseAmount}
        onRaiseChange={setRaiseAmount}
        callAmount={callAmount}
        minimumWager={minimumWager}
        maximumWager={maximumWager}
        winners={winners}
        showdown={showdown}
        turnLeft={turnLeft}
        dealing={dealing}
      />
    </div>
  );
}
