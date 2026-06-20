'use client';
import { AnimatedTabs } from '@/components/cult/animated-tabs';
import { useCathedralStore } from '@/lib/store';

const tabs = [
  { label: 'Dashboard', value: 'dashboard' },
  { label: 'Engine', value: 'engine' },
  { label: 'Wormhole', value: 'wormhole' },
  { label: 'DePIN', value: 'depin' },
];

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  const { activeTab, setActiveTab } = useCathedralStore();
  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <header className="border-b border-zinc-800 p-4">
        <div className="container mx-auto flex items-center justify-between">
          <span className="text-2xl font-bold">🏛️ Cathedral ARKHE</span>
          <span className="text-sm text-zinc-400">v3.0.0</span>
        </div>
      </header>
      <div className="container mx-auto p-4">
        <AnimatedTabs tabs={tabs} activeTab={activeTab} onChange={setActiveTab} className="mb-6" />
        <main>{children}</main>
      </div>
    </div>
  );
}
