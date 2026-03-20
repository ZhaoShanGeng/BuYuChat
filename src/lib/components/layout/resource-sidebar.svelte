<script lang="ts">
  import Button from "$ui/button.svelte";
  import { cn } from "$lib/utils";
  import type { SidebarItem, WorkspaceId } from "$lib/state/app-shell.svelte";

  export let workspace: WorkspaceId;
  export let items: SidebarItem[] = [];
  export let activeId = "";
  export let onSelect: (id: string) => void;

  const labels: Record<WorkspaceId, { title: string; action: string }> = {
    chat: { title: "Conversations", action: "New" },
    agents: { title: "Agents", action: "Create" },
    presets: { title: "Presets", action: "Create" },
    lorebooks: { title: "Lorebooks", action: "Create" },
    workflows: { title: "Workflows", action: "Create" },
    settings: { title: "Settings", action: "Open" }
  };
</script>

<aside class="flex h-full flex-col gap-5 border-r border-[var(--border-soft)] bg-white/74 px-4 py-5 backdrop-blur-xl">
  <header class="space-y-3">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--brand)]">Workspace</p>
        <h2 class="mt-1 text-lg font-bold text-[var(--fg-primary)]">{labels[workspace].title}</h2>
      </div>
      <Button size="sm">{labels[workspace].action}</Button>
    </div>

    <label class="flex items-center gap-2 rounded-[1.2rem] border border-[var(--border-soft)] bg-[var(--bg-panel-strong)] px-3 py-2">
      <span class="text-sm text-[var(--fg-muted)]">Search</span>
      <input
        class="w-full bg-transparent text-sm text-[var(--fg-primary)] outline-none placeholder:text-[var(--fg-muted)]"
        placeholder={`Filter ${labels[workspace].title.toLowerCase()}`}
      />
    </label>
  </header>

  <div class="flex flex-1 flex-col gap-2 overflow-auto pr-1">
    {#each items as item}
      <button
        type="button"
        class={cn(
          "cursor-pointer rounded-[1.35rem] border px-3 py-3 text-left transition-all duration-200",
          item.id === activeId
            ? "border-[var(--brand)] bg-[var(--brand-soft)] shadow-[0_12px_24px_rgba(37,99,235,0.10)]"
            : "border-transparent bg-white/72 hover:border-[var(--border-strong)] hover:bg-white"
        )}
        on:click={() => onSelect(item.id)}
      >
        <div class="flex items-center justify-between gap-3">
          <h3 class="text-sm font-semibold text-[var(--fg-primary)]">{item.title}</h3>
          {#if item.id === activeId}
            <span class="h-2.5 w-2.5 rounded-full bg-[var(--brand)]"></span>
          {/if}
        </div>
        <p class="mt-1 text-xs text-[var(--fg-muted)]">{item.meta}</p>
      </button>
    {/each}
  </div>
</aside>
