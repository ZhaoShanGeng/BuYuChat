<script lang="ts">
  import { Bot, Cable, BookOpenText, MessagesSquare, SlidersHorizontal, Workflow } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import type { NavItem, WorkspaceId } from "$lib/state/app-shell.svelte";

  const icons = {
    chat: MessagesSquare,
    agents: Bot,
    presets: SlidersHorizontal,
    lorebooks: BookOpenText,
    workflows: Workflow,
    settings: Cable
  } satisfies Record<WorkspaceId, typeof MessagesSquare>;

  export let items: NavItem[] = [];
  export let active: WorkspaceId;
  export let onSelect: (id: WorkspaceId) => void;
</script>

<aside class="hidden min-h-screen w-[88px] flex-col gap-5 border-r border-[var(--border-soft)] bg-white/72 px-4 py-6 backdrop-blur-xl lg:flex">
  <div class="flex h-14 w-14 items-center justify-center rounded-[1.5rem] bg-[var(--fg-primary)] text-white shadow-[0_20px_40px_rgba(15,23,42,0.18)]">
    <span class="text-lg font-black tracking-[-0.08em]">步语</span>
  </div>

  <nav class="flex flex-1 flex-col gap-2">
    {#each items as item}
      {@const Icon = icons[item.id]}
      <button
        type="button"
        class={cn(
          "group flex cursor-pointer flex-col items-center gap-2 rounded-[1.4rem] px-2 py-3 text-[11px] font-semibold tracking-[0.04em] transition-all duration-200",
          item.id === active
            ? "bg-[var(--brand)] text-white shadow-[0_16px_32px_rgba(37,99,235,0.22)]"
            : "text-[var(--fg-muted)] hover:bg-white hover:text-[var(--fg-primary)]"
        )}
        on:click={() => onSelect(item.id)}
      >
        <Icon size={18} />
        <span>{item.label}</span>
      </button>
    {/each}
  </nav>
</aside>
