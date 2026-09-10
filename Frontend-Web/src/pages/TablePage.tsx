import { useEffect, useRef, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { getTable, leaveTable } from "@/api/client";
import type { PlayerWsData, PotWsData, ServerMessage, ShowdownEntry, TableResponse } from "@/api/types";
import { TableSocket, type WsStatus } from "@/api/ws";
import { PokerTable, handNamePt } from "@/components/PokerTable";
import { PlayingCard } from "@/components/PlayingCard";
import { HandJournal } from "@/components/HandJournal";
import {
  appendHand,
  loadHands,
  pushSnapshot,
  type HandSnapshot,
  type JournalHand,
} from "@/lib/handJournal";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";
import { variantHint } from "@/lib/gameLabels";

export function TablePage() {
  const { id = "" } = useParams();
  const navigate = useNavigate();
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
  const [tableMeta, setTableMeta] = useState<TableResponse | null>(null);
  const [sittingOut, setSittingOut] = useState(false);
  const [winners, setWinners] = useState<string[]>([]);
  const [showdown, setShowdown] = useState<ShowdownEntry[]>([]);
  // Resultado fixo até o jogador dispensar ou chegar o próximo showdown.
  const [lastResult, setLastResult] = useState<{
    key: number;
    names: string[];
    hand: string | null;
    entries: { name: string; hand: string | null; cards: string[] }[];
  } | null>(null);
  const [resultOpen, setResultOpen] = useState(false);
  const winnersRef = useRef<string[]>([]);
  // Diário de mãos: snapshots da mão atual + modal de replay/download.
  const localIdRef = useRef<string | null>(null);
  const tableNameRef = useRef<string>(id);
  const snapsRef = useRef<HandSnapshot[]>([]);
  const handStartRef = useRef<number>(Date.now());
  const [journalOpen, setJournalOpen] = useState(false);
  const [replayHand, setReplayHand] = useState<JournalHand | null>(null);
  const [lastJournal, setLastJournal] = useState<JournalHand | null>(null);
  const [journalCount, setJournalCount] = useState(0);
  const [turnLeft, setTurnLeft] = useState<number | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [boardStaggerFrom, setBoardStaggerFrom] = useState(0);
  const prevBoardLen = useRef(0);
  const turnActiveRef = useRef(false);
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
            localIdRef.current = msg.player_id;
            {
              const stored = loadHands(id);
              setJournalCount(stored.length);
              if (stored[0]) setLastJournal(stored[0]);
            }
            break;
          case "table_state":
            setPlayers(msg.players ?? []);
            {
              const board = msg.community_cards ?? [];
              const prev = prevBoardLen.current;
              if (board.length === 3 && prev < 3) setBoardStaggerFrom(0);
              else if (board.length === 4 && prev === 3) setBoardStaggerFrom(3);
              else if (board.length === 5 && prev === 4) setBoardStaggerFrom(4);
              else if (board.length === 0) setBoardStaggerFrom(0);
              prevBoardLen.current = board.length;
              setCommunity(board);
            }
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
                // Nova mão no diário: zera snapshots e marca o início.
                snapsRef.current = [];
                handStartRef.current = Date.now();
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
                const winnerNames = w.map(nameOf);
                const winningHand =
                  sd.find((e) => w.includes(e.player_id))?.hand_name ??
                  sd[0]?.hand_name ??
                  null;
                const entries = sd.map((e) => ({
                  name: e.player_name ?? nameOf(e.player_id),
                  hand: e.hand_name ?? null,
                  cards: e.cards ?? [],
                }));
                setLastResult({
                  key: Date.now(),
                  names: winnerNames,
                  hand: winningHand,
                  entries,
                });
                setResultOpen(true);
                // Grava a mão no diário com os snapshots acumulados.
                if (snapsRef.current.length > 0) {
                  const me = (msg.players ?? []).find((p) => p.id === localIdRef.current);
                  const journal: JournalHand = {
                    key: Date.now(),
                    tableId: id,
                    tableName: tableNameRef.current,
                    startedAt: handStartRef.current,
                    endedAt: Date.now(),
                    heroName: me?.name ?? "você",
                    winners: winnerNames,
                    winningHand,
                    snapshots: snapsRef.current,
                    showdown: entries,
                  };
                  appendHand(journal);
                  setLastJournal(journal);
                  setJournalCount(loadHands(id).length);
                }
              }
            }
            setCallAmount(msg.call_amount ?? 0);
            setMinimumWager(msg.minimum_wager ?? 0);
            setMaximumWager(msg.maximum_wager ?? 0);
            // Snapshot para o diário/replay (só com minhas cartas na mesa).
            {
              const me = (msg.players ?? []).find((p) => p.id === localIdRef.current);
              const heroCards = me?.cards ?? [];
              const board = msg.community_cards ?? [];
              if (me && (heroCards.length > 0 || board.length > 0)) {
                const pot = (msg.pots ?? []).reduce((s, p) => s + (p.amount ?? 0), 0);
                snapsRef.current = pushSnapshot(snapsRef.current, {
                  street: msg.stage ?? "waiting",
                  board,
                  heroCards,
                  pot,
                  bets: (msg.players ?? []).map((p) => ({
                    name: p.name,
                    bet: p.bet ?? 0,
                    folded: p.folded ?? false,
                    chips: p.chips ?? 0,
                  })),
                  winners: msg.winners ?? [],
                });
              }
            }
            // Relógio do servidor: reconectar não ganha 30s novos.
            if ((msg.available_actions ?? []).length > 0) {
              turnActiveRef.current = true;
              const bank = typeof msg.time_bank === "number" ? msg.time_bank : 30;
              setTurnLeft(bank);
            } else if (turnActiveRef.current) {
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
            {
              const meId = localIdRef.current;
              if (meId) {
                const me = (msg.players ?? []).find((p) => p.id === meId);
                if (me && typeof me.is_sitting === "boolean") {
                  setSittingOut(!me.is_sitting);
                }
              }
            }
            break;
          case "table_info":
            setTableName(msg.name || id);
            tableNameRef.current = msg.name || id;
            break;
          case "deflator_triggered":
            setDeflatorMsg(
              `Loss Deflator: ${msg.loser_name} recebeu ${formatBrlFromCents(msg.cashback_amount)} de volta.`,
            );
            break;
          case "error":
            setActionError(msg.message);
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
      .then((table) => {
        setMoneyMode(table.money_mode ?? null);
        setTableMeta(table);
      })
      .catch(() => setMoneyMode(null));
  }, [id]);

  // Regressiva de 1s do turno (servidor folda aos 30s).
  useEffect(() => {
    if (turnLeft === null || turnLeft <= 0) return;
    const t = window.setTimeout(() => setTurnLeft((v) => (v === null ? v : v - 1)), 1000);
    return () => window.clearTimeout(t);
  }, [turnLeft]);

  function onAction(action: string, amount = 0) {
    setActionError(null);
    socketRef.current?.sendAction(action, amount);
  }

  async function handleLeave() {
    const inHand = stage !== "waiting" && stage !== "finished";
    if (inHand && !window.confirm("Sair no meio da mão? Você pode ser foldado.")) return;
    try {
      await leaveTable(id);
    } catch {
      /* still leave UI */
    }
    socketRef.current?.disconnect();
    navigate("/lobby");
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
          <div className="mt-1 flex flex-wrap gap-1">
            {moneyMode === "play" ? (
              <span className="zt-chip">Play Money</span>
            ) : moneyMode === "real" ? (
              <span className="zt-chip">Jogo Real</span>
            ) : null}
            {tableMeta?.max_players ? (
              <span className="zt-chip">{tableMeta.max_players}-max</span>
            ) : null}
            {tableMeta?.small_blind != null && tableMeta?.big_blind != null ? (
              <span className="zt-chip font-mono">
                {formatBrlFromCents(tableMeta.small_blind)}/{formatBrlFromCents(tableMeta.big_blind)}
              </span>
            ) : null}
          </div>
          {variantHint(tableMeta?.poker_variant) ? (
            <p className="mt-1 text-[11px] text-felt-300">{variantHint(tableMeta?.poker_variant)}</p>
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
              setReplayHand(lastJournal);
              setJournalOpen(true);
            }}
            disabled={!lastJournal}
            title={lastJournal ? "Rever a última mão" : "Jogue uma mão até o fim"}
          >
            ↺ Replay
          </button>
          <button
            type="button"
            className="zt-btn-secondary"
            onClick={() => {
              setReplayHand(null);
              setJournalOpen(true);
            }}
          >
            📥 Mãos{journalCount > 0 ? ` (${journalCount})` : ""}
          </button>
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
          <button type="button" className="zt-btn-secondary" onClick={() => void handleLeave()}>
            Sair da mesa
          </button>
        </div>
      </div>

      {actionError && (
        <div className="rounded border-2 border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-200" role="alert">
          {actionError}
          <button type="button" className="ml-3 text-xs underline" onClick={() => setActionError(null)}>
            fechar
          </button>
        </div>
      )}

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
              {lastResult.names.join(" + ")} {lastResult.names.length > 1 ? "venceram" : "venceu"}
              {lastResult.hand ? <> com {handNamePt(lastResult.hand)}</> : null}
            </span>
            <button
              type="button"
              className="text-xs underline text-gold-soft"
              onClick={() => setResultOpen(false)}
            >
              fechar
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
                      <span key={`${entry.name}-${i}`} className={lastResult.names.includes(entry.name) ? "zt-win-pop" : undefined}>
                        <PlayingCard key={`${entry.name}-${i}`} code={c} size="sm" highlight={lastResult.names.includes(entry.name)} />
                      </span>
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
        maxPlayers={tableMeta?.max_players ?? 9}
        boardStaggerFrom={boardStaggerFrom}
      />
      {journalOpen && (
        <HandJournal
          tableId={id}
          tableName={tableName}
          initialHand={replayHand}
          onClose={() => {
            setJournalOpen(false);
            setReplayHand(null);
            setJournalCount(loadHands(id).length);
          }}
        />
      )}
    </div>
  );
}
