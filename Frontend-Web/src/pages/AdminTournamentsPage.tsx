import { FormEvent, useCallback, useEffect, useState } from "react";
import {
  createAdminTournament,
  listAdminTournamentPlayers,
  listAdminTournaments,
  patchAdminTournament,
  rescheduleAdminTournament,
  updateAdminTournamentRebuyAddon,
  type RebuyAddonInput,
} from "@/api/client";
import type { AdminTournamentItem, AdminTournamentPlayer } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

function maxSeatsForVariant(variant: string): number {
  return variant === "holdem" ? 9 : variant === "short_deck" ? 8 : 6;
}

function formatAdminSchedule(epoch: number | null): string {
  if (!epoch) return "Sem agenda";
  return new Date(epoch * 1000).toLocaleString("pt-BR", {
    timeZone: "America/Sao_Paulo",
    dateStyle: "short",
    timeStyle: "short",
  });
}

/** Campos vazios ficam de fora: o servidor usa o padrão (criação) ou mantém o valor (edição). */
export function readRebuyAddon(fd: FormData): RebuyAddonInput {
  const num = (key: string) => {
    const raw = String(fd.get(key) ?? "").trim();
    return raw === "" ? undefined : Number(raw);
  };
  const cents = (key: string) => {
    const value = num(key);
    return value === undefined ? undefined : Math.round(value * 100);
  };
  return {
    allow_rebuy: fd.get("allow_rebuy") === "on",
    rebuy_cost_cents: cents("rebuy_cost"),
    rebuy_chips: num("rebuy_chips"),
    rebuy_max_count: num("rebuy_max_count"),
    rebuy_max_level: num("rebuy_max_level"),
    rebuy_stack_threshold: num("rebuy_stack_threshold"),
    allow_addon: fd.get("allow_addon") === "on",
    addon_cost_cents: cents("addon_cost"),
    addon_chips: num("addon_chips"),
    addon_max_level: num("addon_max_level"),
  };
}

function rebuyAddonSummary(t: AdminTournamentItem): string {
  const rebuy = t.allow_rebuy
    ? `R ${t.rebuy_max_count > 0 ? `${t.rebuy_max_count}×` : "∞"} nív.${t.rebuy_max_level}`
    : "R —";
  const addon = t.allow_addon ? `A nív.${t.addon_max_level}` : "A —";
  return `${rebuy} · ${addon}`;
}

function RebuyAddonFields({ item }: { item?: AdminTournamentItem }) {
  return (
    <div className="space-y-2 border-t border-felt-800 pt-3">
      <div className="flex flex-wrap items-end gap-3">
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" name="allow_rebuy" defaultChecked={item?.allow_rebuy ?? false} />
          Rebuy
        </label>
        <label className="flex flex-col gap-1 text-sm">
          Preço R$
          <input name="rebuy_cost" type="number" min={0} step="0.01" placeholder="= buy-in" defaultValue={item ? item.rebuy_cost / 100 : undefined} className="zt-input w-24" />
        </label>
        <label className="flex flex-col gap-1 text-sm">
          Fichas
          <input name="rebuy_chips" type="number" min={0} step={100} placeholder="= stack" defaultValue={item?.rebuy_chips} className="zt-input w-24" />
        </label>
        <label className="flex flex-col gap-1 text-sm">
          Vezes (0 = sem limite)
          <input name="rebuy_max_count" type="number" min={0} max={100} defaultValue={item?.rebuy_max_count ?? 1} className="zt-input w-20" />
        </label>
        <label className="flex flex-col gap-1 text-sm">
          Até o nível
          <input name="rebuy_max_level" type="number" min={0} max={26} defaultValue={item?.rebuy_max_level ?? 6} className="zt-input w-20" />
        </label>
        <label className="flex flex-col gap-1 text-sm">
          Stack-limite (0 = só eliminado)
          <input name="rebuy_stack_threshold" type="number" min={0} step={100} defaultValue={item?.rebuy_stack_threshold ?? 0} className="zt-input w-28" />
        </label>
      </div>
      <div className="flex flex-wrap items-end gap-3">
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" name="allow_addon" defaultChecked={item?.allow_addon ?? false} />
          Add-on (1 por jogador)
        </label>
        <label className="flex flex-col gap-1 text-sm">
          Preço R$
          <input name="addon_cost" type="number" min={0} step="0.01" placeholder="= buy-in" defaultValue={item ? item.addon_cost / 100 : undefined} className="zt-input w-24" />
        </label>
        <label className="flex flex-col gap-1 text-sm">
          Fichas
          <input name="addon_chips" type="number" min={0} step={100} placeholder="= stack" defaultValue={item?.addon_chips} className="zt-input w-24" />
        </label>
        <label className="flex flex-col gap-1 text-sm">
          Até o nível
          <input name="addon_max_level" type="number" min={0} max={26} placeholder="= rebuy" defaultValue={item?.addon_max_level} className="zt-input w-20" />
        </label>
      </div>
      <p className="text-[11px] text-felt-400">
        Sem taxa em rebuy e add-on: o valor vai inteiro para a premiação. As fichas de quem está na mesa entram no começo da mão seguinte.
      </p>
    </div>
  );
}

function scheduleInputValue(epoch: number | null): string {
  if (!epoch) return "";
  return new Intl.DateTimeFormat("sv-SE", {
    timeZone: "America/Sao_Paulo",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  })
    .format(new Date(epoch * 1000))
    .replace(" ", "T");
}

export function AdminTournamentsPage() {
  const [items, setItems] = useState<AdminTournamentItem[]>([]);
  const [players, setPlayers] = useState<AdminTournamentPlayer[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [newVariant, setNewVariant] = useState("holdem");
  const [newTableMax, setNewTableMax] = useState(9);
  const [editing, setEditing] = useState<AdminTournamentItem | null>(null);

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

  async function onSaveRebuyAddon(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!editing) return;
    const input = readRebuyAddon(new FormData(e.currentTarget));
    setBusy(true);
    setError(null);
    setMsg(null);
    try {
      await updateAdminTournamentRebuyAddon(editing.id, input);
      setMsg(`Rebuy e add-on de "${editing.name}" salvos.`);
      setEditing(null);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro ao salvar rebuy e add-on");
    } finally {
      setBusy(false);
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
    const scheduledStartAt = Math.floor(new Date(when).getTime() / 1000);
    if (!when || !Number.isFinite(scheduledStartAt) || scheduledStartAt <= 0) {
      setError("Data e hora de início são obrigatórias");
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
        scheduled_start_at: scheduledStartAt,
        ...readRebuyAddon(fd),
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
            <select name="variant" className="zt-input" value={newVariant} onChange={(event) => {
              const variant = event.target.value;
              setNewVariant(variant);
              setNewTableMax(maxSeatsForVariant(variant));
            }}>
              <option value="holdem">Texas Hold'em (9)</option>
              <option value="omaha">Omaha 4 (6)</option>
              <option value="pineapple_draw">Brazilian Pineapple (6)</option>
              <option value="short_deck">Texas Hold’em Short Deck (8)</option>
            </select>
          </label>
          <label className="flex flex-col gap-1 text-sm">
            Por mesa
            <input name="tablemax" type="number" min={2} max={maxSeatsForVariant(newVariant)} value={newTableMax} onChange={(event) => setNewTableMax(Number(event.target.value))} className="zt-input w-20" />
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
            <input name="when" type="datetime-local" className="zt-input" required />
          </label>
        </div>
        <RebuyAddonFields />
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
              <th>Rebuy · Add-on</th>
              <th>Início (São Paulo)</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            {items.map((t) => (
              <tr key={t.id} className="cursor-default!">
                <td className="font-semibold text-cream">{t.name}</td>
                <td className="font-mono text-gold-soft">
                  {t.is_freeroll ? "Grátis" : formatBrlFromCents(t.buy_in)}
                </td>
                <td className="font-mono">{formatBrlFromCents(t.guaranteed_prize)}</td>
                <td className="font-mono">
                  {t.registered_players}/{t.max_players} · {t.table_max_players}-max
                </td>
                <td className="font-mono text-xs">{t.status}</td>
                <td className="font-mono text-xs">
                  {rebuyAddonSummary(t)}
                  {t.total_rebuys + t.total_addons > 0 ? (
                    <span className="block text-felt-400">
                      {formatBrlFromCents(t.total_rebuys)} · {formatBrlFromCents(t.total_addons)}
                    </span>
                  ) : null}
                </td>
                <td className="font-mono text-xs">{formatAdminSchedule(t.scheduled_start_at)}</td>
                <td className="space-x-1">
                  <input
                    type="datetime-local"
                    className="zt-input px-1! py-0.5! text-[10px]!"
                    aria-label="Agendar início"
                    defaultValue={scheduleInputValue(t.scheduled_start_at)}
                    onChange={(e) => void reschedule(t.id, e.currentTarget.value)}
                  />
                  <button type="button" className="zt-btn-secondary px-2! py-0.5! text-[10px]!" onClick={() => void openPlayers(t.id)}>
                    Inscritos
                  </button>
                  {t.status === "registering" ? (
                    <button type="button" className="zt-btn-secondary px-2! py-0.5! text-[10px]!" onClick={() => setEditing(t)}>
                      Rebuy/Add-on
                    </button>
                  ) : null}
                  <button type="button" className="zt-btn-secondary px-2! py-0.5! text-[10px]!" onClick={() => void setStatus(t.id, "registering")}>
                    Open
                  </button>
                  <button type="button" className="zt-btn-secondary px-2! py-0.5! text-[10px]!" onClick={() => void setStatus(t.id, "paused")}>
                    Pause
                  </button>
                  <button type="button" className="zt-btn-danger px-2! py-0.5! text-[10px]!" onClick={() => void setStatus(t.id, "cancelled")}>
                    Cancel
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {editing && (
        <form key={editing.id} onSubmit={onSaveRebuyAddon} className="zt-panel space-y-3 p-4">
          <div className="zt-panel-title">Rebuy e add-on · {editing.name}</div>
          <RebuyAddonFields item={editing} />
          <div className="flex gap-2">
            <button type="submit" className="zt-btn-primary" disabled={busy}>
              Salvar
            </button>
            <button type="button" className="zt-btn-secondary" onClick={() => setEditing(null)}>
              Fechar
            </button>
          </div>
        </form>
      )}

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
                  <tr className="cursor-default!">
                    <td colSpan={5} className="text-felt-400">
                      Ninguém inscrito
                    </td>
                  </tr>
                ) : (
                  players.map((p) => (
                    <tr key={p.player_id} className="cursor-default!">
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
