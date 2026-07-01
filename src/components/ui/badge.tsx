import { splitProps } from "solid-js";
import { type ComponentProps } from "solid-js";
import { cva } from "../../lib/utils";

export interface BadgeProps extends ComponentProps<"span"> {
  type?: "string" | "hash" | "list" | "set" | "zset" | "none" | string;
}

const badgeVariants = cva({
  base: "text-[10px] font-bold px-1.5 py-0.5 rounded border leading-none shrink-0 text-center tracking-wider",
  variants: {
    type: {
      string: "bg-[var(--color-badge-string)] text-white border-transparent",
      hash: "bg-[var(--color-badge-hash)] text-white border-transparent",
      list: "bg-[var(--color-badge-list)] text-white border-transparent",
      set: "bg-[var(--color-badge-set)] text-white border-transparent",
      zset: "bg-[var(--color-badge-zset)] text-white border-transparent",
      none: "bg-[var(--color-badge-none)] text-white border-transparent",
    },
  },
  defaultVariants: {
    type: "none",
  },
});

export function Badge(props: BadgeProps) {
  const [local, rest] = splitProps(props, ["class", "type"]);

  const type = () => (local.type?.toLowerCase() || "none");

  return (
    <span
      class={badgeVariants({ type: type(), class: local.class })}
      {...rest}
    >
      {type() === "none" ? "..." : type().toUpperCase()}
    </span>
  );
}
