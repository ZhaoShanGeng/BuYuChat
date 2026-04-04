<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import MessageCircleIcon from "@lucide/svelte/icons/message-circle";
  import SparklesIcon from "@lucide/svelte/icons/sparkles";
  import WandSparklesIcon from "@lucide/svelte/icons/wand-sparkles";
  import type { Conversation } from "../lib/transport/conversations";
  import type { MessageNode } from "../lib/transport/messages";
  import type { Notice } from "./workspace-state";
  import MessageCard from "./MessageCard.svelte";
  import PaperTexture from "$lib/components/ui/paper-texture/paper-texture.svelte";
  import WelcomeScreen from "$lib/components/ui/welcome-screen/welcome-screen.svelte";

  const BOTTOM_STICK_THRESHOLD = 80;

  type Props = {
    conversation: Conversation | null;
    loading: boolean;
    loadingOlderMessages: boolean;
    hasOlderMessages: boolean;
    messages: MessageNode[];
    thinkingTags: string[];
    notice: Notice | null;
    dryRunSummary: string | null;
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
    onLoadOlderMessages: () => void | Promise<void>;
  };

  const {
    conversation,
    loading,
    loadingOlderMessages,
    hasOlderMessages,
    messages,
    thinkingTags,
    notice,
    dryRunSummary,
    onCancel,
    onReroll,
    onSwitchVersion,
    onDeleteVersion,
    onEditMessage,
    onLoadVersionContent,
    onLoadOlderMessages
  }: Props = $props();

  let scrollRef = $state<HTMLDivElement>();
  let shouldStickToBottom = $state(true);
  let pendingAnchor = $state<{ nodeId: string; offset: number } | null>(null);
  let previousConversationId = $state<string | null>(null);

  function captureAnchor(nodeId: string) {
    if (!scrollRef) {
      return;
    }

    const nodeElement = scrollRef.querySelector<HTMLElement>(`[data-node-id="${nodeId}"]`);
    if (!nodeElement) {
      return;
    }

    const containerTop = scrollRef.getBoundingClientRect().top;
    pendingAnchor = {
      nodeId,
      offset: nodeElement.getBoundingClientRect().top - containerTop
    };
  }

  function handleScroll() {
    if (!scrollRef) {
      return;
    }

    const remaining = scrollRef.scrollHeight - scrollRef.clientHeight - scrollRef.scrollTop;
    shouldStickToBottom = remaining < BOTTOM_STICK_THRESHOLD;
  }

  async function runAnchored<T>(nodeId: string, task: () => Promise<T>) {
    captureAnchor(nodeId);
    return task();
  }

  async function handleLoadOlderMessages() {
    if (loadingOlderMessages || !hasOlderMessages || messages.length === 0) {
      return;
    }

    captureAnchor(messages[0].id);
    await onLoadOlderMessages();
  }

  $effect(() => {
    const conversationId = conversation?.id ?? null;
    if (conversationId === previousConversationId) {
      return;
    }

    previousConversationId = conversationId;
    shouldStickToBottom = true;
    pendingAnchor = null;

    requestAnimationFrame(() => {
      if (!scrollRef) {
        return;
      }

      scrollRef.scrollTop = scrollRef.scrollHeight;
    });
  });

  $effect(() => {
    void messages.length;
    void loadingOlderMessages;

    requestAnimationFrame(() => {
      if (!scrollRef) {
        return;
      }

      if (pendingAnchor) {
        const nodeElement = scrollRef.querySelector<HTMLElement>(
          `[data-node-id="${pendingAnchor.nodeId}"]`
        );
        if (nodeElement) {
          const containerTop = scrollRef.getBoundingClientRect().top;
          const nextOffset = nodeElement.getBoundingClientRect().top - containerTop;
          scrollRef.scrollTop += nextOffset - pendingAnchor.offset;
        }
        pendingAnchor = null;
        return;
      }

      if (shouldStickToBottom) {
        scrollRef.scrollTop = scrollRef.scrollHeight;
      }
    });
  });
</script>

<div class="relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
  <PaperTexture opacity={0.03} />
  <div
    bind:this={scrollRef}
    class="message-timeline min-h-0 min-w-0 flex-1 overflow-y-auto overscroll-y-contain overflow-x-hidden relative z-10"
    onscroll={handleScroll}
  >
    <div class="message-timeline__inner mx-auto flex w-full min-w-0 max-w-[56rem] flex-col gap-3 px-3 py-3 sm:gap-4 sm:px-5 sm:py-5">
      {#if dryRunSummary}
        <div class="rounded-2xl border border-dashed bg-muted/20 px-4 py-3">
          <div class="mb-1 flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
            <WandSparklesIcon class="size-3.5" />
            Prompt 预览
          </div>
          <pre class="whitespace-pre-wrap text-xs leading-relaxed text-muted-foreground">{dryRunSummary}</pre>
        </div>
      {/if}

      {#if !conversation}
        <WelcomeScreen class="min-h-[60vh] -mx-4" />
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
        <div class="flex min-w-0 flex-col gap-4">
          {#if hasOlderMessages || loadingOlderMessages}
            <div class="flex justify-center">
              <Button
                disabled={loadingOlderMessages}
                onclick={() => void handleLoadOlderMessages()}
                size="sm"
                variant="secondary"
              >
                {#if loadingOlderMessages}
                  <LoaderCircleIcon class="mr-2 size-3.5 animate-spin" />
                  正在加载更早消息
                {:else}
                  加载更早消息
                {/if}
              </Button>
            </div>
          {/if}

          {#each messages as node, index (node.id)}
            <div data-node-id={node.id}>
              <MessageCard
                isLast={index === messages.length - 1}
                {node}
                {thinkingTags}
                onCancel={onCancel}
                onDeleteVersion={(nodeId, versionId) =>
                  runAnchored(nodeId, () => Promise.resolve(onDeleteVersion(nodeId, versionId)))
                }
                onEditMessage={(nodeId, content, options) =>
                  runAnchored(nodeId, () => Promise.resolve(onEditMessage(nodeId, content, options)))
                }
                onLoadVersionContent={onLoadVersionContent}
                onReroll={(nodeId) => runAnchored(nodeId, () => Promise.resolve(onReroll(nodeId)))}
                onSwitchVersion={(nodeId, versionId) =>
                  runAnchored(nodeId, () => Promise.resolve(onSwitchVersion(nodeId, versionId)))
                }
              />
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  {#if notice}
    <div class="pointer-events-none absolute inset-x-0 top-0 z-10 flex justify-center px-4 pt-2">
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
