<script lang="ts">
  import { Plus, Search, Pencil, Trash2, MoreHorizontal, Loader2 } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import type { SidebarItem, WorkspaceId } from "$lib/state/app-shell.svelte";
  import { i18n } from "$lib/i18n.svelte";
  import SearchField from "$components/shared/search-field.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import { formatRelativeTimestamp } from "$lib/time";

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

  let filterText = $state("");

  // Context menu state
  let contextMenuId = $state("");
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let showContextMenu = $state(false);

  // Inline rename state
  let renamingId = $state("");
  let renameText = $state("");
  let renameInput = $state<HTMLInputElement | undefined>(undefined);

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
      <ActionIconButton title={i18n.t("sidebar.new")} onClick={onCreateNew} className="icon-hover h-7 w-7 hover:bg-[var(--bg-active)] hover:text-[var(--brand)]">
        <Plus size={16} />
      </ActionIconButton>
    {/if}
  </div>

  <!-- Search -->
  <div class="px-3 pb-2">
    <SearchField
      bind:value={filterText}
      placeholder={`${i18n.t("sidebar.search")}${i18n.t(labelKeys[workspace])}…`}
    />
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
        <div
          class={cn(
            "group/item relative mb-0.5 flex items-center gap-2 rounded-[var(--radius-md)]",
            item.id === activeId ? "bg-[var(--bg-active)]" : "hover:bg-[var(--bg-hover)]"
          )}
          role="group"
          oncontextmenu={(e) => handleContextMenu(e, item.id)}
        >
          {#if item.id === activeId}
            <span class="sidebar-active-bar"></span>
          {/if}

          <button
            type="button"
            class="flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 rounded-[var(--radius-md)] px-2.5 py-2.5 text-left text-[var(--ink-body)] transition-colors duration-100"
            onclick={() => onSelect(item.id)}
          >
            <div
              class={cn(
                "flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-br text-xs font-bold text-white shadow-sm",
                getAvatarColor(item.title)
              )}
            >
              {getInitial(item.title)}
            </div>

            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-medium text-[var(--ink-strong)]">{item.title}</p>
              <p class="mt-0.5 truncate text-xs text-[var(--ink-faint)]">
                {#if item.updatedAt}
                  {formatRelativeTimestamp(item.updatedAt)}
                {:else}
                  {item.meta ?? ""}
                {/if}
              </p>
            </div>

            {#if item.busyCount || item.unreadCount}
              <div class="flex flex-shrink-0 items-center gap-1">
                {#if item.busyCount}
                  <span class="inline-flex h-6 min-w-6 items-center justify-center gap-1 rounded-full border border-[var(--border-soft)] bg-[var(--bg-surface)] px-2 text-[10px] font-semibold text-[var(--brand)]">
                    <Loader2 size={10} class="animate-spin" />
                    {item.busyCount}
                  </span>
                {/if}
                {#if item.unreadCount}
                  <span class="inline-flex h-6 min-w-6 items-center justify-center rounded-full bg-[var(--brand)] px-2 text-[10px] font-semibold text-white">
                    {item.unreadCount}
                  </span>
                {/if}
              </div>
            {/if}
          </button>

          {#if onRename || onDelete}
            <ActionIconButton
              title={i18n.t("sidebar.rename")}
              className="mr-2 h-6 w-6 flex-shrink-0 opacity-0 group-hover/item:opacity-100"
              onClick={(e) => {
                e.stopPropagation();
                handleContextMenu(e, item.id);
              }}
            >
              <MoreHorizontal size={14} />
            </ActionIconButton>
          {/if}
        </div>
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
