interface ActionButtonsProps {
  phase: string;
  isCurrentTurn: boolean;
  playerStack: number;
  currentBet: number;
  minRaise: number;
  bigBlind: number;
  onAction: (action: string, amount?: number) => void;
  disabled?: boolean;
}

export default function ActionButtons({
  phase,
  isCurrentTurn,
  playerStack,
  currentBet,
  minRaise,
  bigBlind,
  onAction,
  disabled,
}: ActionButtonsProps) {
  if (phase === 'waiting' || phase === 'showdown') return null;

  const callAmount = currentBet;
  const canCheck = currentBet === 0;
  const minRaiseAmount = Math.max(minRaise, currentBet + bigBlind);
  const maxRaiseAmount = playerStack;

  return (
    <div className="flex items-center gap-2">
      {isCurrentTurn ? (
        <>
          {canCheck ? (
            <button
              onClick={() => onAction('check')}
              disabled={disabled}
              className="px-4 py-2 rounded-lg text-sm font-semibold bg-green-600/30 text-green-400 hover:bg-green-600/50 border border-green-500/30 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
            >
              ✓ Check
            </button>
          ) : (
            <button
              onClick={() => onAction('call')}
              disabled={disabled}
              className="px-4 py-2 rounded-lg text-sm font-semibold bg-blue-600/30 text-blue-400 hover:bg-blue-600/50 border border-blue-500/30 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
            >
              Call ${callAmount}
            </button>
          )}

          <button
            onClick={() => onAction('raise', minRaiseAmount)}
            disabled={disabled || playerStack < minRaiseAmount}
            className="px-4 py-2 rounded-lg text-sm font-semibold bg-yellow-600/30 text-yellow-400 hover:bg-yellow-600/50 border border-yellow-500/30 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
          >
            Raise ${minRaiseAmount}
          </button>

          <button
            onClick={() => onAction('allIn')}
            disabled={disabled}
            className="px-4 py-2 rounded-lg text-sm font-semibold bg-red-600/30 text-red-400 hover:bg-red-600/50 border border-red-500/30 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
          >
            All-in ${playerStack}
          </button>

          <button
            onClick={() => onAction('fold')}
            disabled={disabled}
            className="px-4 py-2 rounded-lg text-sm font-semibold bg-gray-600/30 text-gray-400 hover:bg-gray-600/50 border border-gray-500/30 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
          >
            ✕ Fold
          </button>
        </>
      ) : (
        <div className="text-white/30 text-sm italic">
          Aguardando jogadores...
        </div>
      )}
    </div>
  );
}