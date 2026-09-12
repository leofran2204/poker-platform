import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { fetchEstrutura } from "@/api/client";
import type { EstruturaResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

export function EstruturaPage() {
  const [data, setData] = useState<EstruturaResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

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

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold text-gold-bright">Minha Estrutura</h1>
      <p className="text-sm text-felt-300">
        Você vê só dois níveis. 18% do rake de quem entrou pelo seu convite, 12% de quem
        entrou pelo convite deles. Sem cadastro, só mão jogada. Clube não leva fatia.
      </p>
      <div className="zt-panel p-4 text-sm space-y-2">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-felt-800 pb-2">
          <div>
            <span className="text-xs text-felt-400 block">Pontos Acumulados</span>
            <span className="font-mono text-lg font-bold text-gold-bright">{data.estrutura_points}</span>{" "}
            <span className="text-xs text-felt-400">({formatBrlFromCents(data.estrutura_points)} em rake)</span>
          </div>
          <div className="text-right">
            <span className="text-xs text-felt-400 block">Ciclo de Pagamento</span>
            <span className="text-xs font-semibold text-felt-200">Todo dia 25 de cada mês</span>
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs pt-1">
          <div>
            <span className="text-felt-400">Comissões da Semana:</span>{" "}
            <span className="font-semibold text-felt-200">L1: {data.points_week_l1} · L2: {data.points_week_l2}</span>
            {data.withheld_week > 0 && (
              <span className="text-amber-400 block">⚠️ Retido (sem qualificação): {data.withheld_week} pts</span>
            )}
          </div>
          <div>
            <span className="text-felt-400">Status da Semana:</span>{" "}
            {data.eligible ? (
              <span className="text-emerald-400 font-semibold">✅ Qualificado (ativo)</span>
            ) : (
              <span className="text-amber-300 font-semibold">⏳ Pendente meta semanal</span>
            )}
          </div>
        </div>

        <div className="rounded border border-felt-800 bg-felt-950/60 p-3 text-xs space-y-1">
          <p className="font-semibold text-gold-bright">Critério de Ativação Semanal (Segunda a Domingo):</p>
          <ul className="list-disc pl-4 space-y-0.5 text-felt-300">
            <li>
              <strong>Volume de jogo:</strong> {data.hands_this_week}/{data.vp_hands_needed} mãos concluídas na semana{" "}
              {data.hands_this_week >= data.vp_hands_needed ? "✅" : `(faltam ${Math.max(0, data.vp_hands_needed - data.hands_this_week)})`}
            </li>
          </ul>
          <p className="text-[11px] text-felt-400 pt-1">
            Conforme a Cláusula 4 dos <Link to="/termos" className="text-gold underline hover:text-gold-bright">Termos de Uso</Link>, afiliados inativos não acumulam bonificação de rede retroativa.
          </p>
        </div>

        {invite && (
          <div className="pt-2">
            <span className="text-xs text-felt-400 block">Seu link de indicação direta:</span>
            <p className="break-all font-mono text-xs text-felt-200 bg-felt-900/80 p-2 rounded border border-felt-800">{invite}</p>
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
    return <p className="p-4 text-sm text-felt-400">Ninguém neste nível ainda.</p>;
  }
  return (
    <table className="w-full text-left text-sm">
      <thead>
        <tr className="text-xs uppercase text-felt-400">
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
            <td className="px-3 py-2 font-mono">{formatBrlFromCents(r.rake_generated_week)}</td>
            <td className="px-3 py-2 font-mono">{r.commission_paid_week}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
