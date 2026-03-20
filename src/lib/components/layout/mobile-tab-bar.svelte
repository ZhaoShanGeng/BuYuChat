<script lang="ts">
  import { Bot, BookOpenText, Cable, MessagesSquare, SlidersHorizontal, Workflow } from "lucide-svelte";
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

<nav class="fixed inset-x-0 bottom-0 z-30 border-t border-[var(--border-soft)] bg-[color:rgba(255,255,255,0.92)] px-2 py-2 backdrop-blur-xl lg:hidden">
  <div class="grid grid-cols-6 gap-1">
    {#each items as item}
      {@const Icon = icons[item.id]}
      <button
        type="button"
        class={cn(
          "flex cursor-pointer flex-col items-center gap-1 rounded-[1rem] px-1 py-2 text-[10px] font-semibold transition-colors duration-200",
          item.id === active
            ? "bg-[var(--fg-primary)] text-white"
            : "text-[var(--fg-muted)] hover:bg-[var(--bg-soft)]"
        )}
        on:click={() => onSelect(item.id)}
      >
        <Icon size={16} />
        <span>{item.label}</span>
      </button>
    {/each}
  </div>
</nav>
