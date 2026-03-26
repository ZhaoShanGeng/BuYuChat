<script lang="ts">
  import { onDestroy } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import * as Collapsible from "$lib/components/ui/collapsible/index.js";
  import * as Avatar from "$lib/components/ui/avatar/index.js";
  import * as Textarea from "$lib/components/ui/textarea/index.js";
  import BotIcon from "@lucide/svelte/icons/bot";
  import BrainIcon from "@lucide/svelte/icons/brain";
  import CheckIcon from "@lucide/svelte/icons/check";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CircleUserIcon from "@lucide/svelte/icons/circle-user";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import SquareIcon from "@lucide/svelte/icons/square";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import { extractThinkingTags } from "../lib/thinking-tags";
  import RichTextContent from "./RichTextContent.svelte";
  import type { MessageNode } from "../lib/transport/messages";
  import { getActiveVersion, isNodeGenerating } from "./workspace-state";

  type Props = {
    node: MessageNode;
    thinkingTags: string[];
    onCancel: (versionId: string) => void | Promise<void>;
    onReroll: (nodeId: string) => void | Promise<void>;
    onSwitchVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onDeleteVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onEditMessage: (
      nodeId: string,
      content: string,
      options?: { resend?: boolean }
    ) => void | Promise<void>;
    onLoadVersionContent: (nodeId: string, versionId: string) => Promise<string>;
  };

  const {
    node,
    thinkingTags,
    onCancel,
    onReroll,
    onSwitchVersion,
    onDeleteVersion,
    onEditMessage,
    onLoadVersionContent
  }: Props =
    $props();

  let activeVersion = $derived.by(() => getActiveVersion(node));
  let activeIndex = $derived(node.versions.findIndex((version) => version.id === node.activeVersionId) + 1);
  let extractedThinking = $derived.by(() =>
    node.role === "assistant"
      ? extractThinkingTags(activeVersion?.content ?? "", thinkingTags)
      : { thinking: null, body: activeVersion?.content ?? "" }
  );
  let displayBodyContent = $derived(
    node.role === "assistant" ? extractedThinking.body : (activeVersion?.content ?? "")
  );
  let displayThinking = $derived.by(() => {
    const parts = [activeVersion?.thinkingContent, extractedThinking.thinking]
      .map((part) => part?.trim() ?? "")
      .filter((part) => part.length > 0);
    return parts.length > 0 ? parts.join("\n\n") : null;
  });
  let copied = $state(false);
  let generating = $derived(isNodeGenerating(node));
  let editing = $state(false);
  let loadingEditContent = $state(false);
  let saving = $state(false);
  let draft = $state("");
  let toolbarVisible = $state(false);
  let copyResetTimer: ReturnType<typeof setTimeout> | null = null;
  let showToolbar = $derived(toolbarVisible || editing);

  onDestroy(() => {
    if (copyResetTimer) {
      clearTimeout(copyResetTimer);
    }
  });

  function handlePointerEnter() {
    toolbarVisible = true;
  }

  function handlePointerLeave() {
    if (!editing) {
      toolbarVisible = false;
    }
  }

  async function handleCopy() {
    const text = displayBodyContent;
    if (!text) return;
    await navigator.clipboard.writeText(text);
    copied = true;
    if (copyResetTimer) {
      clearTimeout(copyResetTimer);
    }
    copyResetTimer = setTimeout(() => {
      copied = false;
      copyResetTimer = null;
    }, 1500);
  }

  async function startEdit() {
    if (!activeVersion) return;
    loadingEditContent = true;
    try {
      draft =
        activeVersion.content ??
        (await onLoadVersionContent(node.id, activeVersion.id));
      editing = true;
    } finally {
      loadingEditContent = false;
    }
  }

  function cancelEdit() {
    editing = false;
    draft = "";
  }

  async function saveEdit(resend = false) {
    if (!draft.trim()) {
      return;
    }

    saving = true;
    try {
      await onEditMessage(node.id, draft, { resend });
      editing = false;
      draft = "";
    } finally {
      saving = false;
    }
  }

  function handleDelete() {
    if (!activeVersion) return;
    void onDeleteVersion(node.id, activeVersion.id);
  }

  function prevVersion() {
    const index = activeIndex - 2;
    if (index >= 0) {
      void onSwitchVersion(node.id, node.versions[index].id);
    }
  }

  function nextVersion() {
    const index = activeIndex;
    if (index < node.versions.length) {
      void onSwitchVersion(node.id, node.versions[index].id);
    }
  }
</script>

{#if node.role === "user"}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex items-start justify-end gap-2.5 pl-20"
    onmouseenter={handlePointerEnter}
    onmouseleave={handlePointerLeave}
  >
    <div class="relative max-w-[80%]">
      <Card.Root class="overflow-hidden rounded-2xl rounded-br-sm border-0 bg-primary text-primary-foreground shadow-sm">
        <Card.Content class="space-y-3 px-4 py-3 text-sm leading-relaxed">
          {#if activeVersion?.images.length}
            <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
              {#each activeVersion.images as image (image.base64)}
                <img
                  alt="用户上传图片"
                  class="max-h-48 rounded-2xl object-cover"
                  src={`data:${image.mimeType};base64,${image.base64}`}
                />
              {/each}
            </div>
          {/if}

        {#if editing}
            <div class="space-y-3">
            <Textarea.Root
              bind:value={draft}
              class="min-h-[120px] resize-y border-primary-foreground/20 bg-primary/20 text-primary-foreground placeholder:text-primary-foreground/45"
              placeholder="编辑当前消息"
            />
            <div class="flex items-center justify-end gap-2">
              <Button disabled={saving} onclick={cancelEdit} size="sm" variant="secondary">取消</Button>
              <Button disabled={saving || !draft.trim()} onclick={() => void saveEdit(false)} size="sm" variant="secondary">
                保存
              </Button>
              <Button disabled={saving || !draft.trim()} onclick={() => void saveEdit(true)} size="sm">
                保存并重新发送
              </Button>
            </div>
            </div>
        {:else}
            <RichTextContent content={displayBodyContent} />
        {/if}
        </Card.Content>
      </Card.Root>

      <div class="relative mt-1 h-6">
        {#if showToolbar}
          <div class="absolute right-0 top-0 flex items-center gap-0.5">
            <Button class="size-6" onclick={handleCopy} size="icon" variant="ghost" title="复制">
              {#if copied}
                <CheckIcon class="size-3 text-emerald-500" />
              {:else}
                <CopyIcon class="size-3 text-muted-foreground" />
              {/if}
            </Button>
            <Button class="size-6" disabled={loadingEditContent || saving} onclick={() => void startEdit()} size="icon" variant="ghost" title="编辑">
              {#if loadingEditContent}
                <LoaderCircleIcon class="size-3 animate-spin text-muted-foreground" />
              {:else}
                <PencilIcon class="size-3 text-muted-foreground" />
              {/if}
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
        {/if}
      </div>
    </div>

    <Avatar.Root class="size-7 shrink-0 rounded-full">
      <Avatar.Fallback class="bg-primary/10">
        <CircleUserIcon class="size-4 text-primary" />
      </Avatar.Fallback>
    </Avatar.Root>
  </div>
{:else}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex items-start gap-2.5 pr-20"
    onmouseenter={handlePointerEnter}
    onmouseleave={handlePointerLeave}
  >
    <Avatar.Root class="size-7 shrink-0 rounded-full border">
      <Avatar.Fallback class="bg-muted">
        <BotIcon class="size-3.5 text-muted-foreground" />
      </Avatar.Fallback>
    </Avatar.Root>
    <div class="min-w-0 flex-1">
      {#if activeVersion?.status === "failed"}
        <div class="mb-1 text-xs text-destructive">生成失败</div>
      {:else if activeVersion?.status === "cancelled"}
        <div class="mb-1 text-xs text-muted-foreground">已取消</div>
      {/if}

      <div class="rounded-2xl rounded-tl-sm px-1 py-0.5 text-sm leading-relaxed">
        {#if editing}
          <Card.Root class="rounded-2xl shadow-sm">
            <Card.Content class="space-y-3 px-4 py-3">
              <Textarea.Root
                bind:value={draft}
                class="min-h-[120px] resize-y"
                placeholder="编辑当前消息"
              />
              <div class="flex items-center justify-end gap-2">
                <Button disabled={saving} onclick={cancelEdit} size="sm" variant="outline">取消</Button>
                <Button disabled={saving || !draft.trim()} onclick={() => void saveEdit(false)} size="sm" variant="outline">
                  保存
                </Button>
                <Button disabled={saving || !draft.trim()} onclick={() => void saveEdit(true)} size="sm">
                  保存并重新发送
                </Button>
              </div>
            </Card.Content>
          </Card.Root>
        {:else}
          <Card.Root class="rounded-[1.4rem] border-0 bg-transparent shadow-none">
            <Card.Content class="space-y-3 px-0 py-0">
              <RichTextContent content={displayBodyContent} throttleMs={generating ? 160 : 0} />
              {#if displayThinking}
                <Collapsible.Root class="rounded-2xl border bg-muted/35 px-3 py-2.5">
                  <Collapsible.Trigger class="flex w-full items-center gap-1.5 text-left text-xs text-muted-foreground">
                    <BrainIcon class="size-3.5" />
                    <span>思考过程</span>
                    <ChevronRightIcon class="size-3 transition-transform data-[state=open]:rotate-90" />
                  </Collapsible.Trigger>
                  <Collapsible.Content class="pt-2">
                    <RichTextContent
                      class="text-sm text-muted-foreground"
                      content={displayThinking}
                    />
                  </Collapsible.Content>
                </Collapsible.Root>
              {/if}
            </Card.Content>
          </Card.Root>
          {#if generating}
            <span class="ml-0.5 inline-block size-1.5 animate-pulse rounded-full bg-foreground/40"></span>
          {/if}
        {/if}
      </div>

      <div class="relative mt-2 h-6">
        {#if showToolbar}
          <div class="absolute left-0 top-0 flex min-w-0 items-center gap-0.5">
            <Button class="size-6" onclick={handleCopy} size="icon" variant="ghost" title="复制">
              {#if copied}
                <CheckIcon class="size-3 text-emerald-500" />
              {:else}
                <CopyIcon class="size-3 text-muted-foreground" />
              {/if}
            </Button>
            <Button class="size-6" disabled={loadingEditContent || saving} onclick={() => void startEdit()} size="icon" variant="ghost" title="编辑">
              {#if loadingEditContent}
                <LoaderCircleIcon class="size-3 animate-spin text-muted-foreground" />
              {:else}
                <PencilIcon class="size-3 text-muted-foreground" />
              {/if}
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
              <span class="ml-2 max-w-[16rem] truncate text-[10px] text-muted-foreground/60">
                {activeVersion.modelName ?? ""}{activeVersion.promptTokens !== null ? ` · ${(activeVersion.promptTokens ?? 0) + (activeVersion.completionTokens ?? 0)} tokens` : ""}
              </span>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
