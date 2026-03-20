<script lang="ts">
  import { cn } from "$lib/utils";

  export let className = "";
</script>

<span
  class={cn(
    "inline-flex items-center rounded-full bg-blue-50 px-3 py-1 text-sm font-semibold text-blue-700",
    className
  )}
>
  <slot />
</span>
