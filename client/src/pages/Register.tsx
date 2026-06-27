import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import { UserPlus, Swords } from 'lucide-react';

export default function Register() {
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [localError, setLocalError] = useState('');
  const { register, loading, error } = useAuthStore();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError('');

    if (password !== confirmPassword) {
      setLocalError('Senhas não conferem');
      return;
    }

    if (password.length < 6) {
      setLocalError('Senha deve ter no mínimo 6 caracteres');
      return;
    }

    if (username.length < 3) {
      setLocalError('Usuário deve ter no mínimo 3 caracteres');
      return;
    }

    try {
      await register(username, email, password);
      navigate('/');
    } catch {}
  };

  const displayError = localError || error;

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-[#0a0a1a] via-[#1a1a2e] to-[#0d0d1f] p-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-br from-yellow-500 to-yellow-700 mb-4 shadow-lg shadow-yellow-500/20">
            <Swords size={32} className="text-white" />
          </div>
          <h1 className="text-3xl font-bold font-poker text-yellow-400">Poker Platform</h1>
              <p className="text-white/40 text-sm mt-1">Texas Hold'em</p>
        </div>

        {/* Form */}
        <div className="bg-white/5 backdrop-blur-xl rounded-2xl p-8 border border-white/10 shadow-xl">
          <h2 className="text-xl font-semibold text-white mb-6">Criar Conta</h2>

          {displayError && (
            <div className="bg-red-500/10 border border-red-500/30 text-red-400 text-sm rounded-lg px-4 py-2 mb-4">
              {displayError}
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm text-white/60 mb-1">Usuário</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="3-30 caracteres"
                required
                minLength={3}
                maxLength={30}
                autoFocus
              />
            </div>

            <div>
              <label className="block text-sm text-white/60 mb-1">Email</label>
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="seu@email.com"
                required
              />
            </div>

            <div>
              <label className="block text-sm text-white/60 mb-1">Senha</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Mínimo 6 caracteres"
                required
                minLength={6}
              />
            </div>

            <div>
              <label className="block text-sm text-white/60 mb-1">Confirmar Senha</label>
              <input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Repita a senha"
                required
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="btn-primary w-full flex items-center justify-center gap-2"
            >
              {loading ? (
                <span className="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              ) : (
                <UserPlus size={18} />
              )}
              {loading ? 'Criando...' : 'Criar Conta'}
            </button>
          </form>

          <div className="mt-6 text-center">
            <span className="text-white/40 text-sm">Já tem conta? </span>
            <Link to="/login" className="text-yellow-400 hover:text-yellow-300 text-sm font-semibold transition-colors">
              Fazer login
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}