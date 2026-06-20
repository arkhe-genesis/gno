'use client';
import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';

interface Tab {
  label: string;
  value: string;
}

interface AnimatedTabsProps {
  tabs: Tab[];
  activeTab: string;
  onChange: (value: string) => void;
  className?: string;
}

export function AnimatedTabs({ tabs, activeTab, onChange, className }: AnimatedTabsProps) {
  return (
    <div className={cn('flex gap-1 p-1 bg-zinc-900 rounded-xl', className)}>
      {tabs.map((tab) => (
        <button
          key={tab.value}
          onClick={() => onChange(tab.value)}
          className={cn(
            'relative px-4 py-2 text-sm font-medium rounded-lg transition-colors',
            activeTab === tab.value ? 'text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'
          )}
        >
          {activeTab === tab.value && (
            <motion.div layoutId="activeTab" className="absolute inset-0 bg-zinc-800 rounded-lg" />
          )}
          <span className="relative z-10">{tab.label}</span>
        </button>
      ))}
    </div>
  );
}
