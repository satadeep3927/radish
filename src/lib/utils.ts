import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export type Variants = Record<string, Record<string, string>>;

interface CvaConfig {
  base?: string;
  variants?: Variants;
  defaultVariants?: Record<string, string>;
}

export function cva(config: CvaConfig) {
  return (props?: Record<string, string | undefined>) => {
    const resolved = { ...config.defaultVariants };
    if (props) {
      for (const key of Object.keys(props)) {
        if (props[key] !== undefined) {
          resolved[key] = props[key]!;
        }
      }
    }
    const variantClasses = config.variants
      ? Object.entries(config.variants).map(([variantKey, variantStyles]) => {
          const value = resolved[variantKey];
          return value && variantStyles[value] ? variantStyles[value] : "";
        })
      : [];
    const className = (props as Record<string, any> | undefined)?.["class"] || (props as Record<string, any> | undefined)?.["className"] || "";
    return cn(config.base, ...variantClasses, className);
  };
}
