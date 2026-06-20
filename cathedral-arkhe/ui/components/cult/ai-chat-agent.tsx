'use client';
import { useState } from 'react';
import { AnimatedCard } from './animated-card';

export function AiChatAgent({ className }: { className?: string }) {
  const [messages, setMessages] = useState<{ role: string; content: string }[]>([]);
  const [input, setInput] = useState('');

  const send = async () => {
    if (!input.trim()) return;
    setMessages([...messages, { role: 'user', content: input }]);
    try {
      const res = await fetch('/api/engine', {
        method: 'POST',
        body: JSON.stringify({ prompt: input }),
        headers: { 'Content-Type': 'application/json' },
      });
      const data = await res.json();
      setMessages(prev => [...prev, { role: 'assistant', content: data.response }]);
    } catch (e) {
      setMessages(prev => [...prev, { role: 'assistant', content: 'Error communicating with engine.' }]);
    }
    setInput('');
  };

  return (
    <AnimatedCard className={`p-4 flex flex-col ${className || ''}`}>
      <div className="flex-1 overflow-y-auto space-y-2 mb-2">
        {messages.map((m, i) => (
          <div key={i} className={`p-2 rounded ${m.role === 'user' ? 'bg-blue-600' : 'bg-zinc-700'}`}>
            {m.content}
          </div>
        ))}
      </div>
      <div className="flex gap-2">
        <input
          className="flex-1 bg-zinc-800 p-2 rounded"
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={e => e.key === 'Enter' && send()}
        />
        <button onClick={send} className="bg-amber-500 px-4 py-2 rounded text-zinc-950 font-medium">Send</button>
      </div>
    </AnimatedCard>
  );
}
