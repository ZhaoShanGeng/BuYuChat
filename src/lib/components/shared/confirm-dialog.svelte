<script lang="ts">
  import { X, AlertTriangle } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import { i18n } from "$lib/i18n.svelte";

  let {
    open = false,
    title = i18n.t("confirm_dialog.title") ?? "确认操作",
    description = "",
    confirmText = i18n.t("confirm_dialog.confirm") ?? "确认",
    cancelText = i18n.t("confirm_dialog.cancel") ?? "取消",
    isDanger = false,
    confirming = false,
    onConfirm,
    onCancel
  }: {
    open: boolean;
    title?: string;
    description?: string;
    confirmText?: string;
    cancelText?: string;
    isDanger?: boolean;
    confirming?: boolean;
    onConfirm: () => void | Promise<void>;
    onCancel: () => void;
  } = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "Escape" && !confirming) {
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- Overlay -->
    <div 
      class="absolute inset-0 bg-black/40 backdrop-blur-sm transition-opacity"
      role="button"
      tabindex="-1"
      onclick={() => !confirming && onCancel()}
    ></div>

    <!-- Dialog -->
    <div
      class="relative z-10 w-full max-w-sm animate-in fade-in zoom-in-95 rounded-[var(--radius-xl)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-6 shadow-[var(--shadow-float)] duration-200"
      role="dialog"
      aria-modal="true"
    >
      <div class="mb-4 flex items-start gap-4">
        <div class={cn(
          "flex h-10 w-10 shrink-0 items-center justify-center rounded-full",
          isDanger ? "bg-red-100 text-[var(--danger)] dark:bg-red-500/20" : "bg-[var(--brand-soft)] text-[var(--brand)]"
        )}>
          {#if isDanger}
            <AlertTriangle size={20} />
          {:else}
            <!-- Default icon -->
            <div class="h-5 w-5 rounded-full border-2 border-current"></div>
          {/if}
        </div>
        <div class="flex-1 pt-1">
          <h2 class="text-base font-semibold text-[var(--ink-strong)]">{title}</h2>
          {#if description}
            <p class="mt-2 leading-relaxed text-sm text-[var(--ink-muted)]">{description}</p>
          {/if}
        </div>
      </div>

      <div class="mt-6 flex justify-end gap-3">
        <button
          type="button"
          class="inline-flex h-9 items-center justify-center rounded-[var(--radius-sm)] border border-[var(--border-medium)] bg-transparent px-4 text-sm font-medium text-[var(--ink-body)] transition-colors hover:bg-[var(--bg-hover)] disabled:opacity-50"
          onclick={onCancel}
          disabled={confirming}
        >
          {cancelText}
        </button>
        <button
          type="button"
          class={cn(
            "inline-flex h-9 items-center justify-center rounded-[var(--radius-sm)] px-4 text-sm font-semibold text-white transition-colors disabled:opacity-50",
            isDanger
              ? "bg-[var(--danger)] hover:bg-red-700 dark:hover:bg-red-500"
              : "bg-[var(--brand)] hover:bg-[var(--brand-strong)]"
          )}
          onclick={onConfirm}
          disabled={confirming}
        >
          {#if confirming}
            <div class="flex items-center gap-2">
              <span class="h-4 w-4 animate-spin rounded-full border-2 border-white/20 border-t-white"></span>
              {i18n.t("confirm_dialog.processing") ?? "处理中..."}
            </div>
          {:else}
            {confirmText}
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
