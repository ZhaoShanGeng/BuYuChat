<script lang="ts">
  import { Paperclip, SendHorizontal, Loader2, X } from "lucide-svelte";
  import Button from "$components/ui/button.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import { i18n } from "$lib/i18n.svelte";
  import { cn } from "$lib/utils";

  type RecipientOption = {
    id: string;
    label: string;
    secondaryLabel?: string | null;
  };

  type AttachmentChip = {
    id: string;
    name: string;
    meta: string;
  };

  let {
    value = $bindable(""),
    sending = false,
    canSend = false,
    busy = false,
    attachments = [],
    availableRecipients = [],
    selectedRecipientIds = [],
    onSend = () => {},
    onAttachFiles = () => {},
    onRemoveAttachment = () => {},
    onToggleRecipient = () => {}
  }: {
    value?: string;
    sending?: boolean;
    canSend?: boolean;
    busy?: boolean;
    attachments?: AttachmentChip[];
    availableRecipients?: RecipientOption[];
    selectedRecipientIds?: string[];
    onSend?: () => void;
    onAttachFiles?: (files: FileList | null) => void;
    onRemoveAttachment?: (attachmentId: string) => void;
    onToggleRecipient?: (recipientId: string) => void;
  } = $props();

  let composerTextarea = $state<HTMLTextAreaElement | undefined>(undefined);
  let attachmentInput = $state<HTMLInputElement | undefined>(undefined);

  function getComposerMaxHeight() {
    if (typeof window === "undefined") return 240;
    return Math.max(180, Math.min(Math.round(window.innerHeight * 0.26), 320));
  }

  function resizeComposer() {
    if (!composerTextarea) return;
    composerTextarea.style.height = "auto";
    const maxHeight = getComposerMaxHeight();
    composerTextarea.style.height = `${Math.min(composerTextarea.scrollHeight, maxHeight)}px`;
    composerTextarea.style.overflowY =
      composerTextarea.scrollHeight > maxHeight ? "auto" : "hidden";
  }

  $effect(() => {
    void value;
    resizeComposer();
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      if (canSend) {
        onSend();
      }
    }
  }

  function openAttachmentPicker() {
    attachmentInput?.click();
  }

  function handleAttachmentChange(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    onAttachFiles(target.files);
    target.value = "";
  }
</script>

<div class="shrink-0 px-4 pb-4 pt-2">
  <div class="composer-card mx-auto w-full max-w-[var(--composer-max-width)] px-4 pb-3 pt-3">
    {#if availableRecipients.length > 1}
      <div class="mb-3">
        <div class="mb-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--ink-faint)]">
          {i18n.t("chat.recipient_selector")}
        </div>
        <div class="flex flex-wrap gap-2">
          {#each availableRecipients as recipient}
            {@const selected = selectedRecipientIds.includes(recipient.id)}
            <button
              type="button"
              class={cn(
                "inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-medium transition-colors",
                selected
                  ? "border-[var(--brand)] bg-[var(--brand-soft)] text-[var(--brand)]"
                  : "border-[var(--border-soft)] bg-[var(--bg-app)] text-[var(--ink-muted)] hover:border-[var(--border-medium)] hover:text-[var(--ink-strong)]"
              )}
              onclick={() => onToggleRecipient(recipient.id)}
              disabled={sending}
            >
              <span>{recipient.label}</span>
              {#if recipient.secondaryLabel}
                <span class="text-[10px] opacity-70">{recipient.secondaryLabel}</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if attachments.length > 0}
      <div class="mb-3 flex flex-wrap gap-2">
        {#each attachments as attachment}
          <div class="inline-flex max-w-full items-center gap-2 rounded-full border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-1.5 text-xs text-[var(--ink-body)]">
            <div class="min-w-0">
              <p class="truncate font-medium text-[var(--ink-strong)]">{attachment.name}</p>
              <p class="truncate text-[10px] text-[var(--ink-faint)]">{attachment.meta}</p>
            </div>
            <ActionIconButton
              title={i18n.t("chat.attachment_remove")}
              className="h-6 w-6 flex-shrink-0"
              onClick={() => onRemoveAttachment(attachment.id)}
            >
              <X size={12} />
            </ActionIconButton>
          </div>
        {/each}
      </div>
    {/if}

    <textarea
      bind:this={composerTextarea}
      class="block w-full resize-none bg-transparent text-sm leading-relaxed text-[var(--ink-body)] outline-none placeholder:text-[var(--ink-faint)]"
      placeholder={i18n.t("chat.input_placeholder")}
      rows="3"
      bind:value
      onkeydown={handleKeydown}
      disabled={sending}
      style="overflow-y: hidden;"
    ></textarea>

    <input
      bind:this={attachmentInput}
      type="file"
      class="hidden"
      multiple
      onchange={handleAttachmentChange}
    />

    <div class="mt-2 flex items-center justify-between gap-3">
      <div class="flex items-center gap-2">
        <ActionIconButton
          title={i18n.t("chat.attachment")}
          className="icon-hover"
          onClick={openAttachmentPicker}
          disabled={sending}
        >
          <Paperclip size={16} />
        </ActionIconButton>
        {#if busy}
          <span class="inline-flex items-center gap-1 rounded-full bg-[var(--brand-soft)] px-2.5 py-1 text-[10px] font-medium text-[var(--brand)]">
            <Loader2 size={10} class="animate-spin" />
            {i18n.t("chat.in_progress")}
          </span>
        {/if}
      </div>

      <Button
        title={`${i18n.t("chat.send")} (Enter)`}
        variant="default"
        size="sm"
        className="gap-1.5"
        onclick={onSend}
        disabled={!canSend || sending}
      >
        {#if sending}
          <Loader2 size={14} class="animate-spin" />
        {:else}
          <SendHorizontal size={14} />
        {/if}
        {i18n.t("chat.send")}
      </Button>
    </div>
  </div>
</div>
