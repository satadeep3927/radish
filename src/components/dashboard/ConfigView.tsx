import { Settings, Save, Server, FileText, Shield, Lock } from "lucide-solid";
import { Button } from "../ui/button";
import { useConfig } from "../../hooks/useConfig";
import { Show } from "solid-js";

export function ConfigView() {
  const {
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
  } = useConfig();

  return (
    <div class="flex-1 flex flex-col min-h-0 bg-[var(--color-surface-0)] text-[var(--color-text-primary)]">
      {/* Header */}
      <div class="flex items-center justify-between px-6 py-4 slick-border-b bg-[var(--color-surface-1)]">
        <div class="flex items-center gap-2 text-[var(--color-text-secondary)] font-sans">
          <Settings class="w-5 h-5 text-[var(--color-brand)]" />
          <span class="font-medium text-[var(--color-text-primary)]">Configuration</span>
        </div>
        <Button 
          variant="outline"
          onClick={handleSave} 
          class="h-8 px-3 gap-1.5 text-xs text-[var(--color-text-primary)] bg-[var(--color-surface-2)] border-[var(--color-border-strong)] hover:bg-[var(--color-surface-3)]"
        >
          <Save class="w-3.5 h-3.5 text-blue-500" />
          Save Changes
        </Button>
      </div>

      <div class="flex-1 overflow-y-auto p-6 max-w-4xl mx-auto w-full">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          
          {/* General Section */}
          <div class="bg-[var(--color-surface-1)] rounded-md border border-[var(--color-border-strong)] overflow-hidden">
            <div class="px-4 py-3 bg-[var(--color-surface-2)] border-b border-[var(--color-border-strong)] flex items-center gap-2">
              <Server class="w-4 h-4 text-[var(--color-text-muted)]" />
              <h3 class="text-sm font-medium">Server Settings</h3>
            </div>
            <div class="p-4 space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">Bind Address</label>
                  <input 
                    type="text" 
                    value={bind()}
                    onInput={(e) => setBind(e.currentTarget.value)}
                    class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-strong)] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[var(--color-brand)] transition-colors"
                  />
                </div>
                <div>
                  <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">Port Number</label>
                  <input 
                    type="number" 
                    value={port()}
                    onInput={(e) => { const v = parseInt(e.currentTarget.value); setPort(Number.isNaN(v) ? 6379 : v); }}
                    class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-strong)] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[var(--color-brand)] transition-colors"
                  />
                </div>
              </div>
              
              <div>
                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">Max Memory</label>
                <input 
                  type="text" 
                  value={maxMemory()}
                  onInput={(e) => setMaxMemory(e.currentTarget.value)}
                  class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-strong)] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[var(--color-brand)] transition-colors"
                />
                <p class="text-[10px] text-[var(--color-text-muted)] mt-1">Memory limit (e.g. 0 for unlimited, 256mb, 1gb).</p>
              </div>
            </div>
          </div>

          {/* Security Section */}
          <div class="bg-[var(--color-surface-1)] rounded-md border border-[var(--color-border-strong)] overflow-hidden">
            <div class="px-4 py-3 bg-[var(--color-surface-2)] border-b border-[var(--color-border-strong)] flex items-center gap-2">
              <Shield class="w-4 h-4 text-[var(--color-text-muted)]" />
              <h3 class="text-sm font-medium">Security Settings</h3>
            </div>
            <div class="p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <label class="block text-xs font-medium text-[var(--color-text-primary)]">Require Authentication</label>
                  <p class="text-[10px] text-[var(--color-text-muted)]">Clients must authenticate using AUTH.</p>
                </div>
                <input 
                  type="checkbox"
                  checked={requiresAuth()}
                  onChange={(e) => setRequiresAuth(e.currentTarget.checked)}
                  class="w-4 h-4 accent-[var(--color-brand)] cursor-pointer"
                />
              </div>

              <Show when={requiresAuth()}>
                <div>
                  <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">Server Password</label>
                  <input 
                    type="password" 
                    value={password()}
                    onInput={(e) => setPassword(e.currentTarget.value)}
                    placeholder="Enter password"
                    class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-strong)] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[var(--color-brand)] transition-colors"
                  />
                </div>
              </Show>
            </div>
          </div>

          {/* Persistence Section */}
          <div class="bg-[var(--color-surface-1)] rounded-md border border-[var(--color-border-strong)] overflow-hidden md:col-span-2">
            <div class="px-4 py-3 bg-[var(--color-surface-2)] border-b border-[var(--color-border-strong)] flex items-center gap-2">
              <FileText class="w-4 h-4 text-[var(--color-text-muted)]" />
              <h3 class="text-sm font-medium">Persistence & Snapshots</h3>
            </div>
            <div class="p-4 grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">Save Interval (seconds)</label>
                <input 
                  type="number" 
                  value={saveInterval()}
                  onInput={(e) => { const v = parseInt(e.currentTarget.value); setSaveInterval(Number.isNaN(v) ? 0 : v); }}
                  class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-strong)] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[var(--color-brand)] transition-colors"
                />
                <p class="text-[10px] text-[var(--color-text-muted)] mt-1">Snapshot interval (0 to disable auto-saves).</p>
              </div>

              <div>
                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">Database Dump Path</label>
                <input 
                  type="text" 
                  value={dumpPath()}
                  onInput={(e) => setDumpPath(e.currentTarget.value)}
                  class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-strong)] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[var(--color-brand)] transition-colors"
                />
                <p class="text-[10px] text-[var(--color-text-muted)] mt-1">Snapshot save destination filename.</p>
              </div>
            </div>
          </div>
          
        </div>
      </div>
    </div>
  );
}
