import { createSignal, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useRadish } from "./useRadish";
import { useConnections } from "./useConnections";
import { useServerConfig } from "./useServerConfig";

export interface PubSubMessage {
  id: string;
  channel: string;
  data: string;
  time: string;
}

export function usePubSub() {
  const { executeCommand } = useRadish();
  const { getConnectionString } = useConnections();
  const { serverConfig } = useServerConfig();
  const [activeChannels, setActiveChannels] = createSignal<string[]>([]);
  const [messages, setMessages] = createSignal<PubSubMessage[]>([]);

  const getPassword = () => {
    const cfg = serverConfig();
    return cfg.requires_auth && cfg.password ? cfg.password : undefined;
  };

  // Subscribe to channel on backend and track locally
  const subscribe = async (channel: string) => {
    if (!channel || activeChannels().includes(channel)) return;
    try {
      await invoke("subscribe_channel", { 
        connectionString: getConnectionString(),
        channel,
        password: getPassword(),
      });
      setActiveChannels((prev) => [...prev, channel]);
    } catch (e) {
      console.error("Failed to subscribe:", e);
    }
  };

  const unsubscribe = async (channel: string) => {
    try {
      await invoke("unsubscribe_channel", { channel });
      setActiveChannels((prev) => prev.filter((c) => c !== channel));
    } catch (e) {
      console.error("Failed to unsubscribe:", e);
    }
  };

  // Publish a message to a channel
  const publish = async (channel: string, message: string) => {
    if (!channel) return;
    try {
      await executeCommand(["PUBLISH", channel, message]);
    } catch (e) {
      console.error("Failed to publish message", e);
    }
  };

  const clearMessages = () => {
    setMessages([]);
  };

  onMount(async () => {
    // Listen to "pubsub-message" emitted from Tauri backend
    const unlisten = await listen<any>("pubsub-message", (event) => {
      const payload = event.payload;
      if (Array.isArray(payload) && payload.length >= 3) {
        const type = payload[0];
        if (type === "message" || type === "pmessage") {
          const isPattern = type === "pmessage";
          const channel = isPattern ? payload[2] : payload[1];
          const data = isPattern ? payload[3] : payload[2];
          
          const newMessage: PubSubMessage = {
            id: Math.random().toString(36).substring(2, 9),
            channel,
            data: typeof data === "string" ? data : JSON.stringify(data),
            time: new Date().toLocaleTimeString(),
          };

          setMessages((prev) => [newMessage, ...prev].slice(0, 500));
        }
      }
    });

    onCleanup(async () => {
      // Clean up tauri listener
      unlisten();

      // Unsubscribe all active channels to clean up backend sockets
      const channels = activeChannels();
      for (const channel of channels) {
        try {
          await invoke("unsubscribe_channel", { channel });
        } catch (e) {
          console.error("Failed to clean up channel on exit", e);
        }
      }
    });
  });

  return {
    activeChannels,
    messages,
    subscribe,
    unsubscribe,
    publish,
    clearMessages,
  };
}
