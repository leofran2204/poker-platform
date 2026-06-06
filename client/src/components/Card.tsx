interface CardProps {
  suit: 'h' | 'd' | 'c' | 's';
  rank: string;
  hidden?: boolean;
  size?: 'small' | 'normal' | 'large';
}

const suitSymbols: Record<string, string> = {
  h: '♥',
  d: '♦',
  c: '♣',
  s: '♠',
};

const suitColors: Record<string, string> = {
  h: 'card-red',
  d: 'card-red',
  c: 'card-black',
  s: 'card-black',
};

const sizeClasses = {
  small: 'card-small',
  normal: '',
  large: 'card-large',
};

export default function Card({ suit, rank, hidden, size = 'normal' }: CardProps) {
  if (hidden) {
    return (
      <div className={`card card-back ${sizeClasses[size]}`}>
        <div className="text-white/30 text-xs font-bold">♠♥</div>
      </div>
    );
  }

  const isRed = suit === 'h' || suit === 'd';

  return (
    <div className={`card ${suitColors[suit]} ${sizeClasses[size]} animate-card-deal`}>
      <div className="flex items-center justify-between w-full px-1.5">
        <span className="text-xs leading-none">{rank}</span>
      </div>
      <span className={`text-xl ${size === 'small' ? 'text-base' : ''} ${size === 'large' ? 'text-2xl' : ''}`}>
        {suitSymbols[suit]}
      </span>
      <div className="flex items-center justify-between w-full px-1.5 rotate-180">
        <span className="text-xs leading-none">{rank}</span>
      </div>
    </div>
  );
}