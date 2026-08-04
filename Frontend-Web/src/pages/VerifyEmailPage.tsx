import { FormEvent, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { applyAuthTokens, resendVerification, verifyEmail } from "@/api/client";
import { saveUsername } from "@/lib/auth";

export function VerifyEmailPage() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const [email, setEmail] = useState(params.get("email") ?? "");
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [resending, setResending] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setInfo(null);
    setLoading(true);
    try {
      const res = await verifyEmail(email.trim(), code.trim());
      if (res.already_verified) {
        setInfo(res.message ?? "E-mail já verificado. Faça login.");
        return;
      }
      if (res.token) {
        applyAuthTokens({
          token: res.token,
          refresh_token: res.refresh_token,
          expires_in: res.expires_in,
        });
        if (res.username) saveUsername(res.username);
        navigate("/lobby");
        return;
      }
      setInfo(res.message ?? "Verificação concluída.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Código inválido");
    } finally {
      setLoading(false);
    }
  }

  async function onResend() {
    setError(null);
    setInfo(null);
    setResending(true);
    try {
      const res = await resendVerification(email.trim());
      setInfo(res.message);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha ao reenviar");
    } finally {
      setResending(false);
    }
  }

  return (
    <div className="mx-auto max-w-md">
      <div className="zt-panel">
        <div className="zt-panel-title">Confirmar e-mail</div>
        <form className="space-y-4 p-5" onSubmit={onSubmit}>
          <p className="text-sm leading-relaxed text-felt-200">
            Enviamos um código de 6 dígitos para o seu e-mail — é o dealer
            pedindo para confirmar que a cadeira é sua. Em ambiente demo o
            código também aparece nos logs da API (<code className="text-gold-soft">EMAIL_PROVIDER=log</code>).
          </p>
          <div>
            <label className="zt-label" htmlFor="email">
              E-mail
            </label>
            <input
              id="email"
              type="email"
              className="zt-input"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
          <div>
            <label className="zt-label" htmlFor="code">
              Código (6 dígitos)
            </label>
            <input
              id="code"
              className="zt-input font-mono tracking-[0.35em] text-center text-lg"
              required
              inputMode="numeric"
              pattern="[0-9]{6}"
              maxLength={6}
              placeholder="000000"
              value={code}
              onChange={(e) => setCode(e.target.value.replace(/\D/g, "").slice(0, 6))}
            />
          </div>
          {error && (
            <p className="rounded border border-red-800 bg-red-950/50 px-3 py-2 text-sm text-red-200">
              {error}
            </p>
          )}
          {info && (
            <p className="rounded border border-rail bg-felt-850 px-3 py-2 text-sm text-gold-soft">
              {info}
            </p>
          )}
          <button type="submit" className="zt-btn-primary w-full" disabled={loading}>
            {loading ? "Validando…" : "Ativar conta"}
          </button>
          <button
            type="button"
            className="zt-btn-secondary w-full"
            disabled={resending || !email}
            onClick={() => void onResend()}
          >
            {resending ? "Reenviando…" : "Reenviar código"}
          </button>
          <p className="text-center text-sm text-felt-300">
            <Link to="/login" className="font-semibold text-gold-bright hover:underline">
              Voltar ao login
            </Link>
          </p>
        </form>
      </div>
    </div>
  );
}
