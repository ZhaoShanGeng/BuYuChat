<script lang="ts">
  import { tv, type VariantProps } from "tailwind-variants";
  import { cn } from "$lib/utils";
  import type { HTMLButtonAttributes } from "svelte/elements";

  const buttonVariants = tv({
    base: "inline-flex items-center justify-center font-medium transition-colors duration-150 outline-none focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)] disabled:pointer-events-none disabled:opacity-50",
    variants: {
      variant: {
        default:
          "rounded-[var(--radius-md)] bg-[var(--brand)] text-white shadow-[var(--shadow-sm)] hover:bg-[var(--brand-strong)]",
        secondary:
          "rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] text-[var(--ink-strong)] hover:bg-[var(--bg-hover)]",
        ghost:
          "rounded-[var(--radius-md)] bg-transparent text-[var(--ink-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)]",
        destructive:
          "rounded-[var(--radius-md)] bg-[var(--danger)] text-white hover:bg-red-700"
      },
      size: {
        sm: "h-8 gap-1.5 px-2.5 text-xs",
        md: "h-9 gap-2 px-3 text-sm",
        lg: "h-10 gap-2 px-4 text-sm"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "md"
    }
  });

  import type { Snippet } from "svelte";
  let {
    variant = "default",
    size = "md",
    type = "button",
    className = "",
    children,
    ...rest
  }: {
    variant?: VariantProps<typeof buttonVariants>["variant"];
    size?: VariantProps<typeof buttonVariants>["size"];
    type?: "button" | "submit" | "reset";
    className?: string;
    children?: Snippet;
  } & HTMLButtonAttributes = $props();
</script>

<button {...rest} type={type} class={cn(buttonVariants({ variant, size }), className)}>
  {#if children}{@render children()}{/if}
</button>
