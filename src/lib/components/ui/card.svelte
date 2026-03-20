<script lang="ts">
  import { cn } from "$lib/utils";

  import type { Snippet } from "svelte";
  let { className = "", children }: { className?: string; children?: Snippet } = $props();
</script>

<section
  class={cn(
    "rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-sm)]",
    className
  )}
>
  {#if children}{@render children()}{/if}
</section>
