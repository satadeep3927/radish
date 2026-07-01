import { createSignal, createEffect, on } from "solid-js";
import { useRadish } from "./useRadish";
import { useConnections } from "./useConnections";

async function runConcurrent<T>(items: T[], concurrency: number, fn: (item: T) => Promise<void>): Promise<void> {
  let index = 0;
  const workers = Array.from({ length: concurrency }, async () => {
    while (index < items.length) {
      const i = index++;
      await fn(items[i]);
    }
  });
  await Promise.all(workers);
}

export function useKeys() {
  const { executeCommand, isConnected } = useRadish();
  const { activeConnectionId } = useConnections();
  const [keys, setKeys] = createSignal<string[]>([]);
  const [keyTypes, setKeyTypes] = createSignal<Record<string, string>>({});
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  let scanId = 0;

  const fetchKeys = async () => {
    if (!isConnected()) return;
    const id = ++scanId;
    setIsLoading(true);
    try {
      // Use SCAN instead of KEYS * — non-blocking, cursor-based
      const response = await executeCommand(["SCAN", "0", "MATCH", "*", "COUNT", "1000"]);
      if (id !== scanId) return;
      let allKeys: string[] = [];
      if (Array.isArray(response) && response.length >= 2) {
        const keysArr = response[1];
        if (Array.isArray(keysArr)) {
          allKeys = keysArr.filter(k => typeof k === "string");
        }
      }
      setKeys(allKeys);

      // Fetch types in batches with limited concurrency
      if (allKeys.length > 0) {
        const types: Record<string, string> = {};
        const batchSize = 50;

        for (let i = 0; i < allKeys.length; i += batchSize) {
          if (id !== scanId) return;
          const chunk = allKeys.slice(i, i + batchSize);
          await runConcurrent(chunk, 5, async (k) => {
            try {
              const t = await executeCommand(["TYPE", k]);
              if (typeof t === "string") types[k] = t;
            } catch (e) {}
          });
        }
        if (id !== scanId) return;
        setKeyTypes(types);
      } else {
        setKeyTypes({});
      }
      setError(null);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      if (id === scanId) setIsLoading(false);
    }
  };

  // Clear stale data when connection switches
  createEffect(on(activeConnectionId, () => {
    scanId++;
    setKeys([]);
    setKeyTypes({});
    setError(null);
  }, { defer: true }));

  // Re-fetch when connected (fires after connection switch too)
  createEffect(() => {
    if (isConnected()) {
      fetchKeys();
    }
  });

  const flushDb = async () => {
    if (!isConnected()) return;
    try {
      await executeCommand(["FLUSHDB"]);
      await fetchKeys();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  return {
    keys,
    keyTypes,
    isLoading,
    error,
    fetchKeys,
    flushDb
  };
}
