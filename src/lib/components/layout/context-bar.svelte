<script lang="ts">
  import Button from "$ui/button.svelte";
  import Badge from "$ui/badge.svelte";
  import { PanelLeft, PanelRight, Search, Sparkles } from "lucide-svelte";

  export let title = "Chat";
  export let subtitle = "Workspace";
  export let meta: string[] = [];
  export let onToggleSidebar: () => void;
  export let onToggleInspector: () => void;
</script>

<header class="sticky top-0 z-20 border-b border-[var(--border-soft)] bg-[color:rgba(248,250,252,0.88)] px-4 py-3 backdrop-blur-xl md:px-6">
  <div class="flex items-start justify-between gap-4">
    <div class="flex min-w-0 items-center gap-3">
      <button
        type="button"
        class="inline-flex h-10 w-10 cursor-pointer items-center justify-center rounded-[1rem] border border-[var(--border-soft)] bg-white text-[var(--fg-primary)] shadow-[0_8px_20px_rgba(15,23,42,0.06)] lg:hidden"
        on:click={onToggleSidebar}
      >
        <PanelLeft size={18} />
      </button>

      <div class="min-w-0">
        <p class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--brand)]">{subtitle}</p>
        <div class="mt-1 flex min-w-0 items-center gap-2">
          <h1 class="truncate text-xl font-black tracking-[-0.04em] text-[var(--fg-primary)] md:text-2xl">
            {title}
          </h1>
          <Badge className="hidden bg-[var(--brand-soft)] text-[var(--brand)] md:inline-flex">
            Incremental
          </Badge>
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <Button variant="secondary" size="sm" className="hidden md:inline-flex">
        <Search size={16} />
        <span class="ml-2">Search</span>
      </Button>
      <Button size="sm" className="hidden md:inline-flex">
        <Sparkles size={16} />
        <span class="ml-2">Command</span>
      </Button>
      <button
        type="button"
        class="inline-flex h-10 w-10 cursor-pointer items-center justify-center rounded-[1rem] border border-[var(--border-soft)] bg-white text-[var(--fg-primary)] shadow-[0_8px_20px_rgba(15,23,42,0.06)] xl:hidden"
        on:click={onToggleInspector}
      >
        <PanelRight size={18} />
      </button>
    </div>
  </div>

  {#if meta.length > 0}
    <div class="mt-3 flex flex-wrap gap-2">
      {#each meta as item}
        <Badge className="bg-white text-[var(--fg-secondary)]">{item}</Badge>
      {/each}
    </div>
  {/if}
</header>
