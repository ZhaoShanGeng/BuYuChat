<script lang="ts">
  /**
   * 消息时间线 — 可滚动消息列表，自动滚到底部。
   */
  import SparklesIcon from "@lucide/svelte/icons/sparkles";
  import MessageCircleIcon from "@lucide/svelte/icons/message-circle";
  import WandSparklesIcon from "@lucide/svelte/icons/wand-sparkles";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import type { Conversation } from "../lib/transport/conversations";
  import type { MessageNode } from "../lib/transport/messages";
  import type { Notice } from "./workspace-state";
  import MessageCard from "./MessageCard.svelte";

  type Props = {
    conversation: Conversation | null;
    loading: boolean;
    messages: MessageNode[];
    notice: Notice | null;
    dryRunSummary: string | null;
    onCancel: (versionId: string) => void | Promise<void>;
    onReroll: (nodeId: string) => void | Promise<void>;
    onSwitchVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onDeleteVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onEditMessage: (nodeId: string, versionId: string, content: string) => void | Promise<void>;
  };

  const {
    conversation, loading, messages, notice, dryRunSummary,
    onCancel, onReroll, onSwitchVersion, onDeleteVersion, onEditMessage
  }: Props = $props();

  let scrollRef: HTMLDivElement | undefined = $state();

  /** 消息变化时自动滚到底部。 */
  $effect(() => {
    void messages.length;
    if (scrollRef) {
      requestAnimationFrame(() => { scrollRef!.scrollTop = scrollRef!.scrollHeight; });
    }
  });
</script>

<div class="relative flex min-h-0 flex-1 flex-col">
  <!-- 滚动区域 -->
  <div bind:this={scrollRef} class="min-h-0 flex-1 overflow-y-auto">
    <div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-8">
      <!-- Dry Run -->
      {#if dryRunSummary}
        <div class="rounded-lg border border-dashed bg-muted/20 px-4 py-3">
          <div class="mb-1 flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
            <WandSparklesIcon class="size-3.5" />
            Prompt 预览
          </div>
          <pre class="whitespace-pre-wrap text-xs leading-relaxed text-muted-foreground">{dryRunSummary}</pre>
        </div>
      {/if}

      {#if !conversation}
        <div class="flex min-h-[60vh] flex-col items-center justify-center">
          <MessageCircleIcon class="mb-4 size-10 text-muted-foreground/20" />
          <h3 class="text-base font-medium text-foreground/70">BuYu</h3>
          <p class="mt-1 text-sm text-muted-foreground">创建或选择一个会话开始对话</p>
        </div>
      {:else if loading && messages.length === 0}
        <div class="flex min-h-[40vh] items-center justify-center">
          <LoaderCircleIcon class="size-5 animate-spin text-muted-foreground/40" />
        </div>
      {:else if messages.length === 0}
        <div class="flex min-h-[50vh] flex-col items-center justify-center">
          <SparklesIcon class="mb-4 size-10 text-muted-foreground/20" />
          <h3 class="text-base font-medium text-foreground/70">有什么可以帮你的？</h3>
          <p class="mt-1 text-sm text-muted-foreground">在下方输入你的问题</p>
        </div>
      {:else}
        <div class="flex flex-col gap-5">
          {#each messages as node}
            <MessageCard
              isLastUserNode={node.role === "user" && messages.at(-1)?.id === node.id}
              {node}
              {onCancel}
              {onDeleteVersion}
              {onEditMessage}
              {onReroll}
              {onSwitchVersion}
            />
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- 顶部通知（非持久） -->
  {#if notice}
    <div class="absolute inset-x-0 top-0 z-10 flex justify-center px-4 pt-2">
      <div
        class={`inline-flex items-center gap-2 rounded-full px-4 py-1.5 text-xs font-medium shadow-sm ${
          notice.kind === "success"
            ? "bg-emerald-500/10 text-emerald-600"
            : notice.kind === "info"
              ? "bg-blue-500/10 text-blue-600"
              : "bg-rose-500/10 text-rose-600"
        }`}
      >
        <SparklesIcon class="size-3" />
        {notice.text}
      </div>
    </div>
  {/if}
</div>
