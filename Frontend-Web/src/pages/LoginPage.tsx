import { FormEvent, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { applyAuthTokens, login, verifyMfa } from "@/api/client";
import { saveUsername } from "@/lib/auth";

export function LoginPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const requestedReturnTo = searchParams.get("returnTo");
  const returnTo =
    requestedReturnTo?.startsWith("/") && !requestedReturnTo.startsWith("//")
      ? requestedReturnTo
      : "/lobby";
  const sessionExpired = searchParams.get("reason") === "session-expired";
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [mfaChallenge, setMfaChallenge] = useState<string | null>(null);
  const [mfaCode, setMfaCode] = useState("");
  const [loading, setLoading] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      if (mfaChallenge) {
        const tokens = await verifyMfa(mfaChallenge, mfaCode.trim());
        applyAuthTokens(tokens);
        saveUsername(tokens.username ?? (email.trim().split("@")[0] || "jogador"));
        navigate(returnTo, { replace: true });
        return;
      }

      const res = await login(email.trim(), password);
      if (res.email_verification_required) {
        navigate(
          `/verify-email?email=${encodeURIComponent(res.email ?? email.trim())}`,
        );
        return;
      }
      if (res.mfa_required) {
        if (!res.mfa_challenge) {
          setError("O servidor não forneceu um challenge MFA válido.");
          return;
        }
        setMfaChallenge(res.mfa_challenge);
        setPassword("");
        return;
      }
      if (!res.token) {
        setError(res.message ?? "Login incompleto");
        return;
      }
      applyAuthTokens({
        token: res.token,
        refresh_token: res.refresh_token,
        expires_in: res.expires_in,
      });
      saveUsername(res.username ?? (email.trim().split("@")[0] || "jogador"));
      navigate(returnTo, { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha no login");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="mx-auto max-w-5xl">
      <div className="grid gap-6 lg:grid-cols-[420px_1fr]">
        <div className="zt-panel h-fit">
          <div className="zt-panel-title">Entrar</div>
          <form className="space-y-4 p-5" onSubmit={onSubmit}>
          {sessionExpired && (
            <p
              className="rounded border border-amber-600 bg-amber-950/60 px-3 py-2 text-sm text-amber-100"
              role="alert"
            >
              Sua sessão expirou ou foi encerrada. Entre novamente para continuar com segurança.
            </p>
          )}
          {mfaChallenge ? (
            <div>
              <label className="zt-label" htmlFor="mfa-code">
                Código do autenticador
              </label>
              <input
                id="mfa-code"
                type="text"
                className="zt-input"
                autoComplete="one-time-code"
                inputMode="numeric"
                pattern="[0-9]{6}"
                maxLength={6}
                required
                autoFocus
                value={mfaCode}
                onChange={(e) => setMfaCode(e.target.value.replace(/\D/g, ""))}
              />
              <button
                type="button"
                className="mt-3 text-sm text-felt-200 hover:underline"
                onClick={() => {
                  setMfaChallenge(null);
                  setMfaCode("");
                }}
              >
                Voltar ao login
              </button>
            </div>
          ) : (
            <>
              <div>
                <label className="zt-label" htmlFor="email">
                  E-mail
                </label>
                <input
                  id="email"
                  type="email"
                  className="zt-input"
                  autoComplete="email"
                  required
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                />
              </div>
              <div>
                <label className="zt-label" htmlFor="password">
                  Senha
                </label>
                <input
                  id="password"
                  type="password"
                  className="zt-input"
                  autoComplete="current-password"
                  required
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
              </div>
            </>
          )}
          {error && (
            <p className="rounded border border-red-800 bg-red-950/50 px-3 py-2 text-sm text-red-200">
              {error}
            </p>
          )}
          <button type="submit" className="zt-btn-primary w-full" disabled={loading}>
            {loading
              ? mfaChallenge ? "Verificando…" : "Entrando…"
              : mfaChallenge
                ? "Confirmar MFA"
                : "Entrar"}
          </button>
          <p className="text-center text-sm text-felt-300">
            Novo por aqui?{" "}
            <Link to="/register" className="font-semibold text-gold-bright hover:underline">
              Criar conta
            </Link>
            {" · "}
            <Link to="/verify-email" className="font-semibold text-felt-200 hover:underline">
              Verificar e-mail
            </Link>
          </p>
          </form>
        </div>

        <div className="space-y-4">
          <div className="zt-panel p-5">
            <h3 className="text-sm font-bold uppercase tracking-wide text-gold-bright">
              Por que entrar agora?
            </h3>
            <ul className="mt-3 list-disc space-y-1.5 pl-5 text-sm leading-relaxed text-felt-200">
              <li>Lobby ao vivo com mesas por stake e variante</li>
              <li>Mínimo 2 na mesma mesa para a mão começar — chame um amigo</li>
              <li>Play Money e Jogo Real separados, sem misturar saldos</li>
            </ul>
            <Link to="/register" className="zt-btn-primary mt-4 w-full justify-center">
              Criar minha conta
            </Link>
          </div>

          <div className="zt-panel p-5">
            <h3 className="text-sm font-bold text-gold-bright">Dica para iniciantes</h3>
            <p className="mt-2 text-sm leading-relaxed text-felt-300">
              Comece no <span className="font-semibold text-cream">Botão</span> com mãos largas e no{" "}
              <span className="font-semibold text-cream">UTG</span> só com 12% das mãos. Posição é tudo.
            </p>
            <Link to="/" className="mt-3 inline-flex text-xs font-semibold text-gold-soft hover:text-gold-bright hover:underline">
              Ver história e dicas →
            </Link>
          </div>

          <div className="rounded border border-felt-600 bg-felt-850 px-4 py-3 text-xs leading-relaxed text-felt-400">
            Demo · Sem dinheiro real obrigatório · Verificação por e-mail em 6 dígitos
          </div>
        </div>
      </div>
    </div>
  );
}
