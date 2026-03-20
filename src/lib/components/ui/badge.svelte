<script lang="ts">
  import { cn } from "$lib/utils";

  import type { Snippet } from "svelte";
  let { className = "", children }: { className?: string; children?: Snippet } = $props();
</script>

<span
  class={cn(
    "inline-flex items-center rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-2 py-0.5 text-[11px] font-medium text-[var(--ink-muted)]",
    className
  )}
>
  {#if children}{@render children()}{/if}
</span>
