import { createSignal, createEffect, on, onCleanup } from "solid-js";
import { useRadish } from "./useRadish";
import { useConnections } from "./useConnections";

export interface ServerStats {
  uptime: number;
  clients: number;
  memory: number;
  rawProperties: Record<string, string>;
}

export function useStats() {
  const { executeCommand, isConnected } = useRadish();
  const { activeConnectionId } = useConnections();
  const [stats, setStats] = createSignal<ServerStats | null>(null);

  const fetchStats = async () => {
    if (!isConnected()) return;
    try {
      const response = await executeCommand(["INFO"]);
      if (typeof response === "string") {
        const lines = response.split("\r\n");
        const parsed: Record<string, string> = {};
        for (const line of lines) {
          if (line.includes(":") && !line.startsWith("#")) {
            const idx = line.indexOf(":");
            const key = line.slice(0, idx).trim();
            const val = line.slice(idx + 1).trim();
            parsed[key] = val;
          }
        }
        
        setStats({
          uptime: parseInt(parsed.uptime_in_seconds, 10) || 0,
          clients: parseInt(parsed.connected_clients, 10) || 0,
          memory: parseInt(parsed.used_memory, 10) || 0,
          rawProperties: parsed
        });
      }
    } catch (e) {
      console.error("Failed to fetch stats", e);
    }
  };

  // Clear stale stats when connection switches
  createEffect(on(activeConnectionId, () => {
    setStats(null);
  }, { defer: true }));

  // Start/stop polling when isConnected changes
  createEffect(on(isConnected, (connected) => {
    if (connected) {
      fetchStats();
      const interval = setInterval(fetchStats, 2000);
      onCleanup(() => clearInterval(interval));
    } else {
      setStats(null);
    }
  }));

  return { stats };
}
