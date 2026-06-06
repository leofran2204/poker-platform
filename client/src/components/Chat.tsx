import { useState, useRef, useEffect } from 'react';
import { Send } from 'lucide-react';

interface ChatMessage {
  userId: string;
  username: string;
  message: string;
  timestamp: string;
}

interface ChatProps {
  messages: ChatMessage[];
  onSendMessage: (message: string) => void;
  currentUserId?: string;
}

export default function Chat({ messages, onSendMessage, currentUserId }: ChatProps) {
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim()) {
      onSendMessage(input.trim());
      setInput('');
    }
  };

  return (
    <div className="flex flex-col h-full bg-black/20 rounded-lg overflow-hidden border border-white/5">
      {/* Header */}
      <div className="px-3 py-2 border-b border-white/10">
        <span className="text-xs font-semibold text-white/60 uppercase tracking-wider">Chat</span>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {messages.length === 0 && (
          <div className="text-center text-white/20 text-xs mt-4">
            Nenhuma mensagem ainda
          </div>
        )}
        {messages.map((msg, i) => (
          <div
            key={i}
            className={`text-xs leading-relaxed ${
              msg.userId === currentUserId ? 'text-yellow-300/80' : 'text-white/70'
            }`}
          >
            <span className="font-semibold">{msg.username}: </span>
            <span>{msg.message}</span>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <form onSubmit={handleSubmit} className="flex items-center gap-1 p-2 border-t border-white/10">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Digite sua mensagem..."
          className="flex-1 bg-white/5 border border-white/10 rounded px-2 py-1.5 text-xs text-white placeholder-white/20 outline-none focus:border-yellow-500/50 transition-colors"
          maxLength={200}
        />
        <button
          type="submit"
          disabled={!input.trim()}
          className="p-1.5 rounded bg-yellow-600/30 text-yellow-400 hover:bg-yellow-600/50 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
        >
          <Send size={14} />
        </button>
      </form>
    </div>
  );
}