import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { joinTable, listTables } from "@/api/client";
import type { TableResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

type StakeFilter = "all" | "micro" | "low" | "mid";

export function LobbyPage() {
  const navigate = useNavigate();
  const [tables, setTables] = useState<TableResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hideFull, setHideFull] = useState(false);
  const [stake, setStake] = useState<StakeFilter>("all");
  const [joiningId, setJoiningId] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!isAuthenticated()) {
      setError("Faça login para ver o lobby.");
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const data = await listTables();
      setTables(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao carregar mesas");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
    const t = window.setInterval(() => void load(), 15_000);
    return () => window.clearInterval(t);
  }, [load]);

  const filtered = useMemo(() => {
    return tables.filter((t) => {
      if (hideFull && t.players >= t.max_players) return false;
      const bb = t.big_blind;
      if (stake === "micro" && bb > 50) return false;
      if (stake === "low" && (bb < 50 || bb > 200)) return false;
      if (stake === "mid" && bb < 200) return false;
      return true;
    });
  }, [tables, hideFull, stake]);

  async function handleJoin(table: TableResponse) {
    if (table.players >= table.max_players) return;
    setJoiningId(table.id);
    setError(null);
    try {
      await joinTable(table.id, table.min_buy_in);
      navigate(`/table/${table.id}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Não foi possível entrar na mesa");
    } finally {
      setJoiningId(null);
    }
  }

  if (!isAuthenticated()) {
    return (
      <div className="zt-panel p-8 text-center">
        <h1 className="text-xl font-bold text-gold-bright">Lobby</h1>
        <p className="mt-2 text-felt-300">Entre na sua conta para listar mesas.</p>
        <Link to="/login" className="zt-btn-primary mt-6 inline-flex">
          Entrar
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold text-gold-bright">Lobby</h1>
          <p className="text-sm text-felt-300">Cash games · atualiza a cada 15s</p>
        </div>
        <button type="button" className="zt-btn-secondary" onClick={() => void load()}>
          Atualizar
        </button>
      </div>

      <div className="zt-panel p-4">
        <div className="flex flex-wrap items-end gap-4">
          <div>
            <label className="zt-label" htmlFor="stake">
              Stakes
            </label>
            <select
              id="stake"
              className="zt-input w-40"
              value={stake}
              onChange={(e) => setStake(e.target.value as StakeFilter)}
            >
              <option value="all">Todos</option>
              <option value="micro">Micro (BB ≤ 0,50)</option>
              <option value="low">Low (0,50–2)</option>
              <option value="mid">Mid (BB ≥ 2)</option>
            </select>
          </div>
          <label className="flex items-center gap-2 pb-2 text-sm text-cream">
            <input
              type="checkbox"
              className="accent-gold"
              checked={hideFull}
              onChange={(e) => setHideFull(e.target.checked)}
            />
            Ocultar mesas cheias
          </label>
        </div>
      </div>

      {error && (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">
          {error}
        </p>
      )}

      <div className="zt-panel overflow-hidden">
        <div className="zt-panel-title">Mesas abertas ({filtered.length})</div>
        {loading && tables.length === 0 ? (
          <p className="p-6 text-sm text-felt-300">Carregando mesas…</p>
        ) : filtered.length === 0 ? (
          <p className="p-6 text-sm text-felt-300">Nenhuma mesa com esses filtros.</p>
        ) : (
          <div className="zt-table-wrap">
            <table className="zt-lobby-table">
              <thead>
                <tr>
                  <th>Nome</th>
                  <th>Tipo</th>
                  <th>Blinds</th>
                  <th>Buy-in</th>
                  <th>Jogadores</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {filtered.map((t) => {
                  const full = t.players >= t.max_players;
                  return (
                    <tr key={t.id}>
                      <td className="font-semibold text-cream">{t.name}</td>
                      <td className="text-felt-200">{t.game_type || "NLHE"}</td>
                      <td className="font-mono text-gold-soft">
                        {formatBrlFromCents(t.small_blind)}/{formatBrlFromCents(t.big_blind)}
                      </td>
                      <td className="font-mono text-felt-200">
                        {formatBrlFromCents(t.min_buy_in)}–{formatBrlFromCents(t.max_buy_in)}
                      </td>
                      <td>
                        <span className={full ? "text-red-300" : "text-cream"}>
                          {t.players}/{t.max_players}
                        </span>
                      </td>
                      <td className="text-right">
                        <button
                          type="button"
                          className={full ? "zt-btn-secondary !py-1 !text-xs" : "zt-btn-primary !py-1 !text-xs"}
                          disabled={full || joiningId === t.id}
                          onClick={() => void handleJoin(t)}
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
    </div>
  );
}
