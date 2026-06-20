export function MetricsChart({ className }: { className?: string }) {
  return (
    <div className={`rounded-2xl border border-zinc-800 bg-zinc-900 p-4 ${className || ''}`}>
      <h3 className="text-sm font-medium text-zinc-400 mb-4">Performance Metrics</h3>
      <div className="flex h-full items-center justify-center text-zinc-500">
        [Chart visualization placeholder]
      </div>
    </div>
  );
}
