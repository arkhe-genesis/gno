'use client';
import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';

export function AnimatedCard({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <motion.div
      whileHover={{ scale: 1.02 }}
      className={cn('rounded-2xl border border-zinc-800 bg-zinc-900 p-4', className)}
    >
      {children}
    </motion.div>
  );
}
