import { useEffect, useState } from "react";
import { fetchAdminStats } from "@/api/client";
import type { AdminStatsResponse } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

function StatCard({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="zt-card p-4">
      <div className="text-[10px] font-bold uppercase tracking-wider text-felt-400">{label}</div>
      <div className="mt-1 font-mono text-xl text-gold-bright">{value}</div>
    </div>
  );
}

export function AdminOverviewPage() {
  const [stats, setStats] = useState<AdminStatsResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void fetchAdminStats()
      .then(setStats)
      .catch((e) => setError(e instanceof Error ? e.message : "Erro"));
  }, []);

  if (error) {
    return <p className="text-sm text-red-200">{error}</p>;
  }
  if (!stats) {
    return (
      <div className="flex items-center gap-3 text-sm text-felt-300">
        <span className="zt-spinner" aria-hidden /> Carregando…
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard label="Users" value={stats.users_total} />
        <StatCard label="E-mail verificado" value={stats.users_verified} />
        <StatCard label="Online agora" value={stats.online_count} />
        <StatCard label="Saldo total play-money" value={formatBrlFromCents(stats.wallet_balance_sum)} />
        <StatCard label="Mesas OPEN" value={stats.tables_open} />
        <StatCard label="Mesas PAUSED" value={stats.tables_paused} />
        <StatCard label="Torneios abertos" value={stats.tournaments_open} />
        <StatCard label="Inscrições MTT" value={stats.tournament_registrations} />
      </div>
      <div className="zt-panel p-4">
        <div className="zt-panel-title !border-0 !px-0 !pt-0">Users por status</div>
        <ul className="mt-2 space-y-1 text-sm">
          {Object.entries(stats.users_by_status).map(([k, v]) => (
            <li key={k} className="flex justify-between border-b border-felt-700/50 py-1">
              <span className="text-felt-300">{k}</span>
              <span className="font-mono text-cream">{v}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
