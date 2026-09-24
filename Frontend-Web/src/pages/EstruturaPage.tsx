import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { fetchEstrutura } from "@/api/client";
import type { EstruturaResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

export function EstruturaPage() {
  const [data, setData] = useState<EstruturaResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "error">("idle");

  const load = useCallback(async () => {
    setError(null);
    try {
      setData(await fetchEstrutura());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Falha ao carregar Minha Estrutura");
    }
  }, []);

  useEffect(() => {
    if (isAuthenticated()) void load();
  }, [load]);

  if (!isAuthenticated()) {
    return (
      <div className="zt-panel p-8 text-center">
        <p className="text-felt-300">Entre na conta para ver Minha Estrutura.</p>
        <Link to="/login" className="zt-btn-primary mt-4 inline-flex">
          Entrar
        </Link>
      </div>
    );
  }

  if (error) {
    return (
      <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">
        {error}
      </p>
    );
  }
  if (!data) {
    return <p className="text-felt-400">Carregando…</p>;
  }

  const invite =
    data.referral_code && typeof window !== "undefined"
      ? `${window.location.origin}/register?ref=${data.referral_code}`
      : null;
  const handProgress = Math.min(
    100,
    Math.round((data.hands_this_week / Math.max(1, data.vp_hands_needed)) * 100),
  );
  const rakeProgress = Math.min(
    100,
    Math.round((data.personal_rake_cents_week / Math.max(1, data.vp_rake_cents_needed)) * 100),
  );

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
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-xs font-bold uppercase tracking-[0.18em] text-gold-soft">Rede Zero Tilt</p>
          <h1 className="mt-1 text-2xl font-bold text-gold-bright">Minha Rede</h1>
          <p className="mt-1 max-w-3xl text-sm text-felt-200">
            Dois níveis, sem taxa e sem bônus por cadastro. A apuração vem do rake individual de
            quem efetivamente jogou: 18% no 1º nível e 12% no 2º.
          </p>
        </div>
        <Link to="/rede" className="text-sm font-semibold text-gold-soft hover:underline">
          Como a rede funciona
        </Link>
      </div>
      <div className="zt-panel p-4 text-sm space-y-2">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-felt-800 pb-2">
          <div>
            <span className="text-xs text-felt-300 block">ZT Points acumulados</span>
            <span className="font-mono text-lg font-bold text-gold-bright">{data.estrutura_points}</span>{" "}
            <span className="text-xs text-felt-300">apuração separada da carteira</span>
          </div>
          <div className="text-right">
            <span className="text-xs text-felt-300 block">Ciclo de apuração</span>
            <span className="text-xs font-semibold text-felt-200">Todo dia 25 de cada mês</span>
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs pt-1">
          <div>
            <span className="text-felt-300">Pontos da semana:</span>{" "}
            <span className="font-semibold text-felt-200">L1: {data.points_week_l1} · L2: {data.points_week_l2}</span>
            {data.withheld_week > 0 && (
              <span className="text-amber-400 block">⚠️ Retido (sem qualificação): {data.withheld_week} pts</span>
            )}
          </div>
          <div>
            <span className="text-felt-300">Status da semana:</span>{" "}
            {data.eligible ? (
              <span className="text-emerald-400 font-semibold">✅ Qualificado (ativo)</span>
            ) : (
              <span className="text-amber-300 font-semibold">⏳ Pendente meta semanal</span>
            )}
          </div>
        </div>

        <div className="rounded border border-felt-800 bg-felt-950/60 p-3 text-xs space-y-1">
          <p className="font-semibold text-gold-bright">Qualificação semanal (segunda a domingo)</p>
          <p className="text-felt-200">Alcance uma das duas metas:</p>
          <ul className="list-disc pl-4 space-y-2 text-felt-200">
            <li>
              <strong>Mãos:</strong> {data.hands_this_week}/{data.vp_hands_needed}{" "}
              {data.hands_this_week >= data.vp_hands_needed ? "✓" : `(faltam ${Math.max(0, data.vp_hands_needed - data.hands_this_week)})`}
              <div className="mt-1 h-1.5 overflow-hidden rounded bg-felt-900" aria-hidden>
                <div className="h-full rounded bg-gold" style={{ width: `${handProgress}%` }} />
              </div>
            </li>
            <li>
              <strong>Rake pessoal:</strong>{" "}
              {formatBrlFromCents(data.personal_rake_cents_week)}/{formatBrlFromCents(data.vp_rake_cents_needed)}{" "}
              {data.personal_rake_cents_week >= data.vp_rake_cents_needed ? "✓" : ""}
              <div className="mt-1 h-1.5 overflow-hidden rounded bg-felt-900" aria-hidden>
                <div className="h-full rounded bg-gold" style={{ width: `${rakeProgress}%` }} />
              </div>
            </li>
          </ul>
          <p className="text-xs text-felt-300 pt-1">
            A rede não paga por cadastro. Conforme os <Link to="/termos" className="text-gold underline hover:text-gold-bright">Termos de Uso</Link>, atividade sem qualificação não acumula bonificação retroativa.
          </p>
        </div>

        {invite && (
          <div className="border-t border-felt-800 pt-3">
            <span className="text-xs text-felt-300 block">Compartilhe seu convite:</span>
            <div className="mt-1 flex flex-col gap-2 sm:flex-row sm:items-center">
              <p className="min-w-0 flex-1 break-all font-mono text-xs text-felt-200 bg-felt-900/80 p-2 rounded border border-felt-800">{invite}</p>
              <button type="button" className="zt-btn-primary shrink-0 !py-1.5 !text-xs" onClick={() => void copyInvite()}>
                {copyStatus === "copied" ? "Link copiado ✓" : "Copiar convite"}
              </button>
            </div>
            {copyStatus === "error" && (
              <p className="mt-1 text-xs text-red-200" role="alert">Selecione o link e copie manualmente.</p>
            )}
          </div>
        )}
      </div>
      <div className="zt-panel overflow-hidden">
        <div className="zt-panel-title">1º nível (18%)</div>
        <MemberTable rows={data.level1} />
      </div>
      <div className="zt-panel overflow-hidden">
        <div className="zt-panel-title">2º nível (12%)</div>
        <MemberTable rows={data.level2} showSponsor />
      </div>
    </div>
  );
}

function MemberTable({
  rows,
  showSponsor,
}: {
  rows: EstruturaResponse["level1"];
  showSponsor?: boolean;
}) {
  if (rows.length === 0) {
    return <p className="p-4 text-sm text-felt-300">Ninguém neste nível ainda.</p>;
  }
  return (
    <div className="zt-table-wrap">
      <table className="w-full min-w-[38rem] text-left text-sm">
        <thead>
          <tr className="text-xs uppercase text-felt-300">
            <th className="px-3 py-2">Jogador</th>
            {showSponsor ? <th className="px-3 py-2">Entrou por</th> : null}
            <th className="px-3 py-2">Rake gerado (semana)</th>
            <th className="px-3 py-2">Seus pontos</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.username} className="border-t border-felt-700">
              <td className="px-3 py-2">{r.username}</td>
              {showSponsor ? <td className="px-3 py-2">{r.sponsor_username ?? "—"}</td> : null}
              <td className="px-3 py-2 font-mono">
                {formatBrlFromCents(r.rake_generated_week)}
              </td>
              <td className="px-3 py-2 font-mono">{r.commission_paid_week}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
