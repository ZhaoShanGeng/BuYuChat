<script lang="ts">
  import { Bot, Cable, BookOpenText, MessagesSquare, SlidersHorizontal, Workflow, Sun, Moon, Languages } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import type { NavItem, WorkspaceId } from "$lib/state/app-shell.svelte";
  import { i18n } from "$lib/i18n.svelte";
  import { theme } from "$lib/theme.svelte";

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

<aside class="hidden h-screen w-[var(--rail-width)] flex-col items-center bg-[var(--bg-rail)] py-3 lg:flex">
  <!-- Top: logo -->
  <div class="mb-6 flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-[var(--brand)] to-[#3b82f6] shadow-lg">
    <span class="text-sm font-bold text-white">步</span>
  </div>

  <!-- Navigation icons -->
  <nav class="flex flex-1 flex-col items-center gap-1">
    {#each items as item}
      {#if item.id === "settings"}
        <div class="flex-1"></div>
      {/if}
      {@const Icon = icons[item.id]}
      <button
        type="button"
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
        <span class="rail-tooltip">{i18n.t(labelKeys[item.id])}</span>
      </button>
    {/each}
  </nav>

  <!-- Bottom tools -->
  <div class="flex flex-col items-center gap-1 pt-2">
    <!-- Language toggle -->
    <button
      type="button"
      class="rail-btn icon-hover relative flex h-9 w-9 cursor-pointer items-center justify-center rounded-[var(--radius-md)] text-[var(--ink-on-dark-muted)] transition-all duration-150 hover:bg-white/8 hover:text-[var(--ink-on-dark)]"
      onclick={() => i18n.toggleLocale()}
    >
      <Languages size={17} />
      <span class="rail-tooltip">{i18n.locale === "zh-CN" ? "English" : "中文"}</span>
    </button>

    <!-- Theme toggle -->
    <button
      type="button"
      class="rail-btn icon-hover relative flex h-9 w-9 cursor-pointer items-center justify-center rounded-[var(--radius-md)] text-[var(--ink-on-dark-muted)] transition-all duration-150 hover:bg-white/8 hover:text-[var(--ink-on-dark)]"
      onclick={() => theme.toggle()}
    >
      {#if theme.isDark}
        <Sun size={17} />
      {:else}
        <Moon size={17} />
      {/if}
      <span class="rail-tooltip">{theme.isDark ? i18n.t("theme.light") : i18n.t("theme.dark")}</span>
    </button>
  </div>
</aside>
