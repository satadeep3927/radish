import { useStats } from "../../hooks/useStats";
import { formatBytes, formatUptime } from "../../lib/format";

export function StatsRow() {
  const { stats } = useStats();
  const s = stats;

  return (
    <div class="flex items-center gap-6 px-6 py-2.5 slick-border-b bg-[var(--color-surface-1)]">
      
      <div class="flex items-center gap-2">
        <div class="text-[10px] text-[var(--color-text-muted)] font-medium uppercase tracking-wider">Memory</div>
        <div class="text-sm font-semibold text-[var(--color-text-primary)]">
          {s() ? formatBytes(s().memory) : "0 B"}
        </div>
      </div>
      
      <div class="w-px h-4 bg-[var(--color-border)]"></div>

      <div class="flex items-center gap-2">
        <div class="text-[10px] text-[var(--color-text-muted)] font-medium uppercase tracking-wider">Clients</div>
        <div class="text-sm font-semibold text-[var(--color-text-primary)]">{s() ? s().clients : 0}</div>
      </div>

      <div class="w-px h-4 bg-[var(--color-border)]"></div>

      <div class="flex items-center gap-2">
        <div class="text-[10px] text-[var(--color-text-muted)] font-medium uppercase tracking-wider">Uptime</div>
        <div class="text-sm font-semibold text-[var(--color-text-primary)] font-mono">{s() ? formatUptime(s().uptime) : "00:00:00"}</div>
      </div>

    </div>
  );
}
