<script lang="ts">
  import { cn } from "$lib/utils";

  type Props = {
    variant: "user" | "assistant";
    class?: string;
    children: any;
  };

  const { variant, class: cls, children }: Props = $props();
</script>

<div
  class={cn(
    "relative max-w-full md:max-w-[85%] px-4 py-3 shadow-sm",
    /* Notebook aesthetic radius: highly rounded on non-tail corners */
    variant === "user"
      ? "ml-auto bg-primary/10 rounded-2xl rounded-tr-sm text-foreground"
      : "mr-auto bg-card rounded-2xl rounded-tl-sm border text-foreground",
    cls
  )}
>
  {@render children()}
</div>
