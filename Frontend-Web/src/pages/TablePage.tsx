import { useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { getTable, leaveTable } from "@/api/client";
import type { PlayerWsData, PotWsData, ServerMessage, ShowdownEntry } from "@/api/types";
import { TableSocket, type WsStatus } from "@/api/ws";
import { PokerTable } from "@/components/PokerTable";
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
