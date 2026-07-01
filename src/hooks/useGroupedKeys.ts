import { createSignal, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export interface KeyNode {
  name: string;
  full_key: string | null;
  children: KeyNode[];
}

export function useGroupedKeys(keys: () => string[], separator = ":") {
  const [tree, setTree] = createSignal<KeyNode[]>([]);
  const [isBuilding, setIsBuilding] = createSignal(false);

  createEffect(() => {
    const k = keys();
    if (k.length === 0) {
      setTree([]);
      return;
    }
    setIsBuilding(true);
    invoke<KeyNode[]>("group_keys", { keys: k, separator })
      .then((result) => {
        setTree(result);
      })
      .catch((e) => {
        console.error("group_keys failed", e);
        setTree([]);
      })
      .finally(() => {
        setIsBuilding(false);
      });
  });

  return { tree, isBuilding };
}
