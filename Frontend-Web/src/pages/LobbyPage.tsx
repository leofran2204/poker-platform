import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router";
import {
  fetchTournamentRegistration,
  joinTable,
  joinWaitlist,
  listTables,
  listTournaments,
  registerTournament,
  unregisterTournament,
} from "@/api/client";
import type { TableResponse, TournamentInfoResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatCountdown, liveTableIds, nextStartEpoch } from "@/lib/countdown";
import { deckTypeLabel, gameNameLabel } from "@/lib/gameLabels";
import { formatBrlFromCents } from "@/lib/money";
import { getWalletMode } from "@/lib/walletMode";

type LobbyTab = "cash" | "tournaments";
type StakeFilter = "all" | "nl025" | "nl075150" | "omaha050" | "pineapple050";

const STAKE_OPTIONS: { id: StakeFilter; label: string }[] = [
  { id: "all", label: "Todos" },
  { id: "nl025", label: "NL 0,25/0,25" },
  { id: "nl075150", label: "NL 0,75/1,50" },
  { id: "omaha050", label: "Omaha 0,50/0,50" },
  { id: "pineapple050", label: "Pineapple 0,50/0,50" },
];

function formatBuyInRange(min: number, max: number): string {
  if (min === max) return formatBrlFromCents(min);
  return `${formatBrlFromCents(min)}–${formatBrlFromCents(max)}`;
}

function occupancyPct(players: number, max: number): number {
  if (max <= 0) return 0;
  return Math.min(100, Math.round((players / max) * 100));
}

function buyInLabel(t: TournamentInfoResponse): string {
  if (t.is_freeroll) return "Grátis";
  if ((t.fee_cents ?? 0) > 0) {
    return `${formatBrlFromCents(t.buy_in)} + ${formatBrlFromCents(t.fee_cents)}`;
  }
  return formatBrlFromCents(t.buy_in);
}

function occupancyBarClass(pct: number, full: boolean): string {
  if (full || pct >= 100) return "zt-occupancy-bar full";
  if (pct >= 75) return "zt-occupancy-bar high";
  if (pct >= 50) return "zt-occupancy-bar mid";
  return "zt-occupancy-bar";
}

export function LobbyPage() {
  const navigate = useNavigate();
  const [tab, setTab] = useState<LobbyTab>("cash");
  const [tables, setTables] = useState<TableResponse[]>([]);
  const [tournaments, setTournaments] = useState<TournamentInfoResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [registeredIds, setRegisteredIds] = useState<Set<string>>(new Set());
  const [hideFull, setHideFull] = useState(false);
  const [stake, setStake] = useState<StakeFilter>("all");
  const [joiningId, setJoiningId] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [registeringId, setRegisteringId] = useState<string | null>(null);
  const [now, setNow] = useState(() => Date.now());
  const walletMode = getWalletMode();
  const walletModeLabel = walletMode === "real" ? "Jogo Real" : "Play Money";

  const load = useCallback(async () => {
    if (!isAuthenticated()) {
      setError("Faça login para ver o lobby.");
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const mode = getWalletMode();
      const [t, tourneys] = await Promise.all([listTables(mode), listTournaments(mode)]);
      setTables(t);
      setTournaments(tourneys);
      const flags = await Promise.all(
        tourneys.map(async (tourney) => {
          try {
            const r = await fetchTournamentRegistration(tourney.id);
            return r.registered ? tourney.id : null;
          } catch {
            return null;
          }
        }),
      );
      setRegisteredIds(new Set(flags.filter((id): id is string => id != null)));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao carregar lobby");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    const tick = window.setInterval(() => setNow(Date.now()), 1_000);
    return () => window.clearInterval(tick);
  }, []);

  useEffect(() => {
    void load();
    const t = window.setInterval(() => void load(), 15_000);
    const onMode = () => void load();
    window.addEventListener("wallet-mode-changed", onMode);
    return () => {
      window.clearInterval(t);
      window.removeEventListener("wallet-mode-changed", onMode);
    };
  }, [load]);

  const filtered = useMemo(() => {
    return tables.filter((t) => {
      if (hideFull && t.players >= t.max_players) return false;
      const isOmaha = t.poker_variant === "omaha" || t.poker_variant === "short_deck_omaha";
      const isPineapple =
        t.poker_variant === "brazilian_pineapple" || t.poker_variant === "ultimate_pineapple";
      if (stake === "nl025")
        return !isOmaha && !isPineapple && t.small_blind === 25 && t.big_blind === 25;
      if (stake === "nl075150")
        return !isOmaha && !isPineapple && t.small_blind === 75 && t.big_blind === 150;
      if (stake === "omaha050")
        return isOmaha && t.small_blind === 50 && t.big_blind === 50;
      if (stake === "pineapple050")
        return isPineapple && t.small_blind === 50 && t.big_blind === 50;
      return true;
    });
  }, [tables, hideFull, stake]);

  const soonestStart = useMemo(() => nextStartEpoch(tournaments), [tournaments]);

  useEffect(() => {
    if (selectedId && !filtered.some((t) => t.id === selectedId)) {
      setSelectedId(null);
    }
  }, [filtered, selectedId]);

  async function handleWaitlist(table: TableResponse) {
    setJoiningId(table.id);
    setError(null);
    setInfo(null);
    try {
      const wait = await joinWaitlist(table.id);
      setInfo(
        `Fila da mesa: você é o ${wait.position}º de ${wait.length}. Quando abrir vaga, clique em Entrar.`,
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : "Não foi possível entrar na fila");
    } finally {
      setJoiningId(null);
    }
  }

  async function handleJoin(table: TableResponse) {
    if (table.players >= table.max_players) return;
    setJoiningId(table.id);
    setError(null);
    setInfo(null);
    try {
      await joinTable(table.id, table.min_buy_in, getWalletMode());
      navigate(`/table/${table.id}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Não foi possível entrar na mesa");
    } finally {
      setJoiningId(null);
    }
  }

  async function handleRegister(t: TournamentInfoResponse) {
    setRegisteringId(t.id);
    setError(null);
    setInfo(null);
    try {
      await registerTournament(t.id, getWalletMode());
      setInfo("Inscrição confirmada. Abra o torneio para ver a mesa quando começar.");
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Falha na inscrição");
    } finally {
      setRegisteringId(null);
    }
  }

  async function handleUnregister(t: TournamentInfoResponse) {
    if (!window.confirm("Cancelar inscrição? Devolve buy-in + taxa, só antes de começar.")) return;
    setRegisteringId(t.id);
    setError(null);
    setInfo(null);
    try {
      const res = await unregisterTournament(t.id);
      setInfo(
        `Inscrição cancelada. Devolvidos ${formatBrlFromCents(res.refunded_buy_in_cents + res.refunded_fee_cents)}.`,
      );
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Falha ao cancelar");
    } finally {
      setRegisteringId(null);
    }
  }

  if (!isAuthenticated()) {
    return (
      <div className="zt-panel p-8 text-center">
        <h1 className="text-xl font-bold text-gold-bright">Lobby</h1>
        <p className="mt-2 text-felt-300">Entre na sua conta para listar mesas e torneios.</p>
        <Link to="/login" className="zt-btn-primary mt-6 inline-flex">
          Entrar
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h1 className="text-xl font-bold uppercase tracking-wide text-gold-bright">Lobby</h1>
          <p className="text-xs text-felt-300">
            Listando apenas <span className="font-semibold text-gold-soft">{walletModeLabel}</span>
            {" · "}fichas Play Money e Jogo Real não se misturam
          </p>
        </div>
        <div className="flex gap-1 rounded border border-felt-600 bg-felt-950/60 p-0.5">
          <button
            type="button"
            role="tab"
            aria-selected={tab === "cash"}
            className={tab === "cash" ? "zt-tab zt-tab-active" : "zt-tab"}
            onClick={() => setTab("cash")}
          >
            Cash
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={tab === "tournaments"}
            className={tab === "tournaments" ? "zt-tab zt-tab-active" : "zt-tab"}
            onClick={() => setTab("tournaments")}
          >
            Torneios
          </button>
        </div>
      </div>

      {error && (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200" role="alert">
          {error}
        </p>
      )}
      {info && (
        <p className="rounded border border-emerald-800 bg-emerald-950/30 px-3 py-2 text-sm text-emerald-100" role="status">
          {info}
        </p>
      )}

      <div
        className={
          walletMode === "real"
            ? "rounded border border-amber-600/60 bg-amber-950/40 px-3 py-2 text-xs text-amber-100"
            : "rounded border border-emerald-700/50 bg-emerald-950/30 px-3 py-2 text-xs text-emerald-100"
        }
      >
        {walletMode === "real" ? (
          <>
            <strong>Jogo Real:</strong> só saldo real. Fichas Play Money{" "}
            <strong>não</strong> servem nestas mesas/torneios. Sem saldo? Vá em{" "}
            <Link to="/wallet" className="underline text-gold-soft">
              Carteira → Pedir fichas
            </Link>
            .
          </>
        ) : (
          <>
            <strong>Play Money:</strong> fichas de diversão (renovam todo dia).{" "}
            <strong>Não têm valor real</strong> e não podem ser usadas no Jogo Real.
          </>
        )}
      </div>

      {tab === "cash" ? (
        <div className="zt-panel overflow-hidden">
          <div className="zt-lobby-toolbar">
            <div className="min-w-0 flex-1">
              <div className="text-xs font-bold uppercase tracking-wider text-gold-bright">
                Cash games
                <span className="ml-2 font-mono text-felt-300">({filtered.length})</span>
              </div>
              <p className="text-[11px] text-felt-400">
                NL 0,25/0,25 9-max (R$25) · NL 0,75/1,50 9-max (R$150) · Omaha 0,50/0,50 6-max (R$100) · Pineapple 0,50/0,50 5-max (R$75) · ação automática em 15s
              </p>
            </div>

            <div className="zt-lobby-stake-tabs" role="tablist" aria-label="Filtro de stakes">
              {STAKE_OPTIONS.map((opt) => (
                <button
                  key={opt.id}
                  type="button"
                  role="tab"
                  aria-selected={stake === opt.id}
                  className={stake === opt.id ? "zt-tab zt-tab-active" : "zt-tab"}
                  onClick={() => setStake(opt.id)}
                >
                  {opt.label}
                </button>
              ))}
            </div>

            <label className="flex items-center gap-2 text-xs text-cream">
              <input
                type="checkbox"
                className="accent-gold"
                checked={hideFull}
                onChange={(e) => setHideFull(e.target.checked)}
              />
              Ocultar cheias
            </label>

            <button
              type="button"
              className="zt-btn-secondary !px-3 !py-1 !text-xs"
              onClick={() => void load()}
              disabled={loading}
            >
              Atualizar
            </button>
          </div>

          {loading && tables.length === 0 ? (
            <div className="flex items-center justify-center gap-3 p-8 text-sm text-felt-300">
              <span className="zt-spinner" aria-hidden />
              Carregando mesas…
            </div>
          ) : filtered.length === 0 ? (
            <p className="p-6 text-center text-sm text-felt-300">Nenhuma mesa com esses filtros.</p>
          ) : (
            <>
            <div className="space-y-2 p-3 md:hidden">
              {filtered.map((t) => {
                const full = t.players >= t.max_players;
                const pct = occupancyPct(t.players, t.max_players);
                return (
                  <div key={t.id} className="rounded border border-felt-600 bg-felt-950/50 p-3">
                    <div className="flex items-start justify-between gap-2">
                      <div className="min-w-0">
                        <p className="font-semibold text-cream">{gameNameLabel(t, "cash")}</p>
                        <p className="mt-1 font-mono text-sm text-gold-soft">
                          {formatBrlFromCents(t.small_blind)}/{formatBrlFromCents(t.big_blind)}
                          <span className="ml-2 text-felt-300">
                            {formatBuyInRange(t.min_buy_in, t.max_buy_in)}
                          </span>
                        </p>
                        <p className="mt-1 text-[11px] text-felt-400">
                          {deckTypeLabel(t)} · {t.max_players}-max
                        </p>
                      </div>
                      <button
                        type="button"
                        className={
                          full
                            ? "zt-btn-secondary shrink-0 !px-3 !py-1.5 !text-xs"
                            : "zt-btn-primary shrink-0 !px-3 !py-1.5 !text-xs"
                        }
                        disabled={joiningId === t.id}
                        onClick={() => (full ? void handleWaitlist(t) : void handleJoin(t))}
                      >
                        {joiningId === t.id ? "…" : full ? "Fila" : "Entrar"}
                      </button>
                    </div>
                    <div className="zt-occupancy mt-2">
                      <span className={full ? "zt-occupancy-label full" : "zt-occupancy-label"}>
                        {t.players}/{t.max_players}
                      </span>
                      <div className="zt-occupancy-track" aria-hidden>
                        <div className={occupancyBarClass(pct, full)} style={{ width: `${pct}%` }} />
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
            <div className="zt-table-wrap hidden md:block">
              <table className="zt-lobby-table min-w-[58rem] table-fixed">
                <colgroup>
                  <col className="w-[28%]" />
                  <col className="w-[16%]" />
                  <col className="w-[20%]" />
                  <col className="w-[14%]" />
                  <col className="w-[13%]" />
                  <col className="w-[9%]" />
                </colgroup>
                <thead>
                  <tr>
                    <th>Nome</th>
                    <th>Tipo</th>
                    <th>Blinds</th>
                    <th>Frente</th>
                    <th>Jogadores</th>
                    <th className="text-right">Ação</th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((t) => {
                    const full = t.players >= t.max_players;
                    const pct = occupancyPct(t.players, t.max_players);
                    const selected = selectedId === t.id;
                    return (
                      <tr
                        key={t.id}
                        tabIndex={0}
                        className={[
                          selected ? "zt-lobby-row-selected" : "",
                          full ? "zt-lobby-row-full" : "",
                        ]
                          .filter(Boolean)
                          .join(" ")}
                        onClick={() => setSelectedId(t.id)}
                        onDoubleClick={() => {
                          if (!full) void handleJoin(t);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            setSelectedId(t.id);
                            if (e.key === "Enter" && !full) void handleJoin(t);
                          }
                        }}
                      >
                        <td className="font-semibold text-cream">
                          {gameNameLabel(t, "cash")}
                        </td>
                        <td>
                          <div className="flex flex-col items-center gap-1 text-center">
                            <span
                              className={
                                deckTypeLabel(t) === "Short Deck"
                                  ? "zt-chip zt-chip-accent"
                                  : "zt-chip"
                              }
                            >
                              {deckTypeLabel(t)}
                            </span>
                            <span className="zt-chip">{t.max_players}-max</span>
                          </div>
                        </td>
                        <td className="font-mono text-gold-soft">
                          {formatBrlFromCents(t.small_blind)}/{formatBrlFromCents(t.big_blind)}
                        </td>
                        <td className="font-mono text-felt-200">
                          {formatBuyInRange(t.min_buy_in, t.max_buy_in)}
                        </td>
                        <td>
                          <div className="zt-occupancy">
                            <span className={full ? "zt-occupancy-label full" : "zt-occupancy-label"}>
                              {t.players}/{t.max_players}
                            </span>
                            <div className="zt-occupancy-track" aria-hidden>
                              <div
                                className={occupancyBarClass(pct, full)}
                                style={{ width: `${pct}%` }}
                              />
                            </div>
                          </div>
                        </td>
                        <td className="text-right">
                          <button
                            type="button"
                            className={
                              full
                                ? "zt-btn-secondary !px-2.5 !py-1 !text-xs"
                                : "zt-btn-primary !px-2.5 !py-1 !text-xs"
                            }
                            disabled={joiningId === t.id}
                            onClick={(e) => {
                              e.stopPropagation();
                              if (full) {
                                void handleWaitlist(t);
                              } else {
                                void handleJoin(t);
                              }
                            }}
                          >
                            {joiningId === t.id ? "…" : full ? "Fila" : "Entrar"}
                          </button>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
            </>
          )}
        </div>
      ) : (
        <div className="space-y-3">
          <div className="rounded border border-gold/40 bg-felt-950/70 px-4 py-3">
            <p className="text-xs font-bold uppercase tracking-wider text-gold-soft">
              Próximos torneios
            </p>
            <p className="mt-1 text-sm text-cream">
              Data e horário definidos pelo admin · auto-start com 5+ jogadores
              {soonestStart ? ` · ${formatCountdown(soonestStart, now)}` : ""}
            </p>
          </div>
          <div className="zt-panel overflow-hidden">
          <div className="zt-lobby-toolbar">
            <div className="min-w-0 flex-1">
              <div className="text-xs font-bold uppercase tracking-wider text-gold-bright">
                Torneios
                <span className="ml-2 font-mono text-felt-300">({tournaments.length})</span>
              </div>
              <p className="text-[11px] text-felt-400">
                Texas, freeroll, Omaha e Pineapple · taxa 15% por cima do buy-in
              </p>
            </div>
            <button
              type="button"
              className="zt-btn-secondary !px-3 !py-1 !text-xs"
              onClick={() => void load()}
              disabled={loading}
            >
              Atualizar
            </button>
          </div>

          {loading && tournaments.length === 0 ? (
            <div className="flex items-center justify-center gap-3 p-8 text-sm text-felt-300">
              <span className="zt-spinner" aria-hidden />
              Carregando torneios…
            </div>
          ) : tournaments.length === 0 ? (
            <p className="p-6 text-center text-sm text-felt-300">Nenhum torneio aberto.</p>
          ) : (
            <div className="grid gap-3 p-3 sm:grid-cols-2">
              {tournaments.map((t) => {
                const tables = liveTableIds(t);
                const registered = registeredIds.has(t.id);
                const live = t.status === "running" || t.gameplay_ready;
                return (
                  <article
                    key={t.id}
                    className="flex flex-col gap-3 rounded border border-felt-600 bg-felt-950/50 p-3"
                  >
                    <div>
                      <Link
                        to={`/tournament/${t.id}`}
                        className="font-semibold text-cream hover:text-gold-bright"
                      >
                        {gameNameLabel(t, "tournament")}
                      </Link>
                      <p className="mt-1 text-[11px] text-felt-400">
                        {deckTypeLabel(t)} · {t.table_max_players}-max
                        {t.scheduled_start_at
                          ? ` · ${formatCountdown(t.scheduled_start_at, now)}`
                          : ""}
                      </p>
                    </div>
                    <dl className="grid grid-cols-2 gap-2 text-xs">
                      <div>
                        <dt className="text-felt-400">Buy-in</dt>
                        <dd className="font-mono text-gold-soft">{buyInLabel(t)}</dd>
                      </div>
                      <div>
                        <dt className="text-felt-400">GTD</dt>
                        <dd className="font-mono text-cream">
                          {t.guaranteed_prize > 0 ? formatBrlFromCents(t.guaranteed_prize) : "—"}
                        </dd>
                      </div>
                      <div>
                        <dt className="text-felt-400">Inscritos</dt>
                        <dd className="font-mono text-cream">
                          {t.registered_players}/{t.max_players}
                        </dd>
                      </div>
                      <div>
                        <dt className="text-felt-400">Rebuy</dt>
                        <dd className="text-felt-200">
                          {t.allow_rebuy ? `até niv. ${t.rebuy_max_level}` : "não"}
                        </dd>
                      </div>
                    </dl>
                    <div className="mt-auto flex flex-wrap gap-2">
                      {live && tables.length > 0 && registered ? (
                        <Link
                          to={`/table/${tables[0]}`}
                          className="zt-btn-primary !px-3 !py-1.5 !text-xs"
                        >
                          Sentar
                        </Link>
                      ) : registered && t.status === "registering" ? (
                        <button
                          type="button"
                          className="zt-btn-secondary !px-3 !py-1.5 !text-xs"
                          disabled={registeringId === t.id}
                          onClick={() => void handleUnregister(t)}
                        >
                          {registeringId === t.id ? "…" : "Inscrito · Cancelar"}
                        </button>
                      ) : (
                        <button
                          type="button"
                          className="zt-btn-primary !px-3 !py-1.5 !text-xs"
                          disabled={
                            registeringId === t.id ||
                            registered ||
                            t.status === "finished" ||
                            t.status === "cancelled"
                          }
                          onClick={() => void handleRegister(t)}
                        >
                          {registeringId === t.id ? "…" : registered ? "Inscrito" : "Inscrever"}
                        </button>
                      )}
                      <Link
                        to={`/tournament/${t.id}`}
                        className="zt-btn-secondary !px-3 !py-1.5 !text-xs"
                      >
                        Detalhes
                      </Link>
                    </div>
                  </article>
                );
              })}
            </div>
          )}
        </div>
        </div>
      )}

      <p className="text-[11px] text-felt-400">
        Cash: mín. 2 na mesma mesa para iniciar a mão. No computador, duplo clique entra.
      </p>
    </div>
  );
}
