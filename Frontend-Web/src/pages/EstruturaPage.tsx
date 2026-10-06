import { useCallback, useEffect, useRef, useState } from "react";
import { Link } from "react-router";
import { fetchEstrutura } from "@/api/client";
import type { AgentDirectPlayer, EstruturaResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

type MoneyMode = "play" | "real";

function formatAmount(cents: number, mode: MoneyMode) {
  return mode === "real"
    ? formatBrlFromCents(cents)
    : `${cents.toLocaleString("pt-BR")} pts`;
}

function monthLabel(cycleStart: string) {
  return new Intl.DateTimeFormat("pt-BR", {
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  }).format(new Date(`${cycleStart}T00:00:00Z`));
}

export function EstruturaPage() {
  const [data, setData] = useState<EstruturaResponse | null>(null);
  const [mode, setMode] = useState<MoneyMode>("play");
  const [selectedMonth, setSelectedMonth] = useState("");
  const loadSequence = useRef(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "error">("idle");

  const load = useCallback(async () => {
    const sequence = ++loadSequence.current;
    setLoading(true);
    setError(null);
    try {
      const response = await fetchEstrutura(selectedMonth ? `${selectedMonth}-01` : undefined);
      if (sequence === loadSequence.current) setData(response);
    } catch (e) {
      if (sequence === loadSequence.current) setError(e instanceof Error ? e.message : "Falha ao carregar o painel do agente");
    } finally {
      if (sequence === loadSequence.current) setLoading(false);
    }
  }, [selectedMonth]);

  useEffect(() => {
    if (isAuthenticated()) void load();
  }, [load]);

  if (!isAuthenticated()) {
    return (
      <div className="zt-panel p-8 text-center">
        <p className="text-felt-300">Entre na conta para acessar o painel do Agente ZT.</p>
        <Link to="/login" className="zt-btn-primary mt-4 inline-flex">Entrar</Link>
      </div>
    );
  }

  if (error) {
    return <div className="space-y-3"><p role="alert" className="rounded-sm border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">{error}</p><button className="zt-btn-secondary" onClick={() => void load()}>Tentar novamente</button></div>;
  }
  if (!data) return <p className="text-felt-400">Carregando…</p>;

  const summary = mode === "play" ? data.play : data.real;
  const targetProgress = summary.target_ngr_cents > 0
    ? Math.min(100, Math.round((summary.ngr_cents / summary.target_ngr_cents) * 100))
    : 0;
  const houseCents = Math.max(0, summary.ngr_cents - summary.projected_commission_cents);
  const invite = data.referral_code && typeof window !== "undefined"
    ? `${window.location.origin}/register?ref=${data.referral_code}`
    : null;

  async function copyInvite() {
    if (!invite) return;
    try {
      await navigator.clipboard.writeText(invite);
      setCopyStatus("copied");
      window.setTimeout(() => setCopyStatus("idle"), 2500);
    } catch {
      setCopyStatus("error");
    }
  }

  return (
    <div className="space-y-5">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-xs font-bold uppercase tracking-[0.18em] text-gold-soft">Agente ZT Poker</p>
          <h1 className="mt-1 text-2xl font-bold text-gold-bright">Meu painel</h1>
          <p className="mt-1 max-w-3xl text-sm text-felt-200">
            Um nível direto. Comissão de 30% do NGR, chegando a 35% no mês da meta.
          </p>
        </div>
        <span className={`zt-chip ${data.agent_status === "active" ? "border-emerald-700 text-emerald-300" : "border-amber-700 text-amber-200"}`}>
          {data.agent_status === "active" ? "Agente ativo" : data.agent_status === "suspended" ? "Agente suspenso" : "Aguardando aprovação"}
        </span>
      </header>

      <div className="zt-panel p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-xs text-felt-300">Ciclo consultado</p>
            <p className="font-semibold capitalize text-cream">{monthLabel(data.cycle_start)}</p>
            <div className="mt-2 flex items-center gap-2">
              <input type="month" aria-label="Consultar mês" className="zt-input w-40! py-1! text-xs" value={selectedMonth || data.cycle_start.slice(0, 7)} onChange={(e) => setSelectedMonth(e.target.value)} />
              <button className="zt-btn-secondary py-1! text-xs!" disabled={loading} onClick={() => { if (selectedMonth) setSelectedMonth(""); else void load(); }}>Mês atual</button>
            </div>
            {loading && <p role="status" className="mt-1 text-xs text-felt-400">Atualizando…</p>}
          </div>
          <div className="flex gap-2" role="tablist" aria-label="Tipo de carteira">
            {(["play", "real"] as const).map((item) => (
              <button
                key={item}
                type="button"
                role="tab"
                aria-selected={mode === item}
                className={mode === item ? "zt-btn-primary py-1.5! text-xs!" : "zt-btn-secondary py-1.5! text-xs!"}
                onClick={() => setMode(item)}
              >
                {item === "play" ? "Play Money" : "Jogo Real"}
              </button>
            ))}
          </div>
        </div>

        {data.agent_status !== "active" && (
          <p className="mt-3 rounded-sm border border-amber-700/70 bg-amber-950/30 p-3 text-xs text-amber-100">
            A apuração começa quando o administrador aprovar o agente. O vínculo dos jogadores permanece preservado.
          </p>
        )}

        <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <Metric label="Jogadores ativos" value={`${summary.active_players_count}/${data.direct_players_count}`} />
          <Metric label="NGR elegível" value={formatAmount(summary.ngr_cents, mode)} />
          <Metric label="Percentual projetado" value={`${summary.commission_percent}%`} highlight />
          <Metric label="Comissão projetada" value={formatAmount(summary.projected_commission_cents, mode)} highlight />
        </div>

        <div className="mt-4 rounded-sm border border-felt-700 bg-felt-950/60 p-4">
          <div className="flex flex-wrap items-end justify-between gap-2">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wider text-gold-soft">Meta mensal</p>
              <p className="mt-1 text-sm text-felt-200">
                {summary.target_ngr_cents > 0
                  ? `${formatAmount(summary.ngr_cents, mode)} de ${formatAmount(summary.target_ngr_cents, mode)}`
                  : "Meta ainda não definida pelo administrador."}
              </p>
            </div>
            <strong className={summary.target_reached ? "text-emerald-300" : "text-cream"}>
              {summary.target_reached ? "35% projetados" : `${targetProgress}%`}
            </strong>
          </div>
          <div className="mt-3 h-2 overflow-hidden rounded-sm bg-felt-900" aria-hidden>
            <div
              className={`h-full rounded-sm ${summary.target_reached ? "bg-emerald-500" : "bg-gold"}`}
              style={{ width: `${targetProgress}%` }}
            />
          </div>
        </div>

        <div className="mt-4 grid gap-2 text-xs sm:grid-cols-4">
          <Breakdown label="Receita bruta" value={formatAmount(summary.gross_revenue_cents, mode)} />
          <Breakdown label="Deduções" value={formatAmount(summary.deductions_cents, mode)} />
          <Breakdown label="NGR" value={formatAmount(summary.ngr_cents, mode)} />
          <Breakdown label="Parcela da plataforma" value={formatAmount(houseCents, mode)} />
        </div>
        <p className="mt-2 text-xs text-felt-400">NGR é a receita atribuível menos as deduções. A comissão prevista pode mudar até a conciliação e o fechamento do mês.</p>
        {summary.adjustments.length > 0 && (
          <details className="mt-4 rounded-sm border border-felt-700 p-3">
            <summary className="cursor-pointer text-sm text-gold-soft">Deduções do período</summary>
            <ul className="mt-3 max-h-64 space-y-2 overflow-y-auto text-xs text-felt-200">
              {summary.adjustments.map((item) => (
                <li key={item.id} className="flex justify-between gap-3 border-t border-felt-800 pt-2">
                  <span>{item.note}</span><strong className="shrink-0 font-mono">{formatAmount(item.amount_cents, mode)}</strong>
                </li>
              ))}
            </ul>
            <p className="mt-2 text-xs text-felt-400">Até 100 deduções recentes. O total considera todos os lançamentos.</p>
          </details>
        )}

        {invite && (
          <div className="mt-4 border-t border-felt-800 pt-4">
            <span className="text-xs text-felt-300">Seu convite direto</span>
            <div className="mt-1 flex flex-col gap-2 sm:flex-row sm:items-center">
              <p className="min-w-0 flex-1 break-all rounded-sm border border-felt-800 bg-felt-900/80 p-2 font-mono text-xs text-felt-200">{invite}</p>
              <button type="button" className="zt-btn-primary shrink-0 py-1.5! text-xs!" onClick={() => void copyInvite()}>
                {copyStatus === "copied" ? "Link copiado ✓" : "Copiar convite"}
              </button>
            </div>
            {copyStatus === "error" && <p className="mt-1 text-xs text-red-200">Selecione o link e copie manualmente.</p>}
          </div>
        )}
      </div>

      <DirectPlayersTable rows={data.direct_players} mode={mode} />

      <section className="zt-panel overflow-hidden">
        <div className="zt-panel-title">Fechamentos mensais</div>
        {data.recent_cycles.length === 0 ? (
          <p className="p-4 text-sm text-felt-300">Nenhum ciclo fechado ainda.</p>
        ) : (
          <div className="zt-table-wrap">
            <table className="w-full min-w-176 text-left text-sm">
              <thead>
                <tr className="text-xs uppercase text-felt-300">
                  <th className="px-3 py-2">Ciclo</th>
                  <th className="px-3 py-2">Modo</th>
                  <th className="px-3 py-2">NGR</th>
                  <th className="px-3 py-2">Percentual</th>
                  <th className="px-3 py-2">Comissão</th>
                  <th className="px-3 py-2">Status</th>
                </tr>
              </thead>
              <tbody>
                {data.recent_cycles.map((cycle) => (
                  <tr key={`${cycle.cycle_start}-${cycle.money_mode}`} className="border-t border-felt-700">
                    <td className="px-3 py-2 capitalize">{monthLabel(cycle.cycle_start)}</td>
                    <td className="px-3 py-2">{cycle.money_mode === "play" ? "Play" : "Real"}</td>
                    <td className="px-3 py-2 font-mono">{formatAmount(cycle.ngr_cents, cycle.money_mode)}</td>
                    <td className="px-3 py-2 font-mono">{cycle.commission_percent ? `${cycle.commission_percent}%` : "—"}</td>
                    <td className="px-3 py-2 font-mono">{formatAmount(cycle.commission_cents, cycle.money_mode)}</td>
                    <td className="px-3 py-2">{cycle.status === "closed" ? "Fechado" : "Em apuração"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <p className="text-xs text-felt-400">
        Saldo fechado: {data.estrutura_points.toLocaleString("pt-BR")} ZT Points no Play Money e{" "}
        {formatBrlFromCents(data.agent_commission_balance_cents)} em comissões do Jogo Real, separadas da carteira de jogo. Pontos Play não são convertidos em dinheiro. Valores reais dependem das condições operacionais aplicáveis.
      </p>
    </div>
  );
}

function Metric({ label, value, highlight = false }: { label: string; value: string; highlight?: boolean }) {
  return (
    <div className={`rounded-sm border p-3 ${highlight ? "border-gold/50 bg-gold/10" : "border-felt-700 bg-felt-950/60"}`}>
      <p className="text-xs text-felt-300">{label}</p>
      <p className={`mt-1 font-mono text-lg font-bold ${highlight ? "text-gold-bright" : "text-cream"}`}>{value}</p>
    </div>
  );
}

function Breakdown({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-sm border border-felt-800 p-2">
      <span className="text-felt-400">{label}</span>
      <strong className="mt-0.5 block font-mono text-felt-200">{value}</strong>
    </div>
  );
}

function DirectPlayersTable({ rows, mode }: { rows: AgentDirectPlayer[]; mode: MoneyMode }) {
  if (rows.length === 0) {
    return <div className="zt-panel p-4 text-sm text-felt-300">Nenhum jogador direto ainda.</div>;
  }
  return (
    <section className="zt-panel overflow-hidden">
      <div className="zt-panel-title">Jogadores diretos</div>
      <div className="zt-table-wrap">
        <table className="w-full min-w-152 text-left text-sm">
          <thead>
            <tr className="text-xs uppercase text-felt-300">
              <th className="px-3 py-2">Jogador</th>
              <th className="px-3 py-2">Cash</th>
              <th className="px-3 py-2">Fees MTT</th>
              <th className="px-3 py-2">Receita bruta</th>
              <th className="px-3 py-2">Atividade</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => {
              const cash = mode === "play" ? row.play_cash_rake_cents : row.real_cash_rake_cents;
              const fees = mode === "play" ? row.play_tournament_fees_cents : row.real_tournament_fees_cents;
              const gross = mode === "play" ? row.play_gross_revenue_cents : row.real_gross_revenue_cents;
              return (
                <tr key={row.username} className="border-t border-felt-700">
                  <td className="px-3 py-2 font-semibold text-cream">{row.username}</td>
                  <td className="px-3 py-2 font-mono">{formatAmount(cash, mode)}</td>
                  <td className="px-3 py-2 font-mono">{formatAmount(fees, mode)}</td>
                  <td className="px-3 py-2 font-mono text-gold-soft">{formatAmount(gross, mode)}</td>
                  <td className="px-3 py-2">{gross > 0 ? "Ativo" : "Sem atividade"}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
}
