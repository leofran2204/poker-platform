import { FormEvent, useCallback, useEffect, useState } from "react";
import {
  ensureBotsPool,
  fetchBotsStatus,
  listTables,
  startBots,
  stopBots,
} from "@/api/client";
import type { BotTableStatus } from "@/api/client";
import type { TableResponse } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

export function AdminBotsPage() {
  const [tables, setTables] = useState<TableResponse[]>([]);
  const [status, setStatus] = useState<{ pool_total: number; pool_free: number; strategies: string[]; tables: BotTableStatus[] } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      const [lobby, st] = await Promise.all([listTables("play"), fetchBotsStatus()]);
      setTables(lobby.filter((t) => t.game_type === "cash"));
      setStatus(st);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }, []);

  useEffect(() => {
    void load();
    const t = window.setInterval(() => void load(), 5_000);
    return () => window.clearInterval(t);
  }, [load]);

  async function onEnsurePool() {
    setBusy(true);
    setError(null);
    try {
      const r = await ensureBotsPool();
      setMsg(`Elenco pronto: ${r.total}/${r.pool_size} bots (novos: ${r.created})`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    } finally {
      setBusy(false);
    }
  }

  async function onStart(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const tableId = String(fd.get("table") || "");
    const count = Number(fd.get("count") || 6);
    const strategy = String(fd.get("strategy") || "lag_v1");
    if (!tableId) return;
    setBusy(true);
    setError(null);
    try {
      const r = await startBots(tableId, count, strategy);
      setMsg(`${r.bots.length} bots ligados em ${r.table_name}`);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro ao ligar");
    } finally {
      setBusy(false);
    }
  }

  async function onStop(tableId: string, tableName: string) {
    if (!window.confirm(`Desligar bots de ${tableName}?`)) return;
    setBusy(true);
    setError(null);
    try {
      const r = await stopBots(tableId);
      setMsg(`Bots desligados (${formatBrlFromCents(r.refunded_chips)} devolvidos)`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold text-gold-bright">Bots da casa</h1>
      <p className="text-sm text-felt-300">
        Elenco de 72 bots para testes e futuro coach. Escolha a mesa e quantos ligar;
        eles jogam sozinhos até restar 1. Só mesas play money.
      </p>
      {error ? (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">{error}</p>
      ) : null}
      {msg ? (
        <p className="rounded border border-felt-700 bg-felt-800 px-3 py-2 text-sm text-felt-200">{msg}</p>
      ) : null}

      <div className="zt-panel p-4 text-sm">
        <p>
          Elenco: <span className="font-mono text-gold-bright">{status?.pool_total ?? "…"}/72</span>{" "}
          · livres: <span className="font-mono">{status?.pool_free ?? "…"}</span>
        </p>
        <button className="zt-btn-secondary mt-2" disabled={busy} onClick={onEnsurePool}>
          Preparar elenco (criar os 72)
        </button>
      </div>

      <form onSubmit={onStart} className="zt-panel space-y-3 p-4">
        <div className="zt-panel-title">Ligar bots</div>
        <div className="flex flex-wrap gap-3">
          <label className="flex flex-col gap-1 text-sm">
            Mesa
            <select name="table" className="zt-input" defaultValue="">
              <option value="" disabled>Escolha…</option>
              {tables.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name} ({t.players}/{t.max_players})
                </option>
              ))}
            </select>
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Quantos
            <input name="count" type="number" min={1} max={9} defaultValue={6} className="zt-input w-20" />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Estratégia
            <select name="strategy" className="zt-input" defaultValue="lag_v2">
              {(status?.strategies ?? ["lag_v2", "lag_v1"]).map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </select>
          </label>
        </div>
        <button type="submit" className="zt-btn-primary" disabled={busy}>
          Ligar bots
        </button>
      </form>

      {(status?.tables ?? []).map((t) => (
        <div key={t.table_id} className="zt-panel overflow-hidden">
          <div className="zt-panel-title flex flex-wrap items-center justify-between gap-2">
            <span>
              {t.table_name} · {t.variant} · {t.bots_alive}/{t.bots_total} vivos · {t.hands_played} mãos
              {t.leader ? ` · líder ${t.leader.username} (${formatBrlFromCents(t.leader.chips)})` : ""}
            </span>
            <button className="zt-btn-secondary !text-xs" disabled={busy} onClick={() => onStop(t.table_id, t.table_name)}>
              Desligar
            </button>
          </div>
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="text-xs uppercase text-felt-400">
                <th className="px-3 py-2">Bot</th>
                <th className="px-3 py-2">Fichas</th>
                <th className="px-3 py-2">Estado</th>
              </tr>
            </thead>
            <tbody>
              {t.seats.map((s) => (
                <tr key={s.username} className="border-t border-felt-700">
                  <td className="px-3 py-2 font-mono">{s.username}</td>
                  <td className="px-3 py-2 font-mono">{formatBrlFromCents(s.chips)}</td>
                  <td className="px-3 py-2">{s.chips > 0 ? "vivo" : "quebrado"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ))}
      {(status?.tables.length ?? 0) === 0 ? (
        <p className="text-sm text-felt-400">Nenhum deploy ativo.</p>
      ) : null}
    </div>
  );
}
