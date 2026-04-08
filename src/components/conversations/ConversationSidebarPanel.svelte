<script lang="ts">
  import MessageSquarePlusIcon from "@lucide/svelte/icons/message-square-plus";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import SearchIcon from "@lucide/svelte/icons/search";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import type { ConversationSummary } from "../../lib/transport/conversations";
  import type { Agent } from "../../lib/transport/agents";
  import ConversationSidebarItem from "./ConversationSidebarItem.svelte";

  type Props = {
    bootstrapping: boolean;
    conversations: ConversationSummary[];
    agents: Agent[];
    activeConversationId: string | null;
    pendingConversationId: string | null;
    renamingConversationId: string | null;
    renamingConversationTitle: string;
    onHeaderMouseDown: (event: MouseEvent) => void | Promise<void>;
    onCreate: () => void | Promise<void>;
    onSelect: (id: string) => void | Promise<void>;
    onRenameTitleChange: (value: string) => void;
    onCommitRename: () => void | Promise<void>;
    onCancelRename: () => void;
    onStartRename: (conversation: ConversationSummary) => void | Promise<void>;
    onTogglePin: (conversation: ConversationSummary) => void | Promise<void>;
    onToggleArchive: (conversation: ConversationSummary) => void | Promise<void>;
    onDelete: (id: string) => void | Promise<void>;
  };

  const props: Props = $props();
  let highlightedConversationId = $derived(
    props.pendingConversationId ?? props.activeConversationId
  );
  let searchQuery = $state("");
  let collapsedGroups = $state<Set<string>>(new Set());

  /** 按 Agent 分组，每组内按 updatedAt 倒序 */
  type AgentGroup = {
    agentId: string | null;
    agentName: string;
    conversations: ConversationSummary[];
  };

  let agentGroups = $derived.by((): AgentGroup[] => {
    const filtered = searchQuery.trim()
      ? props.conversations.filter((c) =>
          c.title.toLowerCase().includes(searchQuery.trim().toLowerCase())
        )
      : props.conversations;

    const groupMap = new Map<string, AgentGroup>();

    for (const conv of filtered) {
      const key = conv.agentId ?? "__none__";
      if (!groupMap.has(key)) {
        const agent = conv.agentId
          ? props.agents.find((a) => a.id === conv.agentId)
          : null;
        groupMap.set(key, {
          agentId: conv.agentId ?? null,
          agentName: agent?.name ?? "未绑定 Agent",
          conversations: []
        });
      }
      groupMap.get(key)!.conversations.push(conv);
    }

    // Sort groups: "未绑定" last, others alphabetically
    const groups = [...groupMap.values()].sort((a, b) => {
      if (a.agentId === null) return 1;
      if (b.agentId === null) return -1;
      return a.agentName.localeCompare(b.agentName);
    });

    return groups;
  });

  function toggleGroup(agentId: string | null) {
    const key = agentId ?? "__none__";
    const next = new Set(collapsedGroups);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    collapsedGroups = next;
  }

  function isGroupCollapsed(agentId: string | null) {
    return collapsedGroups.has(agentId ?? "__none__");
  }
</script>

<section class="conversation-sidebar workspace-shell__context-panel flex h-full flex-col" data-ui="conversation-sidebar">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="workspace-shell__context-header flex h-12 items-center justify-between border-b px-3" onmousedown={props.onHeaderMouseDown}>
    <span class="text-sm font-semibold" style="font-family: var(--buyu-font-heading)">对话</span>
    <Button
      class="conversation-sidebar__create-button size-8 rounded-xl"
      onclick={() => void props.onCreate()}
      size="icon"
      title="新建会话"
      variant="ghost"
    >
      <MessageSquarePlusIcon class="size-4" />
    </Button>
  </div>

  <!-- 搜索框 -->
  <div class="px-3 py-2">
    <div class="relative">
      <SearchIcon class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
      <Input
        bind:value={searchQuery}
        class="h-8 rounded-lg bg-muted/50 pl-8 text-xs placeholder:text-muted-foreground/50 border-transparent"
        placeholder="搜索会话..."
      />
    </div>
  </div>

  <div class="conversation-sidebar__list min-h-0 flex-1 overflow-y-auto px-2 pb-2">
    {#if props.bootstrapping}
      <div class="px-3 py-8 text-center text-xs text-muted-foreground">加载中...</div>
    {:else if agentGroups.length === 0}
      <div class="px-3 py-8 text-center text-xs text-muted-foreground">
        {searchQuery.trim() ? "没有匹配的会话" : "还没有会话"}
      </div>
    {:else}
      {#each agentGroups as group (group.agentId ?? "__none__")}
        <!-- Agent 分组标题 -->
        <button
          class="flex w-full items-center gap-1.5 rounded-lg px-2 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground transition-colors hover:text-foreground mt-1"
          onclick={() => toggleGroup(group.agentId)}
          type="button"
        >
          <ChevronRightIcon class="size-3 transition-transform duration-200 {isGroupCollapsed(group.agentId) ? '' : 'rotate-90'}" />
          <span class="flex-1 text-left truncate">{group.agentName}</span>
          <span class="text-[10px] font-normal text-muted-foreground/60">{group.conversations.length}</span>
        </button>

        {#if !isGroupCollapsed(group.agentId)}
          <div class="ml-1">
            {#each group.conversations as conversation (conversation.id)}
              <ConversationSidebarItem
                conversation={conversation}
                isActive={conversation.id === highlightedConversationId}
                isRenaming={props.renamingConversationId === conversation.id}
                onCancelRename={props.onCancelRename}
                onCommitRename={props.onCommitRename}
                onDelete={props.onDelete}
                onRenameTitleChange={props.onRenameTitleChange}
                onSelect={props.onSelect}
                onStartRename={props.onStartRename}
                onToggleArchive={props.onToggleArchive}
                onTogglePin={props.onTogglePin}
                renamingTitle={props.renamingConversationTitle}
              />
            {/each}
          </div>
        {/if}
      {/each}
    {/if}
  </div>
</section>
