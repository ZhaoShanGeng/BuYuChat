<script lang="ts">
  /**
   * 消息卡片 — 支持右键菜单、hover 操作栏、用户和助手头像。
   * 两种角色都支持：复制、编辑、重发、删除。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Avatar from "$lib/components/ui/avatar/index.js";
  import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
  import BotIcon from "@lucide/svelte/icons/bot";
  import CircleUserIcon from "@lucide/svelte/icons/circle-user";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import CheckIcon from "@lucide/svelte/icons/check";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import SquareIcon from "@lucide/svelte/icons/square";
  import RichTextContent from "./RichTextContent.svelte";
  import type { MessageNode } from "../lib/transport/messages";
  import { getActiveVersion, isNodeGenerating } from "./workspace-state";

  type Props = {
    node: MessageNode;
    isLastUserNode: boolean;
    onCancel: (versionId: string) => void | Promise<void>;
    onReroll: (nodeId: string) => void | Promise<void>;
    onSwitchVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onDeleteVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onEditMessage: (nodeId: string, versionId: string, content: string) => void | Promise<void>;
  };

  const { node, isLastUserNode, onCancel, onReroll, onSwitchVersion, onDeleteVersion, onEditMessage }: Props = $props();

  let activeVersion = $derived.by(() => getActiveVersion(node));
  let activeIndex = $derived(node.versions.findIndex((v) => v.id === node.activeVersionId) + 1);
  let copied = $state(false);
  let generating = $derived(isNodeGenerating(node));

  /** 复制到剪贴板。 */
  async function handleCopy() {
    const text = activeVersion?.content;
    if (!text) return;
    await navigator.clipboard.writeText(text);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }

  /** 编辑：复制内容到 composer + 删除版本。 */
  function handleEdit() {
    if (!activeVersion) return;
    void onEditMessage(node.id, activeVersion.id, activeVersion.content ?? "");
  }

  /** 删除当前版本。 */
  function handleDelete() {
    if (!activeVersion) return;
    void onDeleteVersion(node.id, activeVersion.id);
  }

  function prevVersion() {
    const idx = activeIndex - 2;
    if (idx >= 0) onSwitchVersion(node.id, node.versions[idx].id);
  }

  function nextVersion() {
    const idx = activeIndex;
    if (idx < node.versions.length) onSwitchVersion(node.id, node.versions[idx].id);
  }
</script>

<ContextMenu.Root>
  <ContextMenu.Trigger>
    {#if node.role === "user"}
      <!-- 用户消息 -->
      <div class="group flex items-start justify-end gap-3 pl-16">
        <div class="max-w-[85%]">
          <div class="rounded-3xl rounded-br-sm bg-primary px-5 py-3 text-[15px] leading-relaxed text-primary-foreground shadow-sm">
            <RichTextContent content={activeVersion?.content ?? ""} />
          </div>
          <!-- hover 操作栏 -->
          <div class="mt-1 flex items-center justify-end gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
            <Button class="size-6" onclick={handleCopy} size="icon" variant="ghost" title="复制">
              {#if copied}<CheckIcon class="size-3 text-emerald-500" />{:else}<CopyIcon class="size-3 text-muted-foreground" />{/if}
            </Button>
            <Button class="size-6" onclick={handleEdit} size="icon" variant="ghost" title="编辑">
              <PencilIcon class="size-3 text-muted-foreground" />
            </Button>
            <Button class="size-6" onclick={() => onReroll(node.id)} size="icon" variant="ghost" title="重发">
              <RotateCcwIcon class="size-3 text-muted-foreground" />
            </Button>
            <Button class="size-6" onclick={handleDelete} size="icon" variant="ghost" title="删除">
              <Trash2Icon class="size-3 text-muted-foreground" />
            </Button>
            {#if node.versions.length > 1}
              <div class="ml-0.5 flex items-center gap-0.5">
                <Button class="size-5" disabled={activeIndex <= 1} onclick={prevVersion} size="icon" variant="ghost">
                  <ChevronLeftIcon class="size-3" />
                </Button>
                <span class="min-w-6 text-center text-[11px] text-muted-foreground">{activeIndex}/{node.versions.length}</span>
                <Button class="size-5" disabled={activeIndex >= node.versions.length} onclick={nextVersion} size="icon" variant="ghost">
                  <ChevronRightIcon class="size-3" />
                </Button>
              </div>
            {/if}
          </div>
        </div>
        <!-- 用户头像 -->
        <Avatar.Root class="size-8 shrink-0 rounded-full shadow-sm ring-1 ring-border">
          <Avatar.Fallback class="bg-primary/5">
            <CircleUserIcon class="size-4.5 text-primary" />
          </Avatar.Fallback>
        </Avatar.Root>
      </div>
    {:else}
      <!-- 助手消息 -->
      <div class="group flex items-start gap-4 pr-16">
        <Avatar.Root class="mt-0.5 size-8 shrink-0 rounded-[0.6rem] border shadow-sm">
          <Avatar.Fallback class="bg-muted/50">
            <BotIcon class="size-4 text-primary" />
          </Avatar.Fallback>
        </Avatar.Root>
        <div class="min-w-0 flex-1">
          <!-- 状态 -->
          {#if activeVersion?.status === "failed"}
            <div class="mb-1 text-xs text-destructive">生成失败</div>
          {:else if activeVersion?.status === "cancelled"}
            <div class="mb-1 text-xs text-muted-foreground">已取消</div>
          {/if}
          <!-- 正文 -->
          <div class="text-[15px] leading-relaxed text-foreground/90">
            <RichTextContent content={activeVersion?.content ?? ""} />
            {#if generating}
              <span class="ml-0.5 inline-block size-1.5 animate-pulse rounded-full bg-foreground/40"></span>
            {/if}
          </div>
          <!-- hover 操作栏 -->
          <div class="mt-2 flex items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
            <Button class="size-6" onclick={handleCopy} size="icon" variant="ghost" title="复制">
              {#if copied}<CheckIcon class="size-3 text-emerald-500" />{:else}<CopyIcon class="size-3 text-muted-foreground" />{/if}
            </Button>
            <Button class="size-6" onclick={handleEdit} size="icon" variant="ghost" title="编辑（复制到输入框 + 重新生成）">
              <PencilIcon class="size-3 text-muted-foreground" />
            </Button>
            <Button class="size-6" onclick={() => onReroll(node.id)} size="icon" variant="ghost" title="重新生成">
              <RotateCcwIcon class="size-3 text-muted-foreground" />
            </Button>
            <Button class="size-6" onclick={handleDelete} size="icon" variant="ghost" title="删除">
              <Trash2Icon class="size-3 text-muted-foreground" />
            </Button>
            {#if generating && activeVersion}
              <Button class="size-6" onclick={() => onCancel(activeVersion.id)} size="icon" variant="ghost" title="停止">
                <SquareIcon class="size-3 text-muted-foreground" />
              </Button>
            {/if}
            {#if node.versions.length > 1}
              <div class="ml-1 flex items-center gap-0.5 border-l pl-1">
                <Button class="size-5" disabled={activeIndex <= 1} onclick={prevVersion} size="icon" variant="ghost">
                  <ChevronLeftIcon class="size-3" />
                </Button>
                <span class="min-w-6 text-center text-[11px] text-muted-foreground">{activeIndex}/{node.versions.length}</span>
                <Button class="size-5" disabled={activeIndex >= node.versions.length} onclick={nextVersion} size="icon" variant="ghost">
                  <ChevronRightIcon class="size-3" />
                </Button>
              </div>
            {/if}
            {#if activeVersion?.status === "committed" && (activeVersion.modelName || activeVersion.promptTokens !== null)}
              <span class="ml-2 text-[10px] text-muted-foreground/50">
                {activeVersion.modelName ?? ""}{activeVersion.promptTokens !== null ? ` · ${(activeVersion.promptTokens ?? 0) + (activeVersion.completionTokens ?? 0)} tokens` : ""}
              </span>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </ContextMenu.Trigger>

  <!-- 右键上下文菜单 -->
  <ContextMenu.Content class="w-40">
    <ContextMenu.Item onclick={handleCopy}>
      <CopyIcon class="text-muted-foreground" />
      复制
    </ContextMenu.Item>
    <ContextMenu.Item onclick={handleEdit}>
      <PencilIcon class="text-muted-foreground" />
      编辑
    </ContextMenu.Item>
    <ContextMenu.Item onclick={() => onReroll(node.id)}>
      <RotateCcwIcon class="text-muted-foreground" />
      重新生成
    </ContextMenu.Item>
    <ContextMenu.Separator />
    <ContextMenu.Item onclick={handleDelete}>
      <Trash2Icon class="text-muted-foreground" />
      删除
    </ContextMenu.Item>
  </ContextMenu.Content>
</ContextMenu.Root>
