import { FormEvent, useCallback, useEffect, useState } from "react";
import {
  createAdminTournament,
  listAdminTournamentPlayers,
  listAdminTournaments,
  patchAdminTournament,
  rescheduleAdminTournament,
} from "@/api/client";
import type { AdminTournamentItem, AdminTournamentPlayer } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

export function AdminTournamentsPage() {
  const [items, setItems] = useState<AdminTournamentItem[]>([]);
  const [players, setPlayers] = useState<AdminTournamentPlayer[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

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

  async function reschedule(id: string, value: string) {
    if (!value) return;
    const epoch = Math.floor(new Date(value).getTime() / 1000);
    if (!Number.isFinite(epoch)) {
      setError("Data/hora inválida");
      return;
    }
    try {
      await rescheduleAdminTournament(id, epoch);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao agendar");
    }
  }

  async function onCreate(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const name = String(fd.get("name") || "").trim();
    const buyInReais = Number(fd.get("buyin") || 0);
    const stack = Number(fd.get("stack") || 10000);
    const tableMax = Number(fd.get("tablemax") || 9);
    const variant = String(fd.get("variant") || "holdem");
    const mode = String(fd.get("mode") || "play");
    const gtdReais = Number(fd.get("gtd") || 0);
    const when = String(fd.get("when") || "");
    if (!name) {
      setError("Nome obrigatório");
      return;
    }
    setBusy(true);
    setError(null);
    setMsg(null);
    try {
      await createAdminTournament({
        name,
        buy_in_cents: Math.round(buyInReais * 100),
        starting_stack: stack,
        table_max_players: tableMax,
        poker_variant: variant,
        money_mode: mode,
        guaranteed_prize_cents: Math.round(gtdReais * 100),
        scheduled_start_at: when ? Math.floor(new Date(when).getTime() / 1000) : null,
      });
      setMsg("Torneio criado (max = 3x por mesa).");
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro ao criar");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-4">
      {error && <p className="text-sm text-red-200">{error}</p>}
      {msg && <p className="text-sm text-emerald-200">{msg}</p>}
      <form onSubmit={onCreate} className="zt-panel space-y-3 p-4">
        <div className="zt-panel-title">Novo torneio (3 mesas, blinds 26 BBA)</div>
        <div className="flex flex-wrap gap-3">
          <label className="flex flex-col gap-1 text-sm">
            Nome
            <input name="name" className="zt-input" placeholder="Ex.: Noite Texas R$20" required />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Buy-in R$
            <input name="buyin" type="number" min={0} step="0.01" defaultValue={10} className="zt-input w-24" />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Stack
            <input name="stack" type="number" min={100} step={100} defaultValue={10000} className="zt-input w-24" />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Variante
            <select name="variant" className="zt-input" defaultValue="holdem">
              <option value="holdem">Texas Hold'em (9)</option>
              <option value="short_deck">Short Deck (6)</option>
              <option value="short_deck_omaha">SD Omaha (5)</option>
              <option value="ultimate_pineapple">Pineapple (6)</option>
            </select>
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Por mesa
            <input name="tablemax" type="number" min={2} max={9} defaultValue={9} className="zt-input w-20" />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Modo
            <select name="mode" className="zt-input" defaultValue="play">
              <option value="play">Play Money</option>
              <option value="real">Jogo Real</option>
            </select>
          </label>
          <label className="flex flex-col gap-1 text-sm">
            GTD R$
            <input name="gtd" type="number" min={0} step="0.01" defaultValue={100} className="zt-input w-24" />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Início
            <input name="when" type="datetime-local" className="zt-input" />
          </label>
        </div>
        <button type="submit" className="zt-btn-primary" disabled={busy}>
          Criar torneio
        </button>
      </form>
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
                  <input
                    type="datetime-local"
                    className="zt-input !px-1 !py-0.5 !text-[10px]"
                    aria-label="Agendar início"
                    onChange={(e) => void reschedule(t.id, e.currentTarget.value)}
                  />
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
                  <th>E-mail</th>
                  <th>Stack</th>
                  <th>Rebuys</th>
                  <th>Registrado</th>
                </tr>
              </thead>
              <tbody>
                {players.length === 0 ? (
                  <tr className="!cursor-default">
                    <td colSpan={5} className="text-felt-400">
                      Ninguém inscrito
                    </td>
                  </tr>
                ) : (
                  players.map((p) => (
                    <tr key={p.player_id} className="!cursor-default">
                      <td className="text-cream">{p.player_name}</td>
                      <td className="text-[11px] text-felt-300">{p.email ?? "—"}</td>
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
