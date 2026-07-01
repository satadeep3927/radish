import { createSignal, createEffect, on, Show, For } from "solid-js";
import { RefreshCw, List, LayoutList, ChevronDown } from "lucide-solid";
import { Button } from "../ui/button";
import { SearchInput } from "../ui/search-input";
import { KeyBrowser } from "./KeyBrowser";
import { Inspector } from "./Inspector";
import { useResizer } from "../../hooks/useResizer";
import type { Accessor } from "solid-js";

const KEY_TYPE_OPTIONS = ["All Key Types", "String", "Hash", "List", "Set", "ZSet"] as const;

interface KeysViewProps {
  keys: Accessor<string[]>;
  keyTypes: Accessor<Record<string, string>>;
  fetchKeys: () => Promise<void>;
  connectionId?: Accessor<string | null>;
}

export function KeysView(props: KeysViewProps) {
  const { keys, keyTypes, fetchKeys } = props;
  const [activeKey, setActiveKey] = createSignal<string | null>(null);
  const [view, setView] = createSignal<"flat" | "tree">("tree");
  const [filter, setFilter] = createSignal("");
  const [keyType, setKeyType] = createSignal("All Key Types");
  const [isKeyTypeDropdownOpen, setIsKeyTypeDropdownOpen] = createSignal(false);

  const { width: leftWidth, isResizing, handlePointerDown } = useResizer(240, 150, 600);

  // Reset local state when connection switches
  createEffect(on(() => props.connectionId?.(), () => {
    setActiveKey(null);
    setFilter("");
    setKeyType("All Key Types");
  }, { defer: true }));

  return (
    <>
      <div class="flex items-center gap-3 px-4 py-3 slick-border-b bg-[var(--color-surface-1)]">
        <div class="flex items-center rounded border border-[var(--color-border-strong)] bg-white overflow-hidden shrink-0 h-9">
          <button
            title="Tree view"
            onClick={() => setView("tree")}
            class={`px-3 h-full transition-colors ${view() === "tree" ? "bg-[var(--color-brand)] text-white" : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-2)]"}`}
          >
            <LayoutList class="w-4 h-4" />
          </button>
          <button
            title="Flat list"
            onClick={() => setView("flat")}
            class={`px-3 h-full transition-colors ${view() === "flat" ? "bg-[var(--color-brand)] text-white" : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-2)]"}`}
          >
            <List class="w-4 h-4" />
          </button>
        </div>

        <Button variant="outline" onClick={fetchKeys} class="h-9 px-3 bg-white border-[var(--color-border-strong)] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-2)] shrink-0">
          <RefreshCw class="w-4 h-4" />
        </Button>

        <div class="relative shrink-0">
          <button
            onClick={() => setIsKeyTypeDropdownOpen(!isKeyTypeDropdownOpen())}
            class="h-9 px-3 flex items-center justify-between gap-2 bg-white border border-[var(--color-border-strong)] rounded text-sm text-[var(--color-text-secondary)] min-w-[140px] hover:border-[var(--color-text-muted)] transition-colors"
          >
            {keyType()}
            <ChevronDown class="w-4 h-4 text-[var(--color-text-muted)]" />
          </button>

          <Show when={isKeyTypeDropdownOpen()}>
            <>
              <div class="fixed inset-0 z-40" onClick={() => setIsKeyTypeDropdownOpen(false)} />
              <div class="absolute top-full left-0 mt-1 w-full bg-white border border-[var(--color-border-strong)] rounded shadow-lg z-50 overflow-hidden text-sm text-[var(--color-text-primary)]">
                {KEY_TYPE_OPTIONS.map(type => (
                  <div
                    class="px-3 py-2 cursor-pointer hover:bg-[var(--color-surface-2)]"
                    onClick={() => { setKeyType(type); setIsKeyTypeDropdownOpen(false); }}
                  >
                    {type}
                  </div>
                ))}
              </div>
            </>
          </Show>
        </div>

        <SearchInput
          placeholder="Filter by Key Name or Pattern"
          value={filter()}
          onInput={(e) => setFilter(e.currentTarget.value)}
          iconPosition="right"
          class="flex-1 min-w-0"
        />
      </div>

      <div class="flex-1 flex min-h-0 overflow-hidden relative">
        <div style={{ width: `${leftWidth()}px` }} class="flex flex-col shrink-0 min-h-0 bg-white">
          <KeyBrowser keys={keys()} keyTypes={keyTypes()} activeKey={activeKey()} setActiveKey={setActiveKey} view={view()} filter={filter()} keyTypeFilter={keyType()} />
        </div>

        <div
          class="w-1 bg-[var(--color-border)] hover:bg-[var(--color-brand)] cursor-col-resize shrink-0 transition-colors"
          onPointerDown={handlePointerDown}
        ></div>

        <Inspector
          activeKey={activeKey}
          onDelete={() => { setActiveKey(null); fetchKeys(); }}
          onRename={(newKey) => { setActiveKey(newKey); fetchKeys(); }}
        />
      </div>
    </>
  );
}
