<script lang="ts">
  import { Bot, Cable, BookOpenText, MessagesSquare, SlidersHorizontal, Workflow } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import type { NavItem, WorkspaceId } from "$lib/state/app-shell.svelte";
  import { i18n } from "$lib/i18n.svelte";
  import Tooltip from "$components/shared/tooltip.svelte";
  import BuYuLogo from "$components/shared/buyu-logo.svelte";

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

<aside class="relative z-50 hidden h-screen w-[var(--rail-width)] flex-col items-center overflow-visible bg-[var(--bg-rail)] py-3 lg:flex">
  <!-- Top: logo -->
  <div class="mb-6">
    <BuYuLogo size={40} rounded="lg" className="shadow-lg" />
  </div>

  <!-- Navigation icons -->
  <nav class="flex flex-1 flex-col items-center gap-1">
    {#each items as item}
      {#if item.id === "settings"}
        <div class="flex-1"></div>
      {/if}
      {@const Icon = icons[item.id]}
      <Tooltip text={i18n.t(labelKeys[item.id])}>
        {#snippet children()}
          <button
            type="button"
            aria-label={i18n.t(labelKeys[item.id])}
            class={cn(
              "rail-btn icon-hover relative flex h-10 w-10 cursor-pointer items-center justify-center rounded-[var(--radius-md)] transition-all duration-150",
              item.id === active
                ? "bg-white/15 text-white"
                : "text-[var(--ink-on-dark-muted)] hover:bg-white/8 hover:text-[var(--ink-on-dark)]"
            )}
            onclick={() => onSelect(item.id)}
          >
            {#if item.id === active}
              <span class="absolute left-0 top-2 bottom-2 w-[3px] rounded-r-full bg-[var(--brand)]"></span>
            {/if}
            <Icon size={20} />
          </button>
        {/snippet}
      </Tooltip>
    {/each}
  </nav>
</aside>
