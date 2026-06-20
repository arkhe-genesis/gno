'use client';
import { useSSE } from '@/hooks/use-sse';
import { useCathedralStore } from '@/lib/store';
import { MetricCard } from '@/components/dashboard/metric-card';
import { MetricsChart } from '@/components/dashboard/metrics-chart';
import { AiChatAgent } from '@/components/cult/ai-chat-agent';

export default function DashboardPage() {
  const { engine, wormhole, depin, setEngine, setWormhole, setDePIN } = useCathedralStore();

  useSSE('/api/metrics', (data) => {
    if (data.type === 'engine') setEngine(data);
    if (data.type === 'wormhole') setWormhole(data);
    if (data.type === 'depin') setDePIN(data);
  });

  return (
    <div className="grid grid-cols-12 gap-6">
      <div className="col-span-3"><MetricCard title="Engine Status" value={engine.status} /></div>
      <div className="col-span-3"><MetricCard title="Wormhole Compression" value={`${(wormhole.compressionRatio * 100).toFixed(1)}%`} /></div>
      <div className="col-span-3"><MetricCard title="DePIN Workers" value={depin.activeWorkers} /></div>
      <div className="col-span-3"><MetricCard title="Avg Reputation" value={`${(depin.avgReputation * 100).toFixed(1)}%`} /></div>
      <div className="col-span-5"><AiChatAgent className="h-96" /></div>
      <div className="col-span-7"><MetricsChart className="h-96" /></div>
    </div>
  );
}
