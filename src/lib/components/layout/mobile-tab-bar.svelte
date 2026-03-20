<script lang="ts">
  import { Bot, BookOpenText, Cable, MessagesSquare, SlidersHorizontal, Workflow } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import type { NavItem, WorkspaceId } from "$lib/state/app-shell.svelte";
  import { i18n } from "$lib/i18n.svelte";

  const icons = {
    chat: MessagesSquare,
    agents: Bot,
    presets: SlidersHorizontal,
    lorebooks: BookOpenText,
    workflows: Workflow,
    settings: Cable
  } satisfies Record<WorkspaceId, typeof MessagesSquare>;

  const labelKeys: Record<WorkspaceId, string> = {
    chat: "nav.chat",
    agents: "nav.agents",
    presets: "nav.presets",
    lorebooks: "nav.lorebooks",
    workflows: "nav.workflows",
    settings: "nav.settings"
  };

  let {
    items = [],
    active,
    onSelect
  }: {
    items?: NavItem[];
    active: WorkspaceId;
    onSelect: (id: WorkspaceId) => void;
  } = $props();
</script>

<nav class="fixed inset-x-0 bottom-0 z-30 border-t border-[var(--border-soft)] bg-[var(--bg-surface)] px-1 pb-[env(safe-area-inset-bottom)] lg:hidden">
  <div class="grid grid-cols-6">
    {#each items as item}
      {@const Icon = icons[item.id]}
      <button
        type="button"
        class={cn(
          "flex cursor-pointer flex-col items-center gap-0.5 py-2 text-[10px] font-medium transition-colors duration-100",
          item.id === active
            ? "text-[var(--brand)]"
            : "text-[var(--ink-faint)] hover:text-[var(--ink-muted)]"
        )}
        onclick={() => onSelect(item.id)}
      >
        <Icon size={18} />
        <span>{i18n.t(labelKeys[item.id])}</span>
      </button>
    {/each}
  </div>
</nav>
