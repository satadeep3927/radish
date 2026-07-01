import { Show, JSX, Accessor } from "solid-js";
import { 
  Database, 
  BarChart2, 
  Radio, 
  TerminalSquare, 
  Server, 
  Settings, 
  Power,
  Book
} from "lucide-solid";
import logoWide from "../../assets/logo-wide.png";

interface SidebarProps {
  activeView: string;
  setActiveView: (view: string) => void;
  keyCount: number;
  onStopServer: () => void;
  isEngineOwnedByStudio: Accessor<boolean>;
}

interface SidebarNavItemProps {
  view: string;
  activeView: string;
  onClick: (view: string) => void;
  icon: JSX.Element;
  label: string;
  badge?: JSX.Element;
}

function SidebarNavItem(props: SidebarNavItemProps) {
  const isActive = () => props.activeView === props.view;

  return (
    <button 
      onClick={() => props.onClick(props.view)}
      class={`flex items-center gap-2 px-2.5 py-[7px] mx-1.5 rounded-md my-[1px] text-xs transition-colors cursor-pointer ${
        isActive() 
          ? "bg-[var(--color-surface-2)] text-[var(--color-text-primary)] border-l-2 border-[var(--color-brand)] !px-2" 
          : "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-2)]"
      }`}
    >
      {props.icon}
      <span>{props.label}</span>
      {props.badge}
    </button>
  );
}

export function Sidebar(props: SidebarProps) {
  return (
    <div class="w-[200px] bg-[var(--color-surface-1)] slick-border-r flex flex-col shrink-0 min-h-0">
      <div class="flex items-center px-4 pt-4 pb-3 slick-border-b shrink-0">
        <img src={logoWide} alt="Radish Studio" class="w-[80px]" />
      </div>
      
      {/* Scrollable nav area */}
      <div class="flex-1 min-h-0 overflow-y-auto flex flex-col items-stretch">
        <div class="px-2.5 pt-2.5 pb-1 text-[10px] tracking-widest uppercase text-[var(--color-text-muted)] font-medium">
          Overview
        </div>
        
        <SidebarNavItem
          view="keys"
          activeView={props.activeView}
          onClick={props.setActiveView}
          icon={<Database class="w-4 h-4 shrink-0" />}
          label="Keys"
          badge={
            <span class={`ml-auto text-[10px] px-1.5 py-[1px] rounded-sm border ${
              props.activeView === "keys" 
                ? "bg-[var(--color-brand-subtle)] text-[var(--color-brand)] border-[var(--color-brand)]" 
                : "bg-[var(--color-surface-0)] text-[var(--color-text-muted)] border-[var(--color-border-strong)]"
            }`}>
              {props.keyCount.toLocaleString()}
            </span>
          }
        />

        <SidebarNavItem
          view="analyze"
          activeView={props.activeView}
          onClick={props.setActiveView}
          icon={<BarChart2 class="w-4 h-4 shrink-0" />}
          label="Analyze"
        />

        <SidebarNavItem
          view="pubsub"
          activeView={props.activeView}
          onClick={props.setActiveView}
          icon={<Radio class="w-4 h-4 shrink-0" />}
          label="Pub/Sub"
        />

        <div class="px-2 pt-3 pb-1 text-[10px] tracking-widest uppercase text-[var(--color-text-muted)] font-medium">
          Tools
        </div>
        
        <SidebarNavItem
          view="cli"
          activeView={props.activeView}
          onClick={props.setActiveView}
          icon={<TerminalSquare class="w-4 h-4 shrink-0" />}
          label="CLI"
        />

        {/* Only show Manage section when Studio actually owns the server process */}
        <Show when={props.isEngineOwnedByStudio()}>
          <div class="px-2 pt-3 pb-1 text-[10px] tracking-widest uppercase text-[var(--color-text-muted)] font-medium">
            Manage
          </div>
          
          <SidebarNavItem
            view="config"
            activeView={props.activeView}
            onClick={props.setActiveView}
            icon={<Settings class="w-4 h-4 shrink-0" />}
            label="Config"
          />

          <SidebarNavItem
            view="service"
            activeView={props.activeView}
            onClick={props.setActiveView}
            icon={<Server class="w-4 h-4 shrink-0" />}
            label="Service"
          />
        </Show>
      </div>

      {/* Stop button pinned at bottom — only when Studio owns the engine */}
      <Show when={props.isEngineOwnedByStudio()}>
        <div class="p-2 slick-border-t shrink-0">
          <button 
            onClick={props.onStopServer}
            class="w-full flex items-center gap-2 px-2.5 py-[7px] rounded-md text-xs text-[var(--color-brand)] hover:bg-[var(--color-brand-subtle)] hover:text-[var(--color-brand-hover)] transition-colors cursor-pointer"
          >
            <Power class="w-4 h-4 shrink-0" />
            Stop Radish
          </button>
        </div>
      </Show>
    </div>
  );
}
