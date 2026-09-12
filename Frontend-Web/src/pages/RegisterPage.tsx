import { FormEvent, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { applyAuthTokens, register } from "@/api/client";
import { ShowcaseTable } from "@/components/ShowcaseTable";
import { saveUsername } from "@/lib/auth";

export function RegisterPage() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [passwordConfirm, setPasswordConfirm] = useState("");
  const [inviteCode, setInviteCode] = useState((params.get("ref") ?? "").toUpperCase());
  const [termsAccepted, setTermsAccepted] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);

    if (!termsAccepted) {
      setError("Você precisa ler e aceitar os Termos de Uso e a Política de Privacidade para continuar.");
      return;
    }

    if (password !== passwordConfirm) {
      setError("As senhas não coincidem.");
      return;
    }
    if (password.length < 8) {
      setError("A senha deve ter no mínimo 8 caracteres.");
      return;
    }
    if (!/[A-Z]/.test(password) || !/[a-z]/.test(password) || !/[0-9]/.test(password)) {
      setError("Use ao menos 1 maiúscula, 1 minúscula e 1 dígito.");
      return;
    }
    if (!/^[A-Za-z0-9_]{3,30}$/.test(username.trim())) {
      setError("Usuário: 3 a 30 caracteres, só letras, números ou _.");
      return;
    }

    setLoading(true);
    try {
      const res = await register(
        username.trim(),
        email.trim(),
        password,
        passwordConfirm,
        inviteCode.trim() || undefined,
      );
      if (res.email_verification_required) {
        navigate(`/verify-email?email=${encodeURIComponent(res.email ?? email.trim())}`);
        return;
      }
      if (res.token) {
        applyAuthTokens({
          token: res.token,
          refresh_token: res.refresh_token,
          expires_in: res.expires_in,
        });
        saveUsername(username.trim());
        navigate("/lobby");
        return;
      }
      setError(res.message ?? "Registro incompleto — tente novamente.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha no registro");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="zt-auth-split">
      <div className="zt-panel">
        <div className="zt-panel-title">Criar conta</div>
        <form className="space-y-4 p-5" onSubmit={onSubmit}>
          <p className="text-sm text-felt-200">
            Dois minutos. Você entra com <strong className="text-gold-soft">R$ 150</strong> de
            Play Money para cash e <strong className="text-gold-soft">R$ 150</strong> para
            torneio. O código de e-mail vem depois.
          </p>
          <div>
            <label className="zt-label" htmlFor="username">
              Usuário
            </label>
            <input
              id="username"
              className="zt-input"
              required
              minLength={3}
              maxLength={30}
              pattern="[A-Za-z0-9_]+"
              title="3 a 30 caracteres: letras, números ou _"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
            />
          </div>
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
            <label className="zt-label" htmlFor="password">
              Senha
            </label>
            <input
              id="password"
              type="password"
              className="zt-input"
              required
              minLength={8}
              autoComplete="new-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
            <p className="mt-1 text-xs text-felt-400">
              Mín. 8 caracteres, com maiúscula, minúscula e número.
            </p>
          </div>
          <div>
            <label className="zt-label" htmlFor="passwordConfirm">
              Confirmar senha
            </label>
            <input
              id="passwordConfirm"
              type="password"
              className="zt-input"
              required
              minLength={8}
              autoComplete="new-password"
              value={passwordConfirm}
              onChange={(e) => setPasswordConfirm(e.target.value)}
            />
          </div>
          <div>
            <label className="zt-label" htmlFor="invite">
              Código de convite <span className="font-normal normal-case tracking-normal text-felt-400">(opcional)</span>
            </label>
            <input
              id="invite"
              className="zt-input uppercase"
              value={inviteCode}
              onChange={(e) => setInviteCode(e.target.value.toUpperCase())}
              placeholder="Se alguém te chamou, cola aqui"
              autoComplete="off"
            />
          </div>

          <div className="rounded border border-felt-700/80 bg-felt-950/60 p-3">
            <label className="flex items-start gap-2.5 text-xs text-felt-200 cursor-pointer">
              <input
                type="checkbox"
                required
                checked={termsAccepted}
                onChange={(e) => setTermsAccepted(e.target.checked)}
                className="mt-0.5 h-4 w-4 rounded border-felt-600 bg-felt-900 text-gold-bright focus:ring-gold-soft cursor-pointer shrink-0"
              />
              <span>
                Li, compreendi e concordo integralmente com os{" "}
                <Link to="/termos" target="_blank" className="font-semibold text-gold-bright underline">
                  Termos de Uso, Política de Privacidade (LGPD) e Regulamento da Rede
                </Link>
                . Declaro ter mais de 18 anos de idade.
              </span>
            </label>
          </div>

          {error && (
            <p className="rounded border border-red-800 bg-red-950/50 px-3 py-2 text-sm text-red-200">
              {error}
            </p>
          )}
          <button type="submit" className="zt-btn-primary w-full" disabled={loading}>
            {loading ? "Criando…" : "Criar conta"}
          </button>
          <p className="text-center text-sm text-felt-300">
            Já tem conta?{" "}
            <Link to="/login" className="font-semibold text-gold-bright hover:underline">
              Entrar
            </Link>
            {" · "}
            <Link to="/verify-email" className="font-semibold text-felt-200 hover:underline">
              Verificar e-mail
            </Link>
          </p>
        </form>
      </div>
      <div className="hidden lg:flex lg:justify-center">
        <ShowcaseTable />
      </div>
    </div>
  );
}
