<script lang="ts">
  import { PanelLeft, PanelRight, Pencil, Check, X, Eraser, Download } from "lucide-svelte";

  let {
    title = "Chat",
    subtitle = "Workspace",
    meta = [],
    editable = false,
    onToggleSidebar,
    onToggleInspector,
    onRename = undefined
  }: {
    title?: string;
    subtitle?: string;
    meta?: string[];
    editable?: boolean;
    onToggleSidebar: () => void;
    onToggleInspector: () => void;
    onRename?: ((title: string) => void) | undefined;
  } = $props();

  let editing = $state(false);
  let editText = $state("");

  function startEdit() {
    if (!editable || !onRename) return;
    editing = true;
    editText = title;
    requestAnimationFrame(() => {
      const input = document.querySelector(".ctx-rename-input") as HTMLInputElement | null;
      input?.focus();
      input?.select();
    });
  }

  function submitEdit() {
    if (editText.trim() && onRename) {
      onRename(editText.trim());
    }
    editing = false;
  }

  function cancelEdit() {
    editing = false;
    editText = "";
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      submitEdit();
    }
    if (event.key === "Escape") {
      cancelEdit();
    }
  }
</script>

<header
  class="flex h-[var(--topbar-height)] items-center gap-3 border-b border-[var(--border-soft)] bg-[var(--bg-surface)]/80 px-4 backdrop-blur-sm"
>
  <!-- Mobile sidebar toggle -->
  <button
    type="button"
    class="icon-hover inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)] lg:hidden"
    onclick={onToggleSidebar}
  >
    <PanelLeft size={18} />
  </button>

  <!-- Title area -->
  <div class="flex min-w-0 flex-1 items-center gap-2">
    {#if editing}
      <div class="flex items-center gap-1.5">
        <input
          class="ctx-rename-input rounded-[var(--radius-sm)] border border-[var(--brand)] bg-[var(--bg-app)] px-2 py-0.5 text-sm font-semibold text-[var(--ink-strong)] outline-none shadow-[0_0_0_2px_var(--brand-glow)]"
          style="width: {Math.max(editText.length * 8 + 24, 100)}px"
          bind:value={editText}
          onkeydown={handleKeydown}
          onblur={submitEdit}
        />
        <button
          type="button"
          class="inline-flex h-6 w-6 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] hover:text-[var(--ink-muted)]"
          onclick={(e) => { e.preventDefault(); cancelEdit(); }}
        >
          <X size={14} />
        </button>
      </div>
    {:else}
      <button
        type="button"
        class="group/title flex items-center gap-1.5 truncate"
        ondblclick={startEdit}
        title={editable ? "双击编辑标题" : ""}
      >
        <h1 class="truncate text-sm font-semibold text-[var(--ink-strong)]">
          {title}
        </h1>
        {#if editable}
          <Pencil size={12} class="flex-shrink-0 text-[var(--ink-faint)] opacity-0 transition-opacity group-hover/title:opacity-100" />
        {/if}
      </button>
    {/if}
    {#if meta.length > 0 && !editing}
      {#each meta as tag}
        <span class="hidden rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-2 py-0.5 text-[11px] font-medium text-[var(--ink-muted)] md:inline-flex">
          {tag}
        </span>
      {/each}
    {/if}
  </div>

  <!-- Right actions -->
  <div class="flex items-center gap-1">
    <!-- Placeholder tool buttons -->
    <button
      type="button"
      title="清空对话"
      class="icon-hover hidden h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-muted)] sm:inline-flex"
    >
      <Eraser size={16} />
    </button>
    <button
      type="button"
      title="导出对话"
      class="icon-hover hidden h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-muted)] sm:inline-flex"
    >
      <Download size={16} />
    </button>

    <div class="mx-1 hidden h-5 w-px bg-[var(--border-soft)] sm:block"></div>

    <button
      type="button"
      class="icon-hover inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)] xl:hidden"
      onclick={onToggleInspector}
    >
      <PanelRight size={18} />
    </button>
  </div>
</header>
