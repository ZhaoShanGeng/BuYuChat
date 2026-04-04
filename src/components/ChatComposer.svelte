<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import ArrowUpIcon from "@lucide/svelte/icons/arrow-up";
  import GlobeIcon from "@lucide/svelte/icons/globe";
  import ImageIcon from "@lucide/svelte/icons/image";
  import PaperclipIcon from "@lucide/svelte/icons/paperclip";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import SquareIcon from "@lucide/svelte/icons/square";
  import WrenchIcon from "@lucide/svelte/icons/wrench";
  import XIcon from "@lucide/svelte/icons/x";
  import {
    mergeComposerFiles,
    mergeComposerImages,
    pickComposerAttachments,
    readComposerAttachmentsFromFiles
  } from "../lib/composer-attachments";
  import { isTauriWindowAvailable } from "../lib/tauri-window";
  import type { Conversation } from "../lib/transport/conversations";
  import type { PendingComposerFile, PendingComposerImage } from "./workspace-shell.svelte.js";

  const MIN_HEIGHT = 52;
  const MAX_HEIGHT = 280;

  type Props = {
    conversation: Conversation | null;
    sending: boolean;
    composer: string;
    pendingImages: PendingComposerImage[];
    pendingFiles: PendingComposerFile[];
    generatingVersionId: string | null;
    enabledTools: string[];
    onComposerChange: (value: string) => void;
    onPendingImagesChange: (images: PendingComposerImage[]) => void;
    onPendingFilesChange: (files: PendingComposerFile[]) => void;
    onEnabledToolsChange: (tools: string[]) => void;
    onDryRun: () => void | Promise<void>;
    onSend: () => void | Promise<void>;
    onCancel: (versionId: string) => void | Promise<void>;
  };

  const {
    conversation,
    sending,
    composer,
    pendingImages,
    pendingFiles,
    generatingVersionId,
    enabledTools,
    onComposerChange,
    onPendingImagesChange,
    onPendingFilesChange,
    onEnabledToolsChange,
    onSend,
    onCancel
  }: Props =
    $props();

  let textareaRef = $state<HTMLTextAreaElement>();
  let fileInput = $state<HTMLInputElement | null>(null);
  let toolPopoverOpen = $state(false);
  let canSend = $derived(
    !!conversation &&
      !sending &&
      (composer.trim().length > 0 || pendingImages.length > 0 || pendingFiles.length > 0)
  );
  let isGenerating = $derived(!!generatingVersionId);
  let fetchEnabled = $derived(enabledTools.includes("fetch"));

  function autoResize() {
    if (!textareaRef) return;
    textareaRef.style.height = "auto";
    const scrollH = textareaRef.scrollHeight;
    textareaRef.style.height = `${Math.min(MAX_HEIGHT, Math.max(MIN_HEIGHT, scrollH))}px`;
  }

  onMount(() => {
    autoResize();
  });

  function handleInput(event: Event) {
    onComposerChange((event.currentTarget as HTMLTextAreaElement).value);
    autoResize();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      if (canSend) {
        void onSend();
      }
    }
  }

  async function appendFiles(files: File[]) {
    const selection = await readComposerAttachmentsFromFiles(files);
    if (selection.images.length === 0 && selection.files.length === 0) {
      return;
    }

    onPendingImagesChange(mergeComposerImages(pendingImages, selection.images));
    onPendingFilesChange(mergeComposerFiles(pendingFiles, selection.files));
  }

  async function handleFileSelection(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const files = Array.from(input.files ?? []);
    await appendFiles(files);
    input.value = "";
  }

  async function handlePaste(event: ClipboardEvent) {
    const files = Array.from(event.clipboardData?.files ?? []);
    if (files.length === 0) {
      return;
    }

    event.preventDefault();
    await appendFiles(files);
  }

  async function openFilePicker() {
    if (!conversation || sending) {
      return;
    }

    if (isTauriWindowAvailable()) {
      try {
        const selection = await pickComposerAttachments();
        if (selection.images.length > 0 || selection.files.length > 0) {
          onPendingImagesChange(mergeComposerImages(pendingImages, selection.images));
          onPendingFilesChange(mergeComposerFiles(pendingFiles, selection.files));
        }
        return;
      } catch {
        return;
      }
    }

    fileInput?.click();
  }

  function removePendingImage(index: number) {
    onPendingImagesChange(pendingImages.filter((_, currentIndex) => currentIndex !== index));
  }

  function removePendingFile(index: number) {
    onPendingFilesChange(pendingFiles.filter((_, currentIndex) => currentIndex !== index));
  }

  function toggleFetchTool() {
    if (fetchEnabled) {
      onEnabledToolsChange(enabledTools.filter((t) => t !== "fetch"));
    } else {
      onEnabledToolsChange([...enabledTools, "fetch"]);
    }
  }
</script>

<div class="mx-auto w-full max-w-[56rem] shrink-0 px-2 pb-3 pt-1 sm:px-4 sm:pb-4 relative z-10">
  <div class="flex flex-col rounded-[var(--buyu-radius-bubble)] border border-border bg-background/95 shadow-sm transition-all focus-within:shadow-md focus-within:border-primary/50">
    <input
      bind:this={fileInput}
      class="hidden"
      multiple
      onchange={handleFileSelection}
      type="file"
    />

    {#if pendingImages.length > 0 || pendingFiles.length > 0}
      <div class="flex gap-2 overflow-x-auto px-4 pb-1 pt-3">
        {#each pendingImages as image, index (image.base64)}
          <div class="group relative shrink-0">
            <img
              alt={image.name}
              class="size-14 rounded-xl border object-cover"
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
        {#each pendingFiles as file, index (`${file.name}-${file.base64.slice(0, 16)}`)}
          <div class="group relative flex min-w-0 max-w-44 shrink-0 items-start gap-2 rounded-xl border bg-muted/30 px-3 py-2">
            <div class="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-lg bg-background">
              <FileTextIcon class="size-4 text-muted-foreground" />
            </div>
            <div class="min-w-0 flex-1">
              <div class="truncate text-xs font-medium text-foreground">{file.name}</div>
              <div class="truncate text-[11px] text-muted-foreground">{file.mimeType}</div>
            </div>
            <button
              class="absolute -right-1.5 -top-1.5 flex size-5 items-center justify-center rounded-full border bg-background text-muted-foreground shadow-sm transition-colors hover:text-foreground"
              onclick={() => removePendingFile(index)}
              type="button"
            >
              <XIcon class="size-3" />
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Auto-resize textarea -->
    <textarea
      bind:this={textareaRef}
      aria-label="聊天输入框"
      class="min-h-[52px] max-h-[280px] flex-1 resize-none bg-transparent px-4 py-3 text-sm leading-relaxed outline-none placeholder:text-muted-foreground/50"
      disabled={!conversation || sending}
      oninput={handleInput}
      onkeydown={handleKeydown}
      onpaste={handlePaste}
      placeholder={conversation ? "输入消息，Enter 发送，Shift+Enter 换行" : "请先选择会话"}
      rows="1"
      value={composer}
    ></textarea>

    <!-- 内嵌工具栏 -->
    <div class="flex items-center justify-between gap-1 px-3 pb-2.5">
      <div class="flex items-center gap-0.5">
        <Button
          class="size-7 rounded-lg"
          disabled={!conversation || sending}
          onclick={() => void openFilePicker()}
          size="icon"
          variant="ghost"
          title="上传附件"
        >
          <PaperclipIcon class="size-4 text-muted-foreground" />
        </Button>
        <div class="relative">
          <Button
            class="size-7 rounded-lg"
            disabled={!conversation || sending}
            onclick={() => (toolPopoverOpen = !toolPopoverOpen)}
            size="icon"
            variant={enabledTools.length > 0 ? "secondary" : "ghost"}
            title="工具"
          >
            <WrenchIcon class="size-4 {enabledTools.length > 0 ? 'text-foreground' : 'text-muted-foreground'}" />
          </Button>
          {#if toolPopoverOpen}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="absolute bottom-full left-0 mb-2 w-48 rounded-xl border bg-popover p-1.5 shadow-lg"
              onmouseleave={() => (toolPopoverOpen = false)}
            >
              <div class="mb-1 px-2 text-[11px] font-medium text-muted-foreground">内置工具</div>
              <button
                class="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-sm transition-colors hover:bg-muted/60"
                onclick={toggleFetchTool}
                type="button"
              >
                <GlobeIcon class="size-4 shrink-0 {fetchEnabled ? 'text-primary' : 'text-muted-foreground'}" />
                <span class="flex-1 text-left">网页抓取</span>
                <div class="flex size-4 items-center justify-center rounded border {fetchEnabled ? 'border-primary bg-primary' : 'border-muted-foreground/30'}">
                  {#if fetchEnabled}
                    <svg class="size-3 text-white" viewBox="0 0 12 12" fill="none"><path d="M2.5 6L5 8.5L9.5 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
                  {/if}
                </div>
              </button>
            </div>
          {/if}
        </div>
      </div>

      <div class="flex items-center gap-1">
        {#if isGenerating}
          <Button
            class="size-8 rounded-xl"
            onclick={() => generatingVersionId && onCancel(generatingVersionId)}
            size="icon"
            variant="destructive"
            title="停止生成"
          >
            <SquareIcon class="size-4" />
          </Button>
        {/if}
        <Button
          class="size-8 rounded-xl"
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
</div>
