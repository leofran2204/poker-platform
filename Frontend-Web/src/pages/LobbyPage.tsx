import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router";
import { motion } from "motion/react";
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
import { layoutForCap } from "@/lib/cards";
import { formatCountdown, liveTableIds, nextStartEpoch } from "@/lib/countdown";
import { deckTypeLabel, gameNameLabel } from "@/lib/gameLabels";
import { formatBrlFromCents } from "@/lib/money";
import { getWalletMode } from "@/lib/walletMode";

type LobbyTab = "cash" | "tournaments";
type StakeFilter = "all" | "nl025" | "nl075150" | "shortdeck050" | "omaha050" | "pineapple050";

const STAKE_OPTIONS: { id: StakeFilter; label: string }[] = [
  { id: "all", label: "Todos" },
  { id: "nl025", label: "NL 0,25/0,25" },
  { id: "nl075150", label: "NL 0,75/1,50" },
  { id: "shortdeck050", label: "Short Deck 0,50/0,50" },
  { id: "omaha050", label: "Omaha 0,50/0,50" },
  { id: "pineapple050", label: "Brazilian Pineapple 0,50/0,50" },
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

/** Identidade visual de cada modalidade no lobby. */
function variantTheme(variant?: string | null): { tone: string; suit: string; tag: string } {
  if (variant === "pineapple_draw") return { tone: "pineapple", suit: "♥", tag: "Criado no Brasil" };
  if (variant === "omaha" || variant === "short_deck_omaha") return { tone: "omaha", suit: "♣", tag: "4 cartas" };
  if (variant === "short_deck") return { tone: "shortdeck", suit: "♦", tag: "36 cartas" };
  return { tone: "holdem", suit: "♠", tag: "Clássico" };
}

/** Mesa em miniatura: o oval com um ponto por lugar, aceso quando ocupado. */
function MiniTable({ players, max }: { players: number; max: number }) {
  const seats = layoutForCap(max).slice(0, max);
  return (
    <div className="zt-mini-table" role="img" aria-label={`${players} de ${max} lugares ocupados`}>
      {seats.map((seat, index) => (
        <span
          key={index}
          className={index < players ? "zt-mini-seat on" : "zt-mini-seat"}
          style={{ top: `${seat.top}%`, left: `${seat.left}%` }}
        />
      ))}
      <span className="zt-mini-count">
        {players}/{max}
      </span>
    </div>
  );
}

const cardMotion = {
  initial: { opacity: 0, y: 14 },
  animate: { opacity: 1, y: 0 },
  whileHover: { y: -4 },
};

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
      const isPineapple = t.poker_variant === "pineapple_draw";
      const isHoldem = !t.poker_variant || t.poker_variant === "holdem";
      if (stake === "nl025")
        return isHoldem && t.small_blind === 25 && t.big_blind === 25;
      if (stake === "nl075150")
        return isHoldem && t.small_blind === 75 && t.big_blind === 150;
      if (stake === "shortdeck050")
        return t.poker_variant === "short_deck" && t.small_blind === 50 && t.big_blind === 50;
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

  const seated = tables.reduce((sum, t) => sum + t.players, 0);

  return (
    <div className="zt-lobby">
      <header className="zt-lobby-head">
        <div>
          <p className="zt-lobby-eyebrow">
            <span className={walletMode === "real" ? "zt-mode-dot real" : "zt-mode-dot"} aria-hidden />
            {walletModeLabel}
          </p>
          <h1>Lobby</h1>
          <p className="zt-lobby-sub">
            {walletMode === "real" ? (
              <>
                Só saldo real nestas mesas. Sem saldo? Vá em{" "}
                <Link to="/wallet" className="underline text-gold-soft">
                  Carteira
                </Link>
                .
              </>
            ) : (
              <>Fichas de diversão, renovadas todo dia. Não têm valor real e não entram no Jogo Real.</>
            )}
          </p>
        </div>
        <div className="zt-lobby-stats" aria-label="Movimento agora">
          <div>
            <strong>{tables.length}</strong>
            <span>mesas</span>
          </div>
          <div>
            <strong>{seated}</strong>
            <span>jogando</span>
          </div>
          <div>
            <strong>{tournaments.length}</strong>
            <span>torneios</span>
          </div>
        </div>
      </header>

      <div className="zt-lobby-tabs" role="tablist" aria-label="Tipo de jogo">
        {([["cash", "Mesas cash"], ["tournaments", "Torneios"]] as [LobbyTab, string][]).map(([id, label]) => (
          <button
            key={id}
            type="button"
            role="tab"
            aria-selected={tab === id}
            className={tab === id ? "zt-lobby-tab on" : "zt-lobby-tab"}
            onClick={() => setTab(id)}
          >
            {label}
            {tab === id && <motion.span layoutId="lobby-tab-line" className="zt-lobby-tab-line" />}
          </button>
        ))}
        <button type="button" className="zt-lobby-refresh" onClick={() => void load()} disabled={loading}>
          {loading ? "Atualizando…" : "Atualizar"}
        </button>
      </div>

      {error && (
        <p className="rounded-sm border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200" role="alert">
          {error}
        </p>
      )}
      {info && (
        <p className="rounded-sm border border-emerald-800 bg-emerald-950/30 px-3 py-2 text-sm text-emerald-100" role="status">
          {info}
        </p>
      )}

      {tab === "cash" ? (
        <section aria-label="Mesas cash">
          <div className="zt-lobby-filters">
            <div className="zt-lobby-chips" role="tablist" aria-label="Filtro de jogo">
              {STAKE_OPTIONS.map((opt) => (
                <button
                  key={opt.id}
                  type="button"
                  role="tab"
                  aria-selected={stake === opt.id}
                  className={stake === opt.id ? "zt-filter-chip on" : "zt-filter-chip"}
                  onClick={() => setStake(opt.id)}
                >
                  {opt.label}
                </button>
              ))}
            </div>
            <label className="flex items-center gap-2 text-xs text-cream">
              <input type="checkbox" className="accent-gold" checked={hideFull} onChange={(e) => setHideFull(e.target.checked)} />
              Ocultar cheias
            </label>
          </div>

          {loading && tables.length === 0 ? (
            <div className="flex items-center justify-center gap-3 p-10 text-sm text-felt-300">
              <span className="zt-spinner" aria-hidden />
              Carregando mesas…
            </div>
          ) : filtered.length === 0 ? (
            <p className="p-8 text-center text-sm text-felt-300">Nenhuma mesa com esses filtros.</p>
          ) : (
            <div className="zt-lobby-grid">
              {filtered.map((t, index) => {
                const full = t.players >= t.max_players;
                const theme = variantTheme(t.poker_variant);
                return (
                  <motion.article
                    key={t.id}
                    {...cardMotion}
                    transition={{ duration: 0.3, delay: Math.min(index, 8) * 0.04 }}
                    className={`zt-table-card ${theme.tone} ${full ? "full" : ""} ${selectedId === t.id ? "selected" : ""}`}
                    tabIndex={0}
                    onClick={() => setSelectedId(t.id)}
                    onDoubleClick={() => {
                      if (!full) void handleJoin(t);
                    }}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !full) void handleJoin(t);
                    }}
                  >
                    <div className="zt-table-card-top">
                      <span className="zt-variant-badge">
                        <span aria-hidden>{theme.suit}</span> {theme.tag}
                      </span>
                      {t.players > 0 && !full && <span className="zt-live-dot">ao vivo</span>}
                      {full && <span className="zt-full-tag">cheia</span>}
                    </div>
                    <h2>{gameNameLabel(t, "cash")}</h2>
                    <MiniTable players={t.players} max={t.max_players} />
                    <dl className="zt-table-card-facts">
                      <div>
                        <dt>Blinds</dt>
                        <dd>
                          {formatBrlFromCents(t.small_blind)}/{formatBrlFromCents(t.big_blind)}
                        </dd>
                      </div>
                      <div>
                        <dt>Entrada</dt>
                        <dd>{formatBuyInRange(t.min_buy_in, t.max_buy_in)}</dd>
                      </div>
                      <div>
                        <dt>Baralho</dt>
                        <dd>{deckTypeLabel(t)}</dd>
                      </div>
                    </dl>
                    <button
                      type="button"
                      className={full ? "zt-btn-secondary w-full" : "zt-btn-primary w-full"}
                      disabled={joiningId === t.id}
                      onClick={(e) => {
                        e.stopPropagation();
                        if (full) void handleWaitlist(t);
                        else void handleJoin(t);
                      }}
                    >
                      {joiningId === t.id ? "Entrando…" : full ? "Entrar na fila" : "Sentar na mesa"}
                    </button>
                  </motion.article>
                );
              })}
            </div>
          )}
          <p className="mt-3 text-[11px] text-felt-400">
            A mão começa com pelo menos 2 jogadores na mesma mesa. Ação automática em 15 segundos. No computador, duplo clique no cartão
            também entra.
          </p>
        </section>
      ) : (
        <section aria-label="Torneios">
          <div className="zt-next-tourney">
            <span>Próximo torneio</span>
            <strong>{soonestStart ? formatCountdown(soonestStart, now) : "sem horário marcado"}</strong>
            <small>Data e horário definidos pela casa · começa sozinho com 5 ou mais inscritos · taxa de 15% por cima do buy-in</small>
          </div>

          {loading && tournaments.length === 0 ? (
            <div className="flex items-center justify-center gap-3 p-10 text-sm text-felt-300">
              <span className="zt-spinner" aria-hidden />
              Carregando torneios…
            </div>
          ) : tournaments.length === 0 ? (
            <p className="p-8 text-center text-sm text-felt-300">Nenhum torneio aberto.</p>
          ) : (
            <div className="zt-lobby-grid">
              {tournaments.map((t, index) => {
                const liveTables = liveTableIds(t);
                const registered = registeredIds.has(t.id);
                const live = t.status === "running" || t.gameplay_ready;
                const theme = variantTheme(t.poker_variant);
                const pct = occupancyPct(t.registered_players, t.max_players);
                return (
                  <motion.article
                    key={t.id}
                    {...cardMotion}
                    transition={{ duration: 0.3, delay: Math.min(index, 8) * 0.04 }}
                    className={`zt-table-card tourney ${theme.tone} ${registered ? "selected" : ""}`}
                  >
                    <div className="zt-table-card-top">
                      <span className="zt-variant-badge">
                        <span aria-hidden>{theme.suit}</span> {theme.tag}
                      </span>
                      {live ? <span className="zt-live-dot">ao vivo</span> : registered ? <span className="zt-full-tag ok">inscrito</span> : null}
                    </div>
                    <h2>
                      <Link to={`/tournament/${t.id}`}>{gameNameLabel(t, "tournament")}</Link>
                    </h2>
                    <p className="zt-tourney-when">
                      {t.scheduled_start_at ? formatCountdown(t.scheduled_start_at, now) : "Aguardando horário"}
                    </p>
                    <div className="zt-tourney-prize">
                      <span>{t.guaranteed_prize > 0 ? "Garantido" : "Buy-in"}</span>
                      <strong>{t.guaranteed_prize > 0 ? formatBrlFromCents(t.guaranteed_prize) : buyInLabel(t)}</strong>
                    </div>
                    <dl className="zt-table-card-facts">
                      <div>
                        <dt>Buy-in</dt>
                        <dd>{buyInLabel(t)}</dd>
                      </div>
                      <div>
                        <dt>Mesa</dt>
                        <dd>{t.table_max_players} lugares</dd>
                      </div>
                      <div>
                        <dt>Rebuy</dt>
                        <dd>{t.allow_rebuy ? `até nív. ${t.rebuy_max_level}` : "não"}</dd>
                      </div>
                    </dl>
                    <div className="zt-tourney-fill" aria-label={`${t.registered_players} de ${t.max_players} inscritos`}>
                      <div className="zt-tourney-fill-track">
                        <motion.div className="zt-tourney-fill-bar" initial={{ width: 0 }} animate={{ width: `${pct}%` }} transition={{ duration: 0.6 }} />
                      </div>
                      <span>
                        {t.registered_players}/{t.max_players} inscritos
                      </span>
                    </div>
                    <div className="mt-auto flex gap-2">
                      {live && liveTables.length > 0 && registered ? (
                        <Link to={`/table/${liveTables[0]}`} className="zt-btn-primary flex-1 text-center">
                          Sentar
                        </Link>
                      ) : registered && t.status === "registering" ? (
                        <button type="button" className="zt-btn-secondary flex-1" disabled={registeringId === t.id} onClick={() => void handleUnregister(t)}>
                          {registeringId === t.id ? "…" : "Cancelar inscrição"}
                        </button>
                      ) : (
                        <button
                          type="button"
                          className="zt-btn-primary flex-1"
                          disabled={registeringId === t.id || registered || t.status === "finished" || t.status === "cancelled"}
                          onClick={() => void handleRegister(t)}
                        >
                          {registeringId === t.id ? "…" : registered ? "Inscrito" : "Inscrever"}
                        </button>
                      )}
                      <Link to={`/tournament/${t.id}`} className="zt-btn-secondary">
                        Detalhes
                      </Link>
                    </div>
                  </motion.article>
                );
              })}
            </div>
          )}
        </section>
      )}
    </div>
  );
}
