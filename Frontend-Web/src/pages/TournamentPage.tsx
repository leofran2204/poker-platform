import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { getTournament, registerTournament } from "@/api/client";
import type { TournamentInfoResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { deckTypeLabel, gameNameLabel } from "@/lib/gameLabels";
import { formatBrlFromCents } from "@/lib/money";
import { getWalletMode } from "@/lib/walletMode";

export function TournamentPage() {
  const { id = "" } = useParams();
  const [info, setInfo] = useState<TournamentInfoResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [registeredMsg, setRegisteredMsg] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!id) return;
    setError(null);
    try {
      setInfo(await getTournament(id));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao carregar torneio");
    }
  }, [id]);

  useEffect(() => {
    void load();
  }, [load]);

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
        `Inscrito com ${res.stack.toLocaleString("pt-BR")} fichas. Gameplay MTT em breve.`,
      );
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Falha na inscrição");
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
                info.poker_variant === "short_deck" ||
                info.poker_variant === "short_deck_omaha"
                  ? "zt-chip zt-chip-accent"
                  : "zt-chip"
              }
            >
              {deckTypeLabel(info)}
            </span>
            <span className="zt-chip ml-1">{info.table_max_players} jogadores</span>
          </p>
        </div>
        <button
          type="button"
          className="zt-btn-primary !px-3 !py-1.5 !text-xs"
          disabled={busy || info.status === "finished"}
          onClick={() => void handleRegister()}
        >
          {busy ? "…" : info.is_freeroll ? "Inscrever (grátis)" : `Inscrever (${formatBrlFromCents(info.buy_in)})`}
        </button>
      </div>

      <div className="rounded border border-amber-700/60 bg-amber-950/30 px-3 py-2 text-xs text-amber-100">
        Gameplay de torneio ainda não está ligado à mesa ao vivo — inscrição e configuração já
        disponíveis. Em breve você joga as mãos MTT aqui.
      </div>

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
            <dd className="text-cream">{info.status}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase text-felt-400">Buy-in</dt>
            <dd className="font-mono text-gold-soft">
              {info.is_freeroll ? "Freeroll" : formatBrlFromCents(info.buy_in)}
            </dd>
          </div>
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
            <dd className="font-mono text-cream">{info.table_max_players}-max</dd>
          </div>
          <div>
            <dt className="text-xs uppercase text-felt-400">Reentrada</dt>
            <dd className="text-felt-200">
              {info.allow_rebuy
                ? `1× até nível ${info.rebuy_max_level}: ${formatBrlFromCents(info.rebuy_cost)} → ${info.rebuy_chips.toLocaleString("pt-BR")} fichas (stack ≤ ${info.rebuy_stack_threshold.toLocaleString("pt-BR")})`
                : "Não"}
            </dd>
          </div>
        </dl>
      </div>

      <div className="zt-panel overflow-hidden">
        <div className="zt-panel-title">Estrutura de blinds (5 min)</div>
        <div className="zt-table-wrap">
          <table className="zt-lobby-table">
            <thead>
              <tr>
                <th>Nível</th>
                <th>Blinds</th>
                <th>Ante</th>
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
