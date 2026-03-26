<script lang="ts">
  import MessageSquarePlusIcon from "@lucide/svelte/icons/message-square-plus";
  import { Button } from "$lib/components/ui/button/index.js";
  import type { ConversationSummary } from "../lib/transport/conversations";
  import ConversationSidebarItem from "./ConversationSidebarItem.svelte";

  type Props = {
    bootstrapping: boolean;
    conversations: ConversationSummary[];
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
</script>

<section class="conversation-sidebar workspace-shell__context-panel flex h-full flex-col" data-ui="conversation-sidebar">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="workspace-shell__context-header flex h-12 items-center justify-between border-b px-4" onmousedown={props.onHeaderMouseDown}>
    <div class="min-w-0">
      <span class="text-sm font-semibold">对话</span>
    </div>
    <div class="min-w-4 flex-1"></div>
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

  <div class="conversation-sidebar__list min-h-0 flex-1 overflow-y-auto p-2">
    {#if props.bootstrapping}
      <div class="px-3 py-8 text-center text-xs text-muted-foreground">加载中...</div>
    {:else if props.conversations.length === 0}
      <div class="px-3 py-8 text-center text-xs text-muted-foreground">还没有会话</div>
    {:else}
      {#each props.conversations as conversation (conversation.id)}
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
    {/if}
  </div>
</section>
