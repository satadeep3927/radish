import { splitProps } from "solid-js";
import { type ComponentProps } from "solid-js";
import { Search } from "lucide-solid";
import { cn } from "../../lib/utils";

export interface SearchInputProps extends ComponentProps<"input"> {
  iconPosition?: "left" | "right";
}

export function SearchInput(props: SearchInputProps) {
  const [local, rest] = splitProps(props, ["class", "iconPosition", "value", "onInput"]);

  const position = () => local.iconPosition || "left";

  return (
    <div class={cn("relative flex items-center w-full", local.class)}>
      {position() === "left" && (
        <Search class="w-4 h-4 absolute left-3 text-[var(--color-text-muted)] pointer-events-none" />
      )}
      <input
        type="text"
        value={local.value}
        onInput={local.onInput}
        class={cn(
          "w-full h-9 bg-[var(--color-surface-2)] text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] border-[0.5px] border-[var(--color-border-strong)] focus:border-[var(--color-brand)] focus:outline-none transition-colors rounded-md",
          {
            "pl-9 pr-4": position() === "left",
            "pl-3 pr-9": position() === "right",
          }
        )}
        {...rest}
      />
      {position() === "right" && (
        <Search class="w-4 h-4 absolute right-3 text-[var(--color-text-muted)] pointer-events-none" />
      )}
    </div>
  );
}
