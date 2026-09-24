import { FormEvent, useCallback, useEffect, useState } from "react";
import { Link } from "react-router";
import { fetchResponsibleGaming, startSelfExclusion, updateResponsibleLimits } from "@/api/client";
import type { ResponsibleGamingStatus, ResponsibleLimits } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

type LimitForm = Record<keyof ResponsibleLimits, string>;

const emptyLimits: LimitForm = {
  deposit_limit_daily_cents: "",
  deposit_limit_weekly_cents: "",
  deposit_limit_monthly_cents: "",
  loss_limit_daily_cents: "",
  loss_limit_weekly_cents: "",
  loss_limit_monthly_cents: "",
  play_time_limit_daily_minutes: "",
};

function toForm(data: ResponsibleGamingStatus): LimitForm {
  const money = (value: number | null) => value == null ? "" : (value / 100).toFixed(2);
  return {
    deposit_limit_daily_cents: money(data.deposit_limit_daily_cents),
    deposit_limit_weekly_cents: money(data.deposit_limit_weekly_cents),
    deposit_limit_monthly_cents: money(data.deposit_limit_monthly_cents),
    loss_limit_daily_cents: money(data.loss_limit_daily_cents),
    loss_limit_weekly_cents: money(data.loss_limit_weekly_cents),
    loss_limit_monthly_cents: money(data.loss_limit_monthly_cents),
    play_time_limit_daily_minutes: data.play_time_limit_daily_minutes?.toString() ?? "",
  };
}

export function ResponsibleGamingPage() {
  const authed = isAuthenticated();
  const [status, setStatus] = useState<ResponsibleGamingStatus | null>(null);
  const [limits, setLimits] = useState<LimitForm>(emptyLimits);
  const [duration, setDuration] = useState<"24h" | "7d" | "30d" | "180d" | "permanent">("7d");
  const [confirmation, setConfirmation] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    if (!authed) return;
    const result = await fetchResponsibleGaming();
    setStatus(result);
    setLimits(toForm(result));
  }, [authed]);

  useEffect(() => {
    void load().catch((err) => setError(err instanceof Error ? err.message : "Falha ao carregar seus controles"));
  }, [load]);

  function moneyValue(value: string): number | null {
    if (!value.trim()) return null;
    const numeric = Number(value.replace(",", "."));
    if (!Number.isFinite(numeric) || numeric < 0) {
      throw new Error("Informe limites monetários válidos e não negativos.");
    }
    return Math.round(numeric * 100);
  }

  function minuteValue(value: string): number | null {
    if (!value.trim()) return null;
    const numeric = Number(value);
    if (!Number.isInteger(numeric) || numeric < 1) {
      throw new Error("Informe o limite diário de tempo em minutos inteiros.");
    }
    return numeric;
  }

  async function saveLimits(e: FormEvent) {
    e.preventDefault(); setSaving(true); setError(null); setNotice(null);
    try {
      const result = await updateResponsibleLimits({
        deposit_limit_daily_cents: moneyValue(limits.deposit_limit_daily_cents),
        deposit_limit_weekly_cents: moneyValue(limits.deposit_limit_weekly_cents),
        deposit_limit_monthly_cents: moneyValue(limits.deposit_limit_monthly_cents),
        loss_limit_daily_cents: moneyValue(limits.loss_limit_daily_cents),
        loss_limit_weekly_cents: moneyValue(limits.loss_limit_weekly_cents),
        loss_limit_monthly_cents: moneyValue(limits.loss_limit_monthly_cents),
        play_time_limit_daily_minutes: minuteValue(limits.play_time_limit_daily_minutes),
      });
      setNotice(result.message); await load();
    } catch (err) { setError(err instanceof Error ? err.message : "Falha ao salvar limites"); }
    finally { setSaving(false); }
  }

  async function exclude(e: FormEvent) {
    e.preventDefault(); setSaving(true); setError(null); setNotice(null);
    try {
      await startSelfExclusion(duration, confirmation);
      setConfirmation(""); setNotice("Autoexclusão ativada. Depósitos e novas entradas de Jogo Real estão bloqueados."); await load();
    } catch (err) { setError(err instanceof Error ? err.message : "Falha ao ativar autoexclusão"); }
    finally { setSaving(false); }
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <header>
        <p className="text-xs font-bold uppercase tracking-[0.18em] text-gold-soft">Proteção do jogador</p>
        <h1 className="mt-2 text-3xl font-bold text-gold-bright">Jogo responsável</h1>
        <p className="mt-3 max-w-3xl text-sm leading-relaxed text-felt-200">
          Poker com dinheiro real envolve risco de perda. Jogue apenas se for maior de 18 anos,
          trate o valor como entretenimento e nunca conte com o jogo para pagar despesas.
        </p>
      </header>

      <section className="grid gap-4 md:grid-cols-2">
        <div className="zt-panel p-5">
          <h2 className="font-bold text-gold-bright">Antes de jogar</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-felt-200">
            <li>Defina um orçamento que pode perder sem afetar sua vida.</li>
            <li>Escolha antecipadamente um horário para encerrar a sessão.</li>
            <li>Não jogue com dinheiro emprestado ou reservado para contas.</li>
            <li>Use Play Money quando quiser treinar sem risco financeiro.</li>
          </ul>
        </div>
        <div className="zt-panel p-5">
          <h2 className="font-bold text-gold-bright">Durante a sessão</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-felt-200">
            <li>Faça pausas e acompanhe o extrato, não apenas o saldo atual.</li>
            <li>Não aumente apostas para tentar recuperar perdas.</li>
            <li>Use “Sit-out” ou saia da mesa ao perceber irritação ou impulso.</li>
            <li>Não pressione convidados da sua rede a depositar ou jogar.</li>
          </ul>
        </div>
      </section>

      <section className="rounded border-2 border-amber-700/70 bg-amber-950/30 p-5">
        <h2 className="font-bold text-amber-100">Sinais para parar</h2>
        <p className="mt-2 text-sm leading-relaxed text-amber-50/90">
          Esconder perdas, quebrar o orçamento, jogar para aliviar ansiedade, pedir dinheiro para
          continuar ou perder sono são sinais de alerta. Interrompa a sessão e procure apoio de
          alguém de confiança e de um profissional de saúde.
        </p>
      </section>

      {authed ? (
        <section className="space-y-4" aria-labelledby="my-protection-title">
          <div className="flex flex-wrap items-end justify-between gap-3">
            <div><h2 id="my-protection-title" className="text-xl font-bold text-gold-bright">Meus controles</h2><p className="mt-1 text-sm text-felt-200">Reduções são imediatas. Aumentos ou remoções entram em vigor após 24 horas.</p></div>
            <Link to="/verificacao" className="zt-btn-secondary">KYC: {status?.kyc_status === "verified" ? "verificado" : "verificar conta"}</Link>
          </div>

          {status && (
            <div className="grid gap-3 sm:grid-cols-3">
              <div className="zt-card p-3"><span className="text-xs text-felt-300">Depositado hoje</span><p className="font-mono font-bold text-cream">{formatBrlFromCents(status.deposited_today_cents)}</p></div>
              <div className="zt-card p-3"><span className="text-xs text-felt-300">Perda computada hoje</span><p className="font-mono font-bold text-cream">{formatBrlFromCents(status.loss_today_cents)}</p></div>
              <div className="zt-card p-3"><span className="text-xs text-felt-300">Tempo real hoje</span><p className="font-mono font-bold text-cream">{Math.floor(status.real_play_seconds_today / 60)} min</p></div>
            </div>
          )}

          <form className="zt-panel space-y-4 p-5" onSubmit={saveLimits}>
            <div><h3 className="font-bold text-gold-bright">Limites pessoais</h3><p className="mt-1 text-xs text-felt-300">Deixe vazio somente se não quiser definir limite para aquele período.</p></div>
            <div className="grid gap-4 md:grid-cols-3">
              {([['deposit_limit_daily_cents','Depósito diário'],['deposit_limit_weekly_cents','Depósito semanal'],['deposit_limit_monthly_cents','Depósito mensal']] as const).map(([key,label]) => <div key={key}><label className="zt-label" htmlFor={key}>{label} (R$)</label><input id={key} className="zt-input" inputMode="decimal" min="0" step="0.01" value={limits[key]} onChange={(e) => setLimits((old) => ({...old,[key]:e.target.value}))} /></div>)}
            </div>
            <div className="grid gap-4 md:grid-cols-3">
              {([['loss_limit_daily_cents','Perda diária'],['loss_limit_weekly_cents','Perda semanal'],['loss_limit_monthly_cents','Perda mensal']] as const).map(([key,label]) => <div key={key}><label className="zt-label" htmlFor={key}>{label} (R$)</label><input id={key} className="zt-input" inputMode="decimal" min="0" step="0.01" value={limits[key]} onChange={(e) => setLimits((old) => ({...old,[key]:e.target.value}))} /></div>)}
            </div>
            <div className="max-w-xs"><label className="zt-label" htmlFor="play_time_limit_daily_minutes">Tempo diário de Jogo Real (minutos)</label><input id="play_time_limit_daily_minutes" className="zt-input" type="number" min="15" max="1440" value={limits.play_time_limit_daily_minutes} onChange={(e) => setLimits((old) => ({...old,play_time_limit_daily_minutes:e.target.value}))} /></div>
            {status?.pending_limits_effective_at && <p className="rounded border border-amber-700 bg-amber-950/30 p-3 text-xs text-amber-100">Há uma flexibilização agendada para {new Date(status.pending_limits_effective_at).toLocaleString("pt-BR")}.</p>}
            <button type="submit" className="zt-btn-primary" disabled={saving}>{saving ? "Salvando…" : "Salvar limites"}</button>
          </form>

          <form className="rounded border-2 border-red-800 bg-red-950/30 p-5" onSubmit={exclude}>
            <h3 className="font-bold text-red-100">Autoexclusão</h3>
            <p className="mt-1 text-sm text-red-50/90">Bloqueia imediatamente depósitos e novas entradas de Jogo Real. Não pode ser cancelada antes do prazo.</p>
            <div className="mt-4 grid gap-3 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
              <div><label className="zt-label" htmlFor="exclusion-duration">Prazo</label><select id="exclusion-duration" className="zt-input" value={duration} onChange={(e) => setDuration(e.target.value as typeof duration)}><option value="24h">24 horas</option><option value="7d">7 dias</option><option value="30d">30 dias</option><option value="180d">180 dias</option><option value="permanent">Permanente</option></select></div>
              <div><label className="zt-label" htmlFor="exclusion-confirm">Digite AUTOEXCLUIR</label><input id="exclusion-confirm" className="zt-input" value={confirmation} onChange={(e) => setConfirmation(e.target.value)} /></div>
              <button type="submit" className="zt-btn-secondary border-red-700 text-red-100" disabled={saving || confirmation.toUpperCase() !== "AUTOEXCLUIR"}>Ativar</button>
            </div>
          </form>
          {notice && <p className="rounded border border-emerald-700 bg-emerald-950/40 p-3 text-sm text-emerald-100" role="status">{notice}</p>}
          {error && <p className="rounded border border-red-800 bg-red-950/50 p-3 text-sm text-red-200" role="alert">{error}</p>}
        </section>
      ) : (
        <div className="zt-panel p-4 text-sm text-felt-200">Entre na conta para definir limites, iniciar autoexclusão e acompanhar seu uso. <Link to="/login?returnTo=/jogo-responsavel" className="font-semibold text-gold-soft underline">Entrar</Link></div>
      )}

      <section className="zt-panel p-5">
        <h2 className="font-bold text-gold-bright">Ferramentas da plataforma</h2>
        <div className="mt-3 grid gap-3 text-sm text-felt-200 sm:grid-cols-3">
          <div className="rounded border border-felt-600 bg-felt-950/50 p-3">
            <strong className="text-cream">Saldos separados</strong>
            <p className="mt-1 text-xs">Play Money nunca se mistura com Jogo Real.</p>
          </div>
          <div className="rounded border border-felt-600 bg-felt-950/50 p-3">
            <strong className="text-cream">Extrato visível</strong>
            <p className="mt-1 text-xs">Depósitos, créditos e saques ficam na carteira.</p>
          </div>
          <div className="rounded border border-felt-600 bg-felt-950/50 p-3">
            <strong className="text-cream">Pausa imediata</strong>
            <p className="mt-1 text-xs">Use Sit-out e Sair da mesa a qualquer momento.</p>
          </div>
        </div>
        <div className="mt-5 flex flex-wrap gap-3">
          <Link to="/curso" className="zt-btn-primary">Ir para a Academy</Link>
          <Link to="/wallet" className="zt-btn-secondary">Ver meu extrato</Link>
          <Link to="/termos" className="zt-btn-ghost">Ler os termos</Link>
        </div>
      </section>
    </div>
  );
}
