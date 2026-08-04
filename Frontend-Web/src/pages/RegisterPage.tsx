import { FormEvent, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { applyAuthTokens, register } from "@/api/client";
import { saveUsername } from "@/lib/auth";

export function RegisterPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      const tokens = await register(username.trim(), email.trim(), password);
      applyAuthTokens(tokens);
      saveUsername(username.trim());
      navigate("/lobby");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha no registro");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="mx-auto max-w-md">
      <div className="zt-panel">
        <div className="zt-panel-title">Criar conta</div>
        <form className="space-y-4 p-5" onSubmit={onSubmit}>
          <p className="text-sm text-felt-300">
            Contas de demo recebem play-money para testar mesas. Sem depósito real nesta fase.
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
              value={password}
              onChange={(e) => setPassword(e.target.value)}
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
          </p>
        </form>
      </div>
    </div>
  );
}
