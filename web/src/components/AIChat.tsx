import { useState, useRef, useEffect, type FormEvent } from 'react';

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

export default function AIChat() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const trimmed = input.trim();
    if (!trimmed || isLoading) return;

    setError(null);
    const userMessage: Message = { role: 'user', content: trimmed };
    const updatedMessages = [...messages, userMessage];
    setMessages(updatedMessages);
    setInput('');
    setIsLoading(true);

    try {
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          messages: updatedMessages.map((m) => ({
            role: m.role,
            content: m.content,
          })),
        }),
      });

      if (!response.ok) {
        throw new Error('Error al conectar con el chat');
      }

      const reader = response.body?.getReader();
      if (!reader) throw new Error('No response stream');

      const decoder = new TextDecoder();
      let assistantContent = '';
      setMessages((prev) => [...prev, { role: 'assistant', content: '' }]);

      let buffer = '';
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          const trimmedLine = line.trim();
          if (!trimmedLine || !trimmedLine.startsWith('data: ')) continue;

          const data = trimmedLine.slice(6);
          if (data === '[DONE]') break;

          try {
            const parsed = JSON.parse(data);
            if (typeof parsed === 'string') {
              assistantContent += parsed;
              setMessages((prev) => {
                const updated = [...prev];
                updated[updated.length - 1] = {
                  role: 'assistant',
                  content: assistantContent,
                };
                return updated;
              });
            } else if (parsed.error) {
              setError(parsed.error);
            }
          } catch {
            // Skip malformed data
          }
        }
      }
    } catch {
      setError('Chat no disponible. Por favor crea un ticket de soporte.');
      // Remove empty assistant message if we added one
      setMessages((prev) => {
        if (prev.length > 0 && prev[prev.length - 1].role === 'assistant' && !prev[prev.length - 1].content) {
          return prev.slice(0, -1);
        }
        return prev;
      });
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="flex flex-col h-[500px] bg-slate-900 border border-slate-800 rounded-xl overflow-hidden">
      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="text-center text-slate-500 mt-8">
            <p>Escribe tu pregunta para comenzar</p>
          </div>
        )}

        {messages.map((msg, i) => (
          <div
            key={i}
            className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[80%] px-4 py-2.5 rounded-xl text-sm whitespace-pre-wrap ${
                msg.role === 'user'
                  ? 'bg-emerald-600 text-white rounded-br-sm'
                  : 'bg-slate-800 text-slate-200 rounded-bl-sm'
              }`}
            >
              {msg.content}
            </div>
          </div>
        ))}

        {isLoading && messages[messages.length - 1]?.role !== 'assistant' && (
          <div className="flex justify-start">
            <div className="bg-slate-800 text-slate-400 px-4 py-2.5 rounded-xl rounded-bl-sm text-sm">
              <span className="inline-flex gap-1">
                <span className="animate-bounce">.</span>
                <span className="animate-bounce" style={{ animationDelay: '0.1s' }}>.</span>
                <span className="animate-bounce" style={{ animationDelay: '0.2s' }}>.</span>
              </span>
            </div>
          </div>
        )}

        <div ref={messagesEndRef}></div>
      </div>

      {/* Error */}
      {error && (
        <div className="px-4 py-3 bg-red-500/10 border-t border-red-500/20 text-red-400 text-sm flex items-center justify-between">
          <span>{error}</span>
          <a
            href="/dashboard/support/new"
            className="text-emerald-400 hover:text-emerald-300 underline ml-2"
          >
            Crear ticket
          </a>
        </div>
      )}

      {/* Input */}
      <form onSubmit={handleSubmit} className="p-4 border-t border-slate-800">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Escribe tu pregunta..."
            disabled={isLoading}
            className="flex-1 px-4 py-2.5 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 outline-none disabled:opacity-50"
          />
          <button
            type="submit"
            disabled={isLoading || !input.trim()}
            className="px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 disabled:bg-slate-700 disabled:text-slate-500 text-white font-medium rounded-lg transition-colors"
          >
            Enviar
          </button>
        </div>
      </form>
    </div>
  );
}
