<script lang="ts">
  import { Plus, Search, Pencil, Trash2, MoreHorizontal } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import type { SidebarItem, WorkspaceId } from "$lib/state/app-shell.svelte";
  import { i18n } from "$lib/i18n.svelte";

  let {
    workspace,
    items = [],
    activeId = "",
    onSelect,
    onCreateNew = undefined,
    onRename = undefined,
    onDelete = undefined
  }: {
    workspace: WorkspaceId;
    items?: SidebarItem[];
    activeId?: string;
    onSelect: (id: string) => void;
    onCreateNew?: (() => void) | undefined;
    onRename?: ((id: string, title: string) => void) | undefined;
    onDelete?: ((id: string) => void) | undefined;
  } = $props();

  const labelKeys: Record<WorkspaceId, string> = {
    chat: "nav.chat",
    agents: "nav.agents",
    presets: "nav.presets",
    lorebooks: "nav.lorebooks",
    workflows: "nav.workflows",
    settings: "nav.settings"
  };

  // Color palette for avatar initials
  const avatarColors = [
    "from-blue-400 to-blue-600",
    "from-violet-400 to-violet-600",
    "from-emerald-400 to-emerald-600",
    "from-amber-400 to-amber-600",
    "from-rose-400 to-rose-600",
    "from-cyan-400 to-cyan-600",
    "from-pink-400 to-pink-600",
    "from-teal-400 to-teal-600"
  ];

  function getAvatarColor(title: string): string {
    let hash = 0;
    for (let i = 0; i < title.length; i++) {
      hash = title.charCodeAt(i) + ((hash << 5) - hash);
    }
    return avatarColors[Math.abs(hash) % avatarColors.length];
  }

  function getInitial(title: string): string {
    return title.trim().charAt(0).toUpperCase() || "?";
  }

  function formatRelativeTime(meta: string): string {
    // meta is already formatted from App.svelte as a date string
    // Try to parse it; if it looks like a date, convert to relative
    try {
      const d = new Date(meta);
      if (isNaN(d.getTime())) return meta;
      const now = new Date();
      const diff = now.getTime() - d.getTime();
      const seconds = Math.floor(diff / 1000);
      const minutes = Math.floor(seconds / 60);
      const hours = Math.floor(minutes / 60);
      const days = Math.floor(hours / 24);
      if (seconds < 60) return "刚刚";
      if (minutes < 60) return `${minutes}分钟前`;
      if (hours < 24) return `${hours}小时前`;
      if (days === 1) return "昨天";
      if (days < 7) return `${days}天前`;
      return meta;
    } catch {
      return meta;
    }
  }

  let filterText = $state("");

  // Context menu state
  let contextMenuId = $state("");
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let showContextMenu = $state(false);

  // Inline rename state
  let renamingId = $state("");
  let renameText = $state("");
  let renameInput: HTMLInputElement | undefined;

  const filteredItems = $derived(
    filterText
      ? items.filter(item =>
          item.title.toLowerCase().includes(filterText.toLowerCase())
        )
      : items
  );

  function handleContextMenu(event: MouseEvent, itemId: string) {
    if (!onRename && !onDelete) return;
    event.preventDefault();
    contextMenuId = itemId;
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    showContextMenu = true;
  }

  function closeContextMenu() {
    showContextMenu = false;
    contextMenuId = "";
  }

  function startRename(id: string) {
    const item = items.find(i => i.id === id);
    if (!item) return;
    renamingId = id;
    renameText = item.title;
    closeContextMenu();
    requestAnimationFrame(() => {
      renameInput?.focus();
      renameInput?.select();
    });
  }

  function submitRename() {
    if (renameText.trim() && onRename && renamingId) {
      onRename(renamingId, renameText.trim());
    }
    renamingId = "";
    renameText = "";
  }

  function cancelRename() {
    renamingId = "";
    renameText = "";
  }

  function handleRenameKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      submitRename();
    }
    if (event.key === "Escape") {
      cancelRename();
    }
  }

  function handleDeleteItem(id: string) {
    closeContextMenu();
    onDelete?.(id);
  }

  function handleWindowClick() {
    if (showContextMenu) closeContextMenu();
  }
</script>

<svelte:window onclick={handleWindowClick} />

<aside class="flex h-full flex-col border-r border-[var(--border-soft)] bg-[var(--bg-sidebar)]">
  <!-- Header -->
  <div class="flex items-center justify-between gap-2 px-3 py-3" data-tauri-drag-region>
    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t(labelKeys[workspace])}</h2>
    {#if onCreateNew}
      <button
        type="button"
        title={i18n.t("sidebar.new")}
        class="icon-hover flex h-7 w-7 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] transition-colors hover:bg-[var(--bg-active)] hover:text-[var(--brand)]"
        onclick={onCreateNew}
      >
        <Plus size={16} />
      </button>
    {/if}
  </div>

  <!-- Search -->
  <div class="px-3 pb-2">
    <label class="search-box flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-2.5 py-1.5">
      <Search size={14} class="flex-shrink-0 text-[var(--ink-faint)]" />
      <input
        class="w-full bg-transparent text-sm text-[var(--ink-body)] outline-none placeholder:text-[var(--ink-faint)]"
        placeholder="{i18n.t('sidebar.search')}{i18n.t(labelKeys[workspace])}…"
        bind:value={filterText}
      />
    </label>
  </div>

  <!-- Item list -->
  <div class="app-scrollbar flex-1 overflow-y-auto px-2 pb-2">
    {#each filteredItems as item (item.id)}
      {#if renamingId === item.id}
        <!-- Inline rename input -->
        <div class="mb-0.5 rounded-[var(--radius-md)] border border-[var(--brand)] bg-[var(--bg-surface)] px-2.5 py-2 shadow-[0_0_0_2px_var(--brand-glow)]">
          <input
            bind:this={renameInput}
            class="w-full bg-transparent text-sm font-medium text-[var(--ink-strong)] outline-none"
            bind:value={renameText}
            onkeydown={handleRenameKeydown}
            onblur={submitRename}
          />
        </div>
      {:else}
        <button
          type="button"
          class={cn(
            "group/item relative mb-0.5 flex w-full cursor-pointer items-center gap-2.5 rounded-[var(--radius-md)] px-2.5 py-2.5 text-left transition-colors duration-100",
            item.id === activeId
              ? "bg-[var(--bg-active)] text-[var(--ink-strong)]"
              : "text-[var(--ink-body)] hover:bg-[var(--bg-hover)]"
          )}
          onclick={() => onSelect(item.id)}
          oncontextmenu={(e) => handleContextMenu(e, item.id)}
        >
          <!-- Active indicator bar -->
          {#if item.id === activeId}
            <span class="sidebar-active-bar"></span>
          {/if}

          <!-- Avatar -->
          <div class={cn(
            "flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-br text-xs font-bold text-white shadow-sm",
            getAvatarColor(item.title)
          )}>
            {getInitial(item.title)}
          </div>

          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-medium">{item.title}</p>
            <p class="mt-0.5 truncate text-xs text-[var(--ink-faint)]">{formatRelativeTime(item.meta)}</p>
          </div>
          <!-- Hover actions -->
          {#if onRename || onDelete}
            <button
              type="button"
              class="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] opacity-0 transition-opacity hover:bg-[var(--bg-hover)] hover:text-[var(--ink-muted)] group-hover/item:opacity-100"
              onclick={(e) => { e.stopPropagation(); handleContextMenu(e, item.id); }}
            >
              <MoreHorizontal size={14} />
            </button>
          {/if}
        </button>
      {/if}
    {/each}

    {#if filteredItems.length === 0}
      <div class="flex flex-col items-center gap-2 px-2 py-8 text-xs text-[var(--ink-faint)]">
        <Search size={20} class="text-[var(--ink-faint)] opacity-50" />
        {filterText ? i18n.t("sidebar.no_match") : i18n.t("sidebar.empty")}
      </div>
    {/if}
  </div>
</aside>

<!-- Context Menu (portal-like) -->
{#if showContextMenu}
  <div
    class="fixed z-50 min-w-[140px] rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] py-1 shadow-[var(--shadow-lg)]"
    style="top: {contextMenuY}px; left: {contextMenuX}px;"
  >
    {#if onRename}
      <button
        type="button"
        class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-[var(--ink-body)] hover:bg-[var(--bg-hover)]"
        onclick={() => startRename(contextMenuId)}
      >
        <Pencil size={14} />
        {i18n.t("sidebar.rename")}
      </button>
    {/if}
    {#if onDelete}
      <button
        type="button"
        class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-[var(--danger)] hover:bg-[var(--bg-hover)]"
        onclick={() => handleDeleteItem(contextMenuId)}
      >
        <Trash2 size={14} />
        {i18n.t("sidebar.delete")}
      </button>
    {/if}
  </div>
{/if}
