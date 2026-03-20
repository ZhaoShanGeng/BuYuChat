<script lang="ts">
  import Badge from "$ui/badge.svelte";
  import Button from "$ui/button.svelte";
  import { cn } from "$lib/utils";
  import type { InspectorTabId } from "$lib/state/app-shell.svelte";

  export let tabs: { id: InspectorTabId; label: string }[] = [];
  export let activeTab: InspectorTabId;
  export let onSelectTab: (id: InspectorTabId) => void;
</script>

<aside class="flex h-full flex-col gap-4 border-l border-[var(--border-soft)] bg-white/76 px-4 py-5 backdrop-blur-xl">
  <div class="space-y-1">
    <p class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--brand)]">Inspector</p>
    <h2 class="text-lg font-bold text-[var(--fg-primary)]">Active context</h2>
  </div>

  <div class="flex flex-wrap gap-2">
    {#each tabs as tab}
      <button
        type="button"
        class={cn(
          "cursor-pointer rounded-full px-3 py-1.5 text-xs font-semibold transition-colors duration-200",
          tab.id === activeTab
            ? "bg-[var(--fg-primary)] text-white"
            : "bg-[var(--bg-soft)] text-[var(--fg-secondary)] hover:bg-white"
        )}
        on:click={() => onSelectTab(tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="space-y-3 rounded-[1.5rem] border border-[var(--border-soft)] bg-[var(--bg-panel-strong)] p-4">
    <div class="flex items-center justify-between gap-3">
      <h3 class="text-sm font-semibold text-[var(--fg-primary)]">Live signals</h3>
      <Badge className="bg-emerald-50 text-emerald-700">Stream ready</Badge>
    </div>
    <ul class="space-y-2 text-sm text-[var(--fg-secondary)]">
      <li>Incremental patch reducer will own all upsert / replace / delete reconciliation.</li>
      <li>Streaming replies append token deltas before final message version commit.</li>
      <li>Large content remains lazy-loaded and never blocks workspace rendering.</li>
    </ul>
  </div>

  <div class="grid gap-3">
    <div class="rounded-[1.35rem] border border-[var(--border-soft)] bg-white p-4">
      <p class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--fg-muted)]">Current tab</p>
      <h3 class="mt-2 text-base font-semibold text-[var(--fg-primary)]">{tabs.find((tab) => tab.id === activeTab)?.label}</h3>
      <p class="mt-2 text-sm leading-6 text-[var(--fg-secondary)]">
        This panel will host bindings, summaries, variables, workflow status and version controls without forcing a full reload.
      </p>
    </div>

    <div class="rounded-[1.35rem] border border-[var(--border-soft)] bg-white p-4">
      <div class="flex items-center justify-between gap-2">
        <h3 class="text-base font-semibold text-[var(--fg-primary)]">Quick actions</h3>
        <Badge className="bg-orange-50 text-orange-700">Desktop first</Badge>
      </div>
      <div class="mt-3 flex flex-wrap gap-2">
        <Button variant="secondary" size="sm">Open summaries</Button>
        <Button variant="secondary" size="sm">Inspect variables</Button>
        <Button variant="secondary" size="sm">Run workflow</Button>
      </div>
    </div>
  </div>
</aside>
