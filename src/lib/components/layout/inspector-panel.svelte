<script lang="ts">
  import { cn } from "$lib/utils";
  import { Layers, GitBranch, FileText, Variable, Link, Workflow } from "lucide-svelte";
  import type { InspectorTabId } from "$lib/state/app-shell.svelte";
  import { i18n } from "$lib/i18n.svelte";

  const tabIcons: Record<InspectorTabId, typeof Layers> = {
    context: Layers,
    versions: GitBranch,
    summaries: FileText,
    variables: Variable,
    bindings: Link,
    workflow: Workflow
  };

  const tabDescKeys: Record<InspectorTabId, string> = {
    context: "inspector.context_desc",
    versions: "inspector.versions_desc",
    summaries: "inspector.summaries_desc",
    variables: "inspector.variables_desc",
    bindings: "inspector.bindings_desc",
    workflow: "inspector.workflow_desc"
  };

  let {
    tabs = [],
    activeTab,
    onSelectTab
  }: {
    tabs?: { id: InspectorTabId; label: string }[];
    activeTab: InspectorTabId;
    onSelectTab: (id: InspectorTabId) => void;
  } = $props();

  const ActiveIcon = $derived(tabIcons[activeTab]);
</script>

<aside class="flex h-full flex-col border-l border-[var(--border-soft)] bg-[var(--bg-sidebar)]">
  <div class="border-b border-[var(--border-soft)] px-4 py-3 pr-[140px]" data-tauri-drag-region>
    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("inspector.title")}</h2>
  </div>

  <div class="flex flex-wrap gap-1 border-b border-[var(--border-soft)] px-3 py-2">
    {#each tabs as tab}
      {@const Icon = tabIcons[tab.id]}
      <button
        type="button"
        class={cn(
          "cursor-pointer inline-flex items-center gap-1 rounded-[var(--radius-full)] px-2.5 py-1 text-xs font-medium transition-colors duration-100",
          tab.id === activeTab
            ? "bg-[var(--ink-strong)] text-white"
            : "text-[var(--ink-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)]"
        )}
        onclick={() => onSelectTab(tab.id)}
      >
        {#if Icon}<Icon size={12} />{/if}
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="app-scrollbar flex-1 overflow-y-auto p-4">
    <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
      <div class="flex items-center gap-2">
        {#if ActiveIcon}
          <div class="flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] bg-[var(--brand-soft)]">
            <ActiveIcon size={16} class="text-[var(--brand)]" />
          </div>
        {/if}
        <h3 class="text-sm font-semibold text-[var(--ink-strong)]">{tabs.find(t => t.id === activeTab)?.label ?? ""}</h3>
      </div>
      <p class="mt-3 text-sm leading-relaxed text-[var(--ink-muted)]">{i18n.t(tabDescKeys[activeTab])}</p>
      <div class="mt-4 rounded-[var(--radius-sm)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-6 text-center text-xs text-[var(--ink-faint)]">
        {i18n.t("inspector.select_msg")}
      </div>
    </div>
  </div>
</aside>
