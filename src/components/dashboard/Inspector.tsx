import { createSignal, Show, Accessor } from "solid-js";
import { Trash2, X, RefreshCw, Maximize2, Minimize2 } from "lucide-solid";
import { useInspector } from "../../hooks/useInspector";
import { useDialog } from "../../hooks/useDialog";
import { Badge } from "../ui/badge";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "../ui/table";

interface InspectorProps {
  activeKey: Accessor<string | null>;
  onDelete: () => void;
  onRename: (newKey: string) => void;
}

export function Inspector(props: InspectorProps) {
  const { details, isLoading, error, deleteKey, updateValue, updateHashField, updateListElement, updateSetElement, refresh } = useInspector(props.activeKey);
  const { confirm, prompt, alert } = useDialog();

  const [isFullscreen, setIsFullscreen] = createSignal(false);

  const handleDelete = () => {
    const onDelete = props.onDelete;
    confirm({
      title: "Delete Key",
      message: `Are you sure you want to delete "${props.activeKey()}"? This cannot be undone.`,
      confirmText: "Delete",
      cancelText: "Cancel",
      variant: "danger",
      onConfirm: async () => {
        await deleteKey();
        onDelete();
      }
    });
  };

  const handleEditString = () => {
    prompt({
      title: "Edit Value",
      label: "Value:",
      defaultValue: String(details()?.value || ""),
      confirmText: "Save",
      onConfirm: async (val) => {
        const ok = await updateValue(val);
        if (ok) alert({ title: "Saved", message: "Value updated successfully." });
      },
    });
  };

  const handleEditHashField = (field: string, current: string) => {
    prompt({
      title: `Edit Field: ${field}`,
      label: "Value:",
      defaultValue: current,
      confirmText: "Save",
      onConfirm: (val) => updateHashField(field, val),
    });
  };

  const handleEditListElement = (index: number, current: string) => {
    prompt({
      title: `Edit List Element at Index: ${index}`,
      label: "Value:",
      defaultValue: current,
      confirmText: "Save",
      onConfirm: (val) => updateListElement(index, val),
    });
  };

  const handleEditSetElement = (current: string) => {
    prompt({
      title: "Edit Set Element",
      label: "New Value:",
      defaultValue: current,
      confirmText: "Save",
      onConfirm: (val) => updateSetElement(current, val),
    });
  };

  return (
    <Show
      when={props.activeKey()}
      fallback={
        <div class="flex-1 self-stretch flex flex-col items-center justify-center min-w-0 text-[var(--color-text-muted)] text-sm gap-2">
          <div class="w-10 h-10 rounded-full bg-[var(--color-surface-1)] flex items-center justify-center text-[var(--color-text-muted)]">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 9h6M9 12h6M9 15h4"/></svg>
          </div>
          <span>Select a key to inspect</span>
        </div>
      }
    >
      <div class={`${isFullscreen() ? "fixed inset-0 z-50 shadow-2xl" : "flex-1 flex flex-col min-w-0 min-h-0"} bg-white flex flex-col`}>
        
        {/* Header row 1: Badge, Key Name, Maximize, Close */}
        <div class="flex items-center gap-3 px-6 pt-5 pb-2">
          <Show when={details()?.type}>
            <Badge type={details()?.type} class="text-[11px] px-2" />
          </Show>
          <div class="font-medium text-[var(--color-text-primary)] text-sm truncate flex-1 tracking-wide">
            {props.activeKey()}
          </div>
          <div class="flex items-center gap-2 text-[var(--color-text-muted)] shrink-0">
            <button class="hover:text-[var(--color-text-primary)] transition-colors p-1" title={isFullscreen() ? "Restore" : "Maximize"} onClick={() => setIsFullscreen(!isFullscreen())}>
              <Show when={isFullscreen()} fallback={<Maximize2 class="w-4 h-4" />}>
                <Minimize2 class="w-4 h-4" />
              </Show>
            </button>
            <button class="hover:text-[var(--color-brand)] transition-colors p-1" onClick={() => { setIsFullscreen(false); props.onDelete(); }} title="Close">
              <X class="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Header row 2: Length, TTL, Refresh, Delete */}
        <div class="flex items-center justify-between px-6 pb-4 slick-border-b">
          <div class="flex items-center gap-4 text-[13px] text-[var(--color-text-secondary)]">
            <span>
              Length: <span class="font-medium ml-1">{details()?.size || "-"}</span>
            </span>
            <span>
              TTL: <span class="font-medium ml-1 text-[var(--color-brand)]">
                {details()?.ttl === -1 ? "No limit" : details()?.ttl === -2 ? "Expired" : `${details()?.ttl}s`}
              </span>
            </span>
          </div>
          
          <div class="flex items-center gap-3 text-[13px] text-[var(--color-text-muted)]">
            <span class="text-[var(--color-text-muted)]">Just now</span>
            <button class="hover:text-[var(--color-text-primary)] transition-colors p-1" onClick={refresh} title="Refresh">
              <RefreshCw class="w-4 h-4" />
            </button>
            <button class="hover:text-[var(--color-brand)] transition-colors p-1" onClick={handleDelete} title="Delete Key">
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Content Area */}
        <div class="p-6 flex-1 overflow-y-auto min-w-0 min-h-0 bg-[var(--color-surface-1)]">
          <Show when={isLoading()}>
            <div class="text-sm text-[var(--color-text-muted)] text-center py-10 animate-pulse">Loading…</div>
          </Show>
          <Show when={error() && !isLoading()}>
            <div class="text-sm text-[var(--color-brand)] bg-[var(--color-brand-subtle)] p-4 rounded border border-[var(--color-brand)]">{error()}</div>
          </Show>
          
          <Show when={!isLoading() && !error() && details()}>
            <div class="flex flex-col h-full gap-4">
              
              {/* String Value Editor */}
              <Show when={details()?.type === "string"}>
                <div 
                  class="flex flex-col flex-1 relative bg-white border border-[var(--color-border-strong)] rounded-md shadow-sm overflow-hidden"
                  onDblClick={handleEditString}
                  title="Double-click to edit"
                >
                  <div
                    class="w-full flex-1 p-4 overflow-y-auto whitespace-pre-wrap outline-none text-sm font-mono text-[var(--color-text-primary)]"
                  >
                    {String(details()?.value || "")}
                  </div>
                </div>
              </Show>

              {/* Hash Editor */}
              <Show when={details()?.type === "hash"}>
                <div class="bg-white border border-[var(--color-border-strong)] rounded-md shadow-sm overflow-hidden">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead class="text-[11px] px-4 py-2 w-1/3">Field</TableHead>
                        <TableHead class="text-[11px] px-4 py-2">Value</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {Object.entries(details()?.value || {}).map(([k, v]) => (
                        <TableRow
                          class="cursor-pointer group"
                          onDblClick={() => handleEditHashField(k, String(v))}
                          title="Double-click to edit"
                        >
                          <TableCell class="text-sm font-mono px-4 py-2.5 text-[var(--color-text-primary)] truncate max-w-[150px]">{k}</TableCell>
                          <TableCell class="text-sm font-mono px-4 py-2.5 text-[var(--color-text-primary)] break-all">
                            {String(v)}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              </Show>

              {/* List / Set Editor */}
              <Show when={details()?.type === "list" || details()?.type === "set"}>
                <div class="bg-white border border-[var(--color-border-strong)] rounded-md shadow-sm overflow-hidden">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead class="text-[11px] px-4 py-2 w-[60px]">Index</TableHead>
                        <TableHead class="text-[11px] px-4 py-2">Value</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {(details()?.value as string[] || []).map((v: string, i: number) => (
                        <TableRow 
                          class="cursor-pointer group"
                          onDblClick={() => details()?.type === "list" ? handleEditListElement(i, String(v)) : handleEditSetElement(String(v))}
                          title="Double-click to edit"
                        >
                          <TableCell class="text-sm font-mono px-4 py-2.5 text-[var(--color-text-muted)]">{i}</TableCell>
                          <TableCell class="text-sm font-mono px-4 py-2.5 text-[var(--color-text-primary)] break-all">{String(v)}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              </Show>
              
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
}
