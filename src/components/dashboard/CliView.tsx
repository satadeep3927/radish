import { createSignal, onMount, For } from "solid-js";
import { TerminalSquare, Trash2 } from "lucide-solid";
import { Button } from "../ui/button";
import { useCli } from "../../hooks/useCli";
import { useConnections } from "../../hooks/useConnections";

export function CliView(props: { cliState: ReturnType<typeof useCli> }) {
  const { getConnectionString } = useConnections();
  const {
    history,
    cmdHistory,
    handleCommandExecution,
    clearHistory
  } = props.cliState;

  const [inputValue, setInputValue] = createSignal("");
  const [cmdHistoryIndex, setCmdHistoryIndex] = createSignal(-1);

  let outputRef: HTMLDivElement | undefined;
  let inputRef: HTMLInputElement | undefined;

  const scrollToBottom = () => {
    if (outputRef) {
      outputRef.scrollTop = outputRef.scrollHeight;
    }
  };

  const handleCommand = async (e: Event) => {
    e.preventDefault();
    const cmdStr = inputValue().trim();
    if (!cmdStr) return;

    setInputValue("");
    setCmdHistoryIndex(-1); // reset index
    await handleCommandExecution(cmdStr);
    setTimeout(scrollToBottom, 0);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "ArrowUp") {
      e.preventDefault();
      const currentHistory = cmdHistory();
      if (currentHistory.length === 0) return;
      
      let newIdx = cmdHistoryIndex() === -1 ? currentHistory.length - 1 : cmdHistoryIndex() - 1;
      if (newIdx < 0) newIdx = 0;
      
      setCmdHistoryIndex(newIdx);
      setInputValue(currentHistory[newIdx]);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      const currentHistory = cmdHistory();
      if (cmdHistoryIndex() === -1 || currentHistory.length === 0) return;

      let newIdx = cmdHistoryIndex() + 1;
      if (newIdx >= currentHistory.length) {
        setCmdHistoryIndex(-1);
        setInputValue("");
      } else {
        setCmdHistoryIndex(newIdx);
        setInputValue(currentHistory[newIdx]);
      }
    }
  };

  onMount(() => {
    if (inputRef) inputRef.focus();
    setTimeout(scrollToBottom, 0);
  });

  return (
    <div class="flex-1 flex flex-col min-h-0 bg-[var(--color-surface-0)] text-[var(--color-text-primary)] font-mono text-sm" onClick={() => inputRef?.focus()}>
      {/* Header */}
      <div class="flex items-center justify-between px-6 py-4 slick-border-b bg-[var(--color-surface-1)]">
        <div class="flex items-center gap-2 text-[var(--color-text-secondary)] font-sans">
          <TerminalSquare class="w-5 h-5 text-[var(--color-brand)]" />
          <span class="font-medium text-[var(--color-text-primary)]">Terminal</span>
        </div>
        <Button 
          variant="ghost" 
          onClick={clearHistory}
          class="h-8 px-3 text-[var(--color-text-secondary)] hover:text-[var(--color-brand)] font-sans text-xs"
        >
          <Trash2 class="w-3.5 h-3.5 mr-1.5" />
          Clear Log
        </Button>
      </div>

      {/* Output Console */}
      <div 
        ref={outputRef}
        class="flex-1 overflow-y-auto p-4 whitespace-pre-wrap select-text break-words"
      >
        <For each={history()}>
          {(block) => (
            <div class={`mb-1.5 ${block.isCommand ? 'text-[var(--color-text-muted)]' : 'text-[var(--color-text-primary)]'}`}>
              {block.content}
            </div>
          )}
        </For>
      </div>

      {/* Input row */}
      <div class="p-4 bg-[var(--color-surface-1)] slick-border-t">
        <form onSubmit={handleCommand} class="flex items-center gap-2">
          <span class="text-[var(--color-text-muted)] select-none shrink-0">{getConnectionString()}&gt;</span>
          <input
            ref={inputRef}
            type="text"
            value={inputValue()}
            onInput={(e) => setInputValue(e.currentTarget.value)}
            onKeyDown={handleKeyDown}
            class="flex-1 bg-transparent border-none outline-none text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] font-mono"
            placeholder="Enter command..."
            autofocus
            autocomplete="off"
            spellcheck={false}
          />
        </form>
      </div>
    </div>
  );
}
