import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useDialog } from "./useDialog";
import { useConnections } from "./useConnections";
import { loadServerConfig } from "./useServerConfig";

export function useConfig() {
  const [port, setPort] = createSignal(6379);
  const [saveInterval, setSaveInterval] = createSignal(60);
  const [maxMemory, setMaxMemory] = createSignal("0");
  const [requiresAuth, setRequiresAuth] = createSignal(false);
  const [password, setPassword] = createSignal("");
  const [dumpPath, setDumpPath] = createSignal("dump.radish");
  const [bind, setBind] = createSignal("127.0.0.1");

  const { confirm, alert } = useDialog();
  const { activeConnectionId, updateConnection, connections } = useConnections();

  onMount(async () => {
    try {
      const cfg: any = await invoke("read_config");
      if (cfg) {
        if (cfg.port !== undefined) setPort(cfg.port);
        if (cfg.save_interval !== undefined && cfg.save_interval !== null) {
          setSaveInterval(cfg.save_interval);
        } else {
          setSaveInterval(0);
        }
        if (cfg.maxmemory !== undefined) setMaxMemory(cfg.maxmemory);
        if (cfg.requires_auth !== undefined) setRequiresAuth(cfg.requires_auth);
        if (cfg.password !== undefined) setPassword(cfg.password);
        if (cfg.dump_path !== undefined) setDumpPath(cfg.dump_path);
        if (cfg.bind !== undefined) setBind(cfg.bind);
      }
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  });

  const handleSave = async () => {
    const cfg = {
      port: port(),
      save_interval: saveInterval() > 0 ? saveInterval() : null,
      requires_auth: requiresAuth(),
      password: password(),
      dump_path: dumpPath(),
      bind: bind(),
      maxmemory: maxMemory(),
    };

    try {
      await invoke("write_config", { config: cfg });

      // Refresh the shared config signal so auth/port changes take effect immediately
      await loadServerConfig();

      // Check if we need to update the default connection profile's port
      const localConn = connections().find((c) => c.id === "local-radish-engine");
      if (localConn && localConn.port !== cfg.port) {
        updateConnection("local-radish-engine", { port: cfg.port });
      }

      if (activeConnectionId() === "local-radish-engine") {
        confirm({
          title: "Configuration Saved",
          message: "Your settings have been saved to radish.toml. You need to restart the Radish engine for changes to take effect.",
          confirmText: "OK",
          cancelText: "",
          variant: "default",
          onConfirm: () => {},
        });
      } else {
        alert({
          title: "Configuration Saved",
          message: "Settings saved to radish.toml.",
        });
      }
    } catch (e: any) {
      alert({
        title: "Save Failed",
        message: `Failed to save configuration: ${e.toString()}`,
      });
    }
  };

  return {
    port,
    setPort,
    saveInterval,
    setSaveInterval,
    maxMemory,
    setMaxMemory,
    requiresAuth,
    setRequiresAuth,
    password,
    setPassword,
    dumpPath,
    setDumpPath,
    bind,
    setBind,
    handleSave,
  };
}
