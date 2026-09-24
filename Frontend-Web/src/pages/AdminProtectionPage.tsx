import { useCallback, useEffect, useState } from "react";
import {
  listAdminKyc,
  listAdminSupportTickets,
  replyAdminSupportTicket,
  reviewAdminKyc,
} from "@/api/client";
import type { AdminKycItem, SupportTicketResponse } from "@/api/types";

export default function AdminProtectionPage() {
  const [kyc, setKyc] = useState<AdminKycItem[]>([]);
  const [tickets, setTickets] = useState<SupportTicketResponse[]>([]);
  const [responses, setResponses] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      const [kycRows, ticketRows] = await Promise.all([
        listAdminKyc("pending"),
        listAdminSupportTickets("open"),
      ]);
      setKyc(kycRows);
      setTickets(ticketRows);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha ao carregar proteção");
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  async function review(userId: string, status: "verified" | "rejected") {
    try { await reviewAdminKyc(userId, status); await load(); }
    catch (err) { setError(err instanceof Error ? err.message : "Falha na análise KYC"); }
  }

  async function answer(ticket: SupportTicketResponse) {
    const text = responses[ticket.id]?.trim();
    if (!text) { setError("Escreva uma resposta antes de concluir o chamado."); return; }
    try { await replyAdminSupportTicket(ticket.id, "resolved", text); await load(); }
    catch (err) { setError(err instanceof Error ? err.message : "Falha ao responder chamado"); }
  }

  return (
    <div className="space-y-6">
      {error && <p className="rounded border border-red-800 bg-red-950/50 p-3 text-sm text-red-200" role="alert">{error}</p>}
      <section><h2 className="text-lg font-bold text-gold-bright">KYC pendente ({kyc.length})</h2><div className="zt-table-wrap mt-3"><table className="w-full min-w-[48rem] text-left text-sm"><thead><tr className="text-xs uppercase text-felt-300"><th className="p-3">Conta</th><th className="p-3">Nome civil</th><th className="p-3">Nascimento</th><th className="p-3">CPF</th><th className="p-3">Ações</th></tr></thead><tbody>{kyc.map((item) => <tr key={item.user_id} className="border-t border-felt-700"><td className="p-3"><strong className="text-cream">{item.username}</strong><br/><span className="text-xs text-felt-300">{item.email}</span></td><td className="p-3">{item.legal_name}</td><td className="p-3">{item.date_of_birth}</td><td className="p-3">final {item.tax_id_last4}</td><td className="p-3"><div className="flex gap-2"><button type="button" className="zt-btn-primary !py-1 !text-xs" onClick={() => void review(item.user_id,"verified")}>Aprovar</button><button type="button" className="zt-btn-secondary !py-1 !text-xs" onClick={() => void review(item.user_id,"rejected")}>Rejeitar</button></div></td></tr>)}</tbody></table>{kyc.length === 0 && <p className="p-4 text-sm text-felt-300">Nenhuma análise pendente.</p>}</div></section>
      <section><h2 className="text-lg font-bold text-gold-bright">Chamados abertos ({tickets.length})</h2><div className="mt-3 space-y-3">{tickets.map((ticket) => <article key={ticket.id} className="zt-panel p-4"><div className="flex flex-wrap justify-between gap-2"><div><span className="text-xs text-gold-soft">{ticket.category} · {ticket.username}</span><h3 className="font-bold text-cream">{ticket.subject}</h3></div><span className="zt-chip">{ticket.status}</span></div><p className="mt-2 whitespace-pre-wrap text-sm text-felt-200">{ticket.message}</p><div className="mt-3 flex flex-col gap-2 sm:flex-row"><textarea className="zt-input min-h-20 flex-1" placeholder="Resposta ao jogador" value={responses[ticket.id] ?? ""} onChange={(e) => setResponses((old) => ({...old,[ticket.id]:e.target.value}))}/><button type="button" className="zt-btn-primary self-end" onClick={() => void answer(ticket)}>Responder e resolver</button></div></article>)}{tickets.length === 0 && <p className="zt-panel p-4 text-sm text-felt-300">Nenhum chamado aberto.</p>}</div></section>
    </div>
  );
}
