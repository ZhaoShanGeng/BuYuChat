<script lang="ts">
  /**
   * 聊天输入框 — 大输入区 + 附件占位 + stop/send 按钮 + toast。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import ArrowUpIcon from "@lucide/svelte/icons/arrow-up";
  import SquareIcon from "@lucide/svelte/icons/square";
  import PaperclipIcon from "@lucide/svelte/icons/paperclip";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import type { Conversation } from "../lib/transport/conversations";

  type Props = {
    conversation: Conversation | null;
    sending: boolean;
    composer: string;
    generatingVersionId: string | null;
    onComposerChange: (value: string) => void;
    onDryRun: () => void | Promise<void>;
    onSend: () => void | Promise<void>;
    onCancel: (versionId: string) => void | Promise<void>;
  };

  const { conversation, sending, composer, generatingVersionId, onComposerChange, onDryRun, onSend, onCancel }: Props = $props();

  let canSend = $derived(!!conversation && !sending && composer.trim().length > 0);
  let isGenerating = $derived(!!generatingVersionId);

  /** 生成中 toast — 3 秒后自动消失。 */
  let toastVisible = $state(false);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (sending) {
      toastVisible = true;
      if (toastTimer) clearTimeout(toastTimer);
      toastTimer = setTimeout(() => { toastVisible = false; }, 3000);
    }
  });

  /** Enter 发送，Shift+Enter 换行。 */
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      if (canSend) void onSend();
    }
  }

  /** 自动高度。 */
  function handleInput(event: Event) {
    const el = event.currentTarget as HTMLTextAreaElement;
    onComposerChange(el.value);
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 240)}px`;
  }
</script>

<div class="mx-auto w-full max-w-3xl px-4 pb-6 pt-2">
  <!-- Toast -->
  {#if toastVisible && isGenerating}
    <div class="mb-2 flex items-center justify-center">
      <div class="inline-flex items-center gap-2 rounded-full bg-muted/80 px-3 py-1 text-xs text-muted-foreground">
        <LoaderCircleIcon class="size-3 animate-spin" />
        正在生成回复...
      </div>
    </div>
  {/if}

  <!-- 输入区域 -->
  <div class="flex flex-col overflow-hidden rounded-[1.5rem] border bg-background/60 shadow-sm backdrop-blur-xl transition-colors focus-within:border-primary/50 focus-within:ring-[3px] focus-within:ring-primary/10">
    <!-- 顶部工具栏 -->
    <div class="flex items-center gap-1 px-3 pt-2">
      <Button class="size-7" disabled size="icon" variant="ghost" title="附件（即将支持）">
        <PaperclipIcon class="size-4 text-muted-foreground/40" />
      </Button>
    </div>

    <!-- 输入框 -->
    <textarea
      aria-label="聊天输入框"
      class="max-h-[240px] min-h-[72px] flex-1 resize-none bg-transparent px-4 py-2 text-sm leading-relaxed outline-none placeholder:text-muted-foreground/50"
      disabled={!conversation || sending}
      oninput={handleInput}
      onkeydown={handleKeydown}
      placeholder={conversation ? "输入消息，Enter 发送，Shift+Enter 换行" : "请先选择会话"}
      rows={2}
      value={composer}
    ></textarea>

    <!-- 底部按钮栏 -->
    <div class="flex items-center justify-end gap-1.5 px-3 pb-2">
      {#if isGenerating}
        <Button
          class="size-8 rounded-lg"
          onclick={() => generatingVersionId && onCancel(generatingVersionId)}
          size="icon"
          variant="destructive"
          title="停止生成"
        >
          <SquareIcon class="size-4" />
        </Button>
      {/if}
      <Button
        class="size-8 rounded-lg"
        disabled={!canSend}
        onclick={() => void onSend()}
        size="icon"
        variant={canSend ? "default" : "secondary"}
        title="发送"
      >
        <ArrowUpIcon class="size-4" />
      </Button>
    </div>
  </div>
</div>
