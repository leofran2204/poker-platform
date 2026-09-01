import { FormEvent, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { applyAuthTokens, register } from "@/api/client";
import { saveUsername } from "@/lib/auth";

export function RegisterPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [passwordConfirm, setPasswordConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);

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

    setLoading(true);
    try {
      const res = await register(username.trim(), email.trim(), password, passwordConfirm);
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
    <div className="mx-auto max-w-5xl">
      <div className="grid gap-6 lg:grid-cols-[420px_1fr]">
        <div className="zt-panel h-fit">
          <div className="zt-panel-title">Criar conta</div>
          <form className="space-y-4 p-5" onSubmit={onSubmit}>
            <p className="text-sm text-felt-300">
              Contas de demo recebem play-money. Após o cadastro, confirme o e-mail
              com o código de 6 dígitos para liberar o lobby.
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
              maxLength={32}
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

        <div className="space-y-4">
          <div className="zt-panel p-5">
            <h3 className="text-sm font-bold uppercase tracking-wide text-gold-bright">
              O que você ganha ao criar?
            </h3>
            <ul className="mt-3 list-disc space-y-1.5 pl-5 text-sm leading-relaxed text-felt-200">
              <li>
                <span className="font-semibold text-cream">R$ 1.000 Play Money</span> por dia para treinar sem risco
              </li>
              <li>Lobby completo: Hold&apos;em, Short Deck e Omaha Short Deck</li>
              <li>História do pôquer e Dica do Pró para evoluir desde a primeira mão</li>
            </ul>
            <Link to="/login" className="zt-btn-secondary mt-4 w-full justify-center">
              Já tenho conta — Entrar
            </Link>
          </div>

          <div className="zt-panel p-5">
            <h3 className="text-sm font-bold text-gold-bright">Como funciona?</h3>
            <ol className="mt-3 list-decimal space-y-1.5 pl-5 text-sm leading-relaxed text-felt-300">
              <li>Crie com e-mail e senha forte (8+ com maiúscula, minúscula e número)</li>
              <li>Confirme com código de 6 dígitos (caixa de entrada e spam)</li>
              <li>Escolha Play Money ou Jogo Real no topo e entre na mesa com +1 amigo</li>
            </ol>
          </div>

          <div className="rounded border border-felt-600 bg-felt-850 px-4 py-3 text-xs leading-relaxed text-felt-400">
            Leva 30 segundos. Sem cartão, sem pegadinha. Jogo limpo desde o cadastro.
          </div>
        </div>
      </div>
    </div>
  );
}
