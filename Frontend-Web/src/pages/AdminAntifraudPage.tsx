import { useEffect, useState } from "react";
import { fetchAntifraudAlerts } from "@/api/client";
import type { AntifraudAlertSummary } from "@/api/types";

export function AdminAntifraudPage() {
  const [data, setData] = useState<AntifraudAlertSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void fetchAntifraudAlerts()
      .then(setData)
      .catch((e) => setError(e instanceof Error ? e.message : "Erro"));
  }, []);

  if (error) return <p className="text-sm text-red-200">{error}</p>;
  if (!data) {
    return (
      <div className="flex items-center gap-3 text-sm text-felt-300">
        <span className="zt-spinner" aria-hidden /> Carregando…
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <p className="rounded border border-amber-700/50 bg-amber-950/30 px-3 py-2 text-xs text-amber-100">
        Endpoint de alerts ainda é <strong>stub</strong> — útil para layout; detectors ao vivo vêm depois.
      </p>
      <div className="grid gap-3 sm:grid-cols-3">
        <div className="zt-card p-4">
          <div className="text-[10px] uppercase text-felt-400">Bot suspects</div>
          <div className="font-mono text-xl text-gold-bright">{data.bot_suspects_count}</div>
        </div>
        <div className="zt-card p-4">
          <div className="text-[10px] uppercase text-felt-400">Collusion</div>
          <div className="font-mono text-xl text-gold-bright">{data.collusion_alerts_count}</div>
        </div>
        <div className="zt-card p-4">
          <div className="text-[10px] uppercase text-felt-400">Chip dumping</div>
          <div className="font-mono text-xl text-gold-bright">{data.chip_dumping_alerts_count}</div>
        </div>
      </div>
      <div className="zt-panel p-4 text-sm text-felt-200">
        Status: <span className="text-cream">{data.system_status}</span>
      </div>
      <div className="zt-panel overflow-hidden">
        <div className="zt-panel-title">Recent alerts</div>
        <ul className="divide-y divide-felt-700 text-sm">
          {data.recent_alerts.length === 0 ? (
            <li className="p-4 text-felt-400">Nenhum alerta</li>
          ) : (
            data.recent_alerts.map((a) => (
              <li key={a.id} className="px-4 py-3">
                <div className="font-semibold text-cream">
                  {a.alert_type} · score {a.risk_score}
                </div>
                <div className="text-xs text-felt-300">{a.description}</div>
                <div className="text-[11px] text-felt-500">
                  {a.player_id} · {a.timestamp}
                </div>
              </li>
            ))
          )}
        </ul>
      </div>
    </div>
  );
}
