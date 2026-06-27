import { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import { useGameStore } from '../store/gameStore';
import { useSocketStore } from '../store/socketStore';
import Card from '../components/Card';
import PlayerSeat from '../components/PlayerSeat';
import Chat from '../components/Chat';
import ActionButtons from '../components/ActionButtons';
import { ArrowLeft, Users, DollarSign, Clock, Swords } from 'lucide-react';

interface ChatMessage {
  userId: string;
  username: string;
  message: string;
  timestamp: string;
}

const seatPositions = [
  { top: '85%', left: '50%' },  // Jogador atual (baixo centro)
  { top: '55%', left: '85%' },  // Direita
  { top: '20%', left: '70%' },  // Superior direito
  { top: '10%', left: '50%' },  // Topo centro
  { top: '20%', left: '30%' },  // Superior esquerdo
  { top: '55%', left: '15%' },  // Esquerda
];

export default function Table() {
  const { tableId } = useParams<{ tableId: string }>();
  const navigate = useNavigate();
  const { user } = useAuthStore();
  const { currentTable, setCurrentTable } = useGameStore();
  const { socket, connected } = useSocketStore();
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [showBuyInModal, setShowBuyInModal] = useState(false);
  const [buyInAmount, setBuyInAmount] = useState(200);
  const [joinError, setJoinError] = useState('');

  // Entrar na mesa
  useEffect(() => {
    if (!socket || !connected || !tableId || !user) return;

    socket.emit('joinTable', { tableId, buyIn: 0 }, (response: any) => {
      if (response?.error) {
        setJoinError(response.error);
        if (response.requiresBuyIn) {
          setBuyInAmount(response.minBuyIn || 200);
          setShowBuyInModal(true);
        }
      }
    });

    // Chat listener
    const handleChat = (data: ChatMessage) => {
      setChatMessages((prev) => [...prev, data]);
    };
    socket.on('chatMessage', handleChat);

    return () => {
      socket.emit('leaveTable', { tableId });
      socket.off('chatMessage', handleChat);
    };
  }, [socket, connected, tableId, user]);

  const handleBuyIn = () => {
    if (!socket || !tableId) return;
    socket.emit('joinTable', { tableId, buyIn: buyInAmount }, (response: any) => {
      if (response?.error) {
        setJoinError(response.error);
      } else {
        setShowBuyInModal(false);
        setJoinError('');
      }
    });
  };

  const handleAction = useCallback(
    (action: string, amount?: number) => {
      if (!socket || !tableId) return;
      socket.emit('playerAction', { tableId, action, amount });
    },
    [socket, tableId]
  );

  const handleSendMessage = useCallback(
    (message: string) => {
      if (!socket || !tableId) return;
      socket.emit('chatMessage', { tableId, message });
    },
    [socket, tableId]
  );

  const handleLeave = () => {
    if (socket && tableId) {
      socket.emit('leaveTable', { tableId });
    }
    setCurrentTable(null);
    navigate('/');
  };

  if (!currentTable) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-[#0a0a1a] via-[#1a1a2e] to-[#0d0d1f] flex items-center justify-center">
        <div className="text-center">
          {joinError ? (
            <>
              <div className="w-16 h-16 mx-auto rounded-full bg-red-500/10 border border-red-500/30 flex items-center justify-center mb-4">
                <Swords size={32} className="text-red-400" />
              </div>
              <p className="text-red-400 mb-2">{joinError}</p>
              <button onClick={() => navigate('/')} className="btn-secondary">
                Voltar ao Lobby
              </button>
            </>
          ) : (
            <>
              <div className="w-10 h-10 border-2 border-yellow-500/30 border-t-yellow-500 rounded-full animate-spin mx-auto mb-4" />
              <p className="text-white/40">Entrando na mesa...</p>
            </>
          )}
        </div>
      </div>
    );
  }

  const currentPlayer = currentTable.players.find((p) => p.userId === user?.id);
  const isCurrentTurn = currentPlayer?.isCurrentTurn || false;

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#0a0a1a] via-[#1a1a2e] to-[#0d0d1f] flex flex-col">
      {/* Header */}
      <header className="border-b border-white/10 bg-black/20 backdrop-blur-xl px-4 py-2 flex items-center justify-between z-10">
        <div className="flex items-center gap-3">
          <button
            onClick={handleLeave}
            className="p-2 rounded-lg hover:bg-white/5 text-white/40 hover:text-white/60 transition-all"
          >
            <ArrowLeft size={18} />
          </button>
          <div>
            <h1 className="text-sm font-semibold text-white">{currentTable.name}</h1>
            <p className="text-xs text-white/30">Texas Hold'em - ${currentTable.smallBlind}/${currentTable.bigBlind}</p>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1.5 text-xs text-white/40">
            <Users size={14} />
            <span>{currentTable.players.length}/{currentTable.maxPlayers}</span>
          </div>
          <div className="flex items-center gap-1.5 text-xs text-white/40">
            <Clock size={14} />
            <span className="capitalize">{currentTable.phase === 'waiting' ? 'Aguardando' : currentTable.phase}</span>
          </div>
          <div className={`w-2 h-2 rounded-full ${connected ? 'bg-green-400' : 'bg-red-400'}`} />
        </div>
      </header>

      {/* Game Area */}
      <div className="flex-1 flex items-stretch p-4 gap-4">
        {/* Mesa */}
        <div className="flex-1 relative flex items-center justify-center">
          <div className="felt-table w-full max-w-4xl aspect-[2/1] flex flex-col items-center justify-center p-8">
            {/* Community Cards */}
            <div className="flex items-center gap-2 mb-4">
              {currentTable.communityCards.length > 0 ? (
                currentTable.communityCards.map((card, i) => (
                  <Card key={i} suit={card.suit} rank={card.rank} />
                ))
              ) : (
                currentTable.phase !== 'waiting' && (
                  <div className="flex gap-2">
                    {[1, 2, 3, 4, 5].map((i) => (
                      <Card key={i} suit="h" rank="A" hidden size="normal" />
                    ))}
                  </div>
                )
              )}
            </div>

            {/* Pot */}
            {currentTable.pot > 0 && (
              <div className="flex items-center gap-2 mb-2">
                <div className="chip chip-gold">
                  <span>$</span>
                </div>
                <span className="text-lg font-bold text-yellow-400">
                  Pot: ${currentTable.pot.toLocaleString()}
                </span>
              </div>
            )}

            {/* Winners */}
            {currentTable.winners && currentTable.winners.length > 0 && (
              <div className="text-center animate-slide-up">
                <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg px-4 py-2">
                  {currentTable.winners.map((w, i) => (
                    <p key={i} className="text-yellow-400 font-semibold">
                      {w.username} venceu ${w.winAmount.toLocaleString()} com {w.handName}
                    </p>
                  ))}
                </div>
              </div>
            )}

            {/* Jogadores */}
            {currentTable.players.map((player, index) => (
              <PlayerSeat
                key={player.id}
                player={player}
                position={seatPositions[index] || seatPositions[0]}
                isCurrentUser={player.userId === user?.id}
              />
            ))}
          </div>
        </div>

        {/* Sidebar */}
        <div className="w-72 flex flex-col gap-3">
          {/* Ações */}
          <div className="bg-black/20 rounded-xl p-4 border border-white/5">
            <h4 className="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3">Ações</h4>
            <ActionButtons
              phase={currentTable.phase}
              isCurrentTurn={isCurrentTurn}
              playerStack={currentPlayer?.stack || 0}
              currentBet={currentTable.currentBet}
              minRaise={currentTable.minRaise}
              bigBlind={currentTable.bigBlind}
              onAction={handleAction}
            />
          </div>

          {/* Informações do Jogador */}
          {currentPlayer && (
            <div className="bg-black/20 rounded-xl p-4 border border-white/5">
              <h4 className="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3">Seu Stack</h4>
              <div className="flex items-center gap-2">
                <DollarSign size={20} className="text-yellow-400" />
                <span className="text-2xl font-bold text-white">{currentPlayer.stack.toLocaleString()}</span>
              </div>
              {currentPlayer.bet > 0 && (
                <p className="text-xs text-red-400 mt-1">Aposta atual: ${currentPlayer.bet}</p>
              )}
            </div>
          )}

          {/* Chat */}
          <div className="flex-1 min-h-0">
            <Chat
              messages={chatMessages}
              onSendMessage={handleSendMessage}
              currentUserId={user?.id}
            />
          </div>
        </div>
      </div>

      {/* Modal Buy-in */}
      {showBuyInModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-[#1a1a2e] rounded-2xl p-8 w-full max-w-sm border border-white/10 shadow-2xl animate-slide-up">
            <h3 className="text-xl font-semibold text-white mb-2">Comprar Fichas</h3>
            <p className="text-white/40 text-sm mb-6">
              Você precisa de fichas para entrar nesta mesa.
            </p>

            {joinError && (
              <div className="bg-red-500/10 border border-red-500/30 text-red-400 text-sm rounded-lg px-4 py-2 mb-4">
                {joinError}
              </div>
            )}

            <div>
              <label className="block text-sm text-white/60 mb-1">Valor</label>
              <input
                type="number"
                value={buyInAmount}
                onChange={(e) => setBuyInAmount(Number(e.target.value))}
                min={currentTable.minBuyIn}
                max={currentTable.maxBuyIn}
              />
              <div className="flex justify-between mt-1 text-xs text-white/30">
                <span>Min: ${currentTable.minBuyIn}</span>
                <span>Max: ${currentTable.maxBuyIn}</span>
              </div>
            </div>

            <div className="flex gap-3 mt-6">
              <button
                onClick={() => {
                  setShowBuyInModal(false);
                  navigate('/');
                }}
                className="btn-secondary flex-1"
              >
                Voltar
              </button>
              <button
                onClick={handleBuyIn}
                disabled={buyInAmount < currentTable.minBuyIn || buyInAmount > currentTable.maxBuyIn}
                className="btn-primary flex-1"
              >
                Comprar Fichas
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}