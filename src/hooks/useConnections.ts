import { createSignal, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { loadServerConfig } from "./useServerConfig";

export interface ConnectionProfile {
  id: string;
  alias: string;
  host: string;
  port: number;
  type: "standalone" | "cluster";
  lastConnection?: string;
}

const DEFAULT_LOCAL_CONNECTION: ConnectionProfile = {
  id: "local-radish-engine",
  alias: "Local Radish Engine",
  host: "127.0.0.1",
  port: 6379,
  type: "standalone",
};

// Global state for connections
const [connections, setConnections] = createSignal<ConnectionProfile[]>([DEFAULT_LOCAL_CONNECTION]);
const [activeConnectionId, setActiveConnectionId] = createSignal<string | null>(null);

// Initialize from local storage
const saved = localStorage.getItem("radish-connections");
if (saved) {
  try {
    const parsed = JSON.parse(saved);
    if (Array.isArray(parsed) && parsed.length > 0) {
      // Ensure the default local engine is always present
      const hasLocal = parsed.some((c) => c.id === DEFAULT_LOCAL_CONNECTION.id);
      if (!hasLocal) {
        setConnections([DEFAULT_LOCAL_CONNECTION, ...parsed]);
      } else {
        setConnections(parsed);
      }
    }
  } catch (e) {
    console.error("Failed to parse saved connections, resetting to defaults", e);
    setConnections([DEFAULT_LOCAL_CONNECTION]);
  }
}

// On startup, load config and sync the local engine port + the shared config store
loadServerConfig().then(() => {
  invoke<{ port: number; bind: string }>("read_config").then((cfg) => {
    if (cfg?.port) {
      setConnections((conns) =>
        conns.map((c) =>
          c.id === "local-radish-engine" ? { ...c, host: cfg.bind || c.host, port: cfg.port } : c
        )
      );
    }
  }).catch(() => {/* silently use defaults if config doesn't exist yet */});
}).catch(() => {});

// Persist to local storage whenever connections change
createEffect(() => {
  localStorage.setItem("radish-connections", JSON.stringify(connections()));
});

export function useConnections() {
  const activeConnection = () => connections().find(c => c.id === activeConnectionId());

  const addConnection = (profile: Omit<ConnectionProfile, "id">) => {
    const newConn: ConnectionProfile = {
      ...profile,
      id: crypto.randomUUID(),
    };
    setConnections([...connections(), newConn]);
    return newConn.id;
  };

  const removeConnection = (id: string) => {
    if (id === DEFAULT_LOCAL_CONNECTION.id) return; // Cannot remove local engine
    setConnections(connections().filter((c) => c.id !== id));
    if (activeConnectionId() === id) {
      setActiveConnectionId(null);
    }
  };

  const updateConnection = (id: string, updates: Partial<ConnectionProfile>) => {
    setConnections(connections().map((c) => (c.id === id ? { ...c, ...updates } : c)));
  };

  const selectConnection = (id: string) => {
    const conn = connections().find(c => c.id === id);
    if (conn) {
      setActiveConnectionId(id);
      // Update lastConnection time
      setConnections(connections().map(c => 
        c.id === id ? { ...c, lastConnection: new Date().toISOString() } : c
      ));
    }
  };

  const getConnectionString = () => {
    const conn = activeConnection();
    if (!conn) return "127.0.0.1:6379";
    return `${conn.host}:${conn.port}`;
  };

  return {
    connections,
    activeConnectionId,
    activeConnection,
    addConnection,
    removeConnection,
    updateConnection,
    selectConnection,
    setActiveConnectionId, // For clearing active connection
    getConnectionString,
  };
}
