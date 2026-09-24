import { FormEvent, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { fetchKyc, submitKyc } from "@/api/client";
import type { KycStatusResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";

const labels: Record<KycStatusResponse["status"], string> = {
  not_submitted: "Não enviada",
  pending: "Em análise",
  verified: "Verificada",
  rejected: "Reenvio necessário",
};

export function KycPage() {
  const [data, setData] = useState<KycStatusResponse | null>(null);
  const [legalName, setLegalName] = useState("");
  const [taxId, setTaxId] = useState("");
  const [birthDate, setBirthDate] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!isAuthenticated()) return;
    void fetchKyc()
      .then((result) => {
        setData(result);
        setLegalName(result.legal_name ?? "");
        setBirthDate(result.date_of_birth ?? "");
      })
      .catch((err) => setError(err instanceof Error ? err.message : "Falha ao carregar KYC"));
  }, []);

  if (!isAuthenticated()) {
    return <div className="zt-panel p-8 text-center"><p className="text-felt-200">Entre para verificar sua conta.</p><Link to="/login?returnTo=/verificacao" className="zt-btn-primary mt-4 inline-flex">Entrar</Link></div>;
  }

  async function submit(e: FormEvent) {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const result = await submitKyc({
        legal_name: legalName.trim(),
        tax_id: taxId,
        date_of_birth: birthDate,
      });
      setData(result);
      setTaxId("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha ao enviar verificação");
    } finally {
      setLoading(false);
    }
  }

  const canSubmit = !data || data.status === "not_submitted" || data.status === "rejected";
  return (
    <div className="mx-auto max-w-3xl space-y-5">
      <header>
        <p className="text-xs font-bold uppercase tracking-[0.18em] text-gold-soft">Conta protegida</p>
        <h1 className="mt-2 text-2xl font-bold text-gold-bright">Verificação de identidade</h1>
        <p className="mt-2 text-sm text-felt-200">
          Play Money continua disponível. Para depositar ou entrar em Jogo Real, a conta precisa estar verificada.
        </p>
      </header>

      {data && (
        <div className="zt-panel flex flex-wrap items-center justify-between gap-3 p-4">
          <div>
            <span className="text-xs text-felt-300">Status</span>
            <p className="font-bold text-cream">{labels[data.status]}</p>
          </div>
          {data.tax_id_last4 && <span className="zt-chip">CPF final {data.tax_id_last4}</span>}
        </div>
      )}

      {canSubmit ? (
        <form className="zt-panel space-y-4 p-5" onSubmit={submit}>
          <div>
            <label className="zt-label" htmlFor="legal-name">Nome civil completo</label>
            <input id="legal-name" className="zt-input" required minLength={5} maxLength={160} autoComplete="name" value={legalName} onChange={(e) => setLegalName(e.target.value)} />
          </div>
          <div className="grid gap-4 sm:grid-cols-2">
            <div>
              <label className="zt-label" htmlFor="tax-id">CPF</label>
              <input id="tax-id" className="zt-input" required inputMode="numeric" autoComplete="off" placeholder="000.000.000-00" value={taxId} onChange={(e) => setTaxId(e.target.value)} />
              <p className="mt-1 text-xs text-felt-300">Guardamos somente hash protegido e os quatro últimos dígitos.</p>
            </div>
            <div>
              <label className="zt-label" htmlFor="kyc-birth-date">Data de nascimento</label>
              <input id="kyc-birth-date" type="date" className="zt-input" required autoComplete="bday" value={birthDate} onChange={(e) => setBirthDate(e.target.value)} />
            </div>
          </div>
          {error && <p className="rounded border border-red-800 bg-red-950/50 p-3 text-sm text-red-200" role="alert">{error}</p>}
          <button type="submit" className="zt-btn-primary" disabled={loading}>{loading ? "Enviando…" : "Enviar para análise"}</button>
        </form>
      ) : (
        <div className="zt-panel p-5 text-sm text-felt-200">
          {data?.status === "pending" ? "Seus dados foram enviados e aguardam análise." : "Sua identidade está verificada para usar o Jogo Real."}
        </div>
      )}
    </div>
  );
}
