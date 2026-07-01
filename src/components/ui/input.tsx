import { splitProps } from "solid-js";
import { type ComponentProps } from "solid-js";
import { cn } from "~/lib/utils";

export interface InputProps extends ComponentProps<"input"> {}

export function Input(props: InputProps) {
  const [local, rest] = splitProps(props, ["class"]);

  return (
    <input
      class={cn(
        "flex h-9 w-full bg-[var(--color-surface-2)] px-3 py-1 text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] border-[0.5px] border-[var(--color-border-strong)] focus:border-[var(--color-brand)] focus:outline-none focus:ring-1 focus:ring-[var(--color-brand-subtle)] disabled:cursor-not-allowed disabled:opacity-50 transition-colors rounded-md",
        local.class
      )}
      {...rest}
    />
  );
}
