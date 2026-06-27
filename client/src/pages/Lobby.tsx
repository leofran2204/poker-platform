import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import { useGameStore } from '../store/gameStore';
import { useSocketStore } from '../store/socketStore';
import { Swords, LogOut, Plus, Users, DollarSign, Clock, RefreshCw } from 'lucide-react';

export default function Lobby() {
  const { user, logout } = useAuthStore();
  const { tables, loading, fetchTables } = useGameStore();
  const { connect, disconnect, connected } = useSocketStore();
  const navigate = useNavigate();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newTable, setNewTable] = useState({
    name: '',
    smallBlind: 10,
    bigBlind: 20,
    minBuyIn: 200,
    maxBuyIn: 2000,
    maxPlayers: 6,
  });
  const [createError, setCreateError] = useState('');

  useEffect(() => {
    const token = useAuthStore.getState().token;
    if (token) {
      fetchTables(token);
      connect();
    }
    return () => {
      disconnect();
    };
  }, []);

  const handleCreateTable = async () => {
    setCreateError('');
    const token = useAuthStore.getState().token;
    if (!token) return;

    try {
      const res = await fetch('http://localhost:3001/api/tables', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify(newTable),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error || 'Erro ao criar mesa');
      setShowCreateModal(false);
      navigate(`/table/${data.table.id}`);
    } catch (err: any) {
      setCreateError(err.message);
    }
  };

  const handleJoinTable = (tableId: string) => {
    navigate(`/table/${tableId}`);
  };

  const handleLogout = () => {
    disconnect();
    logout();
    navigate('/login');
  };

  const blinds = [
    { sb: 5, bb: 10 },
    { sb: 10, bb: 20 },
    { sb: 25, bb: 50 },
    { sb: 50, bb: 100 },
    { sb: 100, bb: 200 },
    { sb: 250, bb: 500 },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#0a0a1a] via-[#1a1a2e] to-[#0d0d1f]">
      {/* Header */}
      <header className="border-b border-white/10 bg-black/20 backdrop-blur-xl">
        <div className="max-w-6xl mx-auto px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-yellow-500 to-yellow-700 flex items-center justify-center shadow-lg shadow-yellow-500/20">
              <Swords size={20} className="text-white" />
            </div>
            <div>
              <h1 className="text-lg font-bold font-poker text-yellow-400">Poker Platform</h1>
              <p className="text-xs text-white/30">Texas Hold'em</p>
            </div>
          </div>

          <div className="flex items-center gap-4">
            {/* Status da conexão */}
            <div className="flex items-center gap-1.5">
              <div className={`w-2 h-2 rounded-full ${connected ? 'bg-green-400' : 'bg-red-400'}`} />
              <span className="text-xs text-white/40">{connected ? 'Conectado' : 'Desconectado'}</span>
            </div>

            {/* Saldo */}
            <div className="flex items-center gap-1.5 bg-white/5 rounded-lg px-3 py-1.5 border border-white/10">
              <DollarSign size={14} className="text-yellow-400" />
              <span className="text-sm font-bold text-white">{user?.balance?.toLocaleString() || 0}</span>
            </div>

            {/* Usuário */}
            <div className="flex items-center gap-2">
              <div
                className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold text-white"
                style={{ background: user?.avatar || 'linear-gradient(135deg, #667eea, #764ba2)' }}
              >
                {user?.username?.charAt(0).toUpperCase()}
              </div>
              <span className="text-sm text-white/80">{user?.username}</span>
            </div>

            {/* Refresh */}
            <button
              onClick={() => fetchTables(useAuthStore.getState().token!)}
              className="p-2 rounded-lg hover:bg-white/5 text-white/40 hover:text-white/60 transition-all"
              title="Atualizar"
            >
              <RefreshCw size={16} />
            </button>

            {/* Logout */}
            <button
              onClick={handleLogout}
              className="p-2 rounded-lg hover:bg-white/5 text-white/40 hover:text-red-400 transition-all"
              title="Sair"
            >
              <LogOut size={16} />
            </button>
          </div>
        </div>
      </header>

      {/* Main */}
      <main className="max-w-6xl mx-auto px-4 py-8">
        {/* Hero */}
        <div className="text-center mb-10">
          <h2 className="text-3xl font-bold text-white mb-2">Bem-vindo ao Poker Platform</h2>
          <p className="text-white/40">Escolha uma mesa ou crie a sua para começar a jogar Texas Hold'em</p>
        </div>

        {/* Mesas */}
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <Users size={20} className="text-yellow-400" />
            Mesas Disponíveis
          </h3>
          <button
            onClick={() => setShowCreateModal(true)}
            className="btn-primary flex items-center gap-2 text-sm"
          >
            <Plus size={16} />
            Criar Mesa
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center py-20">
            <div className="w-8 h-8 border-2 border-yellow-500/30 border-t-yellow-500 rounded-full animate-spin" />
          </div>
        ) : tables.length === 0 ? (
          <div className="text-center py-20 bg-white/[0.02] rounded-2xl border border-white/5">
            <Swords size={48} className="mx-auto text-white/10 mb-4" />
            <p className="text-white/30 text-lg mb-2">Nenhuma mesa disponível</p>
            <p className="text-white/20 text-sm mb-6">Crie uma mesa para começar a jogar</p>
            <button
              onClick={() => setShowCreateModal(true)}
              className="btn-primary inline-flex items-center gap-2"
            >
              <Plus size={16} />
              Criar Primeira Mesa
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {tables.map((table: any) => (
              <div
                key={table.id}
                className="bg-white/5 backdrop-blur-xl rounded-xl p-5 border border-white/10 hover:border-yellow-500/30 transition-all cursor-pointer group"
                onClick={() => handleJoinTable(table.id)}
              >
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <h4 className="font-semibold text-white group-hover:text-yellow-400 transition-colors">
                      {table.name}
                    </h4>
                    <p className="text-xs text-white/30 mt-0.5">Texas Hold'em</p>
                  </div>
                  <div className="flex items-center gap-1 text-white/40 text-xs">
                    <Users size={12} />
                    <span>{table.currentPlayers || 0}/{table.maxPlayers}</span>
                  </div>
                </div>

                <div className="space-y-1.5">
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-white/40">Blinds</span>
                    <span className="text-white/60 font-medium">${table.smallBlind}/${table.bigBlind}</span>
                  </div>
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-white/40">Buy-in</span>
                    <span className="text-white/60 font-medium">${table.minBuyIn} - ${table.maxBuyIn}</span>
                  </div>
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-white/40">Status</span>
                    <span className={`font-medium ${table.status === 'active' ? 'text-green-400' : 'text-yellow-400'}`}>
                      {table.status === 'active' ? 'Ativa' : 'Aguardando'}
                    </span>
                  </div>
                </div>

                <button className="mt-4 w-full py-2 rounded-lg text-sm font-semibold bg-yellow-600/20 text-yellow-400 hover:bg-yellow-600/30 border border-yellow-500/20 transition-all opacity-0 group-hover:opacity-100">
                  Entrar na Mesa
                </button>
              </div>
            ))}
          </div>
        )}
      </main>

      {/* Modal Criar Mesa */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-[#1a1a2e] rounded-2xl p-8 w-full max-w-md border border-white/10 shadow-2xl animate-slide-up">
            <h3 className="text-xl font-semibold text-white mb-6">Criar Nova Mesa</h3>

            {createError && (
              <div className="bg-red-500/10 border border-red-500/30 text-red-400 text-sm rounded-lg px-4 py-2 mb-4">
                {createError}
              </div>
            )}

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-white/60 mb-1">Nome da Mesa</label>
                <input
                  type="text"
                  value={newTable.name}
                  onChange={(e) => setNewTable({ ...newTable, name: e.target.value })}
                  placeholder="Ex: Mesa High Stakes"
                  required
                />
              </div>

              <div>
                <label className="block text-sm text-white/60 mb-1">Blinds</label>
                <select
                  value={`${newTable.smallBlind}/${newTable.bigBlind}`}
                  onChange={(e) => {
                    const [sb, bb] = e.target.value.split('/').map(Number);
                    setNewTable({
                      ...newTable,
                      smallBlind: sb,
                      bigBlind: bb,
                      minBuyIn: sb * 20,
                      maxBuyIn: sb * 200,
                    });
                  }}
                  className="w-full bg-white/5 border border-white/10 text-white rounded-lg px-3 py-2.5 text-sm outline-none focus:border-yellow-500/50"
                >
                  {blinds.map((b) => (
                    <option key={b.sb} value={`${b.sb}/${b.bb}`}>
                      ${b.sb}/${b.bb}
                    </option>
                  ))}
                </select>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-white/60 mb-1">Min Buy-in</label>
                  <input
                    type="number"
                    value={newTable.minBuyIn}
                    onChange={(e) => setNewTable({ ...newTable, minBuyIn: Number(e.target.value) })}
                    min={newTable.smallBlind * 20}
                  />
                </div>
                <div>
                  <label className="block text-sm text-white/60 mb-1">Max Buy-in</label>
                  <input
                    type="number"
                    value={newTable.maxBuyIn}
                    onChange={(e) => setNewTable({ ...newTable, maxBuyIn: Number(e.target.value) })}
                    max={newTable.smallBlind * 200}
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm text-white/60 mb-1">Máximo de Jogadores</label>
                <select
                  value={newTable.maxPlayers}
                  onChange={(e) => setNewTable({ ...newTable, maxPlayers: Number(e.target.value) })}
                  className="w-full bg-white/5 border border-white/10 text-white rounded-lg px-3 py-2.5 text-sm outline-none focus:border-yellow-500/50"
                >
                  {[2, 3, 4, 5, 6].map((n) => (
                    <option key={n} value={n}>
                      {n} jogadores
                    </option>
                  ))}
                </select>
              </div>
            </div>

            <div className="flex gap-3 mt-6">
              <button
                onClick={() => setShowCreateModal(false)}
                className="btn-secondary flex-1"
              >
                Cancelar
              </button>
              <button
                onClick={handleCreateTable}
                disabled={!newTable.name}
                className="btn-primary flex-1"
              >
                Criar Mesa
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}