<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import ArrowUpIcon from "@lucide/svelte/icons/arrow-up";
  import GripHorizontalIcon from "@lucide/svelte/icons/grip-horizontal";
  import ImageIcon from "@lucide/svelte/icons/image";
  import PaperclipIcon from "@lucide/svelte/icons/paperclip";
  import SquareIcon from "@lucide/svelte/icons/square";
  import XIcon from "@lucide/svelte/icons/x";
  import type { Conversation } from "../lib/transport/conversations";
  import type { PendingComposerImage } from "./workspace-shell.svelte.js";

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
    pendingImages: PendingComposerImage[];
    generatingVersionId: string | null;
    onComposerChange: (value: string) => void;
    onPendingImagesChange: (images: PendingComposerImage[]) => void;
    onDryRun: () => void | Promise<void>;
    onSend: () => void | Promise<void>;
    onCancel: (versionId: string) => void | Promise<void>;
  };

  const {
    conversation,
    sending,
    composer,
    pendingImages,
    generatingVersionId,
    onComposerChange,
    onPendingImagesChange,
    onSend,
    onCancel
  }: Props =
    $props();

  let composerHeight = $state(DEFAULT_HEIGHT);
  let fileInput = $state<HTMLInputElement | null>(null);
  let canSend = $derived(
    !!conversation && !sending && (composer.trim().length > 0 || pendingImages.length > 0)
  );
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

  function readFileAsBase64(file: File): Promise<PendingComposerImage | null> {
    if (!file.type.startsWith("image/")) {
      return Promise.resolve(null);
    }

    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(reader.error);
      reader.onload = () => {
        const result = typeof reader.result === "string" ? reader.result : "";
        const base64 = result.split(",")[1];
        if (!base64) {
          resolve(null);
          return;
        }

        resolve({
          name: file.name || "image",
          base64,
          mimeType: file.type || "image/png"
        });
      };
      reader.readAsDataURL(file);
    });
  }

  async function appendFiles(files: File[]) {
    const nextImages = (await Promise.all(files.map(readFileAsBase64))).filter(
      (image): image is PendingComposerImage => image !== null
    );
    if (nextImages.length === 0) {
      return;
    }

    onPendingImagesChange([
      ...pendingImages,
      ...nextImages.filter(
        (nextImage) =>
          !pendingImages.some(
            (existing) =>
              existing.base64 === nextImage.base64 && existing.mimeType === nextImage.mimeType
          )
      )
    ]);
  }

  async function handleFileSelection(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const files = Array.from(input.files ?? []);
    await appendFiles(files);
    input.value = "";
  }

  async function handlePaste(event: ClipboardEvent) {
    const files = Array.from(event.clipboardData?.files ?? []).filter((file) =>
      file.type.startsWith("image/")
    );
    if (files.length === 0) {
      return;
    }

    event.preventDefault();
    await appendFiles(files);
  }

  function openFilePicker() {
    fileInput?.click();
  }

  function removePendingImage(index: number) {
    onPendingImagesChange(pendingImages.filter((_, currentIndex) => currentIndex !== index));
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

    <input
      bind:this={fileInput}
      accept="image/png,image/jpeg,image/jpg,image/gif,image/webp"
      class="hidden"
      multiple
      onchange={handleFileSelection}
      type="file"
    />

    <div class="flex items-center gap-1 px-3">
      <Button
        class="size-7 rounded-xl"
        disabled={!conversation || sending}
        onclick={openFilePicker}
        size="icon"
        variant="ghost"
        title="上传图片"
      >
        <PaperclipIcon class="size-4 text-muted-foreground" />
      </Button>
    </div>

    {#if pendingImages.length > 0}
      <div class="flex gap-2 overflow-x-auto px-4 pb-2 pt-1">
        {#each pendingImages as image, index (image.base64)}
          <div class="group relative shrink-0">
            <img
              alt={image.name}
              class="size-14 rounded-2xl border object-cover"
              src={`data:${image.mimeType};base64,${image.base64}`}
            />
            <button
              class="absolute -right-1.5 -top-1.5 flex size-5 items-center justify-center rounded-full border bg-background text-muted-foreground shadow-sm transition-colors hover:text-foreground"
              onclick={() => removePendingImage(index)}
              type="button"
            >
              <XIcon class="size-3" />
            </button>
            <div class="mt-1 flex items-center gap-1 text-[11px] text-muted-foreground">
              <ImageIcon class="size-3" />
              <span class="max-w-16 truncate">{image.name}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <Textarea
      aria-label="聊天输入框"
      class="min-h-0 flex-1 resize-none rounded-none border-0 bg-transparent px-4 py-3 text-sm leading-relaxed shadow-none ring-0 outline-none focus-visible:border-0 focus-visible:ring-0 placeholder:text-muted-foreground/50"
      disabled={!conversation || sending}
      oninput={(event: Event) => onComposerChange((event.currentTarget as HTMLTextAreaElement).value)}
      onkeydown={handleKeydown}
      onpaste={handlePaste}
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
