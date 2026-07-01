import { createSignal, createEffect } from "solid-js";
import { useRadish } from "./useRadish";

export interface KeyDetails {
  type: string;
  ttl: number;
  value: any;
  size?: number;
}

export function useInspector(activeKey: () => string | null) {
  const { executeCommand, isConnected } = useRadish();
  const [details, setDetails] = createSignal<KeyDetails | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  let requestId = 0;

  const fetchDetails = async (k: string) => {
    const id = ++requestId;
    setIsLoading(true);
    try {
      const typeRes = await executeCommand(["TYPE", k]);
      if (id !== requestId) return;
      if (typeof typeRes !== "string") throw new Error("Invalid type response");
      
      const ttlRes = await executeCommand(["TTL", k]);
      if (id !== requestId) return;
      const ttl = typeof ttlRes === "number" ? ttlRes : -1;

      let value: any = null;
      let size = 0;

      if (typeRes === "string") {
        const valRes = await executeCommand(["GET", k]);
        if (id !== requestId) return;
        value = typeof valRes === "string" ? valRes : "";
        size = value.length;
      } else if (typeRes === "hash") {
        const valRes = await executeCommand(["HGETALL", k]);
        if (id !== requestId) return;
        if (Array.isArray(valRes)) {
          const obj: any = {};
          for (let i = 0; i < valRes.length; i += 2) {
            obj[valRes[i]] = valRes[i+1];
          }
          value = obj;
          size = valRes.length / 2;
        }
      } else if (typeRes === "list") {
        const valRes = await executeCommand(["LRANGE", k, "0", "-1"]);
        if (id !== requestId) return;
        if (Array.isArray(valRes)) {
          value = valRes;
          size = valRes.length;
        }
      } else if (typeRes === "set") {
        const valRes = await executeCommand(["SMEMBERS", k]);
        if (id !== requestId) return;
        if (Array.isArray(valRes)) {
          value = valRes;
          size = valRes.length;
        }
      }

      setDetails({ type: typeRes as KeyDetails["type"], value, ttl, size });
      setError(null);
    } catch (e: any) {
      setError(e.toString());
      setDetails(null);
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    const k = activeKey();
    if (k && isConnected()) {
      fetchDetails(k);
    } else {
      setDetails(null);
    }
  });

  const deleteKey = async () => {
    const k = activeKey();
    if (!k || !isConnected()) return false;
    try {
      await executeCommand(["DEL", k]);
      return true;
    } catch (e: any) {
      setError(e.toString());
      return false;
    }
  };

  const renameKey = async (newKey: string) => {
    const k = activeKey();
    if (!k || !isConnected()) return false;
    try {
      await executeCommand(["RENAME", k, newKey]);
      return true;
    } catch (e: any) {
      setError(e.toString());
      return false;
    }
  };

  const updateValue = async (val: string) => {
    const k = activeKey();
    if (!k || !isConnected() || details()?.type !== "string") return false;
    try {
      await executeCommand(["SET", k, val]);
      fetchDetails(k);
      return true;
    } catch (e: any) {
      setError(e.toString());
      return false;
    }
  };

  const updateHashField = async (field: string, val: string) => {
    const k = activeKey();
    if (!k || !isConnected() || details()?.type !== "hash") return false;
    try {
      await executeCommand(["HSET", k, field, val]);
      fetchDetails(k);
      return true;
    } catch (e: any) {
      setError(e.toString());
      return false;
    }
  };

  const updateListElement = async (index: number, val: string) => {
    const k = activeKey();
    if (!k || !isConnected() || details()?.type !== "list") return false;
    try {
      await executeCommand(["LSET", k, index.toString(), val]);
      fetchDetails(k);
      return true;
    } catch (e: any) {
      setError(e.toString());
      return false;
    }
  };

  const updateSetElement = async (oldVal: string, newVal: string) => {
    const k = activeKey();
    if (!k || !isConnected() || details()?.type !== "set") return false;
    try {
      await executeCommand(["SREM", k, oldVal]);
      try {
        await executeCommand(["SADD", k, newVal]);
      } catch (addErr: any) {
        // Rollback: re-add the old value that was just removed
        await executeCommand(["SADD", k, oldVal]);
        throw addErr;
      }
      fetchDetails(k);
      return true;
    } catch (e: any) {
      setError(e.toString());
      return false;
    }
  };

  return {
    details,
    isLoading,
    error,
    refresh: () => {
      const k = activeKey();
      if (k) fetchDetails(k);
    },
    deleteKey,
    renameKey,
    updateValue,
    updateHashField,
    updateListElement,
    updateSetElement
  };
}
