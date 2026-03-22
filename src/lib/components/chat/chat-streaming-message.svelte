<script lang="ts">
  import { AlertCircle, Loader2, X } from "lucide-svelte";
  import RichContent from "$components/shared/rich-content.svelte";
  import ChatAvatar from "$components/chat/chat-avatar.svelte";
  import { i18n } from "$lib/i18n.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";

  let {
    text = "",
    authorName = "",
    avatarUri = null,
    status = "running",
    errorText = null,
    onDismiss = () => {}
  }: {
    text?: string;
    authorName?: string;
    avatarUri?: string | null;
    status?: "queued" | "running" | "completed" | "failed";
    errorText?: string | null;
    onDismiss?: () => void;
  } = $props();
</script>

<div class="msg-enter mb-6 flex gap-3">
  <ChatAvatar
    name={authorName || i18n.t("chat.assistant")}
    avatarUri={avatarUri}
    kind="agent"
  />
  <div class="min-w-0 flex-1">
    <div class="mb-1.5 flex flex-wrap items-center gap-2">
      <span class="text-sm font-semibold text-[var(--ink-strong)]">{authorName || i18n.t("chat.assistant")}</span>
      {#if status === "failed"}
        <span class="inline-flex items-center gap-1 rounded-[var(--radius-full)] bg-red-50 px-2 py-0.5 text-[10px] font-medium text-[var(--danger)]">
          <AlertCircle size={10} />
          {i18n.t("chat.failed")}
        </span>
        <ActionIconButton title={i18n.t("chat.dismiss_failed")} className="h-6 w-6" onClick={onDismiss}>
          <X size={12} />
        </ActionIconButton>
      {:else}
        <span class="inline-flex items-center gap-1 rounded-[var(--radius-full)] bg-[var(--brand-soft)] px-2 py-0.5 text-[10px] font-medium text-[var(--brand)]">
          <Loader2 size={10} class="animate-spin" />
          {i18n.t("chat.generating")}
        </span>
      {/if}
    </div>

    {#if text}
      <RichContent text={text} />
    {:else if status === "failed"}
      <div class="rounded-[var(--radius-md)] border border-dashed border-red-200 bg-red-50/60 px-3 py-3 text-sm text-[var(--danger)]">
        {errorText ?? i18n.t("chat.generic_error")}
      </div>
    {:else}
      <div class="flex items-center gap-1.5 py-3">
        <span class="typing-dot h-2 w-2 rounded-full bg-[var(--brand)]"></span>
        <span class="typing-dot h-2 w-2 rounded-full bg-[var(--brand)]"></span>
        <span class="typing-dot h-2 w-2 rounded-full bg-[var(--brand)]"></span>
      </div>
    {/if}

    {#if status === "failed" && text}
      <p class="mt-2 text-xs text-[var(--danger)]">{errorText ?? i18n.t("chat.generic_error")}</p>
    {/if}
  </div>
</div>
