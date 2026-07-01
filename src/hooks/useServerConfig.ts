/**
 * Shared reactive store for the server configuration loaded from radish.toml.
 * All hooks that need auth credentials or port should read from here.
 * This is a singleton — the signal is created once and shared across all callers.
 */

import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface ServerConfig {
  port: number;
  bind: string;
  requires_auth: boolean;
  password: string;
  dump_path: string;
  save_interval: number | null;
  maxmemory: string;
}

const DEFAULT_CONFIG: ServerConfig = {
  port: 6379,
  bind: "127.0.0.1",
  requires_auth: false,
  password: "",
  dump_path: "dump.radish",
  save_interval: null,
  maxmemory: "0",
};

// Global singleton — shared across all modules that import this file
const [serverConfig, setServerConfig] = createSignal<ServerConfig>(DEFAULT_CONFIG);
const [configLoaded, setConfigLoaded] = createSignal(false);

export async function loadServerConfig() {
  try {
    const cfg = await invoke<ServerConfig>("read_config");
    if (cfg) setServerConfig({ ...DEFAULT_CONFIG, ...cfg });
  } catch (e) {
    console.warn("Could not load server config, using defaults:", e);
  } finally {
    setConfigLoaded(true);
  }
}

export function useServerConfig() {
  return { serverConfig, configLoaded, loadServerConfig };
}
