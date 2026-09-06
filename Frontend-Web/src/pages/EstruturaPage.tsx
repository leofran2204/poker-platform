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
      <div className="zt-panel p-4 text-sm">
        <p>
          Pontos: <span className="font-mono text-gold-bright">{data.estrutura_points}</span>{" "}
          ({formatBrlFromCents(data.estrutura_points)} em rake)
        </p>
        <p>
          Semana: L1 {data.points_week_l1} · L2 {data.points_week_l2}
          {data.withheld_week > 0 ? ` · retido (sem VP) ${data.withheld_week}` : ""}
        </p>
        <p>
          VP: {data.hands_this_week}/{data.vp_hands_needed} mãos ou{" "}
          {formatBrlFromCents(data.personal_rake_cents_week)} /{" "}
          {formatBrlFromCents(data.vp_rake_cents_needed)} de rake próprio.{" "}
          {data.eligible ? "Elegível nesta semana." : "Ainda não elegível — a linha não pontua."}
        </p>
        {invite && (
          <p className="mt-2 break-all font-mono text-xs text-felt-200">{invite}</p>
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
