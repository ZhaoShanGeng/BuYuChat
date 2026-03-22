<script lang="ts">
  import { onMount } from "svelte";
  import { Dialog } from "bits-ui";
  import { toast } from "svelte-sonner";
  import { BookOpen, Edit3, Hash, Plus, Save, Trash2, X, TextSearch, Tag } from "lucide-svelte";
  import {
    createLorebook,
    createLorebookEntry,
    deleteLorebook,
    deleteLorebookEntry,
    getLorebookDetail,
    listLorebooks,
    replaceLorebookEntryKeys,
    updateLorebook,
    updateLorebookEntry,
    type CreateLorebookEntryInput,
    type CreateLorebookInput,
    type LorebookDetail,
    type LorebookEntryDetail,
    type LorebookSummary,
    type UpdateLorebookInput
  } from "$lib/api/lorebooks";
  import { i18n } from "$lib/i18n.svelte";
  import { cn } from "$lib/utils";
  import SearchField from "$components/shared/search-field.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import Button from "$components/ui/button.svelte";
  import Card from "$components/ui/card.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import PageShell from "$components/layout/page-shell.svelte";

  type TabId = "overview" | "entries" | "testing";
  type LorebookDraft = {
    name: string;
    description: string;
    scanDepth: number;
    tokenBudget: string;
    insertionStrategy: string;
    enabled: boolean;
    sortOrder: number;
  };
  type EntryDraft = {
    title: string;
    content: string;
    activationStrategy: string;
    keywordLogic: string;
    insertionPosition: string;
    insertionOrder: number;
    insertionDepth: string;
    insertionRole: "" | "system" | "user" | "assistant" | "tool";
    outletName: string;
    entryScope: string;
    enabled: boolean;
    sortOrder: number;
    keysText: string;
  };

  const tabs: { id: TabId; label: string }[] = [
    { id: "overview", label: "基础信息" },
    { id: "entries", label: "条目" },
    { id: "testing", label: "匹配测试" }
  ];
  const labelClass = "space-y-1";
  const labelTextClass = "text-xs font-medium text-[var(--ink-muted)]";

  let testText = $state("");
  let testing = $state(false);
  let matchedEntries = $state<LorebookEntryDetail[]>([]);

  let selectedLorebook = $state<string | null>(null);
  let searchQuery = $state("");
  let lorebooks = $state<LorebookSummary[]>([]);
  let activeLorebook = $state<LorebookDetail | null>(null);
  let detailDraft = $state<LorebookDraft>(emptyLorebookDraft());
  let createDraft = $state<LorebookDraft>(emptyLorebookDraft());
  let entryDraft = $state<EntryDraft>(emptyEntryDraft());
  let loadingList = $state(true);
  let loadingDetail = $state(false);
  let savingLorebook = $state(false);
  let deletingLorebook = $state(false);
  let savingEntry = $state(false);
  let deletingEntryId = $state<string | null>(null);
  let createDialogOpen = $state(false);
  let entryDialogOpen = $state(false);
  let editingEntryId = $state<string | null>(null);
  let activeTab = $state<TabId>("overview");

  const filteredLorebooks = $derived(
    searchQuery ? lorebooks.filter((book) => `${book.name} ${book.description ?? ""}`.toLowerCase().includes(searchQuery.toLowerCase())) : lorebooks
  );
  const detailDirty = $derived(activeLorebook ? JSON.stringify(mapLorebookToDraft(activeLorebook)) !== JSON.stringify(detailDraft) : false);

  onMount(() => void loadLorebooks());

  function emptyLorebookDraft(sortOrder = 0): LorebookDraft {
    return { name: "", description: "", scanDepth: 3, tokenBudget: "", insertionStrategy: "append", enabled: true, sortOrder };
  }
  function emptyEntryDraft(sortOrder = 0): EntryDraft {
    return { title: "", content: "", activationStrategy: "keyword", keywordLogic: "any", insertionPosition: "before_assistant", insertionOrder: sortOrder, insertionDepth: "", insertionRole: "", outletName: "", entryScope: "conversation", enabled: true, sortOrder, keysText: "" };
  }
  function normalizeNullable(value: string) { const trimmed = value.trim(); return trimmed ? trimmed : null; }
  function parseOptionalNumber(value: string) { const trimmed = value.trim(); if (!trimmed) return null; const parsed = Number(trimmed); return Number.isFinite(parsed) ? parsed : null; }
  function readContentText(content?: { text_content: string | null; preview_text: string | null } | null) { return content?.text_content?.trim() || content?.preview_text?.trim() || ""; }
  function parseKeys(value: string) { return value.split(/[,\n]/).map((item) => item.trim()).filter(Boolean); }
  function toContentInput(text: string): CreateLorebookEntryInput["primary_content"] {
    const trimmed = text.trim();
    return { content_type: "text", mime_type: "text/plain", text_content: trimmed, source_file_path: null, primary_storage_uri: null, size_bytes_hint: null, preview_text: trimmed ? trimmed.slice(0, 160) : null, config_json: {} };
  }
  function formatError(error: unknown) { if (error instanceof Error && error.message) return error.message; if (typeof error === "string") return error; return "请求失败，请稍后重试"; }
  function mapLorebookToDraft(detail: LorebookDetail): LorebookDraft {
    return { name: detail.lorebook.name, description: detail.lorebook.description ?? "", scanDepth: detail.lorebook.scan_depth, tokenBudget: detail.lorebook.token_budget?.toString() ?? "", insertionStrategy: detail.lorebook.insertion_strategy, enabled: detail.lorebook.enabled, sortOrder: detail.lorebook.sort_order };
  }
  function mapEntryToDraft(entry: LorebookEntryDetail): EntryDraft {
    return { title: entry.title ?? "", content: readContentText(entry.primary_content), activationStrategy: entry.activation_strategy, keywordLogic: entry.keyword_logic, insertionPosition: entry.insertion_position, insertionOrder: entry.insertion_order, insertionDepth: entry.insertion_depth?.toString() ?? "", insertionRole: (entry.insertion_role as EntryDraft["insertionRole"]) ?? "", outletName: entry.outlet_name ?? "", entryScope: entry.entry_scope, enabled: entry.enabled, sortOrder: entry.sort_order, keysText: entry.keys.map((key) => key.pattern_text).join(", ") };
  }
  function buildLorebookInput(draft: LorebookDraft): CreateLorebookInput | UpdateLorebookInput {
    return { name: draft.name.trim() || "未命名世界书", description: normalizeNullable(draft.description), scan_depth: Number.isFinite(draft.scanDepth) ? draft.scanDepth : 3, token_budget: parseOptionalNumber(draft.tokenBudget), insertion_strategy: draft.insertionStrategy.trim() || "append", enabled: draft.enabled, sort_order: Number.isFinite(draft.sortOrder) ? draft.sortOrder : 0, config_json: {} };
  }
  function buildEntryInput(lorebookId: string, draft: EntryDraft): CreateLorebookEntryInput {
    return { lorebook_id: lorebookId, title: normalizeNullable(draft.title), primary_content: toContentInput(draft.content), activation_strategy: draft.activationStrategy.trim() || "keyword", keyword_logic: draft.keywordLogic.trim() || "any", insertion_position: draft.insertionPosition.trim() || "before_assistant", insertion_order: Number.isFinite(draft.insertionOrder) ? draft.insertionOrder : 0, insertion_depth: parseOptionalNumber(draft.insertionDepth), insertion_role: draft.insertionRole || null, outlet_name: normalizeNullable(draft.outletName), entry_scope: draft.entryScope.trim() || "conversation", enabled: draft.enabled, sort_order: Number.isFinite(draft.sortOrder) ? draft.sortOrder : 0, config_json: {} };
  }

  async function loadLorebooks(preferredSelection?: string | null) {
    loadingList = true;
    try {
      const items = await listLorebooks();
      lorebooks = [...items].sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
      if (preferredSelection !== undefined) selectedLorebook = preferredSelection;
    } catch (error) {
      console.error("Failed to load lorebooks:", error);
      lorebooks = [];
      toast.error("读取世界书失败", { description: formatError(error) });
    } finally { loadingList = false; }
  }
  async function openLorebook(id: string) {
    selectedLorebook = id; activeTab = "overview"; loadingDetail = true;
    try { activeLorebook = await getLorebookDetail(id); detailDraft = mapLorebookToDraft(activeLorebook); }
    catch (error) { console.error("Failed to load lorebook detail:", error); activeLorebook = null; toast.error("读取世界书详情失败", { description: formatError(error) }); }
    finally { loadingDetail = false; }
  }
  function closeDetail() { selectedLorebook = null; activeLorebook = null; detailDraft = emptyLorebookDraft(); activeTab = "overview"; }
  function openCreateDialog() { createDraft = emptyLorebookDraft(lorebooks.length); createDialogOpen = true; }
  async function submitCreateLorebook() {
    savingLorebook = true;
    try { const created = await createLorebook(buildLorebookInput(createDraft)); createDialogOpen = false; await loadLorebooks(created.lorebook.id); await openLorebook(created.lorebook.id); toast.success("世界书已创建", { description: created.lorebook.name }); }
    catch (error) { console.error("Failed to create lorebook:", error); toast.error("创建世界书失败", { description: formatError(error) }); }
    finally { savingLorebook = false; }
  }
  async function submitSaveLorebook() {
    if (!selectedLorebook || !activeLorebook) return;
    savingLorebook = true;
    try { const updated = await updateLorebook(selectedLorebook, buildLorebookInput(detailDraft)); activeLorebook = updated; detailDraft = mapLorebookToDraft(updated); await loadLorebooks(selectedLorebook); toast.success("世界书已保存", { description: updated.lorebook.name }); }
    catch (error) { console.error("Failed to update lorebook:", error); toast.error("保存世界书失败", { description: formatError(error) }); }
    finally { savingLorebook = false; }
  }
  async function removeCurrentLorebook() {
    if (!selectedLorebook || !activeLorebook) return;
    if (!confirm(`确定删除世界书“${activeLorebook.lorebook.name}”吗？`)) return;
    deletingLorebook = true;
    try { await deleteLorebook(selectedLorebook); toast.success("世界书已删除", { description: activeLorebook.lorebook.name }); closeDetail(); await loadLorebooks(null); }
    catch (error) { console.error("Failed to delete lorebook:", error); toast.error("删除世界书失败", { description: formatError(error) }); }
    finally { deletingLorebook = false; }
  }
  function openCreateEntryDialog() { if (!activeLorebook) return; editingEntryId = null; entryDraft = emptyEntryDraft(activeLorebook.entries.length); entryDialogOpen = true; }
  function openEditEntryDialog(entry: LorebookEntryDetail) { editingEntryId = entry.id; entryDraft = mapEntryToDraft(entry); entryDialogOpen = true; }

  function runTest() {
    if (!activeLorebook || !testText.trim()) return;
    testing = true;
    setTimeout(() => {
      const lowerText = testText.toLowerCase();
      matchedEntries = activeLorebook!.entries.filter(e => {
        if (!e.enabled) return false;
        if (e.activation_strategy === "always") return true;
        const keys = e.keys.map(k => k.pattern_text.toLowerCase());
        if (keys.length === 0) return false;
        if (e.keyword_logic === "all") return keys.every(k => lowerText.includes(k));
        return keys.some(k => lowerText.includes(k));
      });
      testing = false;
    }, 400);
  }

  async function submitEntry() {
    if (!selectedLorebook) return;
    savingEntry = true;
    try {
      const keys = parseKeys(entryDraft.keysText);
      if (editingEntryId) {
        const input = buildEntryInput(selectedLorebook, entryDraft);
        const { lorebook_id: _lorebookId, ...updateInput } = input;
        await updateLorebookEntry(editingEntryId, updateInput);
        await replaceLorebookEntryKeys(editingEntryId, keys);
        toast.success("条目已保存");
      } else {
        const created = await createLorebookEntry(buildEntryInput(selectedLorebook, entryDraft));
        await replaceLorebookEntryKeys(created.id, keys);
        toast.success("条目已添加");
      }
      entryDialogOpen = false;
      await openLorebook(selectedLorebook);
      activeTab = "entries";
    } catch (error) {
      console.error("Failed to save lorebook entry:", error);
      toast.error("保存条目失败", { description: formatError(error) });
    } finally { savingEntry = false; }
  }
  async function removeEntry(entry: LorebookEntryDetail) {
    if (!confirm(`确定删除条目“${entry.title || entry.id}”吗？`)) return;
    deletingEntryId = entry.id;
    try { await deleteLorebookEntry(entry.id); toast.success("条目已删除"); if (selectedLorebook) { await openLorebook(selectedLorebook); activeTab = "entries"; } }
    catch (error) { console.error("Failed to delete lorebook entry:", error); toast.error("删除条目失败", { description: formatError(error) }); }
    finally { deletingEntryId = null; }
  }
</script>

<PageShell>
  {#snippet header()}
    <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4" data-tauri-drag-region>
      <div class="flex items-center gap-2">
        <BookOpen size={16} class="text-[var(--brand)]" />
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.lorebooks")}</h1>
      </div>
      <HeaderWindowGroup>
        {#snippet children()}
          <Button type="button" size="sm" onclick={openCreateDialog}>
            <Plus size={14} /> {i18n.t("lorebooks.create")}
          </Button>
        {/snippet}
      </HeaderWindowGroup>
    </header>
  {/snippet}

  {#snippet body()}
    <div class="app-scrollbar h-full overflow-y-auto p-5">
      <div class="grid gap-5 xl:grid-cols-[minmax(18rem,22rem)_minmax(0,1fr)]">
        <Card className="flex min-h-[38rem] flex-col overflow-hidden">
          <div class="border-b border-[var(--border-soft)] px-4 py-3">
            <SearchField bind:value={searchQuery} placeholder={i18n.t("lorebooks.search")} />
          </div>

          <div class="app-scrollbar flex-1 overflow-y-auto p-3">
            {#if loadingList}
              <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-4 py-8 text-center text-sm text-[var(--ink-muted)]">正在读取世界书...</div>
            {:else if filteredLorebooks.length === 0}
              <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-4 py-8 text-center text-sm text-[var(--ink-faint)]">还没有世界书，先创建一个。</div>
            {:else}
              <div class="space-y-2">
                {#each filteredLorebooks as book (book.id)}
                  <Button
                    type="button"
                    variant="ghost"
                    size="md"
                    className={cn(
                      "h-auto w-full justify-start rounded-[var(--radius-lg)] border p-3 text-left transition-all hover:border-[var(--border-medium)] hover:shadow-[var(--shadow-sm)]",
                      selectedLorebook === book.id
                        ? "border-[var(--brand)] bg-[var(--brand-soft)] text-[var(--ink-strong)] hover:bg-[var(--brand-soft)]"
                        : "border-[var(--border-soft)] bg-[var(--bg-surface)] text-[var(--ink-strong)]"
                    )}
                    onclick={() => void openLorebook(book.id)}
                  >
                    <div class="flex items-start gap-3">
                      <div class="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-emerald-400 to-emerald-600 text-white shadow-sm">
                        <BookOpen size={18} />
                      </div>
                      <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                          <p class="truncate text-sm font-semibold text-[var(--ink-strong)]">{book.name}</p>
                          {#if !book.enabled}
                            <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">已停用</span>
                          {/if}
                        </div>
                        <p class="mt-1 line-clamp-2 text-xs leading-relaxed text-[var(--ink-faint)]">{book.description || "无描述"}</p>
                      </div>
                      <span class="text-[10px] text-[var(--ink-faint)]">深度 {book.scan_depth}</span>
                    </div>
                  </Button>
                {/each}
              </div>
            {/if}
          </div>
        </Card>

        <Card className="min-h-[38rem] overflow-hidden">
          {#if !selectedLorebook}
            <div class="flex h-full min-h-[38rem] items-center justify-center px-6 py-10">
              <div class="max-w-sm text-center">
                <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-[var(--radius-lg)] bg-[var(--brand-soft)] text-[var(--brand)] shadow-sm">
                  <BookOpen size={24} />
                </div>
                <h2 class="mt-4 text-base font-semibold text-[var(--ink-strong)]">选择一本世界书</h2>
                <p class="mt-1 text-sm leading-relaxed text-[var(--ink-muted)]">从左侧选择世界书后，可在这里维护基础信息和条目内容。</p>
              </div>
            </div>
          {:else if loadingDetail}
            <div class="flex h-full min-h-[38rem] items-center justify-center px-6 py-10 text-sm text-[var(--ink-muted)]">正在读取世界书详情...</div>
          {:else if !activeLorebook}
            <div class="flex h-full min-h-[38rem] items-center justify-center px-6 py-10 text-sm text-[var(--ink-faint)]">无法读取世界书详情</div>
          {:else}
            <div class="flex h-full min-h-[38rem] flex-col">
              <div class="flex items-center justify-between gap-3 border-b border-[var(--border-soft)] px-5 py-4">
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <h2 class="truncate text-base font-semibold text-[var(--ink-strong)]">{activeLorebook.lorebook.name}</h2>
                    {#if !activeLorebook.lorebook.enabled}
                      <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">已停用</span>
                    {/if}
                  </div>
                  <p class="mt-1 text-xs text-[var(--ink-muted)]">当前世界书的基础配置和条目都在这里持续维护，不再切换到单独详情页。</p>
                </div>
                <div class="flex items-center gap-2">
                  <Button type="button" variant="secondary" size="sm" onclick={closeDetail}>取消选择</Button>
                  <Button type="button" size="sm" onclick={() => void submitSaveLorebook()} disabled={!detailDirty || savingLorebook || loadingDetail}>
                    <Save size={12} /> {savingLorebook ? "保存中..." : "保存"}
                  </Button>
                  <Button type="button" variant="ghost" size="sm" className="h-8 w-8 px-0 text-[var(--ink-faint)] hover:text-[var(--danger)]" onclick={() => void removeCurrentLorebook()} disabled={deletingLorebook || savingLorebook}>
                    <Trash2 size={14} />
                  </Button>
                </div>
              </div>

              <div class="flex gap-1 border-b border-[var(--border-soft)] px-4 py-2">
                {#each tabs as tab}
                  <Button type="button" variant={tab.id === activeTab ? "default" : "ghost"} size="sm" className={cn("rounded-[var(--radius-full)]", tab.id !== activeTab && "text-[var(--ink-muted)]")} onclick={() => (activeTab = tab.id)}>{tab.label}</Button>
                {/each}
              </div>

              <div class="app-scrollbar flex-1 overflow-y-auto p-5">
                <div class="mx-auto max-w-3xl space-y-6">
                  {#if activeTab === "overview"}
                    <div class="space-y-4">
                      <label class={labelClass}><span class={labelTextClass}>名称</span><input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.name} /></label>
                      <label class={labelClass}><span class={labelTextClass}>描述</span><textarea rows="5" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.description}></textarea></label>
                      <div class="grid gap-4 sm:grid-cols-2">
                        <label class={labelClass}><span class={labelTextClass}>扫描深度</span><input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.scanDepth} /></label>
                        <label class={labelClass}><span class={labelTextClass}>Token 预算</span><input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.tokenBudget} /></label>
                        <label class={labelClass}><span class={labelTextClass}>插入策略</span><input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.insertionStrategy} /></label>
                        <label class={labelClass}><span class={labelTextClass}>排序值</span><input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.sortOrder} /></label>
                      </div>
                      <label class="flex items-center gap-2 text-sm text-[var(--ink-body)]"><input type="checkbox" bind:checked={detailDraft.enabled} /> 启用该世界书</label>
                    </div>
                  {:else if activeTab === "entries"}
                    <div class="space-y-3">
                      <div class="flex items-center justify-between">
                        <div>
                          <h3 class="text-sm font-semibold text-[var(--ink-strong)]">条目列表</h3>
                          <p class="mt-1 text-xs text-[var(--ink-muted)]">关键词触发、插入位置和正文内容都在这里统一维护。</p>
                        </div>
                        <Button type="button" variant="secondary" size="sm" className="h-8 gap-1 px-3" onclick={openCreateEntryDialog}><Plus size={12} /> 添加条目</Button>
                      </div>
                      {#if activeLorebook.entries.length === 0}
                        <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-6 text-center text-xs text-[var(--ink-faint)]">暂无条目</div>
                      {:else}
                        {#each activeLorebook.entries as entry (entry.id)}
                          <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
                            <div class="mb-2 flex items-start justify-between gap-3">
                              <div class="min-w-0 flex-1">
                                <div class="flex flex-wrap items-center gap-2"><span class="text-sm font-semibold text-[var(--ink-strong)]">{entry.title || "未命名条目"}</span><span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">{entry.activation_strategy}</span><span class="rounded-[var(--radius-full)] bg-[var(--brand-soft)] px-1.5 py-0.5 text-[10px] text-[var(--brand)]">{entry.insertion_position}</span></div>
                                <div class="mt-1 flex flex-wrap gap-1">{#each entry.keys as key}<span class="inline-flex items-center gap-0.5 rounded-[var(--radius-full)] bg-[var(--brand-soft)] px-2 py-0.5 text-[10px] font-medium text-[var(--brand)]"><Hash size={9} /> {key.pattern_text}</span>{/each}</div>
                                <p class="mt-2 text-xs leading-relaxed text-[var(--ink-muted)]">{readContentText(entry.primary_content) || "无内容"}</p>
                              </div>
                              <div class="flex items-center gap-1"><ActionIconButton title="编辑条目" onClick={() => openEditEntryDialog(entry)}><Edit3 size={13} /></ActionIconButton><ActionIconButton title="删除条目" tone="danger" disabled={deletingEntryId === entry.id} onClick={() => void removeEntry(entry)}><Trash2 size={13} /></ActionIconButton></div>
                            </div>
                            <div class="flex flex-wrap items-center gap-2 text-[11px] text-[var(--ink-faint)]"><span>#{entry.sort_order}</span>{#if entry.insertion_depth !== null}<span>深度 {entry.insertion_depth}</span>{/if}{#if entry.insertion_role}<span>角色 {entry.insertion_role}</span>{/if}<span>{entry.enabled ? "已启用" : "已停用"}</span></div>
                          </div>
                        {/each}
                      {/if}
                    </div>
                  {:else if activeTab === "testing"}
                    <div class="space-y-4">
                      <div>
                        <div class="mb-2 flex items-center gap-2">
                          <TextSearch size={14} class="text-[var(--ink-faint)]" />
                          <h3 class="text-sm font-semibold text-[var(--ink-strong)]">本地匹配测试</h3>
                        </div>
                        <p class="mb-4 text-xs text-[var(--ink-muted)]">输入一段文本进行模拟测试，以检查条目的关键词触发逻辑是否符合预期。</p>

                        <div class="mb-4">
                          <textarea rows="4" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={testText} placeholder="在此输入测试文本..."></textarea>
                          <div class="mt-2 flex justify-end">
                            <Button size="sm" onclick={runTest} disabled={testing || !testText.trim()}>
                              {testing ? "测试中..." : "运行测试"}
                            </Button>
                          </div>
                        </div>

                        {#if matchedEntries.length > 0}
                          <div class="space-y-3 border-t border-[var(--border-soft)] pt-4">
                            <h4 class="text-xs font-semibold text-[var(--ink-strong)]">命中的条目 ({matchedEntries.length})</h4>
                            {#each matchedEntries as entry (entry.id)}
                              <div class="rounded-[var(--radius-md)] border border-emerald-500/30 bg-emerald-500/5 px-3 py-2.5">
                                <div class="flex items-center gap-2">
                                  <Tag size={12} class="text-emerald-600" />
                                  <span class="text-sm font-medium text-emerald-700">{entry.title || "未命名条目"}</span>
                                  <span class="rounded-[var(--radius-full)] bg-emerald-500/10 px-1.5 py-0.5 text-[10px] text-emerald-600">{entry.activation_strategy}</span>
                                </div>
                                <div class="mt-1.5 line-clamp-2 text-xs text-[var(--ink-muted)]">
                                  {readContentText(entry.primary_content)}
                                </div>
                              </div>
                            {/each}
                          </div>
                        {:else if testText.trim() && !testing}
                          <div class="border-t border-[var(--border-soft)] pt-4">
                            <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] py-6 text-center text-xs text-[var(--ink-faint)]">
                              未匹配到任何条目
                            </div>
                          </div>
                        {/if}
                      </div>
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          {/if}
        </Card>
      </div>
    </div>
  {/snippet}
</PageShell>

<Dialog.Root bind:open={createDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/20 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(680px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-lg)] outline-none">
      <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-4">
        <div>
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">创建世界书</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">先创建主体，再逐条维护内容和关键词。</p>
        </div>
        <Button type="button" variant="ghost" size="sm" className="h-9 w-9 px-0" onclick={() => (createDialogOpen = false)}>
          <X size={16} />
        </Button>
      </div>
      <div class="app-scrollbar max-h-[72dvh] overflow-y-auto px-6 py-5">
        <div class="space-y-4">
          <label class={labelClass}>
            <span class={labelTextClass}>名称</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.name} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>描述</span>
            <textarea rows="5" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.description}></textarea>
          </label>
          <div class="grid gap-4 sm:grid-cols-2">
            <label class={labelClass}>
              <span class={labelTextClass}>扫描深度</span>
              <input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.scanDepth} />
            </label>
            <label class={labelClass}>
              <span class={labelTextClass}>Token 预算</span>
              <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.tokenBudget} />
            </label>
            <label class={labelClass}>
              <span class={labelTextClass}>插入策略</span>
              <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.insertionStrategy} />
            </label>
            <label class={labelClass}>
              <span class={labelTextClass}>排序值</span>
              <input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.sortOrder} />
            </label>
          </div>
          <label class="flex items-center gap-2 text-sm text-[var(--ink-body)]">
            <input type="checkbox" bind:checked={createDraft.enabled} />
            创建后立即启用
          </label>
        </div>
      </div>
      <div class="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-6 py-4">
        <Button type="button" variant="secondary" onclick={() => (createDialogOpen = false)} disabled={savingLorebook}>取消</Button>
        <Button type="button" onclick={() => void submitCreateLorebook()} disabled={savingLorebook || !createDraft.name.trim()}>
          {savingLorebook ? "创建中..." : "创建世界书"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<Dialog.Root bind:open={entryDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/20 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(760px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-lg)] outline-none">
      <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-4">
        <div>
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">{editingEntryId ? "编辑条目" : "添加条目"}</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">关键词用逗号或换行分隔，保存时会同步替换整组 key。</p>
        </div>
        <Button type="button" variant="ghost" size="sm" className="h-9 w-9 px-0" onclick={() => (entryDialogOpen = false)}>
          <X size={16} />
        </Button>
      </div>
      <div class="app-scrollbar max-h-[72dvh] overflow-y-auto px-6 py-5">
        <div class="grid gap-4 md:grid-cols-2">
          <label class={labelClass}>
            <span class={labelTextClass}>标题</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.title} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>触发策略</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.activationStrategy} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>关键词逻辑</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.keywordLogic} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>插入位置</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.insertionPosition} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>插入排序</span>
            <input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.insertionOrder} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>深度</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.insertionDepth} placeholder="留空表示不限" />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>角色</span>
            <select class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.insertionRole}>
              <option value="">无</option>
              <option value="system">system</option>
              <option value="user">user</option>
              <option value="assistant">assistant</option>
              <option value="tool">tool</option>
            </select>
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>作用域</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.entryScope} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>输出口</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.outletName} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>排序值</span>
            <input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.sortOrder} />
          </label>
          <label class="flex items-center gap-2 pt-6 text-sm text-[var(--ink-body)]">
            <input type="checkbox" bind:checked={entryDraft.enabled} />
            启用条目
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>关键词</span>
            <textarea rows="3" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.keysText} placeholder="支持逗号或换行分隔"></textarea>
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>正文</span>
            <textarea rows="8" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.content}></textarea>
          </label>
        </div>
      </div>
      <div class="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-6 py-4">
        <Button type="button" variant="secondary" onclick={() => (entryDialogOpen = false)} disabled={savingEntry}>取消</Button>
        <Button type="button" onclick={() => void submitEntry()} disabled={savingEntry}>
          {savingEntry ? "保存中..." : editingEntryId ? "保存条目" : "添加条目"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
