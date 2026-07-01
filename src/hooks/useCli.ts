import { createSignal } from "solid-js";
import { useRadish } from "./useRadish";
import { useConnections } from "./useConnections";

export interface HistoryBlock {
  id: string;
  isCommand: boolean;
  content: string | any;
}

function parseCommand(input: string): string[] {
  const regex = /[^\s"']+|"([^"]*)"|'([^']*)'/g;
  const tokens = [];
  let match;
  while ((match = regex.exec(input)) !== null) {
    tokens.push(match[1] || match[2] || match[0]);
  }
  return tokens;
}

function formatResponse(data: any, indentLevel = 0): string {
  if (data === null) {
    return "(nil)";
  }
  if (typeof data === "number") {
    return `(integer) ${data}`;
  }
  if (typeof data === "string") {
    if (indentLevel > 0) {
      return `"${data}"`;
    }
    return data;
  }
  if (Array.isArray(data)) {
    if (data.length === 0) return "(empty array)";
    const isByteArray = data.every(item => typeof item === "number");
    if (isByteArray) {
       return `(raw bytes: ${data.length})`; 
    }

    let out = "";
    const prefix = "  ".repeat(indentLevel);
    for (let i = 0; i < data.length; i++) {
      const itemFormatted = formatResponse(data[i], indentLevel + 1);
      out += `${prefix}${i + 1}) ${itemFormatted}`;
      if (i < data.length - 1) out += "\n";
    }
    return out;
  }
  if (typeof data === "object" && data !== null) {
    if (data.error) {
      return `(error) ${data.error}`;
    }
  }
  return JSON.stringify(data);
}

export function useCli() {
  const { executeCommand } = useRadish();
  const { getConnectionString } = useConnections();
  const [history, setHistory] = createSignal<HistoryBlock[]>([]);
  const [cmdHistory, setCmdHistory] = createSignal<string[]>([]);
  
  // Initialize with a welcome message if empty
  if (history().length === 0) {
    setHistory([
      { id: "init", isCommand: false, content: "Radish CLI initialized. Type 'clear' to clear the screen." }
    ]);
  }

  const pushBlock = (isCommand: boolean, content: any) => {
    setHistory((prev) => [
      ...prev,
      { id: crypto.randomUUID(), isCommand, content },
    ]);
  };

  const handleCommandExecution = async (cmdStr: string) => {
    if (!cmdStr) return;

    pushBlock(true, `${getConnectionString()}> ${cmdStr}`);
    setCmdHistory((prev) => [...prev, cmdStr]);

    if (cmdStr.toLowerCase() === "clear") {
      setHistory([]);
      return;
    }

    try {
      const tokens = parseCommand(cmdStr);
      if (tokens.length === 0) return;
      
      const response = await executeCommand(tokens);
      pushBlock(false, formatResponse(response));
    } catch (e: any) {
      pushBlock(false, `(error) ${e.toString()}`);
    }
  };

  const clearHistory = () => {
    setHistory([]);
  };

  return {
    history,
    cmdHistory,
    handleCommandExecution,
    clearHistory
  };
}
