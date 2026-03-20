<script lang="ts">
  import { cn } from "$lib/utils";

  export let className = "";
</script>

<section
  class={cn(
    "rounded-[1.75rem] border border-slate-200/70 bg-white/90 shadow-[0_20px_60px_rgba(15,23,42,0.08)] backdrop-blur-xl",
    className
  )}
>
  <slot />
</section>
