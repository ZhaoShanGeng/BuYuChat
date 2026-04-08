<script lang="ts">
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { Button } from "$lib/components/ui/button/index.js";
  import type { Agent } from "../../lib/transport/agents";
  import AgentSidebarItem from "./AgentSidebarItem.svelte";

  type Props = {
    agents: Agent[];
    editingId: string | null;
    onHeaderMouseDown: (event: MouseEvent) => void | Promise<void>;
    onCreate: () => void | Promise<void>;
    onSelect: (agent: Agent) => void | Promise<void>;
  };

  const props: Props = $props();
</script>

<section class="agent-sidebar workspace-shell__context-panel flex h-full flex-col" data-ui="agent-sidebar">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="workspace-shell__context-header flex h-12 items-center justify-between border-b px-4" onmousedown={props.onHeaderMouseDown}>
    <div class="min-w-0">
      <span class="text-sm font-semibold">Agent</span>
    </div>
    <div class="min-w-4 flex-1"></div>
    <Button
      class="agent-sidebar__create-button size-8 rounded-xl"
      onclick={() => void props.onCreate()}
      size="icon"
      title="新建 Agent"
      variant="ghost"
    >
      <PlusIcon class="size-4" />
    </Button>
  </div>

  <div class="agent-sidebar__list min-h-0 flex-1 overflow-y-auto p-2">
    {#if props.agents.length === 0}
      <div class="px-3 py-8 text-center text-xs text-muted-foreground">还没有 Agent</div>
    {:else}
      {#each props.agents as agent (agent.id)}
        <AgentSidebarItem
          agent={agent}
          isActive={props.editingId === agent.id}
          onSelect={props.onSelect}
        />
      {/each}
    {/if}
  </div>
</section>
