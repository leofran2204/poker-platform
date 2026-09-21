import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { getTournament, registerTournament, unregisterTournament, fetchTournamentRegistration } from "@/api/client";
import type { TournamentInfoResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatCountdown, liveTableIds } from "@/lib/countdown";
import { deckTypeLabel, gameNameLabel, tournamentStatusLabel } from "@/lib/gameLabels";
import { formatBrlFromCents } from "@/lib/money";
import { getWalletMode } from "@/lib/walletMode";

export function TournamentPage() {
  const { id = "" } = useParams();
  const [info, setInfo] = useState<TournamentInfoResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [registeredMsg, setRegisteredMsg] = useState<string | null>(null);
  const [registered, setRegistered] = useState(false);
  const [now, setNow] = useState(() => Date.now());

  const load = useCallback(async () => {
    if (!id) return;
    setError(null);
    try {
      setInfo(await getTournament(id));
      if (isAuthenticated()) {
        try {
          setRegistered((await fetchTournamentRegistration(id)).registered);
        } catch {
          setRegistered(false);
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao carregar torneio");
    }
  }, [id]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    const tick = window.setInterval(() => setNow(Date.now()), 1_000);
    return () => window.clearInterval(tick);
  }, []);

  async function handleRegister() {
    if (!isAuthenticated()) {
      setError("Faça login para se inscrever.");
      return;
    }
    setBusy(true);
    setError(null);
    setRegisteredMsg(null);
    try {
      const res = await registerTournament(id, getWalletMode());
      setRegisteredMsg(
        `Inscrito com ${res.stack.toLocaleString("pt-BR")} fichas. As 3 mesas ligam sozinhas com ≥5 inscritos — boa sorte!`,
      );
      setRegistered(true);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Falha na inscrição");
    } finally {
      setBusy(false);
    }
  }

  async function handleUnregister() {
    if (!window.confirm("Cancelar inscrição? Devolve buy-in + taxa de 15%. Só vale antes de começar.")) return;
    setBusy(true);
    setError(null);
    setRegisteredMsg(null);
    try {
      const res = await unregisterTournament(id);
      setRegisteredMsg(
        `Inscrição cancelada. Devolvidos ${formatBrlFromCents(res.refunded_buy_in_cents + res.refunded_fee_cents)}.`,
      );
      setRegistered(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Falha ao cancelar");
    } finally {
      setBusy(false);
    }
  }

  if (!info && !error) {
    return (
      <div className="flex items-center gap-3 p-8 text-sm text-felt-300">
        <span className="zt-spinner" aria-hidden />
        Carregando torneio…
      </div>
    );
  }

  if (!info) {
    return (
      <div className="zt-panel p-6">
        <p className="text-red-200">{error ?? "Torneio não encontrado"}</p>
        <Link to="/lobby" className="zt-btn-secondary mt-4 inline-flex !text-xs">
          Voltar ao lobby
        </Link>
      </div>
    );
  }

  const tables = liveTableIds(info);

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <p className="text-[11px] uppercase tracking-wider text-felt-400">
            <Link to="/lobby" className="hover:text-gold-soft">
              Lobby
            </Link>{" "}
            / Torneio
          </p>
          <h1 className="text-xl font-bold text-gold-bright">
            {gameNameLabel(info, "tournament")}
          </h1>
          <p className="mt-1">
            <span
              className={
                deckTypeLabel(info) === "Short Deck"
                  ? "zt-chip zt-chip-accent"
                  : "zt-chip"
              }
            >
              {deckTypeLabel(info)}
            </span>
            <span className="zt-chip ml-1">{info.table_max_players}-max</span>
          </p>
        </div>
        {tables.length > 0 && (info.status === "running" || info.gameplay_ready) ? (
          <Link
            to={`/table/${tables[0]}`}
            className="zt-btn-primary !px-3 !py-1.5 !text-xs"
          >
            Sentar na mesa
          </Link>
        ) : (
          <button
            type="button"
            className="zt-btn-primary !px-3 !py-1.5 !text-xs"
            disabled={busy || info.status === "finished" || info.status === "cancelled" || registered}
            onClick={() => void handleRegister()}
          >
            {busy
              ? "…"
              : registered
                ? "Inscrito"
                : info.is_freeroll
                  ? "Inscrever (grátis)"
                  : `Inscrever (${formatBrlFromCents(info.buy_in + (info.fee_cents ?? 0))})`}
          </button>
        )}
        {registered && info.status === "registering" ? (
          <button
            type="button"
            className="zt-btn-secondary !px-3 !py-1.5 !text-xs"
            disabled={busy}
            onClick={() => void handleUnregister()}
          >
            Cancelar inscrição
          </button>
        ) : null}
      </div>

      {info.scheduled_start_at ? (
        <div className="rounded border border-gold/40 bg-felt-950/70 px-4 py-3">
          <p className="text-xs font-bold uppercase tracking-wider text-gold-soft">Relógio</p>
          <p className="mt-1 text-lg font-semibold text-cream">
            {formatCountdown(info.scheduled_start_at, now)}
          </p>
          <p className="mt-1 text-xs text-felt-300">
            {new Date(info.scheduled_start_at * 1000).toLocaleString("pt-BR", {
              timeZone: "America/Sao_Paulo",
            })}{" "}
            America/Sao_Paulo · auto-start com ≥{info.auto_start_min_players ?? 5}
            {info.status === "registering" &&
            info.registered_players < (info.auto_start_min_players ?? 5)
              ? ` · faltam ${(info.auto_start_min_players ?? 5) - info.registered_players}`
              : ""}
          </p>
        </div>
      ) : null}

      {!info.gameplay_ready ? (
        <div className="rounded border border-amber-700/60 bg-amber-950/30 px-3 py-2 text-xs text-amber-100">
          {info.status === "registering"
            ? "Aguardando início — as 3 mesas ligam sozinhas com ≥5 inscritos."
            : "Mesa ao vivo indisponível no momento — tente recarregar."}
        </div>
      ) : (
        <div className="rounded border border-emerald-800/60 bg-emerald-950/30 px-3 py-2 text-xs text-emerald-100">
          Mãos ao vivo nas mesas do torneio (mesmo WebSocket do cash). Rebalance, mesa final e
          premiação seguem o coordenador.
          {(info.live_table_ids?.length ? info.live_table_ids : info.live_table_id ? [info.live_table_id] : []).length >
          0 ? (
            <span className="mt-1 block space-x-2">
              {(info.live_table_ids?.length
                ? info.live_table_ids
                : info.live_table_id
                  ? [info.live_table_id]
                  : []
              ).map((tableId, index, all) => (
                <Link key={tableId} to={`/table/${tableId}`} className="font-semibold underline">
                  {all.length > 1 ? `Mesa ${index + 1}` : "Ir para a mesa"}
                </Link>
              ))}
            </span>
          ) : null}
        </div>
      )}

      {error && (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">
          {error}
        </p>
      )}
      {registeredMsg && (
        <p className="rounded border border-emerald-800 bg-emerald-950/30 px-3 py-2 text-sm text-emerald-100">
          {registeredMsg}
        </p>
      )}

      <div className="zt-panel overflow-hidden">
        <div className="zt-panel-title">Resumo</div>
        <dl className="grid gap-2 p-4 text-sm sm:grid-cols-2">
          <div>
            <dt className="text-xs uppercase text-felt-400">Status</dt>
            <dd className="text-cream">{tournamentStatusLabel(info.status)}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase text-felt-400">Buy-in</dt>
            <dd className="font-mono text-gold-soft">
              {info.is_freeroll ? "Freeroll" : formatBrlFromCents(info.buy_in)}
            </dd>
          </div>
          {!info.is_freeroll && (info.fee_cents ?? 0) > 0 ? (
            <div>
              <dt className="text-xs uppercase text-felt-400">Taxa (15% p/ a rede)</dt>
              <dd className="font-mono text-cream">{formatBrlFromCents(info.fee_cents ?? 0)}</dd>
            </div>
          ) : null}
          <div>
            <dt className="text-xs uppercase text-felt-400">Premiação garantida / premiação atual</dt>
            <dd className="font-mono text-cream">
              {formatBrlFromCents(info.guaranteed_prize)} / {formatBrlFromCents(info.prize_pool)}
            </dd>
          </div>
          <div>
            <dt className="text-xs uppercase text-felt-400">Stack inicial</dt>
            <dd className="font-mono text-cream">{info.starting_stack.toLocaleString("pt-BR")}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase text-felt-400">Inscritos</dt>
            <dd className="font-mono text-cream">
              {info.registered_players}/{info.max_players}
            </dd>
          </div>
          <div>
            <dt className="text-xs uppercase text-felt-400">Formato das mesas</dt>
            <dd className="font-mono text-cream">
              {info.table_max_players}-max
            </dd>
          </div>
          <div>
            <dt className="text-xs uppercase text-felt-400">Reentrada</dt>
            <dd className="text-felt-200">
              {info.allow_rebuy
                ? `1× até nível ${info.rebuy_max_level}: ${formatBrlFromCents(info.rebuy_cost)} → ${info.rebuy_chips.toLocaleString("pt-BR")} fichas${
                    info.rebuy_stack_threshold > 0
                      ? ` (stack ≤ ${info.rebuy_stack_threshold.toLocaleString("pt-BR")})`
                      : " após eliminação"
                  }`
                : "Não"}
            </dd>
          </div>
        </dl>
      </div>

      <div className="zt-panel overflow-hidden">
        <div className="zt-panel-title">Estrutura de blinds com Big Blind Ante (5 min)</div>
        <div className="zt-table-wrap">
          <table className="zt-lobby-table">
            <thead>
              <tr>
                <th>Nível</th>
                <th>Blinds</th>
                <th>Big Blind Ante</th>
                <th>Min</th>
              </tr>
            </thead>
            <tbody>
              {info.blind_levels.map((b) => (
                <tr key={b.level} className="!cursor-default">
                  <td className="font-mono text-cream">{b.level}</td>
                  <td className="font-mono text-gold-soft">
                    {b.small_blind}/{b.big_blind}
                  </td>
                  <td className="font-mono text-felt-200">{b.ante || "—"}</td>
                  <td className="font-mono text-felt-200">{b.duration_minutes}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
