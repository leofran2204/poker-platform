import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { joinTable, listTables, listTournaments, registerTournament } from "@/api/client";
import type { TableResponse, TournamentInfoResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { deckTypeLabel, gameNameLabel } from "@/lib/gameLabels";
import { formatBrlFromCents } from "@/lib/money";
import { getWalletMode } from "@/lib/walletMode";

type LobbyTab = "cash" | "tournaments";
type StakeFilter = "all" | "nl025" | "sd025050" | "sdOmaha050";

const STAKE_OPTIONS: { id: StakeFilter; label: string }[] = [
  { id: "all", label: "Todos" },
  { id: "nl025", label: "NL 0,25/0,25" },
  { id: "sd025050", label: "SD 0,25/0,50" },
  { id: "sdOmaha050", label: "SD Omaha 0,50/0,50" },
];

function formatBuyInRange(min: number, max: number): string {
  if (min === max) return formatBrlFromCents(min);
  return `${formatBrlFromCents(min)}–${formatBrlFromCents(max)}`;
}

function occupancyPct(players: number, max: number): number {
  if (max <= 0) return 0;
  return Math.min(100, Math.round((players / max) * 100));
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
  const [hideFull, setHideFull] = useState(false);
  const [stake, setStake] = useState<StakeFilter>("all");
  const [joiningId, setJoiningId] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [registeringId, setRegisteringId] = useState<string | null>(null);
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
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao carregar lobby");
    } finally {
      setLoading(false);
    }
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
      const isOmaha = t.poker_variant === "short_deck_omaha";
      const isSd = t.poker_variant === "short_deck";
      if (stake === "nl025")
        return !isSd && !isOmaha && t.small_blind === 25 && t.big_blind === 25;
      if (stake === "sd025050")
        return isSd && t.small_blind === 25 && t.big_blind === 50;
      if (stake === "sdOmaha050")
        return isOmaha && t.small_blind === 50 && t.big_blind === 50;
      return true;
    });
  }, [tables, hideFull, stake]);

  useEffect(() => {
    if (selectedId && !filtered.some((t) => t.id === selectedId)) {
      setSelectedId(null);
    }
  }, [filtered, selectedId]);

  async function handleJoin(table: TableResponse) {
    if (table.players >= table.max_players) return;
    setJoiningId(table.id);
    setError(null);
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
    try {
      await registerTournament(t.id, getWalletMode());
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Falha na inscrição");
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
            className={tab === "cash" ? "zt-tab zt-tab-active" : "zt-tab"}
            onClick={() => setTab("cash")}
          >
            Cash
          </button>
          <button
            type="button"
            className={tab === "tournaments" ? "zt-tab zt-tab-active" : "zt-tab"}
            onClick={() => setTab("tournaments")}
          >
            Torneios
          </button>
        </div>
      </div>

      {error && (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">
          {error}
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
                NL 0,25/0,25 (R$25) · SD 0,25/0,50 (R$75) · SD Omaha 0,50/0,50 4-max (R$100) · auto 15s
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
            <div className="zt-table-wrap">
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
                      >
                        <td className="font-semibold text-cream">
                          {gameNameLabel(t, "cash")}
                        </td>
                        <td>
                          <span
                            className={
                              t.poker_variant === "short_deck" ||
                              t.poker_variant === "short_deck_omaha"
                                ? "zt-chip zt-chip-accent"
                                : "zt-chip"
                            }
                          >
                            {deckTypeLabel(t)}
                          </span>
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
                            disabled={full || joiningId === t.id}
                            onClick={(e) => {
                              e.stopPropagation();
                              void handleJoin(t);
                            }}
                          >
                            {full ? "Cheia" : joiningId === t.id ? "…" : "Entrar"}
                          </button>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      ) : (
        <div className="zt-panel overflow-hidden">
          <div className="zt-lobby-toolbar">
            <div className="min-w-0 flex-1">
              <div className="text-xs font-bold uppercase tracking-wider text-gold-bright">
                Torneios
                <span className="ml-2 font-mono text-felt-300">({tournaments.length})</span>
              </div>
              <p className="text-[11px] text-felt-400">
                Hold’em tradicional, freeroll tradicional (Mesa Final Short Deck) e Omaha 4 cartas Short Deck · Big Blind Ante desde o nível 1 · 26 níveis · mãos MTT em breve
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
            <div className="zt-table-wrap">
              <table className="zt-lobby-table min-w-[68rem] table-fixed">
                <colgroup>
                  <col className="w-[22%]" />
                  <col className="w-[20%]" />
                  <col className="w-[10%]" />
                  <col className="w-[12%]" />
                  <col className="w-[9%]" />
                  <col className="w-[15%]" />
                  <col className="w-[12%]" />
                </colgroup>
                <thead>
                  <tr>
                    <th>Nome</th>
                    <th>Tipo</th>
                    <th>Buy-in</th>
                    <th>GTD</th>
                    <th>Inscritos</th>
                    <th>Rebuy</th>
                    <th className="text-right">Ação</th>
                  </tr>
                </thead>
                <tbody>
                  {tournaments.map((t) => (
                    <tr
                      key={t.id}
                      className="!cursor-pointer"
                      onDoubleClick={() => navigate(`/tournament/${t.id}`)}
                    >
                      <td className="font-semibold text-cream">
                        <Link to={`/tournament/${t.id}`} className="hover:text-gold-bright">
                          {gameNameLabel(t, "tournament")}
                        </Link>
                      </td>
                      <td>
                        <span
                          className={
                            t.poker_variant === "short_deck" ||
                            t.poker_variant === "short_deck_omaha" ||
                            t.final_table_variant === "short_deck"
                              ? "zt-chip zt-chip-accent"
                              : "zt-chip"
                          }
                        >
                          {deckTypeLabel(t)}
                        </span>
                        <span className="zt-chip ml-1">{t.table_max_players}-max</span>
                      </td>
                      <td className="font-mono text-gold-soft">
                        {t.is_freeroll ? "Grátis" : formatBrlFromCents(t.buy_in)}
                      </td>
                      <td className="font-mono text-felt-200">
                        {formatBrlFromCents(t.guaranteed_prize)}
                      </td>
                      <td className="font-mono text-cream">
                        {t.registered_players}/{t.max_players}
                      </td>
                      <td className="text-xs text-felt-300">
                        {t.allow_rebuy
                          ? `${formatBrlFromCents(t.rebuy_cost)} → ${t.rebuy_chips.toLocaleString("pt-BR")} (≤niv.${t.rebuy_max_level})`
                          : "—"}
                      </td>
                      <td className="space-x-1 text-right">
                        <Link
                          to={`/tournament/${t.id}`}
                          className="zt-btn-secondary inline-flex !px-2.5 !py-1 !text-xs"
                          onClick={(e) => e.stopPropagation()}
                        >
                          Ver
                        </Link>
                        <button
                          type="button"
                          className="zt-btn-primary !px-2.5 !py-1 !text-xs"
                          disabled={registeringId === t.id}
                          onClick={(e) => {
                            e.stopPropagation();
                            void handleRegister(t);
                          }}
                        >
                          {registeringId === t.id ? "…" : "Inscrever"}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      <p className="text-[11px] text-felt-400">
        Cash: clique seleciona · duplo clique entra · mín. 2 na mesma mesa para iniciar mão
      </p>
    </div>
  );
}
