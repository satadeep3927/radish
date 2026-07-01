import { createSignal, createMemo, Accessor, createEffect, on } from "solid-js";
import { useRadish } from "./useRadish";
import { useConnections } from "./useConnections";

export function useDatabaseStats(
  keys: Accessor<string[]>,
  keyTypes: Accessor<Record<string, string>>
) {
  const { executeCommand, isConnected } = useRadish();
  const { activeConnectionId } = useConnections();
  const [keySizes, setKeySizes] = createSignal<Record<string, number>>({});
  const [isFetchingSizes, setIsFetchingSizes] = createSignal(false);
  const [nsSort, setNsSort] = createSignal<"keys" | "memory">("keys");
  const [topSort, setTopSort] = createSignal<"size" | "length">("size");

  let sizeTimer: ReturnType<typeof setTimeout> | undefined;
  let sizeId = 0;

  const fetchKeySizes = async (keyList: string[]) => {
    if (!isConnected() || keyList.length === 0) return;
    const id = ++sizeId;
    setIsFetchingSizes(true);
    const sizes: Record<string, number> = {};
    const batchSize = 50;

    for (let i = 0; i < keyList.length; i += batchSize) {
      if (id !== sizeId) return;
      const chunk = keyList.slice(i, i + batchSize);
      await Promise.all(
        chunk.map(async (k) => {
          try {
            const resp = await executeCommand(["MEMORY", "USAGE", k]);
            if (typeof resp === "number") sizes[k] = resp;
          } catch (e) {}
        })
      );
    }
    if (id !== sizeId) return;
    setKeySizes(sizes);
    setIsFetchingSizes(false);
  };

  createEffect(on(activeConnectionId, () => {
    clearTimeout(sizeTimer);
    setKeySizes({});
  }, { defer: true }));

  createEffect(() => {
    const k = keys();
    if (isConnected() && k.length > 0) {
      clearTimeout(sizeTimer);
      sizeTimer = setTimeout(() => fetchKeySizes(k), 500);
    }
  });

  const stats = createMemo(() => {
    let stringCount = 0;
    let hashCount = 0;
    let listCount = 0;
    let setCount = 0;
    let zsetCount = 0;

    const sizes = keySizes();
    const namespaces: Record<string, { count: number; memory: number; types: Set<string>; nestedKeys: string[] }> = {};

    keys().forEach(k => {
      const type = (keyTypes()[k] || "none").toLowerCase();
      if (type === "string") stringCount++;
      if (type === "hash") hashCount++;
      if (type === "list") listCount++;
      if (type === "set") setCount++;
      if (type === "zset") zsetCount++;

      const parts = k.split(":");
      const ns = parts.length > 1 ? parts[0] + ":*" : "(root)";

      if (!namespaces[ns]) {
        namespaces[ns] = { count: 0, memory: 0, types: new Set(), nestedKeys: [] };
      }
      namespaces[ns].count++;
      namespaces[ns].types.add(type);
      namespaces[ns].nestedKeys.push(k);
      namespaces[ns].memory += sizes[k] ?? 0;
    });

    const total = keys().length || 1;

    const nsArray = Object.entries(namespaces).map(([name, data]) => ({
      name,
      ...data,
      types: Array.from(data.types),
    }));

    return {
      total: keys().length,
      stringCount,
      hashCount,
      listCount,
      setCount,
      zsetCount,
      stringPct: (stringCount / total) * 100,
      hashPct: (hashCount / total) * 100,
      listPct: (listCount / total) * 100,
      setPct: (setCount / total) * 100,
      zsetPct: (zsetCount / total) * 100,
      namespaces: nsArray,
    };
  });

  const sortedNamespaces = createMemo(() => {
    return [...stats().namespaces].sort((a, b) => {
      if (nsSort() === "keys") return b.count - a.count;
      return b.memory - a.memory;
    });
  });

  const topKeys = createMemo(() => {
    const sizes = keySizes();
    return keys().slice(0, 15).map(k => {
      const type = keyTypes()[k] || "none";
      const size = sizes[k] ?? 0;
      return {
        name: k,
        type,
        ttl: -1,
        size,
        length: k.length,
      };
    }).sort((a, b) => topSort() === "size" ? b.size - a.size : b.length - a.length);
  });

  const getConicGradient = () => {
    const s = stats();
    let current = 0;
    const parts: string[] = [];

    if (s.stringPct > 0) {
      parts.push(`var(--color-badge-string) ${current}% ${current + s.stringPct}%`);
      current += s.stringPct;
    }
    if (s.hashPct > 0) {
      parts.push(`var(--color-badge-hash) ${current}% ${current + s.hashPct}%`);
      current += s.hashPct;
    }
    if (s.listPct > 0) {
      parts.push(`var(--color-badge-list) ${current}% ${current + s.listPct}%`);
      current += s.listPct;
    }
    if (s.setPct > 0) {
      parts.push(`var(--color-badge-set) ${current}% ${current + s.setPct}%`);
      current += s.setPct;
    }
    if (s.zsetPct > 0) {
      parts.push(`var(--color-badge-zset) ${current}% ${current + s.zsetPct}%`);
      current += s.zsetPct;
    }

    if (parts.length === 0) return "#e5e7eb 0% 100%";
    return parts.join(", ");
  };

  return {
    isFetchingSizes,
    nsSort,
    setNsSort,
    topSort,
    setTopSort,
    stats,
    sortedNamespaces,
    topKeys,
    getConicGradient,
  };
}
