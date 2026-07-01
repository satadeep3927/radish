import { getCurrentWindow } from "@tauri-apps/api/window";
import { X, Minus, Square, HelpCircle, Settings, Database, ChevronDown } from "lucide-solid";
import { useNavigation } from "../hooks/useNavigation";
import { useConnections } from "../hooks/useConnections";
import { createSignal } from "solid-js";

export function Titlebar() {
  const appWindow = getCurrentWindow();
  const { setActiveView } = useNavigation();
  const { activeConnection, setActiveConnectionId, connections, selectConnection } = useConnections();
  const [dropdownOpen, setDropdownOpen] = createSignal(false);

  return (
    <div
      data-tauri-drag-region
      class="h-10 bg-[var(--color-brand)] flex items-center justify-between select-none z-50 shrink-0 text-white relative"
    >
      {/* Draggable Title Area */}
      <div 
        data-tauri-drag-region 
        class="flex items-center h-full gap-3 px-3 w-48"
      >
        <span data-tauri-drag-region class="font-semibold text-[11px] tracking-wide mt-0.5 pointer-events-none opacity-80">
          RADISH STUDIO
        </span>
      </div>

      {/* Connection Selector (Center) */}
      <div data-tauri-drag-region class="flex-1 flex justify-center h-full items-center">
        {activeConnection() && (
          <div class="relative">
            <button
              onClick={(e) => {
                e.stopPropagation();
                setDropdownOpen(!dropdownOpen());
              }}
              class="flex items-center gap-2 px-3 py-1 rounded bg-white/10 hover:bg-white/20 transition-colors text-sm font-medium"
            >
              <Database class="w-3.5 h-3.5 opacity-80" />
              <span>{activeConnection()?.alias}</span>
              <span class="text-xs opacity-60 font-mono ml-1">({activeConnection()?.host}:{activeConnection()?.port})</span>
              <ChevronDown class="w-3.5 h-3.5 ml-1 opacity-70" />
            </button>

            {dropdownOpen() && (
              <>
                {/* Invisible overlay to catch clicks outside */}
                <div 
                  class="fixed inset-0 z-40" 
                  onClick={() => setDropdownOpen(false)}
                />
                <div 
                  onClick={(e) => e.stopPropagation()}
                  class="absolute top-full mt-1.5 left-1/2 -translate-x-1/2 w-64 bg-white rounded shadow-lg border border-gray-200 py-1 text-gray-800 z-50"
                >
                <div class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider border-b border-gray-100 mb-1">
                  Switch Connection
                </div>
                <div class="max-h-64 overflow-y-auto">
                  {connections().map(conn => (
                    <button
                      onClick={() => {
                        selectConnection(conn.id);
                        setDropdownOpen(false);
                      }}
                      class={`w-full text-left px-3 py-2 hover:bg-[var(--color-brand-subtle)] hover:text-[var(--color-brand)] transition-colors flex items-center justify-between ${
                        activeConnection()?.id === conn.id ? 'bg-[var(--color-brand-subtle)] text-[var(--color-brand)]' : ''
                      }`}
                    >
                      <span class="text-sm font-medium truncate">{conn.alias}</span>
                      {activeConnection()?.id === conn.id && <div class="w-2 h-2 rounded-full bg-[var(--color-brand)]"></div>}
                    </button>
                  ))}
                </div>
                <div class="border-t border-gray-100 mt-1 pt-1">
                  <button
                    onClick={() => {
                      setActiveConnectionId(null);
                      setDropdownOpen(false);
                    }}
                    class="w-full text-left px-3 py-2 text-sm text-gray-600 hover:bg-gray-50 transition-colors flex items-center gap-2"
                  >
                    <Database class="w-4 h-4" />
                    All Databases
                  </button>
                </div>
              </div>
              </>
            )}
          </div>
        )}
      </div>

      {/* Control Icons Area */}
      <div class="flex items-center h-full gap-1 px-2 mr-2">
        <button 
          onClick={() => setActiveView("help")}
          class="w-7 h-7 flex items-center justify-center hover:bg-white/20 rounded transition-colors"
        >
          <HelpCircle class="w-4 h-4" />
        </button>
        <button 
          onClick={() => setActiveView("config")}
          class="w-7 h-7 flex items-center justify-center hover:bg-white/20 rounded transition-colors"
        >
          <Settings class="w-4 h-4" />
        </button>
      </div>

      {/* Window Controls */}
      <div class="flex h-full">
        <div
          class="w-11 h-full flex justify-center items-center hover:bg-white/10 transition-colors cursor-pointer text-white"
          onClick={() => appWindow.minimize()}
        >
          <Minus class="w-4 h-4" />
        </div>
        <div
          class="w-11 h-full flex justify-center items-center hover:bg-white/10 transition-colors cursor-pointer text-white"
          onClick={() => appWindow.toggleMaximize()}
        >
          <Square class="w-3.5 h-3.5" />
        </div>
        <div
          class="w-11 h-full flex justify-center items-center hover:bg-red-600 transition-colors cursor-pointer text-white"
          onClick={() => appWindow.close()}
        >
          <X class="w-4 h-4" />
        </div>
      </div>
    </div>
  );
}
