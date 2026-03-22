<script lang="ts">
  import { X, Blocks, History, GitBranch, Variable, Link, Network } from "lucide-svelte";
  import Button from "$components/ui/button.svelte";
  import InspectorContext from "./inspector-context.svelte";
  import InspectorVersions from "./inspector-versions.svelte";
  import InspectorSummaries from "./inspector-summaries.svelte";
  import InspectorVariables from "./inspector-variables.svelte";
  import InspectorBindings from "./inspector-bindings.svelte";
  import InspectorWorkflow from "./inspector-workflow.svelte";
  import { cn } from "$lib/utils";

  let { conversationId = null, onClose }: { conversationId?: string | null; onClose: () => void } = $props();

  type TabId = "context" | "versions" | "summaries" | "variables" | "bindings" | "workflow";

  let activeTab = $state<TabId>("context");

  const tabs = [
    { id: "context", label: "上下文装配", icon: Blocks },
    { id: "versions", label: "版本浏览", icon: GitBranch },
    { id: "summaries", label: "摘要管理", icon: History },
    { id: "variables", label: "变量状态", icon: Variable },
    { id: "bindings", label: "绑定管理", icon: Link },
    { id: "workflow", label: "工作流视图", icon: Network }
  ] satisfies { id: TabId; label: string; icon: any }[];
</script>

<div class="flex h-full w-[320px] shrink-0 flex-col border-l border-[var(--border-soft)] bg-[var(--bg-app)]">
  <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-4 py-3">
    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">Inspector</h2>
    <Button variant="ghost" size="sm" className="h-7 w-7 px-0" onclick={onClose}>
      <X size={16} />
    </Button>
  </div>

  <div class="border-b border-[var(--border-soft)] p-3">
    <div class="grid grid-cols-2 gap-2">
      {#each tabs as t}
        <button
          class={cn(
            "flex items-center gap-2 rounded-[var(--radius-sm)] border px-2.5 py-2 text-left text-xs transition-colors",
            activeTab === t.id
              ? "border-[var(--brand)] bg-[var(--brand-soft)] text-[var(--brand)] shadow-[0_0_0_1px_var(--brand)]"
              : "border-[var(--border-soft)] bg-[var(--bg-surface)] text-[var(--ink-muted)] hover:border-[var(--border-medium)] hover:text-[var(--ink-body)]"
          )}
          onclick={() => (activeTab = t.id)}
        >
          <t.icon size={14} />
          <span class="font-medium">{t.label}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="app-scrollbar flex-1 overflow-y-auto p-4 bg-[var(--bg-surface)]">
    {#if activeTab === "context"}
      <InspectorContext {conversationId} />
    {:else if activeTab === "versions"}
      <InspectorVersions {conversationId} />
    {:else if activeTab === "summaries"}
      <InspectorSummaries {conversationId} />
    {:else if activeTab === "variables"}
      <InspectorVariables {conversationId} />
    {:else if activeTab === "bindings"}
      <InspectorBindings {conversationId} />
    {:else if activeTab === "workflow"}
      <InspectorWorkflow {conversationId} />
    {/if}
  </div>
</div>
