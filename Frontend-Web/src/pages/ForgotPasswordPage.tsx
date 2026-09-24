import { FormEvent, useState } from "react";
import { Link, useNavigate } from "react-router";
import { forgotPassword, resetPassword } from "@/api/client";

export function ForgotPasswordPage() {
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [codeRequested, setCodeRequested] = useState(false);
  const [code, setCode] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function requestCode(e: FormEvent) {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const result = await forgotPassword(email.trim());
      setMessage(result.message);
      setCodeRequested(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Não foi possível solicitar o código");
    } finally {
      setLoading(false);
    }
  }

  async function changePassword(e: FormEvent) {
    e.preventDefault();
    if (password !== confirm) {
      setError("As senhas não coincidem.");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      await resetPassword({
        email: email.trim(),
        code: code.trim(),
        password,
        password_confirm: confirm,
      });
      navigate("/login?passwordReset=1", { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Não foi possível redefinir a senha");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="mx-auto max-w-lg">
      <div className="zt-panel p-5">
        <h1 className="text-xl font-bold text-gold-bright">Recuperar senha</h1>
        <p className="mt-2 text-sm text-felt-200">
          O código vale por 15 minutos, pode ser usado uma vez e revoga as sessões anteriores.
        </p>
        <form className="mt-5 space-y-4" onSubmit={codeRequested ? changePassword : requestCode}>
          <div>
            <label className="zt-label" htmlFor="recovery-email">E-mail</label>
            <input
              id="recovery-email"
              type="email"
              className="zt-input"
              required
              autoComplete="email"
              disabled={codeRequested}
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
          {codeRequested && (
            <>
              <div>
                <label className="zt-label" htmlFor="recovery-code">Código de 6 dígitos</label>
                <input
                  id="recovery-code"
                  className="zt-input font-mono tracking-[0.3em]"
                  required
                  inputMode="numeric"
                  pattern="[0-9]{6}"
                  maxLength={6}
                  autoComplete="one-time-code"
                  value={code}
                  onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                />
              </div>
              <div>
                <label className="zt-label" htmlFor="new-password">Nova senha</label>
                <input
                  id="new-password"
                  type="password"
                  className="zt-input"
                  required
                  minLength={8}
                  autoComplete="new-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
              </div>
              <div>
                <label className="zt-label" htmlFor="new-password-confirm">Confirmar nova senha</label>
                <input
                  id="new-password-confirm"
                  type="password"
                  className="zt-input"
                  required
                  minLength={8}
                  autoComplete="new-password"
                  value={confirm}
                  onChange={(e) => setConfirm(e.target.value)}
                />
              </div>
            </>
          )}
          {message && <p className="rounded border border-felt-600 bg-felt-950/60 p-3 text-sm text-felt-200">{message}</p>}
          {error && <p className="rounded border border-red-800 bg-red-950/50 p-3 text-sm text-red-200" role="alert">{error}</p>}
          <button type="submit" className="zt-btn-primary w-full" disabled={loading}>
            {loading ? "Processando…" : codeRequested ? "Redefinir senha" : "Enviar código"}
          </button>
          {codeRequested && (
            <button type="button" className="zt-btn-ghost w-full" onClick={() => setCodeRequested(false)}>
              Usar outro e-mail
            </button>
          )}
        </form>
        <p className="mt-4 text-center text-sm text-felt-300">
          <Link to="/login" className="font-semibold text-gold-soft hover:underline">Voltar ao login</Link>
        </p>
      </div>
    </div>
  );
}
