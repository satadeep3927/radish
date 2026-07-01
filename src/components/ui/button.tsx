import { splitProps } from "solid-js";
import { type ComponentProps } from "solid-js";
import { cva } from "~/lib/utils";

const buttonVariants = cva({
  base: "inline-flex items-center justify-center font-medium transition-colors border-[0.5px] border-[var(--color-border-strong)] rounded-md disabled:opacity-50 disabled:pointer-events-none text-xs",
  variants: {
    variant: {
      default: "bg-[var(--color-brand)] hover:bg-[var(--color-brand-hover)] text-white border-transparent",
      secondary: "bg-white text-black",
      outline: "bg-transparent text-black hover:bg-black/5",
      ghost: "border-none box-shadow-none hover:bg-black/5",
      danger: "bg-red-600 hover:bg-red-700 text-white border-transparent",
    },
    size: {
      default: "h-12 px-6 py-2 text-base",
      sm: "h-9 px-3 text-sm",
      lg: "h-16 px-8 text-lg",
      icon: "h-10 w-10",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "default",
  },
});

export interface ButtonProps extends ComponentProps<"button"> {
  variant?: "default" | "secondary" | "outline" | "ghost" | "danger";
  size?: "default" | "sm" | "lg" | "icon";
}

export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, ["class", "variant", "size"]);
  return (
    <button
      class={buttonVariants({ variant: local.variant, size: local.size, class: local.class })}
      {...rest}
    />
  );
}
