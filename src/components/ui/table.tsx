import { splitProps } from "solid-js";
import { type ComponentProps } from "solid-js";
import { cn } from "../../lib/utils";

export function Table(props: ComponentProps<"table">) {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <table
      class={cn("w-full caption-bottom text-sm border-collapse", local.class)}
      {...rest}
    />
  );
}

export function TableHeader(props: ComponentProps<"thead">) {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <thead class={cn("[&_tr]:border-b bg-[var(--color-surface-2)] border-[var(--color-border-strong)]", local.class)} {...rest} />
  );
}

export function TableBody(props: ComponentProps<"tbody">) {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <tbody class={cn("[&_tr:last-child]:border-0", local.class)} {...rest} />
  );
}

export function TableRow(props: ComponentProps<"tr">) {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <tr
      class={cn(
        "border-b border-[var(--color-border-strong)] transition-colors hover:bg-[var(--color-surface-2)]",
        local.class
      )}
      {...rest}
    />
  );
}

export function TableHead(props: ComponentProps<"th">) {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <th
      class={cn(
        "h-10 px-4 text-left align-middle font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider text-xs",
        local.class
      )}
      {...rest}
    />
  );
}

export function TableCell(props: ComponentProps<"td">) {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <td
      class={cn(
        "p-4 align-middle",
        local.class
      )}
      {...rest}
    />
  );
}
