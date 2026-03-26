<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import PenLineIcon from "@lucide/svelte/icons/pen-line";
  import PinIcon from "@lucide/svelte/icons/pin";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import type { ConversationSummary } from "../lib/transport/conversations";
  import { formatRelativeTime } from "./workspace-state";

  type Props = {
    conversation: ConversationSummary;
    isActive: boolean;
    isRenaming: boolean;
    renamingTitle: string;
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
</script>

<ContextMenu.Root>
  <ContextMenu.Trigger>
    <div class="conversation-sidebar__item group relative" data-active={props.isActive} data-ui="conversation-sidebar-item">
      <button
        class={`conversation-sidebar__item-button flex w-full items-center rounded-2xl px-3 py-2.5 text-left transition-colors ${
          props.isActive ? "bg-accent" : "hover:bg-accent/50"
        }`}
        onclick={() => void props.onSelect(props.conversation.id)}
        type="button"
      >
        <div class="conversation-sidebar__item-main min-w-0 flex-1">
          <div class="conversation-sidebar__item-title-row flex items-center gap-1.5">
            {#if props.conversation.pinned}
              <PinIcon class="size-3 shrink-0 text-muted-foreground" />
            {/if}

            {#if props.isRenaming}
              <Input
                class="conversation-sidebar__rename-input h-8 rounded-xl border-transparent bg-background shadow-none"
                onblur={() => void props.onCommitRename()}
                oninput={(event) =>
                  props.onRenameTitleChange((event.currentTarget as HTMLInputElement).value)}
                onkeydown={(event) => {
                  if (event.key === "Enter") void props.onCommitRename();
                  if (event.key === "Escape") props.onCancelRename();
                }}
                value={props.renamingTitle}
              />
            {:else}
              <span class="conversation-sidebar__item-title truncate text-[13px] font-medium">
                {props.conversation.title}
              </span>
            {/if}
          </div>

          <span class="conversation-sidebar__item-meta text-[11px] text-muted-foreground">
            {formatRelativeTime(props.conversation.updatedAt)}
          </span>
        </div>
      </button>

      <div class="conversation-sidebar__item-actions absolute right-1 top-1/2 -translate-y-1/2 opacity-0 transition-opacity group-hover:opacity-100">
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props: triggerProps })}
              <Button
                {...triggerProps}
                class="conversation-sidebar__menu-trigger size-7 rounded-xl"
                size="icon"
                variant="ghost"
              >
                <EllipsisIcon class="size-3.5" />
              </Button>
            {/snippet}
          </DropdownMenu.Trigger>

          <DropdownMenu.Content align="end" class="w-40">
            <DropdownMenu.Item onclick={() => void props.onStartRename(props.conversation)}>
              <PenLineIcon class="text-muted-foreground" />
              重命名
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => void props.onTogglePin(props.conversation)}>
              <PinIcon class="text-muted-foreground" />
              {props.conversation.pinned ? "取消置顶" : "置顶"}
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => void props.onToggleArchive(props.conversation)}>
              <ArchiveIcon class="text-muted-foreground" />
              {props.conversation.archived ? "取消归档" : "归档"}
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item onclick={() => void props.onDelete(props.conversation.id)} variant="destructive">
              <Trash2Icon />
              删除
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </div>
    </div>
  </ContextMenu.Trigger>

  <ContextMenu.Content class="w-40">
    <ContextMenu.Item onclick={() => void props.onStartRename(props.conversation)}>
      <PenLineIcon class="text-muted-foreground" />
      重命名
    </ContextMenu.Item>
    <ContextMenu.Item onclick={() => void props.onTogglePin(props.conversation)}>
      <PinIcon class="text-muted-foreground" />
      {props.conversation.pinned ? "取消置顶" : "置顶"}
    </ContextMenu.Item>
    <ContextMenu.Item onclick={() => void props.onToggleArchive(props.conversation)}>
      <ArchiveIcon class="text-muted-foreground" />
      {props.conversation.archived ? "取消归档" : "归档"}
    </ContextMenu.Item>
    <ContextMenu.Separator />
    <ContextMenu.Item onclick={() => void props.onDelete(props.conversation.id)} variant="destructive">
      <Trash2Icon />
      删除
    </ContextMenu.Item>
  </ContextMenu.Content>
</ContextMenu.Root>
