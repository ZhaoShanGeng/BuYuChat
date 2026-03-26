<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import ArrowUpIcon from "@lucide/svelte/icons/arrow-up";
  import GripHorizontalIcon from "@lucide/svelte/icons/grip-horizontal";
  import PaperclipIcon from "@lucide/svelte/icons/paperclip";
  import SquareIcon from "@lucide/svelte/icons/square";
  import type { Conversation } from "../lib/transport/conversations";

  const STORAGE_KEY = "buyu:composer-height";
  const DEFAULT_HEIGHT = 188;
  const MIN_HEIGHT = 140;
  const MIN_VIEWPORT_HEIGHT = 112;
  const MAX_HEIGHT = 360;
  const RESERVED_VIEWPORT_HEIGHT = 240;

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

  const { conversation, sending, composer, generatingVersionId, onComposerChange, onSend, onCancel }: Props =
    $props();

  let composerHeight = $state(DEFAULT_HEIGHT);
  let canSend = $derived(!!conversation && !sending && composer.trim().length > 0);
  let isGenerating = $derived(!!generatingVersionId);

  function getHeightBounds() {
    const viewportMax =
      typeof window === "undefined"
        ? MAX_HEIGHT
        : Math.max(MIN_VIEWPORT_HEIGHT, window.innerHeight - RESERVED_VIEWPORT_HEIGHT);
    const maxHeight = Math.max(MIN_VIEWPORT_HEIGHT, Math.min(MAX_HEIGHT, viewportMax));
    const minHeight = Math.min(MIN_HEIGHT, maxHeight);

    return { minHeight, maxHeight };
  }

  function clampHeight(nextHeight: number) {
    const { minHeight, maxHeight } = getHeightBounds();
    return Math.max(minHeight, Math.min(maxHeight, nextHeight));
  }

  onMount(() => {
    const saved = window.localStorage.getItem(STORAGE_KEY);
    if (saved) {
      const numericHeight = Number(saved);
      if (Number.isFinite(numericHeight)) {
        composerHeight = clampHeight(numericHeight);
      }
    }

    const handleResize = () => {
      const nextHeight = clampHeight(composerHeight);
      if (nextHeight !== composerHeight) {
        composerHeight = nextHeight;
      }
    };

    handleResize();
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  });

  function persistHeight(nextHeight: number) {
    composerHeight = clampHeight(nextHeight);
    window.localStorage.setItem(STORAGE_KEY, String(composerHeight));
  }

  function handleResizeStart(event: PointerEvent) {
    const startY = event.clientY;
    const startHeight = composerHeight;
    const handle = event.currentTarget as HTMLElement;
    handle.setPointerCapture(event.pointerId);

    const onMove = (moveEvent: PointerEvent) => {
      const delta = startY - moveEvent.clientY;
      persistHeight(startHeight + delta);
    };

    const onEnd = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onEnd);
      window.removeEventListener("pointercancel", onEnd);
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onEnd);
    window.addEventListener("pointercancel", onEnd);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      if (canSend) {
        void onSend();
      }
    }
  }
</script>

<div class="mx-auto w-full max-w-[56rem] shrink-0 px-4 pb-4 pt-1">
  <div class="flex flex-col rounded-[1.75rem] border bg-background shadow-sm transition-shadow focus-within:shadow-md" style={`height:${composerHeight}px;`}>
    <button
      aria-label="调整输入框高度"
      class="flex h-7 items-center justify-center rounded-t-[1.75rem] text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
      onpointerdown={handleResizeStart}
      type="button"
    >
      <GripHorizontalIcon class="size-4" />
    </button>

    <div class="flex items-center gap-1 px-3">
      <Button class="size-7 rounded-xl" disabled size="icon" variant="ghost" title="附件（即将支持）">
        <PaperclipIcon class="size-4 text-muted-foreground/40" />
      </Button>
    </div>

    <Textarea
      aria-label="聊天输入框"
      class="min-h-0 flex-1 resize-none rounded-none border-0 bg-transparent px-4 py-3 text-sm leading-relaxed shadow-none ring-0 outline-none focus-visible:border-0 focus-visible:ring-0 placeholder:text-muted-foreground/50"
      disabled={!conversation || sending}
      oninput={(event: Event) => onComposerChange((event.currentTarget as HTMLTextAreaElement).value)}
      onkeydown={handleKeydown}
      placeholder={conversation ? "输入消息，Enter 发送，Shift+Enter 换行" : "请先选择会话"}
      value={composer}
    />

    <div class="flex items-center justify-end gap-1.5 px-3 pb-3">
      {#if isGenerating}
        <Button
          class="size-9 rounded-xl"
          onclick={() => generatingVersionId && onCancel(generatingVersionId)}
          size="icon"
          variant="destructive"
          title="停止生成"
        >
          <SquareIcon class="size-4" />
        </Button>
      {/if}
      <Button
        class="size-9 rounded-xl"
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
