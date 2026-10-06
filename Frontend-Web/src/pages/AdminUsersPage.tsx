import { FormEvent, useCallback, useEffect, useRef, useState } from "react";
import {
  addAgentAdjustment,
  adjustUserBalance,
  closeAgentCycle,
  configureAgent,
  executeEstruturaBackfill,
  listAdminUsers,
  patchAdminUser,
  previewEstruturaBackfill,
  previewAgentCycle,
} from "@/api/client";
import type { AdminUserResponse, AgentModeSummary, EstruturaBackfillPreview } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

type AgentMoneyMode = "play" | "real";

function monthInput(offset = 0) {
  const parts = new Intl.DateTimeFormat("en-CA", {
    year: "numeric",
    month: "2-digit",
    timeZone: "America/Sao_Paulo",
  }).formatToParts(new Date());
  const year = Number(parts.find((part) => part.type === "year")?.value);
  const month = Number(parts.find((part) => part.type === "month")?.value);
  return new Date(Date.UTC(year, month - 1 + offset, 1)).toISOString().slice(0, 7);
}

function previousMonthInput() {
  return monthInput(-1);
}

function agentAmount(amount: number, mode: AgentMoneyMode) {
  return mode === "real" ? formatBrlFromCents(amount) : `${amount.toLocaleString("pt-BR")} pts`;
}

function parseAgentAmount(value: FormDataEntryValue | null, mode: AgentMoneyMode) {
  const raw = String(value ?? "");
  if (!(mode === "real" ? /^\d+(\.\d{1,2})?$/ : /^\d+$/).test(raw)) return null;
  const [whole, fraction = ""] = raw.split(".");
  const amount = mode === "real" ? Number(whole) * 100 + Number(fraction.padEnd(2, "0")) : Number(whole);
  return Number.isSafeInteger(amount) ? amount : null;
}

export function AdminUsersPage() {
  const [users, setUsers] = useState<AdminUserResponse[]>([]);
  const [total, setTotal] = useState(0);
  const [q, setQ] = useState("");
  const [status, setStatus] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [backfill, setBackfill] = useState<EstruturaBackfillPreview | null>(null);
  const [backfillBusy, setBackfillBusy] = useState(false);
  const [agentBusy, setAgentBusy] = useState(false);
  const agentPending = useRef(false);
  const [closing, setClosing] = useState<{
    id: string; month: string; mode: AgentMoneyMode; summary: AgentModeSummary;
  } | null>(null);
  const [reconciled, setReconciled] = useState(false);

  const load = useCallback(async () => {
    setError(null);
    try {
      const res = await listAdminUsers({
        q: q || undefined,
        status: status || undefined,
        limit: 100,
      });
      setUsers(res.users);
      setTotal(res.total);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }, [q, status]);

  useEffect(() => {
    void load();
  }, [load]);

  const loadBackfill = useCallback(async () => {
    try {
      setBackfill(await previewEstruturaBackfill());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao carregar prévia da rede");
    }
  }, []);

  useEffect(() => {
    void loadBackfill();
  }, [loadBackfill]);

  async function onBackfill() {
    if (!backfill || backfill.unlinked_accounts === 0 || backfill.root_has_sponsor) return;
    if (
      !window.confirm(
        `Vincular ${backfill.unlinked_accounts} conta(s) sem patrocinador a ${backfill.root_username}?`,
      )
    ) {
      return;
    }
    setBackfillBusy(true);
    setError(null);
    setMsg(null);
    try {
      const result = await executeEstruturaBackfill(backfill.root_user_id);
      setMsg(`${result.updated_accounts} conta(s) vinculada(s) a ${result.root_username}.`);
      await Promise.all([load(), loadBackfill()]);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao executar backfill da rede");
    } finally {
      setBackfillBusy(false);
    }
  }

  async function onStatus(id: string, next: string) {
    if (!window.confirm(`Alterar status para ${next}?`)) return;
    setMsg(null);
    try {
      await patchAdminUser(id, { status: next });
      setMsg(`Status → ${next}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }

  async function onRole(id: string, next: string) {
    if (!window.confirm(`Alterar role para ${next}?`)) return;
    setMsg(null);
    try {
      await patchAdminUser(id, { role: next });
      setMsg(`Role → ${next}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }

  async function onAdjust(e: FormEvent<HTMLFormElement>, id: string) {
    e.preventDefault();
    const form = e.currentTarget;
    const fd = new FormData(form);
    const reais = Number(fd.get("reais"));
    const reason = String(fd.get("reason") || "");
    const wallet = String(fd.get("wallet") || "") as "pm_cash" | "pm_mtt" | "real";
    if (!Number.isFinite(reais) || reais === 0) return;
    if (wallet !== "pm_cash" && wallet !== "pm_mtt" && wallet !== "real") {
      setError("Escolha a carteira (PM cash, PM torneio ou Real).");
      return;
    }
    if (!window.confirm(`Ajustar ${wallet} em R$ ${reais.toFixed(2)}?`)) return;
    setMsg(null);
    try {
      const res = await adjustUserBalance(id, Math.round(reais * 100), reason, wallet);
      setMsg(`Novo saldo (${res.wallet}): ${formatBrlFromCents(res.balance)}`);
      form.reset();
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro");
    }
  }

  async function onAgentStatus(id: string, next: "inactive" | "active" | "suspended") {
    if (!window.confirm(`Alterar Agente ZT para ${next}?`)) return;
    setError(null);
    setMsg(null);
    try {
      await configureAgent(id, { status: next });
      setMsg(`Agente ZT → ${next}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao alterar agente");
    }
  }

  async function onAgentTarget(e: FormEvent<HTMLFormElement>, id: string) {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const mode = String(fd.get("agent_mode")) as AgentMoneyMode;
    const month = String(fd.get("agent_month"));
    const amount = parseAgentAmount(fd.get("agent_target"), mode);
    if ((mode !== "play" && mode !== "real") || !/^\d{4}-\d{2}$/.test(month)) return;
    if (amount === null || amount < 0) {
      setError("Informe pontos inteiros no Play ou reais com até duas casas decimais.");
      return;
    }
    setError(null);
    setMsg(null);
    try {
      await configureAgent(id, {
        cycle_start: `${month}-01`,
        money_mode: mode,
        target_ngr_cents: amount,
      });
      setMsg(`Meta ${mode} atualizada.`);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro ao definir meta");
    }
  }

  async function onAgentAdjustment(e: FormEvent<HTMLFormElement>, id: string) {
    e.preventDefault();
    if (agentPending.current) return;
    const form = e.currentTarget;
    const fd = new FormData(form);
    const mode = String(fd.get("adjustment_mode")) as AgentMoneyMode;
    const month = String(fd.get("adjustment_month"));
    const category = String(fd.get("adjustment_category")) as
      | "reward"
      | "refund"
      | "chargeback"
      | "tax"
      | "payment_cost"
      | "other";
    const amount = parseAgentAmount(fd.get("adjustment_amount"), mode);
    const note = String(fd.get("adjustment_note") || "");
    if ((mode !== "play" && mode !== "real") || !/^\d{4}-\d{2}$/.test(month)) return;
    if (amount === null || amount <= 0 || !note.trim()) {
      setError("Informe uma dedução positiva e seu motivo: pontos inteiros no Play ou reais com até duas casas decimais.");
      return;
    }
    setError(null);
    setMsg(null);
    agentPending.current = true;
    setAgentBusy(true);
    try {
      const payload = {
        cycle_start: `${month}-01`,
        money_mode: mode,
        category,
        amount_cents: amount,
        note,
      };
      if (payload.amount_cents <= 0 || !Number.isSafeInteger(payload.amount_cents)) {
        throw new Error("Informe um valor válido: pontos inteiros no Play ou reais com até duas casas decimais.");
      }
      const storageKey = `zt-agent-adjustment-${id}`;
      const fingerprint = JSON.stringify(payload);
      const saved = sessionStorage.getItem(storageKey);
      const pending = saved ? JSON.parse(saved) as { fingerprint: string; key: string } : null;
      const requestId = pending?.fingerprint === fingerprint ? pending.key : crypto.randomUUID();
      sessionStorage.setItem(storageKey, JSON.stringify({ fingerprint, key: requestId }));
      await addAgentAdjustment(id, { ...payload, request_id: requestId });
      sessionStorage.removeItem(storageKey);
      setMsg("Dedução de NGR registrada.");
      setClosing(null);
      form.reset();
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro ao registrar dedução");
    } finally {
      agentPending.current = false;
      setAgentBusy(false);
    }
  }

  async function onAgentClose(e: FormEvent<HTMLFormElement>, id: string) {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const mode = String(fd.get("close_mode")) as AgentMoneyMode;
    const month = String(fd.get("close_month"));
    if ((mode !== "play" && mode !== "real") || !/^\d{4}-\d{2}$/.test(month)) return;
    if (agentPending.current) return;
    setError(null);
    setMsg(null);
    agentPending.current = true;
    setAgentBusy(true);
    try {
      const summary = await previewAgentCycle(id, `${month}-01`, mode);
      setClosing({ id, month, mode, summary });
      setReconciled(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro ao consultar ciclo");
    } finally {
      agentPending.current = false;
      setAgentBusy(false);
    }
  }

  async function confirmAgentClose() {
    if (!closing || !reconciled || agentPending.current) return;
    agentPending.current = true;
    setAgentBusy(true);
    setError(null);
    try {
      const { id, month, mode, summary } = closing;
      const result = await closeAgentCycle(id, `${month}-01`, mode, summary);
      setMsg(
        `${result.already_closed ? "Ciclo já fechado" : "Ciclo fechado"}: ${result.commission_percent}% = ${agentAmount(result.commission_cents, mode)}.`,
      );
      setClosing(null);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro ao fechar ciclo");
    } finally {
      agentPending.current = false;
      setAgentBusy(false);
    }
  }

  return (
    <div className="space-y-3">
      {closing && (
        <section className="zt-panel space-y-3 p-4" aria-label="Conferência do fechamento">
          <h2 className="font-semibold text-gold-bright">
            Conferência · {users.find((u) => u.id === closing.id)?.username ?? closing.id} · {closing.month} · {closing.mode === "play" ? "Play Money" : "Jogo Real"}
          </h2>
          <div className="grid gap-3 text-sm sm:grid-cols-4">
            <p>Receita: <strong>{agentAmount(closing.summary.gross_revenue_cents, closing.mode)}</strong></p>
            <p>Deduções: <strong>{agentAmount(closing.summary.deductions_cents, closing.mode)}</strong></p>
            <p>NGR: <strong>{agentAmount(closing.summary.ngr_cents, closing.mode)}</strong></p>
            <p>Comissão ({closing.summary.commission_percent}%): <strong>{agentAmount(closing.summary.projected_commission_cents, closing.mode)}</strong></p>
          </div>
          <p className="text-xs text-felt-300">Meta: {agentAmount(closing.summary.target_ngr_cents, closing.mode)}. Fechamento a partir do dia 25 do mês seguinte, em ordem cronológica.</p>
          <ul className="max-h-52 space-y-1 overflow-y-auto text-xs text-felt-200">
            {closing.summary.adjustments.map((item) => <li key={item.id}>{item.note} · {agentAmount(item.amount_cents, closing.mode)}</li>)}
          </ul>
          <p className="text-xs text-felt-400">Exibidas até 100 deduções recentes. O total inclui todos os lançamentos.</p>
          {closing.summary.deductions_cents > closing.summary.gross_revenue_cents && (
            <p className="text-sm text-amber-200">Comissão zero. O déficit de {agentAmount(closing.summary.deductions_cents - closing.summary.gross_revenue_cents, closing.mode)} será descontado no próximo ciclo.</p>
          )}
          <label className="flex items-start gap-2 text-sm">
            <input type="checkbox" checked={reconciled} onChange={(e) => setReconciled(e.target.checked)} />
            Conferi receitas, recompensas, estornos, tributos e custos atribuíveis deste ciclo.
          </label>
          <div className="flex gap-2">
            <button className="zt-btn-primary" disabled={!reconciled || agentBusy} onClick={() => void confirmAgentClose()}>Confirmar fechamento</button>
            <button className="zt-btn-secondary" disabled={agentBusy} onClick={() => setClosing(null)}>Cancelar</button>
          </div>
        </section>
      )}
      {backfill && (
        <div className="zt-panel p-4 text-sm">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="font-semibold text-gold-bright">Backfill da rede</p>
              <p className="mt-1 text-xs text-felt-300">
                Raiz: {backfill.root_username} · contas sem patrocinador: {backfill.unlinked_accounts}
              </p>
              {backfill.root_has_sponsor && (
                <p className="mt-1 text-xs text-red-200">
                  A raiz já possui patrocinador. Revise a rede antes de executar.
                </p>
              )}
            </div>
            <button
              type="button"
              className="zt-btn-primary py-1.5! text-xs!"
              disabled={
                backfillBusy || backfill.unlinked_accounts === 0 || backfill.root_has_sponsor
              }
              onClick={() => void onBackfill()}
            >
              {backfillBusy ? "Vinculando…" : "Vincular contas antigas"}
            </button>
          </div>
        </div>
      )}
      <div className="zt-lobby-toolbar">
        <input
          className="zt-input max-w-xs"
          placeholder="Buscar user/email"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        <select className="zt-input w-44" value={status} onChange={(e) => setStatus(e.target.value)}>
          <option value="">Todos status</option>
          <option value="active">active</option>
          <option value="suspended">suspended</option>
          <option value="banned">banned</option>
          <option value="pending_email_verification">pending_email_verification</option>
        </select>
        <button type="button" className="zt-btn-secondary py-1! text-xs!" onClick={() => void load()}>
          Filtrar ({total})
        </button>
      </div>
      {error && <p className="text-sm text-red-200">{error}</p>}
      {msg && <p className="text-sm text-emerald-200">{msg}</p>}
      <div className="zt-table-wrap zt-panel overflow-hidden">
        <table className="zt-lobby-table">
          <thead>
            <tr>
              <th>User</th>
              <th>Role</th>
              <th>Status</th>
              <th>Saldo</th>
              <th>E-mail</th>
              <th>Agente ZT</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            {users.map((u) => (
              <tr key={u.id} className="cursor-default! align-top">
                <td>
                  <div className="font-semibold text-cream">{u.username}</div>
                  <div className="text-[11px] text-felt-400">{u.email}</div>
                </td>
                <td>
                  <select
                    className="zt-input py-1! text-xs"
                    value={u.role}
                    onChange={(e) => void onRole(u.id, e.target.value)}
                  >
                    <option value="player">player</option>
                    <option value="moderator">moderator</option>
                    <option value="admin">admin</option>
                  </select>
                </td>
                <td className="text-xs">
                  <div className="font-mono text-cream">{u.status}</div>
                  <div className="mt-1 flex flex-wrap gap-1">
                    <button type="button" className="zt-btn-secondary px-2! py-0.5! text-[10px]!" onClick={() => void onStatus(u.id, "active")}>
                      active
                    </button>
                    <button type="button" className="zt-btn-secondary px-2! py-0.5! text-[10px]!" onClick={() => void onStatus(u.id, "suspended")}>
                      suspend
                    </button>
                    <button type="button" className="zt-btn-danger px-2! py-0.5! text-[10px]!" onClick={() => void onStatus(u.id, "banned")}>
                      ban
                    </button>
                  </div>
                </td>
                <td className="font-mono text-gold-soft">{formatBrlFromCents(u.balance)}</td>
                <td className="text-xs text-felt-300">{u.email_verified ? "ok" : "não"}</td>
                <td className="min-w-[20rem] text-xs">
                  <div className="flex flex-wrap items-center gap-2">
                    <select
                      className="zt-input w-28! py-1! text-xs"
                      value={u.agent_status}
                      onChange={(e) => void onAgentStatus(
                        u.id,
                        e.target.value as "inactive" | "active" | "suspended",
                      )}
                    >
                      <option value="inactive">inactive</option>
                      <option value="active">active</option>
                      <option value="suspended">suspended</option>
                    </select>
                    <span className="text-felt-300">
                      Meta atual · Play: {agentAmount(u.agent_play_target_cents, "play")} · Real: {formatBrlFromCents(u.agent_real_target_cents)}
                    </span>
                  </div>

                  <form className="mt-2 flex flex-wrap items-end gap-1" onSubmit={(e) => void onAgentTarget(e, u.id)}>
                    <select name="agent_mode" className="zt-input w-20! py-1! text-xs" defaultValue="play" onChange={(e) => {
                      const input = e.currentTarget.form?.elements.namedItem("agent_target");
                      if (input instanceof HTMLInputElement) input.step = e.target.value === "play" ? "1" : "0.01";
                    }}>
                      <option value="play">Play</option>
                      <option value="real">Real</option>
                    </select>
                    <input aria-label="Mês da meta" name="agent_month" type="month" className="zt-input w-32! py-1! text-xs" defaultValue={monthInput(1)} min={monthInput(1)} required />
                    <input aria-label="Meta em pontos Play ou reais" name="agent_target" type="number" min="0" step="1" placeholder="Meta" className="zt-input w-24! py-1! text-xs" required />
                    <button type="submit" className="zt-btn-secondary px-2! py-1! text-[10px]!">Definir meta</button>
                  </form>
                  <p className="mt-1 text-felt-400">Valores: pontos inteiros no Play; reais no Jogo Real. Metas somente para meses futuros.</p>

                  <details className="mt-2 rounded-sm border border-felt-800 p-2">
                    <summary className="cursor-pointer text-gold-soft">Conciliação e fechamento</summary>
                    <form className="mt-2 flex flex-wrap items-end gap-1" onSubmit={(e) => void onAgentAdjustment(e, u.id)}>
                      <select name="adjustment_mode" className="zt-input w-20! py-1! text-xs" defaultValue="play" onChange={(e) => {
                        const input = e.currentTarget.form?.elements.namedItem("adjustment_amount");
                        if (input instanceof HTMLInputElement) {
                          input.step = e.target.value === "play" ? "1" : "0.01";
                          input.min = e.target.value === "play" ? "1" : "0.01";
                        }
                      }}>
                        <option value="play">Play</option>
                        <option value="real">Real</option>
                      </select>
                      <input name="adjustment_month" type="month" className="zt-input w-32! py-1! text-xs" defaultValue={previousMonthInput()} required />
                      <select name="adjustment_category" className="zt-input w-28! py-1! text-xs" defaultValue="reward">
                        <option value="reward">recompensa</option>
                        <option value="refund">estorno</option>
                        <option value="chargeback">chargeback</option>
                        <option value="tax">tributo</option>
                        <option value="payment_cost">pagamento</option>
                        <option value="other">outro</option>
                      </select>
                      <input name="adjustment_amount" type="number" min="1" step="1" placeholder="Valor" className="zt-input w-20! py-1! text-xs" required />
                      <input name="adjustment_note" maxLength={240} placeholder="Motivo da dedução" className="zt-input w-36! py-1! text-xs" required />
                      <button type="submit" disabled={agentBusy} className="zt-btn-secondary px-2! py-1! text-[10px]!">Deduzir</button>
                    </form>
                    <form className="mt-2 flex flex-wrap items-end gap-1" onSubmit={(e) => void onAgentClose(e, u.id)}>
                      <select name="close_mode" className="zt-input w-20! py-1! text-xs" defaultValue="play">
                        <option value="play">Play</option>
                        <option value="real">Real</option>
                      </select>
                      <input name="close_month" type="month" className="zt-input w-32! py-1! text-xs" defaultValue={previousMonthInput()} required />
                      <button type="submit" disabled={agentBusy} className="zt-btn-primary px-2! py-1! text-[10px]!">Conferir mês</button>
                      <span className="text-felt-400">Saldo real: {formatBrlFromCents(u.agent_commission_balance_cents)}</span>
                    </form>
                  </details>
                </td>
                <td>
                  <form className="flex flex-wrap items-end gap-1" onSubmit={(e) => void onAdjust(e, u.id)}>
                    <select name="wallet" className="zt-input w-28! py-1! text-xs" defaultValue="pm_cash" required>
                      <option value="pm_cash">PM cash</option>
                      <option value="pm_mtt">PM torneio</option>
                      <option value="real">Jogo Real</option>
                    </select>
                    <input name="reais" type="number" step="0.01" placeholder="± R$" className="zt-input w-20! py-1! text-xs" required />
                    <input name="reason" placeholder="motivo" className="zt-input w-28! py-1! text-xs" required maxLength={200} />
                    <button type="submit" className="zt-btn-primary px-2! py-1! text-[10px]!">
                      Ajustar
                    </button>
                  </form>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
