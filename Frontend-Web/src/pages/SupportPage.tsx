import { FormEvent, useCallback, useEffect, useState } from "react";
import { Link } from "react-router";
import { createSupportTicket, listMySupportTickets } from "@/api/client";
import type { SupportTicketResponse } from "@/api/types";
import { isAuthenticated } from "@/lib/auth";

const categoryLabels: Record<string, string> = {
  account: "Conta e acesso",
  payments: "Depósitos e saques",
  responsible_gaming: "Jogo responsável",
  technical: "Problema técnico",
  other: "Outro assunto",
};

export function SupportPage() {
  const [tickets, setTickets] = useState<SupportTicketResponse[]>([]);
  const [category, setCategory] = useState("account");
  const [subject, setSubject] = useState("");
  const [message, setMessage] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    if (!isAuthenticated()) return;
    try { setTickets(await listMySupportTickets()); }
    catch (err) { setError(err instanceof Error ? err.message : "Falha ao carregar chamados"); }
  }, []);

  useEffect(() => { void load(); }, [load]);

  if (!isAuthenticated()) {
    return <div className="zt-panel p-8 text-center"><p className="text-felt-200">Entre para abrir e acompanhar chamados.</p><Link to="/login?returnTo=/suporte" className="zt-btn-primary mt-4 inline-flex">Entrar</Link></div>;
  }

  async function submit(e: FormEvent) {
    e.preventDefault();
    setLoading(true); setError(null); setNotice(null);
    try {
      await createSupportTicket({ category, subject: subject.trim(), message: message.trim() });
      setSubject(""); setMessage(""); setNotice("Chamado aberto. Acompanhe a resposta nesta página.");
      await load();
    } catch (err) { setError(err instanceof Error ? err.message : "Falha ao abrir chamado"); }
    finally { setLoading(false); }
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <header><p className="text-xs font-bold uppercase tracking-[0.18em] text-gold-soft">Atendimento</p><h1 className="mt-2 text-2xl font-bold text-gold-bright">Central de suporte</h1><p className="mt-2 text-sm text-felt-200">Nunca informe senha, código de e-mail, código MFA ou chave privada em um chamado.</p></header>
      <form className="zt-panel space-y-4 p-5" onSubmit={submit}>
        <div className="grid gap-4 sm:grid-cols-2">
          <div><label className="zt-label" htmlFor="support-category">Categoria</label><select id="support-category" className="zt-input" value={category} onChange={(e) => setCategory(e.target.value)}>{Object.entries(categoryLabels).map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></div>
          <div><label className="zt-label" htmlFor="support-subject">Assunto</label><input id="support-subject" className="zt-input" required minLength={5} maxLength={120} value={subject} onChange={(e) => setSubject(e.target.value)} /></div>
        </div>
        <div><label className="zt-label" htmlFor="support-message">Como podemos ajudar?</label><textarea id="support-message" className="zt-input min-h-32" required minLength={10} maxLength={5000} value={message} onChange={(e) => setMessage(e.target.value)} /></div>
        {notice && <p className="rounded border border-emerald-700 bg-emerald-950/40 p-3 text-sm text-emerald-100" role="status">{notice}</p>}
        {error && <p className="rounded border border-red-800 bg-red-950/50 p-3 text-sm text-red-200" role="alert">{error}</p>}
        <button type="submit" className="zt-btn-primary" disabled={loading}>{loading ? "Enviando…" : "Abrir chamado"}</button>
      </form>
      <section><h2 className="text-lg font-bold text-gold-bright">Meus chamados</h2><div className="mt-3 space-y-3">{tickets.length === 0 ? <p className="zt-panel p-4 text-sm text-felt-300">Nenhum chamado aberto.</p> : tickets.map((ticket) => <article key={ticket.id} className="zt-panel p-4"><div className="flex flex-wrap justify-between gap-2"><div><span className="text-xs text-gold-soft">{categoryLabels[ticket.category] ?? ticket.category}</span><h3 className="font-bold text-cream">{ticket.subject}</h3></div><span className="zt-chip">{ticket.status}</span></div><p className="mt-2 whitespace-pre-wrap text-sm text-felt-200">{ticket.message}</p>{ticket.admin_response && <div className="mt-3 rounded border border-gold/40 bg-felt-950/60 p-3"><strong className="text-sm text-gold-bright">Resposta do suporte</strong><p className="mt-1 whitespace-pre-wrap text-sm text-felt-200">{ticket.admin_response}</p></div>}</article>)}</div></section>
    </div>
  );
}
