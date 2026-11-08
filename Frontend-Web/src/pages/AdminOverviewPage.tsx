import { useEffect, useState } from "react";
import { fetchAdminLiabilities, fetchAdminStats } from "@/api/client";
import type { AdminLiabilitiesResponse, AdminStatsResponse } from "@/api/types";
import { computeCoverage } from "@/lib/liabilities";
import { formatBrlFromCents, parseBrlToCents } from "@/lib/money";

function LiabilityRow({ label, cents, hint }: { label: string; cents: number; hint?: string }) {
  return (
    <li className="flex items-baseline justify-between gap-3 border-b border-felt-700/50 py-1.5">
      <span className="text-felt-300">
        {label}
        {hint ? <span className="block text-[11px] text-felt-400">{hint}</span> : null}
      </span>
      <span className="font-mono text-cream">{formatBrlFromCents(cents)}</span>
    </li>
  );
}

/** Quanto do caixa pertence aos jogadores no Jogo Real (conciliação diária). */
function LiabilitiesPanel() {
  const [data, setData] = useState<AdminLiabilitiesResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  // Fica só nesta tela: não é enviado nem salvo.
  const [balanceInput, setBalanceInput] = useState("");

  const load = () => {
    setLoading(true);
    setError(null);
    fetchAdminLiabilities()
      .then(setData)
      .catch((e) => setError(e instanceof Error ? e.message : "Erro ao carregar passivo"))
      .finally(() => setLoading(false));
  };
  useEffect(load, []);

  const balanceCents = parseBrlToCents(balanceInput);
  const coverage =
    data && balanceCents !== null ? computeCoverage(balanceCents, data.total_owed_to_players_cents) : null;

  return (
    <div className="zt-panel p-4">
      <div className="flex items-center justify-between gap-3">
        <div className="zt-panel-title border-0! px-0! pt-0!">Passivo com jogadores (Jogo Real)</div>
        <button type="button" className="zt-btn-secondary text-xs" onClick={load} disabled={loading}>
          {loading ? "Atualizando…" : "Atualizar"}
        </button>
      </div>
      {error && <p className="mt-2 text-sm text-red-200">{error}</p>}
      {data && (
        <>
          <ul className="mt-2 space-y-0.5 text-sm">
            <LiabilityRow label="Saldos das carteiras" cents={data.wallet_balances_cents} />
            <LiabilityRow label="Fichas em mesas de cash" cents={data.cash_table_chips_cents} />
            <LiabilityRow
              label="Prêmios de torneios abertos"
              cents={data.open_tournament_prizes_cents}
              hint="maior entre o acumulado e o garantido"
            />
            <LiabilityRow
              label="Saques pedidos, ainda não pagos"
              cents={data.pending_withdrawals_cents}
            />
          </ul>
          <div className="mt-3 flex items-baseline justify-between border-t border-gold-700/60 pt-2">
            <span className="text-sm font-bold uppercase tracking-wide text-gold-bright">Total devido</span>
            <span className="font-mono text-xl text-gold-bright">
              {formatBrlFromCents(data.total_owed_to_players_cents)}
            </span>
          </div>
          {data.provisional_deposits_cents > 0 && (
            <p className="mt-2 text-xs text-amber-100">
              Risco: {formatBrlFromCents(data.provisional_deposits_cents)} creditados de forma provisória,
              com o Pix ainda não liquidado.
            </p>
          )}

          <div className="mt-4">
            <label className="zt-label" htmlFor="account-balance">
              Saldo disponível na conta recebedora
            </label>
            <input
              id="account-balance"
              className="zt-input max-w-xs"
              inputMode="decimal"
              placeholder="R$ 0,00"
              value={balanceInput}
              onChange={(e) => setBalanceInput(e.target.value)}
            />
            <p className="mt-1 text-[11px] text-felt-400">Só para conferência nesta tela; não é salvo nem enviado.</p>
          </div>
          {coverage && (
            <p
              className={`mt-2 rounded-sm border px-3 py-2 text-sm ${
                coverage.state === "covered"
                  ? "border-emerald-700/60 bg-emerald-950/30 text-emerald-100"
                  : "border-red-700/60 bg-red-950/30 text-red-100"
              }`}
            >
              {coverage.state === "covered" ? (
                <>
                  Coberto. Sobra de <strong>{formatBrlFromCents(coverage.differenceCents)}</strong> além do que é
                  dos jogadores.
                </>
              ) : (
                <>
                  <strong>Alerta:</strong> faltam {formatBrlFromCents(-coverage.differenceCents)} para cobrir o que é
                  dos jogadores. Investigue antes de novos saques.
                </>
              )}
            </p>
          )}
          <p className="mt-3 text-[11px] text-felt-400">
            Gerado em {new Date(data.generated_at).toLocaleString("pt-BR")}.
          </p>
        </>
      )}
    </div>
  );
}

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
      <LiabilitiesPanel />
      <div className="zt-panel p-4">
        <div className="zt-panel-title border-0! px-0! pt-0!">Users por status</div>
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
