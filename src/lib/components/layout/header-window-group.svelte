<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children?: Snippet } = $props();
</script>

<div class="flex shrink-0 items-center gap-1" data-no-drag>
  {#if children}
    {@render children()}
  {/if}
</div>
