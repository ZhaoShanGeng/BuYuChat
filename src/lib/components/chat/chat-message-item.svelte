<script lang="ts">
  import {
    Check,
    ChevronLeft,
    ChevronRight,
    Copy,
    Edit3,
    Loader2,
    RefreshCw,
    Trash2,
    X
  } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import type { MessageVersionView } from "$lib/api/messages";
  import { i18n } from "$lib/i18n.svelte";
  import { formatRelativeTimestamp, formatFullTimestamp } from "$lib/time";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import RichContent from "$components/shared/rich-content.svelte";
  import ChatAvatar from "$components/chat/chat-avatar.svelte";

  let {
    message,
    text,
    authorName = "",
    avatarUri = null,
    authorKind = "agent",
    versionInfo = null,
    isEditing = false,
    editText = $bindable(""),
    editSaving = false,
    copied = false,
    generationLocked = false,
    selected = false,
    animationDelay = "0ms",
    onLoadVersions = () => {},
    onStartEdit = () => {},
    onCancelEdit = () => {},
    onSubmitEdit = () => {},
    onCopy = () => {},
    onDelete = () => {},
    onRegenerate = () => {},
    onSelect = () => {},
    onPrevVersion = () => {},
    onNextVersion = () => {}
  }: {
    message: MessageVersionView;
    text: string;
    authorName?: string;
    avatarUri?: string | null;
    authorKind?: "agent" | "human" | "system";
    versionInfo?: { current: number; total: number } | null;
    isEditing?: boolean;
    editText?: string;
    editSaving?: boolean;
    copied?: boolean;
    generationLocked?: boolean;
    selected?: boolean;
    animationDelay?: string;
    onLoadVersions?: () => void;
    onStartEdit?: () => void;
    onCancelEdit?: () => void;
    onSubmitEdit?: () => void;
    onCopy?: () => void;
    onDelete?: () => void;
    onRegenerate?: () => void;
    onSelect?: () => void;
    onPrevVersion?: () => void;
    onNextVersion?: () => void;
  } = $props();

  const attachmentItems = $derived(
    [...message.content_refs].sort((a, b) => a.sort_order - b.sort_order)
  );
  const showAttachmentPlaceholder = $derived(!text && attachmentItems.length > 0);

  function handleEditKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      onSubmitEdit();
    }

    if (event.key === "Escape") {
      onCancelEdit();
    }
  }

  function editAutoResize(node: HTMLTextAreaElement) {
    const getEditMaxHeight = () => {
      if (typeof window === "undefined") return 300;
      return Math.max(220, Math.min(Math.round(window.innerHeight * 0.32), 380));
    };

    function resize() {
      node.style.height = "auto";
      node.style.height = `${Math.min(node.scrollHeight, getEditMaxHeight())}px`;
    }

    node.addEventListener("input", resize);
    requestAnimationFrame(resize);

    return {
      destroy() {
        node.removeEventListener("input", resize);
      }
    };
  }

  function formatAttachmentLabel(messageRef: MessageVersionView["content_refs"][number]) {
    return (
      messageRef.content.preview_text ??
      messageRef.content.primary_storage_uri?.split(/[\\/]/).pop() ??
      messageRef.content.mime_type ??
      messageRef.content.content_type
    );
  }

  function formatAttachmentMeta(messageRef: MessageVersionView["content_refs"][number]) {
    const parts = [messageRef.ref_role];
    if (messageRef.content.mime_type) {
      parts.push(messageRef.content.mime_type);
    }
    if (messageRef.content.size_bytes) {
      parts.push(`${Math.max(1, Math.round(messageRef.content.size_bytes / 1024))} KB`);
    }
    return parts.join(" · ");
  }

  function handleSelectKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelect();
    }
  }
</script>

<div
  class={cn(
    "group msg-enter mb-6 rounded-[var(--radius-lg)] transition-colors",
    selected && "bg-[var(--brand-soft)]/45"
  )}
  style={`animation-delay: ${animationDelay}`}
  role="button"
  aria-pressed={selected}
  tabindex="0"
  onmouseenter={onLoadVersions}
  onclick={onSelect}
  onkeydown={handleSelectKeydown}
>
  {#if message.role === "user"}
    <div class="flex justify-end gap-3">
      <div class="message-bubble-shell">
        <div class="mb-1.5 flex items-center justify-end gap-2 text-[10px] text-[var(--ink-faint)]">
          {#if message.prompt_tokens || message.completion_tokens}
            <span>In: {message.prompt_tokens ?? 0}</span>
          {/if}
          <span title={formatFullTimestamp(message.created_at)}>{formatRelativeTimestamp(message.created_at)}</span>
        </div>
        {#if isEditing}
          <div class="rounded-[var(--radius-lg)] border border-[var(--brand)] bg-[var(--bg-app)] p-2 shadow-[0_0_0_2px_var(--brand-glow)]">
            <textarea
              class="block w-full resize-none bg-transparent px-2 py-1 text-sm leading-relaxed text-[var(--ink-body)] outline-none"
              bind:value={editText}
              onkeydown={handleEditKeydown}
              use:editAutoResize
            ></textarea>
            <div class="mt-1.5 flex items-center justify-end gap-1.5">
              <button
                type="button"
                class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] px-2 text-xs text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]"
                onclick={onCancelEdit}
              >
                <X size={12} />
                {i18n.t("chat.cancel")}
              </button>
              <button
                type="button"
                class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--brand)] px-2.5 text-xs text-white hover:bg-[var(--brand-strong)] disabled:opacity-50"
                onclick={onSubmitEdit}
                disabled={editSaving || (!editText.trim() && attachmentItems.length === 0)}
              >
                {#if editSaving}
                  <Loader2 size={12} class="animate-spin" />
                {:else}
                  <Check size={12} />
                {/if}
                {i18n.t("chat.save")}
                <kbd class="ml-1 text-[10px] opacity-60">Ctrl+↵</kbd>
              </button>
            </div>
          </div>
        {:else}
          <div class="rounded-2xl rounded-br-sm bg-[var(--bg-chat-user)] border border-[var(--border-soft)] px-4 py-2.5 text-[15px] leading-relaxed text-[var(--ink-strong)] shadow-[var(--shadow-sm)]">
            {#if text}
              <p class="whitespace-pre-wrap">{text}</p>
            {/if}
            {#if showAttachmentPlaceholder}
              <p class="mb-2 text-xs text-[var(--ink-muted)]">{i18n.t("chat.attachment_empty")}</p>
            {/if}
            {#if attachmentItems.length > 0}
              <div class="mt-2 flex flex-wrap gap-2">
                {#each attachmentItems as attachment (attachment.ref_id)}
                  <div class="max-w-full rounded-[var(--radius-md)] bg-black/5 dark:bg-white/5 border border-[var(--border-soft)] px-3 py-2">
                    <p class="truncate text-xs font-medium text-[var(--ink-strong)]">{formatAttachmentLabel(attachment)}</p>
                    <p class="truncate text-[10px] text-[var(--ink-muted)]">{formatAttachmentMeta(attachment)}</p>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}

        {#if !isEditing}
          <div class="mt-1.5 flex items-center justify-end opacity-0 transition-opacity duration-200 group-hover:opacity-100">
            <div class="flex items-center gap-0.5 rounded-full bg-[var(--bg-surface)] px-1.5 py-1 shadow-[var(--shadow-sm)] border border-[var(--border-soft)]">
              <span class="ml-1 mr-2 text-[10px] text-[var(--ink-faint)] select-none">ID: {message.node_id.substring(0, 6)}</span>
              <ActionIconButton title={i18n.t("chat.regenerate")} className="msg-action-btn" disabled={generationLocked} onClick={onRegenerate}>
                <RefreshCw size={13} />
              </ActionIconButton>
              <ActionIconButton title={i18n.t("chat.edit")} className="msg-action-btn" onClick={onStartEdit}>
                <Edit3 size={13} />
              </ActionIconButton>
              <ActionIconButton title={i18n.t("chat.copy")} className="msg-action-btn" onClick={onCopy}>
                {#if copied}
                  <Check size={13} class="text-[var(--success)]" />
                {:else}
                  <Copy size={13} />
                {/if}
              </ActionIconButton>
              <ActionIconButton title={i18n.t("chat.delete")} className="msg-action-btn hover:text-[var(--danger)] hover:bg-[var(--danger)]/10" tone="danger" onClick={onDelete}>
                <Trash2 size={13} />
              </ActionIconButton>
            </div>
          </div>
        {/if}
      </div>

      <ChatAvatar
        name={authorName || i18n.t("chat.user_label")}
        avatarUri={avatarUri}
        kind="human"
      />
    </div>
  {:else}
    <div class="flex gap-3">
      <ChatAvatar
        name={authorName || (message.role === "assistant" ? i18n.t("chat.assistant") : i18n.t("chat.system"))}
        avatarUri={avatarUri}
        kind={authorKind}
      />

      <div class="min-w-0 flex-1">
        <div class="mb-1.5 flex flex-wrap items-center justify-between gap-2">
          <div class="flex items-center gap-2">
            <span class="text-sm font-semibold text-[var(--ink-strong)]">
              {authorName || (message.role === "assistant" ? i18n.t("chat.assistant") : i18n.t("chat.system"))}
            </span>
            <span class="text-[10px] text-[var(--ink-faint)]" title={formatFullTimestamp(message.created_at)}>
              {formatRelativeTimestamp(message.created_at)}
            </span>
            {#if versionInfo}
              <div class="flex items-center gap-0.5">
                <button
                  type="button"
                  title={i18n.t("chat.prev_version")}
                  class="inline-flex h-4 w-4 items-center justify-center rounded text-[var(--ink-muted)] hover:bg-[var(--bg-hover)] disabled:opacity-30"
                  disabled={versionInfo.current <= 1}
                  onclick={onPrevVersion}
                >
                  <ChevronLeft size={12} />
                </button>
                <span class="text-[10px] font-medium text-[var(--ink-muted)]">
                  {versionInfo.current}/{versionInfo.total}
                </span>
                <button
                  type="button"
                  title={i18n.t("chat.next_version")}
                  class="inline-flex h-4 w-4 items-center justify-center rounded text-[var(--ink-muted)] hover:bg-[var(--bg-hover)] disabled:opacity-30"
                  disabled={versionInfo.current >= versionInfo.total}
                  onclick={onNextVersion}
                >
                  <ChevronRight size={12} />
                </button>
              </div>
            {/if}
          </div>
          
          <div class="flex items-center gap-1.5 text-[10px] text-[var(--ink-faint)] select-none">
            {#if message.api_channel_model_id}
              <span class="rounded-[var(--radius-sm)] bg-[var(--bg-hover)] px-1.5 py-0.5 border border-[var(--border-soft)]">
                API: {message.api_channel_model_id}
              </span>
            {/if}
            {#if message.prompt_tokens || message.completion_tokens}
              <span class="whitespace-nowrap rounded-[var(--radius-sm)] bg-[var(--bg-light)] px-1.5 py-0.5" title="Prompt / Completion / Total">
                In: {message.prompt_tokens ?? 0} &nbsp;|&nbsp; Out: {message.completion_tokens ?? 0} &nbsp;|&nbsp; Total: {message.total_tokens ?? 0}
              </span>
            {/if}
          </div>
        </div>

        {#if isEditing}
          <div class="rounded-[var(--radius-md)] border border-[var(--brand)] bg-[var(--bg-app)] p-2 shadow-[0_0_0_2px_var(--brand-glow)]">
            <textarea
              class="block w-full resize-none bg-transparent px-2 py-1 text-sm leading-relaxed text-[var(--ink-body)] outline-none"
              bind:value={editText}
              onkeydown={handleEditKeydown}
              use:editAutoResize
            ></textarea>
            <div class="mt-1.5 flex items-center justify-end gap-1.5">
              <button
                type="button"
                class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] px-2 text-xs text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]"
                onclick={onCancelEdit}
              >
                <X size={12} />
                {i18n.t("chat.cancel")}
              </button>
              <button
                type="button"
                class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--brand)] px-2.5 text-xs text-white hover:bg-[var(--brand-strong)] disabled:opacity-50"
                onclick={onSubmitEdit}
                disabled={editSaving || (!editText.trim() && attachmentItems.length === 0)}
              >
                {#if editSaving}
                  <Loader2 size={12} class="animate-spin" />
                {:else}
                  <Check size={12} />
                {/if}
                {i18n.t("chat.save")}
                <kbd class="ml-1 text-[10px] opacity-60">Ctrl+↵</kbd>
              </button>
            </div>
          </div>
        {:else}
          <div class="space-y-3 text-[15px] leading-relaxed text-[var(--ink-body)]">
            {#if text}
              <RichContent text={text} />
            {/if}

            {#if showAttachmentPlaceholder}
              <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-3 text-xs text-[var(--ink-faint)]">
                {i18n.t("chat.attachment_empty")}
              </div>
            {/if}

            {#if attachmentItems.length > 0}
              <div class="grid gap-2 sm:grid-cols-2">
                {#each attachmentItems as attachment (attachment.ref_id)}
                  <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-2">
                    <p class="truncate text-xs font-semibold text-[var(--ink-strong)]">{formatAttachmentLabel(attachment)}</p>
                    <p class="truncate text-[10px] text-[var(--ink-faint)]">{formatAttachmentMeta(attachment)}</p>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}

        {#if !isEditing}
          <div class="mt-2 flex items-center opacity-0 transition-opacity duration-200 group-hover:opacity-100">
            <div class="flex items-center gap-0.5 rounded-full bg-[var(--bg-surface)] px-1.5 py-1 shadow-[var(--shadow-sm)] border border-[var(--border-soft)]">
              <span class="ml-1 mr-2 text-[10px] text-[var(--ink-faint)] select-none">ID: {message.node_id.substring(0, 6)}</span>
              <ActionIconButton title={i18n.t("chat.copy")} className="msg-action-btn" onClick={onCopy}>
                {#if copied}
                  <Check size={13} class="text-[var(--success)]" />
                {:else}
                  <Copy size={13} />
                {/if}
              </ActionIconButton>
              {#if message.role === "assistant"}
                <ActionIconButton title={i18n.t("chat.regenerate")} className="msg-action-btn" disabled={generationLocked} onClick={onRegenerate}>
                  <RefreshCw size={13} />
                </ActionIconButton>
              {/if}
              <ActionIconButton title={i18n.t("chat.edit")} className="msg-action-btn" onClick={onStartEdit}>
                <Edit3 size={13} />
              </ActionIconButton>
              <ActionIconButton title={i18n.t("chat.delete")} className="msg-action-btn hover:text-[var(--danger)] hover:bg-[var(--danger)]/10" tone="danger" onClick={onDelete}>
                <Trash2 size={13} />
              </ActionIconButton>

            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .message-bubble-shell {
    width: min(88%, var(--message-max-width));
  }

  @media (max-width: 767px) {
    .message-bubble-shell {
      width: min(92%, var(--message-max-width));
    }
  }
</style>
