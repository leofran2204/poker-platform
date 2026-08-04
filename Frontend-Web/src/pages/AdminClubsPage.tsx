import { FormEvent, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  createClubAgent,
  getClubFinancials,
  listAdminClubs,
  listClubAgents,
} from "@/api/client";
import type { ClubAgentResponse, ClubFinancialsResponse, ClubResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

export function AdminClubsPage() {
  const [clubs, setClubs] = useState<ClubResponse[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [financials, setFinancials] = useState<ClubFinancialsResponse | null>(null);
  const [agents, setAgents] = useState<ClubAgentResponse[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [agentName, setAgentName] = useState("");
  const [agentPct, setAgentPct] = useState(10);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!isAuthenticated()) {
      setLoading(false);
      return;
    }
    void (async () => {
      try {
        const list = await listAdminClubs();
        setClubs(list);
        const first = list[0]?.id;
        if (first) setSelectedId(first);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Erro ao carregar clubes");
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  useEffect(() => {
    if (!selectedId) return;
    void (async () => {
      try {
        const [fin, ag] = await Promise.all([
          getClubFinancials(selectedId),
          listClubAgents(selectedId),
        ]);
        setFinancials(fin);
        setAgents(ag);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Erro ao carregar dados do clube");
      }
    })();
  }, [selectedId]);

  async function onCreateAgent(e: FormEvent) {
    e.preventDefault();
    if (!selectedId) return;
    setError(null);
    try {
      const created = await createClubAgent(selectedId, agentName.trim(), agentPct);
      setAgents((prev) => [...prev, created]);
      setAgentName("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha ao criar agente");
    }
  }

  if (!isAuthenticated()) {
    return (
      <div className="zt-panel p-8 text-center">
        <p className="text-felt-300">Área B2B — faça login com conta admin.</p>
        <Link to="/login" className="zt-btn-primary mt-4 inline-flex">
          Entrar
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold text-gold-bright">Gestão de clubes</h1>
        <p className="text-sm text-felt-300">
          Rake split 15% plataforma / 85% clube · agentes e rakeback
        </p>
      </div>

      {error && (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">
          {error}
        </p>
      )}

      {loading ? (
        <p className="text-sm text-felt-300">Carregando…</p>
      ) : clubs.length === 0 ? (
        <div className="zt-panel p-6 text-sm text-felt-300">
          Nenhum clube retornado pela API. Verifique permissão admin e migration 014.
        </div>
      ) : (
        <>
          <div className="zt-panel p-4">
            <label className="zt-label" htmlFor="club">
              Clube
            </label>
            <select
              id="club"
              className="zt-input max-w-md"
              value={selectedId ?? ""}
              onChange={(e) => setSelectedId(e.target.value)}
            >
              {clubs.map((c) => (
                <option key={c.id ?? c.subdomain} value={c.id ?? ""}>
                  {c.name} ({c.subdomain})
                </option>
              ))}
            </select>
          </div>

          {financials && (
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
              <Stat label="Saldo clube" value={formatBrlFromCents(financials.balance)} />
              <Stat
                label="Rake bruto"
                value={formatBrlFromCents(financials.total_rake_generated)}
              />
              <Stat label="Lucro clube (85%)" value={formatBrlFromCents(financials.net_club_rake)} />
              <Stat
                label="Fee plataforma (15%)"
                value={formatBrlFromCents(financials.platform_fee_paid)}
              />
            </div>
          )}

          <div className="grid gap-4 lg:grid-cols-2">
            <div className="zt-panel overflow-hidden">
              <div className="zt-panel-title">Agentes</div>
              {agents.length === 0 ? (
                <p className="p-4 text-sm text-felt-300">Nenhum agente cadastrado.</p>
              ) : (
                <ul className="divide-y divide-felt-600">
                  {agents.map((a) => (
                    <li key={a.agent_id} className="flex justify-between px-4 py-3 text-sm">
                      <span>
                        <span className="font-semibold text-cream">{a.name}</span>
                        <span className="ml-2 text-felt-400">{a.rakeback_percentage}%</span>
                      </span>
                      <span className="font-mono text-gold-soft">
                        {formatBrlFromCents(a.total_commission_earned)}
                      </span>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            <div className="zt-panel">
              <div className="zt-panel-title">Novo agente</div>
              <form className="space-y-3 p-4" onSubmit={(e) => void onCreateAgent(e)}>
                <div>
                  <label className="zt-label" htmlFor="aname">
                    Nome
                  </label>
                  <input
                    id="aname"
                    className="zt-input"
                    required
                    value={agentName}
                    onChange={(e) => setAgentName(e.target.value)}
                  />
                </div>
                <div>
                  <label className="zt-label" htmlFor="apct">
                    Rakeback % (0–50)
                  </label>
                  <input
                    id="apct"
                    type="number"
                    min={0}
                    max={50}
                    className="zt-input"
                    value={agentPct}
                    onChange={(e) => setAgentPct(Number(e.target.value))}
                  />
                </div>
                <button type="submit" className="zt-btn-primary">
                  Cadastrar
                </button>
              </form>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="zt-panel p-4">
      <p className="text-[11px] font-semibold uppercase tracking-wide text-felt-400">{label}</p>
      <p className="mt-1 font-mono text-xl font-bold text-gold-bright">{value}</p>
    </div>
  );
}
