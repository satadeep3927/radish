import { createSignal, onMount, onCleanup } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import { useRadish } from "./useRadish";

interface LogEntry {
  id: string;
  time: string;
  message: string;
}

export function useService() {
  const { isConnected, connectionError, startServer, stopServer, restartServer } = useRadish();
  const [logs, setLogs] = createSignal<LogEntry[]>([]);

  const addLog = (message: string) => {
    setLogs((prev) => {
      const newLogs = [
        ...prev,
        {
          id: crypto.randomUUID(),
          time: new Date().toLocaleTimeString([], {
            hour12: false,
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
          }),
          message,
        },
      ];
      // Keep last 1000 logs
      return newLogs.length > 1000 ? newLogs.slice(newLogs.length - 1000) : newLogs;
    });
  };

  onMount(() => {
    const unlistenPromise = listen<string>("server-log", (event) => {
      addLog(event.payload);
    });

    onCleanup(async () => {
      const unlisten = await unlistenPromise;
      unlisten();
    });
  });

  const handleStart = () => {
    startServer();
  };

  const handleRestart = () => {
    restartServer();
  };

  return {
    isConnected,
    connectionError,
    logs,
    clearLogs: () => setLogs([]),
    handleStart,
    handleStop: stopServer,
    handleRestart,
  };
}
