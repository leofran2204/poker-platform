import { useCallback, useEffect, useState } from "react";
import {
  listAdminTournamentPlayers,
  listAdminTournaments,
  patchAdminTournament,
} from "@/api/client";
import type { AdminTournamentItem, AdminTournamentPlayer } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

export function AdminTournamentsPage() {
  const [items, setItems] = useState<AdminTournamentItem[]>([]);
  const [players, setPlayers] = useState<AdminTournamentPlayer[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      setItems(await listAdminTournaments());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function openPlayers(id: string) {
    setSelected(id);
    try {
      setPlayers(await listAdminTournamentPlayers(id));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro inscritos");
    }
  }

  async function setStatus(id: string, status: string) {
    if (!window.confirm(`Torneio → ${status}? (cancelamento sem reembolso auto nesta versão)`)) return;
    try {
      await patchAdminTournament(id, status);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }

  return (
    <div className="space-y-4">
      {error && <p className="text-sm text-red-200">{error}</p>}
      <div className="zt-table-wrap zt-panel overflow-hidden">
        <table className="zt-lobby-table">
          <thead>
            <tr>
              <th>Nome</th>
              <th>Buy-in</th>
              <th>GTD</th>
              <th>Inscritos</th>
              <th>Status</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            {items.map((t) => (
              <tr key={t.id} className="!cursor-default">
                <td className="font-semibold text-cream">{t.name}</td>
                <td className="font-mono text-gold-soft">
                  {t.is_freeroll ? "Grátis" : formatBrlFromCents(t.buy_in)}
                </td>
                <td className="font-mono">{formatBrlFromCents(t.guaranteed_prize)}</td>
                <td className="font-mono">
                  {t.registered_players}/{t.max_players} · {t.table_max_players}-max
                </td>
                <td className="font-mono text-xs">{t.status}</td>
                <td className="space-x-1">
                  <button type="button" className="zt-btn-secondary !px-2 !py-0.5 !text-[10px]" onClick={() => void openPlayers(t.id)}>
                    Inscritos
                  </button>
                  <button type="button" className="zt-btn-secondary !px-2 !py-0.5 !text-[10px]" onClick={() => void setStatus(t.id, "registering")}>
                    Open
                  </button>
                  <button type="button" className="zt-btn-secondary !px-2 !py-0.5 !text-[10px]" onClick={() => void setStatus(t.id, "paused")}>
                    Pause
                  </button>
                  <button type="button" className="zt-btn-danger !px-2 !py-0.5 !text-[10px]" onClick={() => void setStatus(t.id, "cancelled")}>
                    Cancel
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {selected && (
        <div className="zt-panel overflow-hidden">
          <div className="zt-panel-title">Inscritos · {selected}</div>
          <div className="zt-table-wrap">
            <table className="zt-lobby-table">
              <thead>
                <tr>
                  <th>Nome</th>
                  <th>Stack</th>
                  <th>Rebuys</th>
                  <th>Registrado</th>
                </tr>
              </thead>
              <tbody>
                {players.length === 0 ? (
                  <tr className="!cursor-default">
                    <td colSpan={4} className="text-felt-400">
                      Ninguém inscrito
                    </td>
                  </tr>
                ) : (
                  players.map((p) => (
                    <tr key={p.player_id} className="!cursor-default">
                      <td className="text-cream">{p.player_name}</td>
                      <td className="font-mono">{p.stack.toLocaleString("pt-BR")}</td>
                      <td className="font-mono">{p.rebuys}</td>
                      <td className="text-xs text-felt-400">
                        {new Date(p.registered_at * 1000).toLocaleString("pt-BR")}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
