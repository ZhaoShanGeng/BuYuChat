<script lang="ts">
  /**
   * 会话侧边栏 — 会话列表、新建、操作菜单。
   */
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import MessageSquarePlusIcon from "@lucide/svelte/icons/message-square-plus";
  import PinIcon from "@lucide/svelte/icons/pin";
  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import type { ConversationSummary } from "../lib/transport/conversations";
  import { formatRelativeTime } from "./workspace-state";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  type Props = {
    conversations: ConversationSummary[];
    activeConversationId: string | null;
    loading: boolean;
    onCreate: () => void | Promise<void>;
    onSelect: (id: string) => void | Promise<void>;
    onTogglePin: (conversation: ConversationSummary) => void | Promise<void>;
    onToggleArchive: (conversation: ConversationSummary) => void | Promise<void>;
    onDelete: (id: string) => void | Promise<void>;
    onOpenSettings: () => void;
  };

  const {
    conversations,
    activeConversationId,
    loading,
    onCreate,
    onSelect,
    onTogglePin,
    onToggleArchive,
    onDelete,
    onOpenSettings
  }: Props = $props();

  const sidebar = Sidebar.useSidebar();

  /** 置顶的会话。 */
  let pinnedConversations = $derived(conversations.filter((c) => c.pinned));

  /** 非置顶的会话。 */
  let unpinnedConversations = $derived(conversations.filter((c) => !c.pinned));
</script>

<Sidebar.Root class="top-(--header-height) h-[calc(100svh-var(--header-height))]!">
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton size="lg">
          {#snippet child({ props })}
            <a href="##" {...props}>
              <div class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg text-xs font-bold">
                步
              </div>
              <div class="grid flex-1 text-start text-sm leading-tight">
                <span class="truncate font-medium">BuYu</span>
                <span class="truncate text-xs text-muted-foreground">AI 对话工作台</span>
              </div>
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>

    <div class="px-2">
      <Button class="w-full justify-start" onclick={onCreate} variant="outline">
        <MessageSquarePlusIcon class="size-4" />
        新建会话
      </Button>
    </div>
  </Sidebar.Header>

  <Sidebar.Content>
    {#if loading}
      <Sidebar.Group>
        <Sidebar.Menu>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton aria-disabled="true">
              <span class="text-muted-foreground">正在加载...</span>
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        </Sidebar.Menu>
      </Sidebar.Group>
    {:else if conversations.length === 0}
      <Sidebar.Group>
        <Sidebar.Menu>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton aria-disabled="true">
              <span class="text-muted-foreground">还没有会话</span>
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        </Sidebar.Menu>
      </Sidebar.Group>
    {:else}
      <!-- 置顶会话 -->
      {#if pinnedConversations.length > 0}
        <Sidebar.Group>
          <Sidebar.GroupLabel>
            <PinIcon class="mr-1 size-3" />
            置顶
          </Sidebar.GroupLabel>
          <Sidebar.Menu>
            {#each pinnedConversations as conversation (conversation.id)}
              {@render conversationItem(conversation)}
            {/each}
          </Sidebar.Menu>
        </Sidebar.Group>
      {/if}

      <!-- 普通会话 -->
      <Sidebar.Group>
        {#if pinnedConversations.length > 0}
          <Sidebar.GroupLabel>会话</Sidebar.GroupLabel>
        {/if}
        <Sidebar.Menu>
          {#each unpinnedConversations as conversation (conversation.id)}
            {@render conversationItem(conversation)}
          {/each}
        </Sidebar.Menu>
      </Sidebar.Group>
    {/if}
  </Sidebar.Content>

  <Sidebar.Footer>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton onclick={onOpenSettings}>
          <Settings2Icon />
          <span>设置</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Footer>
</Sidebar.Root>

{#snippet conversationItem(conversation: ConversationSummary)}
  <Sidebar.MenuItem>
    <Sidebar.MenuButton
      isActive={conversation.id === activeConversationId}
      onclick={() => onSelect(conversation.id)}
      tooltipContent={conversation.title}
    >
      <div class="grid min-w-0 flex-1 text-left">
        <span class="truncate text-sm">{conversation.title}</span>
        <span class="truncate text-xs text-muted-foreground">
          {formatRelativeTime(conversation.updatedAt)}
        </span>
      </div>
    </Sidebar.MenuButton>

    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Sidebar.MenuAction showOnHover {...props}>
            <EllipsisIcon />
            <span class="sr-only">更多</span>
          </Sidebar.MenuAction>
        {/snippet}
      </DropdownMenu.Trigger>

      <DropdownMenu.Content
        align="start"
        class="w-40"
        side={sidebar.isMobile ? "bottom" : "right"}
      >
        <DropdownMenu.Item onclick={() => onTogglePin(conversation)}>
          <PinIcon class="text-muted-foreground" />
          <span>{conversation.pinned ? "取消置顶" : "置顶"}</span>
        </DropdownMenu.Item>
        <DropdownMenu.Item onclick={() => onToggleArchive(conversation)}>
          <ArchiveIcon class="text-muted-foreground" />
          <span>{conversation.archived ? "取消归档" : "归档"}</span>
        </DropdownMenu.Item>
        <DropdownMenu.Separator />
        <DropdownMenu.Item onclick={() => onDelete(conversation.id)}>
          <Trash2Icon class="text-muted-foreground" />
          <span>删除</span>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.MenuItem>
{/snippet}
