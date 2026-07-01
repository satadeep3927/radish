import { invoke } from "@tauri-apps/api/core";
import { createSignal, createEffect, on } from "solid-js";
import { useConnections } from "./useConnections";
import { useServerConfig } from "./useServerConfig";

// Global state so all hooks share the same connection status
const [isConnected, setIsConnected] = createSignal(false);
const [connectionError, setConnectionError] = createSignal<string | null>(null);
// True ONLY when Studio itself successfully started the server process
// Persisted to localStorage so it survives page refreshes
const [isEngineOwnedByStudio, _setIsEngineOwnedByStudio] = createSignal(localStorage.getItem("radish-engine-owned") === "true");

const setIsEngineOwnedByStudio = (v: boolean) => {
  _setIsEngineOwnedByStudio(v);
  if (v) {
    localStorage.setItem("radish-engine-owned", "true");
  } else {
    localStorage.removeItem("radish-engine-owned");
  }
};

export function useRadish() {
  const { getConnectionString, activeConnectionId } = useConnections();
  const { serverConfig } = useServerConfig();

  const getPassword = () => {
    const cfg = serverConfig();
    return cfg.requires_auth && cfg.password ? cfg.password : undefined;
  };

  const executeCommand = async (args: string[]) => {
    try {
      const response = await invoke("execute_command", { 
        connectionString: getConnectionString(),
        args,
        password: getPassword(),
      });
      setConnectionError(null);
      setIsConnected(true);
      return response;
    } catch (e: any) {
      if (e.toString().includes("Connection refused") || e.toString().includes("No connection")) {
        setIsConnected(false);
      }
      setConnectionError(e.toString());
      throw e;
    }
  };

  const checkConnection = async () => {
    try {
      await executeCommand(["PING"]);
      setIsConnected(true);
    } catch (e: any) {
      setIsConnected(false);
    }
  };

  const retryConnection = async (retries: number, delay: number): Promise<boolean> => {
    for (let i = 0; i < retries; i++) {
      await new Promise(resolve => setTimeout(resolve, delay));
      try {
        await executeCommand(["PING"]);
        return true;
      } catch (e) {
        // Not ready yet, keep trying
      }
    }
    return false;
  };

  // When the active connection changes, reset state and re-check
  // defer: true skips the initial value — only fires on actual switches
  createEffect(on(activeConnectionId, () => {
    // Reset state for the new connection
    setIsConnected(false);
    setConnectionError(null);
    // Re-check connectivity against the new connection string
    checkConnection();
  }, { defer: true }));

  const startServer = async () => {
    try {
      await invoke("start_server");
      setIsEngineOwnedByStudio(true);
      setConnectionError(null);
      if (!(await retryConnection(15, 200))) {
        setConnectionError("Server started but could not confirm connection.");
      }
    } catch (e: any) {
      setIsEngineOwnedByStudio(false);
      setConnectionError(e.toString());
    }
  };

  const stopServer = async () => {
    try {
      await invoke("stop_server");
      setIsEngineOwnedByStudio(false);
      setIsConnected(false);
    } catch (e: any) {
      setConnectionError(e.toString());
    }
  };

  const restartServer = async () => {
    try {
      await invoke("restart_server");
      setIsEngineOwnedByStudio(true);
      setConnectionError(null);
      if (!(await retryConnection(15, 200))) {
        setConnectionError("Server restarted but could not confirm connection.");
      }
    } catch (e: any) {
      setIsEngineOwnedByStudio(false);
      setConnectionError(e.toString());
    }
  };

  return {
    isConnected,
    connectionError,
    isEngineOwnedByStudio,
    startServer,
    stopServer,
    restartServer,
    executeCommand,
    checkConnection,
  };
}
