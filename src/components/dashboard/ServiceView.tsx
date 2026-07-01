import { Server, Play, Square, RotateCcw, AlertTriangle, Trash2 } from "lucide-solid";
import { Button } from "../ui/button";
import { useService } from "../../hooks/useService";
import { For, createEffect } from "solid-js";

export function ServiceView() {
  const {
    isConnected,
    connectionError,
    logs,
    clearLogs,
    handleStart,
    handleStop,
    handleRestart,
  } = useService();

  let logsRef: HTMLDivElement | undefined;

  const scrollToBottom = () => {
    if (logsRef) {
      logsRef.scrollTop = logsRef.scrollHeight;
    }
  };

  createEffect(() => {
    // Scroll to bottom when log list updates
    if (logs().length > 0) {
      scrollToBottom();
    }
  });

  return (
    <div class="flex-1 flex flex-col min-h-0 bg-[var(--color-surface-0)] text-[var(--color-text-primary)]">
      {/* Header */}
      <div class="flex items-center justify-between px-6 py-4 slick-border-b bg-[var(--color-surface-1)]">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded bg-[var(--color-brand-subtle)] text-[var(--color-brand)] flex items-center justify-center">
            <Server class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-sm font-semibold text-[var(--color-text-primary)] leading-none">Service Management</h2>
            <div class="flex items-center gap-2 mt-1.5">
              <div class={`px-2 py-0.5 rounded-sm flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wider ${isConnected() ? 'bg-green-50 text-green-600 border border-green-200' : 'bg-[var(--color-surface-2)] text-[var(--color-text-muted)] border border-[var(--color-border-strong)]'}`}>
                <div class={`w-1.5 h-1.5 rounded-full ${isConnected() ? 'bg-green-500' : 'bg-[var(--color-text-muted)]'}`}></div>
                {isConnected() ? "Running" : "Stopped"}
              </div>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <Button 
            variant="outline"
            onClick={handleStart} 
            disabled={isConnected()}
            class="h-8 px-3 gap-1.5 text-xs text-[var(--color-text-secondary)] bg-transparent border-[var(--color-border-strong)] hover:text-green-600 hover:border-green-600/30"
          >
            <Play class="w-3.5 h-3.5" />
            Start
          </Button>
          <Button 
            variant="outline"
            onClick={handleStop} 
            disabled={!isConnected()}
            class="h-8 px-3 gap-1.5 text-xs text-[var(--color-text-secondary)] bg-transparent border-[var(--color-border-strong)] hover:text-red-500 hover:border-red-500/30"
          >
            <Square class="w-3.5 h-3.5" />
            Stop
          </Button>
          <Button 
            variant="outline"
            onClick={handleRestart} 
            disabled={!isConnected()}
            class="h-8 px-3 gap-1.5 text-xs text-[var(--color-text-secondary)] bg-transparent border-[var(--color-border-strong)] hover:text-blue-500 hover:border-blue-500/30"
          >
            <RotateCcw class="w-3.5 h-3.5" />
            Restart
          </Button>
          <div class="w-px h-5 bg-[var(--color-border-strong)] mx-1" />
          <Button 
            variant="ghost" 
            onClick={clearLogs}
            class="h-8 px-3 text-[var(--color-text-secondary)] hover:text-red-500 font-sans text-xs"
          >
            <Trash2 class="w-3.5 h-3.5 mr-1.5" />
            Clear
          </Button>
        </div>
      </div>

      {connectionError() && (
        <div class="px-6 py-3 border-b border-red-500/20 bg-red-500/5 flex items-start gap-2">
          <AlertTriangle class="w-4 h-4 text-red-500 shrink-0 mt-0.5" />
          <p class="text-xs text-red-500 break-words">{connectionError()}</p>
        </div>
      )}

      {/* Live Logs */}
      <div 
        ref={logsRef}
        class="flex-1 overflow-y-auto p-4 font-mono text-sm bg-[var(--color-surface-0)]"
      >
        {logs().length === 0 ? (
          <div class="h-full flex items-center justify-center text-[var(--color-text-muted)] opacity-50 font-sans text-sm">
            No logs available. Start the service to begin capturing output.
          </div>
        ) : (
          <For each={logs()}>
            {(log) => (
              <div class="flex items-start gap-3 mb-1 hover:bg-[var(--color-surface-1)] px-1.5 py-0.5 rounded transition-colors group text-[var(--color-text-primary)]">
                <span class="text-[var(--color-text-muted)] shrink-0 select-none">[{log.time}]</span>
                <span class="break-words whitespace-pre-wrap leading-relaxed">{log.message}</span>
              </div>
            )}
          </For>
        )}
      </div>
    </div>
  );
}
