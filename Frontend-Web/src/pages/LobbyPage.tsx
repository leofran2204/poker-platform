import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { joinTable, listTables } from "@/api/client";
import type { TableResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

type StakeFilter = "all" | "micro" | "low" | "mid";

const STAKE_OPTIONS: { id: StakeFilter; label: string }[] = [
  { id: "all", label: "Todos" },
  { id: "micro", label: "Micro" },
  { id: "low", label: "Low" },
  { id: "mid", label: "Mid" },
];

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
  const [tables, setTables] = useState<TableResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hideFull, setHideFull] = useState(false);
  const [stake, setStake] = useState<StakeFilter>("all");
  const [joiningId, setJoiningId] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);

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
    <div className="space-y-3">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h1 className="text-xl font-bold uppercase tracking-wide text-gold-bright">Lobby</h1>
          <p className="text-xs text-felt-300">Cash games · estilo Full Tilt</p>
        </div>
      </div>

      {error && (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">
          {error}
        </p>
      )}

      <div className="zt-panel overflow-hidden">
        <div className="zt-lobby-toolbar">
          <div className="min-w-0 flex-1">
            <div className="text-xs font-bold uppercase tracking-wider text-gold-bright">
              Cash games
              <span className="ml-2 font-mono text-felt-300">({filtered.length})</span>
            </div>
            <p className="text-[11px] text-felt-400">Atualização automática a cada 15s</p>
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
            <table className="zt-lobby-table">
              <thead>
                <tr>
                  <th>Nome</th>
                  <th>Tipo</th>
                  <th>Blinds</th>
                  <th>Buy-in</th>
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
                      <td className="font-semibold text-cream">{t.name}</td>
                      <td>
                        <span className="zt-chip">{t.game_type || "NLHE"}</span>
                      </td>
                      <td className="font-mono text-gold-soft">
                        {formatBrlFromCents(t.small_blind)}/{formatBrlFromCents(t.big_blind)}
                      </td>
                      <td className="font-mono text-felt-200">
                        {formatBrlFromCents(t.min_buy_in)}–{formatBrlFromCents(t.max_buy_in)}
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

      <p className="text-[11px] text-felt-400">
        Clique para selecionar · duplo clique para entrar · mín. 2 jogadores na mesma mesa para iniciar mão
      </p>
    </div>
  );
}
