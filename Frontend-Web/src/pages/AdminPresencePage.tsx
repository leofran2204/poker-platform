import { useEffect, useState } from "react";
import { fetchAdminPresence } from "@/api/client";
import type { AdminPresenceResponse } from "@/api/types";

export function AdminPresencePage() {
  const [data, setData] = useState<AdminPresenceResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const load = () => {
      void fetchAdminPresence()
        .then(setData)
        .catch((e) => setError(e instanceof Error ? e.message : "Erro"));
    };
    load();
    const t = window.setInterval(load, 10_000);
    return () => window.clearInterval(t);
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
    <div className="zt-panel overflow-hidden">
      <div className="zt-panel-title">Online agora · {data.online_count}</div>
      <div className="zt-table-wrap">
        <table className="zt-lobby-table">
          <thead>
            <tr>
              <th>Username</th>
              <th>User id</th>
              <th>Last seen</th>
            </tr>
          </thead>
          <tbody>
            {data.users.length === 0 ? (
              <tr className="!cursor-default">
                <td colSpan={3} className="text-felt-400">
                  Ninguém online
                </td>
              </tr>
            ) : (
              data.users.map((u) => (
                <tr key={u.user_id} className="!cursor-default">
                  <td className="font-semibold text-cream">{u.username}</td>
                  <td className="font-mono text-[11px] text-felt-400">{u.user_id}</td>
                  <td className="text-xs text-felt-300">
                    {new Date(u.last_seen * 1000).toLocaleString("pt-BR")}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
