<script lang="ts">
  import { tv, type VariantProps } from "tailwind-variants";
  import { cn } from "$lib/utils";

  const buttonVariants = tv({
    base: "inline-flex items-center justify-center rounded-full font-semibold transition-colors duration-200 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-900 disabled:pointer-events-none disabled:opacity-50",
    variants: {
      variant: {
        default: "bg-slate-900 text-white hover:bg-slate-800",
        secondary: "bg-slate-100 text-slate-900 hover:bg-slate-200",
        ghost: "bg-transparent text-slate-700 hover:bg-slate-100"
      },
      size: {
        sm: "h-9 px-3 text-sm",
        md: "h-11 px-4 text-sm",
        lg: "h-12 px-5 text-base"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "md"
    }
  });

  export let variant: VariantProps<typeof buttonVariants>["variant"] = "default";
  export let size: VariantProps<typeof buttonVariants>["size"] = "md";
  export let type: "button" | "submit" | "reset" = "button";
  export let className = "";
</script>

<button type={type} class={cn(buttonVariants({ variant, size }), className)}>
  <slot />
</button>
