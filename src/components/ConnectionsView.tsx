import { createSignal } from "solid-js";
import { Plus, Search, Server, Trash2, ChevronRight } from "lucide-solid";
import { useConnections } from "../hooks/useConnections";
import { useDialog } from "../hooks/useDialog";
import { ConnectionForm } from "./ConnectionForm";
import { Button } from "./ui/button";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "./ui/table";
import { SearchInput } from "./ui/search-input";

export function ConnectionsView() {
  const { connections, selectConnection, addConnection, removeConnection } = useConnections();
  const { alert, confirm } = useDialog();
  const [searchTerm, setSearchTerm] = createSignal("");
  const [showAddModal, setShowAddModal] = createSignal(false);

  const filteredConnections = () => {
    const term = searchTerm().toLowerCase();
    return connections().filter(c => 
      c.alias.toLowerCase().includes(term) || 
      c.host.toLowerCase().includes(term) ||
      c.id.includes(term)
    );
  };

  const handleAdd = (alias: string, host: string, port: number) => {
    addConnection({
      alias: alias.trim(),
      host: host.trim(),
      port: port,
      type: "standalone",
    });
    setShowAddModal(false);
  };

  const handleDelete = (e: Event, id: string, alias: string) => {
    e.stopPropagation(); // Prevent row click
    if (id === "local-radish-engine") {
      alert({ title: "Restricted", message: "You cannot delete the bundled local Radish engine connection.", variant: "danger" });
      return;
    }
    confirm({
      title: "Delete Connection",
      message: `Are you sure you want to delete the connection "${alias}"?`,
      confirmText: "Delete",
      variant: "danger",
      onConfirm: () => {
        removeConnection(id);
      }
    });
  };

  return (
    <div class="flex-1 flex flex-col min-h-0 w-full bg-[var(--color-surface-0)] text-[var(--color-text-primary)]">
      {/* Header */}
      <div class="flex items-center justify-between px-6 py-4 slick-border-b bg-[var(--color-surface-1)]">
        <div class="flex items-center gap-2">
          <h1 class="text-base font-medium tracking-tight text-[var(--color-text-primary)]">Radish Databases</h1>
        </div>
      </div>

      {/* Toolbar */}
      <div class="flex items-center justify-between px-6 py-4 slick-border-b bg-[var(--color-surface-0)]">
        <div class="flex items-center gap-4">
          <Button 
            onClick={() => setShowAddModal(true)}
            variant="secondary"
            class="flex items-center gap-2 h-9 px-4 text-sm"
          >
            <Plus class="w-4 h-4" />
            Connect database
          </Button>
        </div>
        <SearchInput
          placeholder="Database List Search"
          value={searchTerm()}
          onInput={(e) => setSearchTerm(e.currentTarget.value)}
          class="w-64"
        />
      </div>

      {/* Table */}
      <div class="flex-1 overflow-auto p-6">
        <div class="border border-[var(--color-border-strong)] rounded-lg bg-[var(--color-surface-1)] overflow-hidden shadow-sm">
          <Table>
            <TableHeader>
              <TableRow class="bg-[var(--color-surface-0)] hover:bg-transparent">
                <TableHead class="px-6 py-3 w-1/4">Database Alias</TableHead>
                <TableHead class="px-6 py-3 w-1/4">Host:Port</TableHead>
                <TableHead class="px-6 py-3 w-1/6">Connection Type</TableHead>
                <TableHead class="px-6 py-3 w-1/4">Last Connection</TableHead>
                <TableHead class="px-6 py-3 w-16 text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredConnections().length === 0 ? (
                <TableRow>
                  <TableCell colspan="5" class="px-6 py-12 text-center text-[var(--color-text-secondary)]">
                    No databases found. Click "Connect database" to add one.
                  </TableCell>
                </TableRow>
              ) : (
                filteredConnections().map((conn) => (
                  <TableRow 
                    class="cursor-pointer group"
                    onClick={() => selectConnection(conn.id)}
                  >
                    <TableCell class="px-6 py-4">
                      <div class="flex items-center gap-3">
                        <div class={`w-8 h-8 rounded flex items-center justify-center ${conn.id === "local-radish-engine" ? "bg-[var(--color-brand-subtle)] text-[var(--color-brand)]" : "bg-[var(--color-surface-3)] text-[var(--color-text-secondary)]"}`}>
                          <Server class="w-4 h-4" />
                        </div>
                        <span class="font-medium text-sm text-[var(--color-text-primary)]">{conn.alias}</span>
                      </div>
                    </TableCell>
                    <TableCell class="px-6 py-4 text-sm font-mono text-[var(--color-text-secondary)]">
                      {conn.host}:{conn.port}
                    </TableCell>
                    <TableCell class="px-6 py-4 text-sm text-[var(--color-text-secondary)] capitalize">
                      {conn.type}
                    </TableCell>
                    <TableCell class="px-6 py-4 text-sm text-[var(--color-text-secondary)]">
                      {conn.lastConnection ? new Date(conn.lastConnection).toLocaleString() : "Never"}
                    </TableCell>
                    <TableCell class="px-6 py-4 text-right">
                      <div class="flex items-center justify-end gap-2">
                        {conn.id !== "local-radish-engine" && (
                          <button 
                            onClick={(e) => handleDelete(e, conn.id, conn.alias)}
                            class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-brand)] hover:bg-[var(--color-brand-subtle)] rounded transition-colors opacity-0 group-hover:opacity-100"
                            title="Delete Connection"
                          >
                            <Trash2 class="w-4 h-4" />
                          </button>
                        )}
                        <button 
                          class="flex items-center gap-1 px-3 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] bg-[var(--color-surface-2)] hover:bg-[var(--color-brand)] hover:text-white rounded transition-colors"
                        >
                          Connect
                          <ChevronRight class="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      </div>

      {/* Add Connection Modal */}
      {showAddModal() && (
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
          <div class="bg-[var(--color-surface-0)] rounded-lg shadow-xl border border-[var(--color-border-strong)] overflow-hidden">
            <div class="px-6 py-4 border-b border-[var(--color-border-strong)] flex items-center justify-between bg-[var(--color-surface-1)]">
              <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">Connect Database</h2>
            </div>
            
            <div class="p-6 bg-[var(--color-surface-0)]">
              <ConnectionForm 
                onConnect={handleAdd}
                isLoading={false}
                onCancel={() => setShowAddModal(false)}
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
