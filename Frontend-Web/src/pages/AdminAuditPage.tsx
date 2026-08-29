import { useEffect, useState } from "react";
import { listAuditLogs } from "@/api/client";
import type { AuditLogItem } from "@/api/types";

export function AdminAuditPage() {
  const [rows, setRows] = useState<AuditLogItem[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void listAuditLogs({ limit: 80 })
      .then(setRows)
      .catch((e) => setError(e instanceof Error ? e.message : "Erro"));
  }, []);

  if (error) return <p className="text-sm text-red-200">{error}</p>;

  return (
    <div className="zt-panel overflow-hidden">
      <div className="zt-panel-title">Audit log (recentes)</div>
      <div className="zt-table-wrap">
        <table className="zt-lobby-table">
          <thead>
            <tr>
              <th>Quando</th>
              <th>Action</th>
              <th>User</th>
              <th>Metadata</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id} className="!cursor-default align-top">
                <td className="whitespace-nowrap text-xs text-felt-300">
                  {new Date(r.created_at).toLocaleString("pt-BR")}
                </td>
                <td className="font-mono text-gold-soft text-xs">{r.action}</td>
                <td className="font-mono text-[11px] text-felt-400">{r.user_id}</td>
                <td className="max-w-md truncate font-mono text-[11px] text-felt-200">
                  {JSON.stringify(r.metadata)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
