import Card from './Card';

interface Player {
  id: string;
  userId: string;
  username: string;
  avatar: string;
  stack: number;
  bet: number;
  holeCards: Array<{ suit: 'h' | 'd' | 'c' | 's'; rank: string }>;
  folded: boolean;
  isDealer: boolean;
  isCurrentTurn: boolean;
  isAllIn: boolean;
  isSittingOut: boolean;
  lastAction: string | null;
  seatIndex: number;
}

interface PlayerSeatProps {
  player: Player;
  position: { top: string; left: string };
  isCurrentUser?: boolean;
}

const actionLabels: Record<string, string> = {
  fold: 'Fold',
  check: 'Check',
  call: 'Call',
  raise: 'Raise',
  allIn: 'All-in',
};

export default function PlayerSeat({ player, position, isCurrentUser }: PlayerSeatProps) {
  return (
    <div
      className="absolute flex flex-col items-center animate-fade-in"
      style={{ top: position.top, left: position.left, transform: 'translate(-50%, -50%)' }}
    >
      {/* Avatar e nome */}
      <div className="flex flex-col items-center mb-1">
        <div
          className={`w-12 h-12 rounded-full flex items-center justify-center text-lg font-bold mb-1 border-2 transition-all ${
            player.isCurrentTurn
              ? 'border-yellow-400 shadow-lg shadow-yellow-400/30 animate-pulse-glow'
              : 'border-white/20'
          } ${player.folded ? 'opacity-40' : ''}`}
          style={{ background: player.avatar || 'linear-gradient(135deg, #667eea, #764ba2)' }}
        >
          {player.username.charAt(0).toUpperCase()}
        </div>
        <span className={`text-xs font-semibold ${isCurrentUser ? 'text-yellow-400' : 'text-white/80'}`}>
          {player.username}
          {isCurrentUser && ' (Você)'}
        </span>
      </div>

      {/* Fichas */}
      <div className="flex items-center gap-1 mb-1">
        <div className="chip chip-gold">
          <span>$</span>
        </div>
        <span className="text-sm font-bold text-white">{player.stack.toLocaleString()}</span>
      </div>

      {/* Aposta atual */}
      {player.bet > 0 && !player.folded && (
        <div className="flex items-center gap-1 mb-1 animate-chip-stack">
          <div className="chip chip-red">
            <span>$</span>
          </div>
          <span className="text-xs font-semibold text-red-400">{player.bet.toLocaleString()}</span>
        </div>
      )}

      {/* Cartas do jogador */}
      {player.holeCards.length > 0 && (
        <div className="flex gap-1 mb-1">
          {player.holeCards.map((card, i) => (
            <Card key={i} suit={card.suit} rank={card.rank} size="small" />
          ))}
        </div>
      )}

      {/* Última ação */}
      {player.lastAction && (
        <span className={`text-xs font-bold px-2 py-0.5 rounded ${
          player.lastAction === 'fold' ? 'bg-red-500/20 text-red-400' :
          player.lastAction === 'raise' || player.lastAction === 'allIn' ? 'bg-yellow-500/20 text-yellow-400' :
          'bg-green-500/20 text-green-400'
        }`}>
          {actionLabels[player.lastAction] || player.lastAction}
        </span>
      )}

      {/* Dealer button */}
      {player.isDealer && (
        <div className="dealer-button absolute -top-2 -right-2">
          D
        </div>
      )}

      {/* All-in badge */}
      {player.isAllIn && (
        <span className="text-xs font-bold text-red-400 bg-red-500/20 px-2 py-0.5 rounded mt-1">
          ALL-IN
        </span>
      )}

      {/* Sitting out */}
      {player.isSittingOut && (
        <span className="text-xs text-white/40 mt-1">Ausente</span>
      )}
    </div>
  );
}