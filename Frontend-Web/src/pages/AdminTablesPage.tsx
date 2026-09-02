import { FormEvent, Fragment, useCallback, useEffect, useState } from "react";
import {
  createAdminCashTable,
  listAdminTables,
  patchAdminTableStatus,
} from "@/api/client";
import type { AdminTableListItem, AdminTableSeat } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

export function AdminTablesPage() {
  const [tables, setTables] = useState<AdminTableListItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      setTables(await listAdminTables());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }, []);

  useEffect(() => {
    void load();
    const t = window.setInterval(() => void load(), 8_000);
    return () => window.clearInterval(t);
  }, [load]);

  async function setStatus(id: string, status: string) {
    if (!window.confirm(`Definir mesa como ${status}?`)) return;
    try {
      await patchAdminTableStatus(id, status);
      setMsg(`Mesa ${status}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }

  async function onCreate(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const name = String(fd.get("name") || "").trim();
    const sb = Math.round(Number(fd.get("sb")) * 100);
    const bb = Math.round(Number(fd.get("bb")) * 100);
    const min = Math.round(Number(fd.get("min")) * 100);
    const max = Math.round(Number(fd.get("max")) * 100);
    const maxPlayers = Number(fd.get("max_players") || 9);
    try {
      await createAdminCashTable({
        name,
        small_blind: sb,
        big_blind: bb,
        min_buy_in: min,
        max_buy_in: max,
        max_players: maxPlayers,
        rake_basis_points: 500,
        rake_cap: Math.max(bb * 5, 100),
      });
      setMsg(`Mesa criada: ${name}`);
      e.currentTarget.reset();
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro ao criar");
    }
  }

  return (
    <div className="space-y-4">
      {error && <p className="text-sm text-red-200">{error}</p>}
      {msg && <p className="text-sm text-emerald-200">{msg}</p>}

      <form className="zt-panel grid gap-2 p-4 sm:grid-cols-3 lg:grid-cols-6" onSubmit={(e) => void onCreate(e)}>
        <div className="sm:col-span-2">
          <label className="zt-label">Nome</label>
          <input name="name" className="zt-input" required maxLength={100} />
        </div>
        <div>
          <label className="zt-label">SB (R$)</label>
          <input name="sb" type="number" step="0.01" className="zt-input" required defaultValue={0.25} />
        </div>
        <div>
          <label className="zt-label">BB (R$)</label>
          <input name="bb" type="number" step="0.01" className="zt-input" required defaultValue={0.5} />
        </div>
        <div>
          <label className="zt-label">Min frente</label>
          <input name="min" type="number" step="0.01" className="zt-input" required defaultValue={25} />
        </div>
        <div>
          <label className="zt-label">Max</label>
          <input name="max" type="number" step="0.01" className="zt-input" required defaultValue={50} />
        </div>
        <div>
          <label className="zt-label">Max players</label>
          <input name="max_players" type="number" min={2} max={9} className="zt-input" defaultValue={9} />
        </div>
        <div className="flex items-end sm:col-span-2">
          <button type="submit" className="zt-btn-primary !text-xs">
            Criar mesa cash
          </button>
        </div>
      </form>

      <div className="zt-table-wrap zt-panel overflow-hidden">
        <table className="zt-lobby-table">
          <thead>
            <tr>
              <th>Nome</th>
              <th>Status</th>
              <th>Blinds</th>
              <th>Frente</th>
              <th>Players</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            {tables.map((t) => (
              <Fragment key={t.id}>
                <tr className="!cursor-default">
                  <td className="font-semibold text-cream">
                    {t.name}
                    <div className="text-[10px] text-felt-400">{t.visibility}</div>
                  </td>
                  <td className="font-mono text-xs text-gold-soft">{t.status}</td>
                  <td className="font-mono text-felt-200">
                    {formatBrlFromCents(t.small_blind)}/{formatBrlFromCents(t.big_blind)}
                  </td>
                  <td className="font-mono text-felt-200">
                    {formatBrlFromCents(t.min_buy_in)}–{formatBrlFromCents(t.max_buy_in)}
                  </td>
                  <td className="font-mono">
                    {t.current_players}/{t.max_players}
                  </td>
                  <td className="space-x-1">
                    <button type="button" className="zt-btn-secondary !px-2 !py-0.5 !text-[10px]" onClick={() => void setStatus(t.id, "OPEN")}>
                      OPEN
                    </button>
                    <button type="button" className="zt-btn-secondary !px-2 !py-0.5 !text-[10px]" onClick={() => void setStatus(t.id, "PAUSED")}>
                      PAUSE
                    </button>
                    <button type="button" className="zt-btn-danger !px-2 !py-0.5 !text-[10px]" onClick={() => void setStatus(t.id, "CLOSED")}>
                      CLOSE
                    </button>
                  </td>
                </tr>
                <tr className="!cursor-default">
                  <td colSpan={6} className="bg-felt-950/70 py-2">
                    {!(t.seats ?? []).length ? (
                      <p className="text-[11px] text-felt-400">Nenhum assento ocupado</p>
                    ) : (
                      <ul className="grid gap-1 sm:grid-cols-2 lg:grid-cols-3">
                        {(t.seats as AdminTableSeat[]).map((s) => (
                          <li key={`${t.id}-${s.seat}`} className="rounded border border-felt-700 px-2 py-1 text-[11px]">
                            <span className="font-mono text-gold-soft">#{s.seat}</span>{" "}
                            <span className="font-semibold text-cream">{s.username}</span>
                            <div className="truncate text-felt-300">{s.email}</div>
                          </li>
                        ))}
                      </ul>
                    )}
                  </td>
                </tr>
              </Fragment>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
