<script lang="ts">
  import { PanelLeft, PanelRight, Pencil, X, Eraser, Download } from "lucide-svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import { i18n } from "$lib/i18n.svelte";

  let {
    conversationTitle = "Conversation",
    editable = false,
    onRename = undefined,
    onToggleSidebar = () => {},
    onToggleInspector = () => {}
  }: {
    conversationTitle?: string;
    editable?: boolean;
    onRename?: ((title: string) => void) | undefined;
    onToggleSidebar?: () => void;
    onToggleInspector?: () => void;
  } = $props();

  let headerEditing = $state(false);
  let headerEditText = $state("");
  let titleInput = $state<HTMLInputElement | undefined>(undefined);

  function startHeaderEdit() {
    if (!editable || !onRename) return;
    headerEditing = true;
    headerEditText = conversationTitle;
    requestAnimationFrame(() => {
      titleInput?.focus();
      titleInput?.select();
    });
  }

  function submitHeaderEdit() {
    if (headerEditText.trim() && onRename) {
      onRename(headerEditText.trim());
    }
    headerEditing = false;
  }

  function cancelHeaderEdit() {
    headerEditing = false;
    headerEditText = "";
  }

  function handleHeaderKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      submitHeaderEdit();
    }

    if (event.key === "Escape") {
      cancelHeaderEdit();
    }
  }
</script>

<header class="flex h-12 flex-shrink-0 items-center gap-3 border-b border-[var(--border-soft)] px-4" data-tauri-drag-region>
  <ActionIconButton title={i18n.t("nav.chat")} className="icon-hover lg:hidden" onClick={onToggleSidebar}>
    <PanelLeft size={18} />
  </ActionIconButton>

  <div class="flex min-w-0 flex-1 items-center gap-2">
    {#if headerEditing}
      <div class="flex items-center gap-1.5">
        <input
          bind:this={titleInput}
          class="rounded-[var(--radius-sm)] border border-[var(--brand)] bg-[var(--bg-app)] px-2 py-0.5 text-sm font-semibold text-[var(--ink-strong)] outline-none shadow-[0_0_0_2px_var(--brand-glow)]"
          style={`width: ${Math.max(headerEditText.length * 8 + 24, 100)}px`}
          bind:value={headerEditText}
          onkeydown={handleHeaderKeydown}
          onblur={submitHeaderEdit}
        />
        <ActionIconButton title={i18n.t("chat.cancel")} className="h-6 w-6" onClick={(event) => {
          event.preventDefault();
          cancelHeaderEdit();
        }}>
          <X size={14} />
        </ActionIconButton>
      </div>
    {:else}
      <button
        type="button"
        class="group/title flex items-center gap-1.5 truncate"
        ondblclick={startHeaderEdit}
        title={editable ? i18n.t("chat.edit_title") : ""}
      >
        <h1 class="truncate text-sm font-semibold text-[var(--ink-strong)]">{conversationTitle}</h1>
        {#if editable}
          <Pencil size={12} class="flex-shrink-0 text-[var(--ink-faint)] opacity-0 transition-opacity group-hover/title:opacity-100" />
        {/if}
      </button>
    {/if}
  </div>

  <HeaderWindowGroup>
    {#snippet children()}
      <ActionIconButton title={i18n.t("chat.clear")} className="icon-hover hidden sm:inline-flex">
        <Eraser size={16} />
      </ActionIconButton>
      <ActionIconButton title={i18n.t("chat.export")} className="icon-hover hidden sm:inline-flex">
        <Download size={16} />
      </ActionIconButton>
      <ActionIconButton title={i18n.t("inspector.title")} className="icon-hover" onClick={onToggleInspector} dataInspectorToggle>
        <PanelRight size={18} />
      </ActionIconButton>
    {/snippet}
  </HeaderWindowGroup>
</header>
