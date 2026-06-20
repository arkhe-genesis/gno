export function MetricCard({ title, value }: { title: string; value: string | number }) {
  return (
    <div className="rounded-2xl border border-zinc-800 bg-zinc-900 p-4">
      <h3 className="text-sm font-medium text-zinc-400 mb-2">{title}</h3>
      <p className="text-2xl font-bold text-zinc-100">{value}</p>
    </div>
  );
}
