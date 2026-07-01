import { createSignal, Show, For } from "solid-js";
import { ChevronDown, ChevronRight, FolderOpen, Folder } from "lucide-solid";
import { useGroupedKeys, KeyNode } from "../../hooks/useGroupedKeys";
import { Badge } from "../ui/badge";

interface KeyBrowserProps {
  keys: string[];
  activeKey: string | null;
  setActiveKey: (key: string) => void;
  view: "flat" | "tree";
  filter: string;
  keyTypeFilter: string;
  keyTypes: Record<string, string>;
}

// ─── Recursive tree node ────────────────────────────────────────────────────

function TreeNode(props: { node: KeyNode; activeKey: string | null; setActiveKey: (k: string) => void; depth: number; keyTypes: Record<string, string> }) {
  const hasChildren = () => props.node.children.length > 0;
  const isFolder = () => hasChildren();
  // Folders start collapsed if they have many children, open otherwise
  const [open, setOpen] = createSignal(props.node.children.length <= 5);

  const indent = () => `${props.depth * 12 + 8}px`;

  return (
    <>
      {/* Folder row — clicking toggles expand */}
      <Show when={isFolder()}>
        <div
          style={{ "padding-left": indent() }}
          class="flex items-center gap-2 py-1.5 pr-4 cursor-pointer hover:bg-[var(--color-surface-2)] select-none group border-b border-transparent"
          onClick={() => setOpen(v => !v)}
        >
          <span class="text-[var(--color-text-muted)] shrink-0 w-4 h-4 flex items-center justify-center">
            {open()
              ? <ChevronDown class="w-4 h-4" />
              : <ChevronRight class="w-4 h-4" />}
          </span>
          {open()
            ? <FolderOpen class="w-4 h-4 text-[var(--color-text-secondary)] shrink-0" />
            : <Folder class="w-4 h-4 text-[var(--color-text-secondary)] shrink-0" />}
          <span class="text-sm font-medium text-[var(--color-text-secondary)] truncate flex-1">{props.node.name}</span>
          <span class="text-xs text-[var(--color-text-muted)] ml-auto shrink-0 font-mono">{props.node.children.length}</span>
        </div>
        {/* If the folder segment is ALSO a real key, show it as a leaf beneath */}
        <Show when={props.node.full_key && open()}>
          <KeyRow
            fullKey={props.node.full_key!}
            label={props.node.name}
            activeKey={props.activeKey}
            setActiveKey={props.setActiveKey}
            depth={props.depth + 1}
            isAlias
            type={props.keyTypes[props.node.full_key!]}
          />
        </Show>
      </Show>

      {/* Pure leaf row (no children) */}
      <Show when={!isFolder() && props.node.full_key}>
        <KeyRow
          fullKey={props.node.full_key!}
          label={props.node.name}
          activeKey={props.activeKey}
          setActiveKey={props.setActiveKey}
          depth={props.depth}
          type={props.keyTypes[props.node.full_key!]}
        />
      </Show>

      {/* Children */}
      <Show when={isFolder() && open()}>
        <For each={props.node.children}>
          {(child) => (
            <TreeNode
              node={child}
              activeKey={props.activeKey}
              setActiveKey={props.setActiveKey}
              depth={props.depth + 1}
              keyTypes={props.keyTypes}
            />
          )}
        </For>
      </Show>
    </>
  );
}

function KeyRow(props: {
  fullKey: string;
  label: string;
  activeKey: string | null;
  setActiveKey: (k: string) => void;
  depth: number;
  isAlias?: boolean;
  type?: string;
}) {
  const isActive = () => props.activeKey === props.fullKey;
  const indent = () => `${props.depth * 12 + 8}px`;

  return (
    <div
      style={{ "padding-left": indent() }}
      class={`flex items-center gap-2 pr-2.5 py-1.5 cursor-pointer slick-border-b transition-colors ${
        isActive()
          ? "bg-[var(--color-brand-subtle)] border-l-2 border-l-[var(--color-brand)]"
          : "hover:bg-[var(--color-surface-2)]"
      }`}
      onClick={() => props.setActiveKey(props.fullKey)}
    >
      <Badge type={props.type} />
      <span
        class={`text-sm font-mono truncate flex-1 ${isActive() ? "text-[var(--color-brand)] font-medium" : "text-[var(--color-text-primary)]"}`}
        title={props.fullKey}
      >
        {props.isAlias ? `${props.label} (key)` : props.label}
      </span>
    </div>
  );
}

// ─── Main KeyBrowser ─────────────────────────────────────────────────────────

export function KeyBrowser(props: KeyBrowserProps) {
  const [isRootOpen, setIsRootOpen] = createSignal(true);

  const filteredKeys = () => {
    let keys = props.keys;
    
    if (props.filter) {
      keys = keys.filter(k => k.toLowerCase().includes(props.filter.toLowerCase()));
    }
    
    if (props.keyTypeFilter && props.keyTypeFilter !== "All Key Types") {
      keys = keys.filter(k => {
        const t = props.keyTypes[k];
        return t && t.toLowerCase() === props.keyTypeFilter.toLowerCase();
      });
    }
    return keys;
  };

  const { tree, isBuilding } = useGroupedKeys(filteredKeys);

  return (
    <div class="flex flex-col h-full min-h-0 bg-[var(--color-surface-1)]">

      {/* Key count header */}
      <div 
        onClick={() => setIsRootOpen(!isRootOpen())}
        class="px-3 py-2 text-xs text-[var(--color-text-muted)] uppercase tracking-wider font-semibold slick-border-b flex items-center gap-1.5 cursor-pointer hover:bg-[var(--color-surface-2)] select-none animate-none"
      >
        <Show when={isRootOpen()} fallback={<ChevronRight class="w-4 h-4" />}>
          <ChevronDown class="w-4 h-4" />
        </Show>
        ALL KEYS
        <span class="ml-auto font-mono">{filteredKeys().length}</span>
      </div>

      {/* List area — min-h-0 is required so flex-1 can actually shrink below content height and allow overflow-y-auto to scroll */}
      <div class="flex-1 overflow-y-auto min-h-0">
        <Show when={isRootOpen()}>
          {/* ── FLAT VIEW ── */}
          <Show when={props.view === "flat"}>
            <Show
              when={filteredKeys().length > 0}
              fallback={<div class="text-center py-8 text-sm text-[var(--color-text-muted)]">No keys found</div>}
            >
              <For each={filteredKeys()}>
                {(key) => {
                  return (
                    <KeyRow
                      fullKey={key}
                      label={key}
                      activeKey={props.activeKey}
                      setActiveKey={props.setActiveKey}
                      depth={0}
                      type={props.keyTypes[key]}
                    />
                  );
                }}
              </For>
            </Show>
          </Show>

          {/* ── TREE VIEW ── */}
          <Show when={props.view === "tree"}>
            <Show when={isBuilding()}>
              <div class="text-center py-8 text-xs text-[var(--color-text-muted)] animate-pulse">Building tree…</div>
            </Show>
            <Show when={!isBuilding() && tree().length === 0}>
              <div class="text-center py-8 text-xs text-[var(--color-text-muted)]">No keys found</div>
            </Show>
            <Show when={!isBuilding() && tree().length > 0}>
              <For each={tree()}>
                {(node) => (
                  <TreeNode
                    node={node}
                    activeKey={props.activeKey}
                    setActiveKey={props.setActiveKey}
                    depth={0}
                    keyTypes={props.keyTypes}
                  />
                )}
              </For>
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  );
}
