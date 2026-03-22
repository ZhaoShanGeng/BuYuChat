<script lang="ts">
  import { onMount } from "svelte";
  import { Dialog } from "bits-ui";
  import { toast } from "svelte-sonner";
  import { Edit3, GripVertical, Layers3, Link, Plus, Save, Sparkles, Trash2, X, ArrowUp, ArrowDown } from "lucide-svelte";
  import {
    createPreset,
    createPresetEntry,
    deletePreset,
    deletePresetEntry,
    getPresetDetail,
    listPresets,
    reorderPresetEntries,
    updatePreset,
    updatePresetEntry,
    type CreatePresetEntryInput,
    type CreatePresetInput,
    type PresetDetail,
    type PresetEntryDetail,
    type PresetSummary,
    type UpdatePresetInput
  } from "$lib/api/presets";
  import { i18n } from "$lib/i18n.svelte";
  import { cn } from "$lib/utils";
  import { listApiChannels, listApiChannelModels, type ApiChannel, type ApiChannelModel } from "$lib/api/api-channels";
  import SearchField from "$components/shared/search-field.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import Button from "$components/ui/button.svelte";
  import Card from "$components/ui/card.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import PageShell from "$components/layout/page-shell.svelte";

  type TabId = "overview" | "entries" | "bindings";
  type PresetDraft = {
    name: string;
    description: string;
    enabled: boolean;
    sortOrder: number;
  };
  type EntryDraft = {
    name: string;
    role: "system" | "user" | "assistant" | "tool";
    positionType: string;
    content: string;
    listOrder: number;
    depth: string;
    depthOrder: number;
    enabled: boolean;
    isPinned: boolean;
  };

  const tabs: { id: TabId; label: string }[] = [
    { id: "overview", label: "基础信息" },
    { id: "entries", label: "条目" },
    { id: "bindings", label: "渠道绑定" }
  ];

  let selectedPreset = $state<string | null>(null);
  let searchQuery = $state("");
  let presets = $state<PresetSummary[]>([]);
  let activePreset = $state<PresetDetail | null>(null);
  let detailDraft = $state<PresetDraft>(emptyPresetDraft());
  let createDraft = $state<PresetDraft>(emptyPresetDraft());
  let entryDraft = $state<EntryDraft>(emptyEntryDraft());
  let loadingList = $state(true);
  let loadingDetail = $state(false);
  let savingPreset = $state(false);
  let deletingPreset = $state(false);
  let savingEntry = $state(false);
  let deletingEntryId = $state<string | null>(null);
  let movingEntryId = $state<string | null>(null);
  let createDialogOpen = $state(false);
  let entryDialogOpen = $state(false);
  let editingEntryId = $state<string | null>(null);
  let activeTab = $state<TabId>("overview");

  let allChannels = $state<ApiChannel[]>([]);
  let allModels = $state<Record<string, ApiChannelModel[]>>({});
  let availableChannels = $derived(allChannels.filter((c) => c.enabled));

  let bindChannelId = $state<string>("");
  let bindModelId = $state<string>("");

  const labelClass = "space-y-1";
  const labelTextClass = "text-xs font-medium text-[var(--ink-muted)]";

  const filteredPresets = $derived(
    searchQuery
      ? presets.filter((preset) =>
          `${preset.name} ${preset.description ?? ""}`.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : presets
  );

  const detailDirty = $derived(
    activePreset ? JSON.stringify(mapPresetToDraft(activePreset)) !== JSON.stringify(detailDraft) : false
  );

  onMount(() => {
    void loadPresets();
    void loadChannels();
  });

  async function loadChannels() {
    try {
      allChannels = await listApiChannels();
      for (const channel of allChannels) {
        if (channel.enabled) {
          try {
            allModels[channel.id] = await listApiChannelModels(channel.id);
          } catch {}
        }
      }
      if (allChannels.length > 0) bindChannelId = allChannels[0].id;
    } catch {}
  }

  function emptyPresetDraft(sortOrder = 0): PresetDraft {
    return {
      name: "",
      description: "",
      enabled: true,
      sortOrder
    };
  }

  function emptyEntryDraft(sortOrder = 0): EntryDraft {
    return {
      name: "",
      role: "system",
      positionType: "before_history",
      content: "",
      listOrder: sortOrder,
      depth: "",
      depthOrder: 0,
      enabled: true,
      isPinned: false
    };
  }

  function normalizeNullable(value: string) {
    const trimmed = value.trim();
    return trimmed ? trimmed : null;
  }

  function parseOptionalNumber(value: string) {
    const trimmed = value.trim();
    if (!trimmed) return null;
    const parsed = Number(trimmed);
    return Number.isFinite(parsed) ? parsed : null;
  }

  function handleAddChannelBinding() {
    if (!activePreset || !bindChannelId) return;
    const exists = activePreset.channel_bindings.some(b => b.channel_id === bindChannelId && b.channel_model_id === (bindModelId || null));
    if (exists) { toast.error("绑定已存在"); return; }
    
    // UI Mockup update (backend API missing for replacePresetChannels)
    activePreset.channel_bindings = [...activePreset.channel_bindings, {
      id: "mock-binding-" + Date.now(),
      channel_id: bindChannelId,
      channel_model_id: bindModelId || null,
      binding_type: "preset_channel",
      enabled: true,
      sort_order: activePreset.channel_bindings.length,
      config_json: {},
      created_at: Date.now(),
      updated_at: Date.now()
    }];
  }

  function handleRemoveChannelBinding(id: string) {
    if (!activePreset) return;
    activePreset.channel_bindings = activePreset.channel_bindings.filter(b => b.id !== id);
  }

  function readContentText(content?: { text_content: string | null; preview_text: string | null } | null) {
    return content?.text_content?.trim() || content?.preview_text?.trim() || "";
  }

  function toContentInput(text: string): CreatePresetEntryInput["primary_content"] {
    const trimmed = text.trim();
    return {
      content_type: "text",
      mime_type: "text/plain",
      text_content: trimmed,
      source_file_path: null,
      primary_storage_uri: null,
      size_bytes_hint: null,
      preview_text: trimmed ? trimmed.slice(0, 160) : null,
      config_json: {}
    };
  }

  function formatError(error: unknown) {
    if (error instanceof Error && error.message) return error.message;
    if (typeof error === "string") return error;
    return "请求失败，请稍后重试";
  }

  function mapPresetToDraft(detail: PresetDetail): PresetDraft {
    return {
      name: detail.preset.name,
      description: detail.preset.description ?? "",
      enabled: detail.preset.enabled,
      sortOrder: detail.preset.sort_order
    };
  }

  function mapEntryToDraft(entry: PresetEntryDetail): EntryDraft {
    return {
      name: entry.name,
      role: entry.role,
      positionType: entry.position_type,
      content: readContentText(entry.primary_content),
      listOrder: entry.list_order,
      depth: entry.depth?.toString() ?? "",
      depthOrder: entry.depth_order,
      enabled: entry.enabled,
      isPinned: entry.is_pinned
    };
  }

  function buildPresetInput(draft: PresetDraft): CreatePresetInput | UpdatePresetInput {
    return {
      name: draft.name.trim() || "未命名预设",
      description: normalizeNullable(draft.description),
      enabled: draft.enabled,
      sort_order: Number.isFinite(draft.sortOrder) ? draft.sortOrder : 0,
      config_json: {}
    };
  }

  function buildEntryInput(presetId: string, draft: EntryDraft): CreatePresetEntryInput {
    return {
      preset_id: presetId,
      name: draft.name.trim() || "未命名条目",
      role: draft.role,
      primary_content: toContentInput(draft.content),
      position_type: draft.positionType.trim() || "before_history",
      list_order: Number.isFinite(draft.listOrder) ? draft.listOrder : 0,
      depth: parseOptionalNumber(draft.depth),
      depth_order: Number.isFinite(draft.depthOrder) ? draft.depthOrder : 0,
      triggers_json: {},
      enabled: draft.enabled,
      is_pinned: draft.isPinned,
      config_json: {}
    };
  }

  async function loadPresets(preferredSelection?: string | null) {
    loadingList = true;
    try {
      const items = await listPresets();
      presets = [...items].sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
      if (preferredSelection !== undefined) selectedPreset = preferredSelection;
    } catch (error) {
      console.error("Failed to load presets:", error);
      presets = [];
      toast.error("读取预设失败", { description: formatError(error) });
    } finally {
      loadingList = false;
    }
  }

  async function openPreset(id: string) {
    selectedPreset = id;
    activeTab = "overview";
    loadingDetail = true;
    try {
      activePreset = await getPresetDetail(id);
      detailDraft = mapPresetToDraft(activePreset);
    } catch (error) {
      console.error("Failed to load preset detail:", error);
      activePreset = null;
      toast.error("读取预设详情失败", { description: formatError(error) });
    } finally {
      loadingDetail = false;
    }
  }

  function closeDetail() {
    selectedPreset = null;
    activePreset = null;
    detailDraft = emptyPresetDraft();
    activeTab = "overview";
  }

  function openCreateDialog() {
    createDraft = emptyPresetDraft(presets.length);
    createDialogOpen = true;
  }

  async function submitCreatePreset() {
    savingPreset = true;
    try {
      const created = await createPreset(buildPresetInput(createDraft));
      createDialogOpen = false;
      await loadPresets(created.preset.id);
      await openPreset(created.preset.id);
      toast.success("预设已创建", { description: created.preset.name });
    } catch (error) {
      console.error("Failed to create preset:", error);
      toast.error("创建预设失败", { description: formatError(error) });
    } finally {
      savingPreset = false;
    }
  }

  async function submitSavePreset() {
    if (!selectedPreset || !activePreset) return;
    savingPreset = true;
    try {
      const updated = await updatePreset(selectedPreset, buildPresetInput(detailDraft));
      activePreset = updated;
      detailDraft = mapPresetToDraft(updated);
      await loadPresets(selectedPreset);
      toast.success("预设已保存", { description: updated.preset.name });
    } catch (error) {
      console.error("Failed to update preset:", error);
      toast.error("保存预设失败", { description: formatError(error) });
    } finally {
      savingPreset = false;
    }
  }

  async function removeCurrentPreset() {
    if (!selectedPreset || !activePreset) return;
    if (!confirm(`确定删除预设“${activePreset.preset.name}”吗？`)) return;
    deletingPreset = true;
    try {
      await deletePreset(selectedPreset);
      toast.success("预设已删除", { description: activePreset.preset.name });
      closeDetail();
      await loadPresets(null);
    } catch (error) {
      console.error("Failed to delete preset:", error);
      toast.error("删除预设失败", { description: formatError(error) });
    } finally {
      deletingPreset = false;
    }
  }

  function openCreateEntryDialog() {
    if (!activePreset) return;
    editingEntryId = null;
    entryDraft = emptyEntryDraft(activePreset.entries.length);
    entryDialogOpen = true;
  }

  function openEditEntryDialog(entry: PresetEntryDetail) {
    editingEntryId = entry.id;
    entryDraft = mapEntryToDraft(entry);
    entryDialogOpen = true;
  }

  async function submitEntry() {
    if (!selectedPreset) return;
    savingEntry = true;
    try {
      if (editingEntryId) {
        const input = buildEntryInput(selectedPreset, entryDraft);
        const { preset_id: _presetId, ...updateInput } = input;
        await updatePresetEntry(editingEntryId, updateInput);
        toast.success("条目已保存");
      } else {
        await createPresetEntry(buildEntryInput(selectedPreset, entryDraft));
        toast.success("条目已添加");
      }
      entryDialogOpen = false;
      await openPreset(selectedPreset);
      activeTab = "entries";
    } catch (error) {
      console.error("Failed to save preset entry:", error);
      toast.error("保存条目失败", { description: formatError(error) });
    } finally {
      savingEntry = false;
    }
  }

  async function removeEntry(entry: PresetEntryDetail) {
    if (!confirm(`确定删除条目“${entry.name}”吗？`)) return;
    deletingEntryId = entry.id;
    try {
      await deletePresetEntry(entry.id);
      toast.success("条目已删除");
      if (selectedPreset) {
        await openPreset(selectedPreset);
        activeTab = "entries";
      }
    } catch (error) {
      console.error("Failed to delete preset entry:", error);
      toast.error("删除条目失败", { description: formatError(error) });
    } finally {
      deletingEntryId = null;
    }
  }

  async function moveEntry(entryId: string, direction: -1 | 1) {
    if (!selectedPreset || !activePreset) return;
    const currentIndex = activePreset.entries.findIndex((entry) => entry.id === entryId);
    const nextIndex = currentIndex + direction;
    if (currentIndex < 0 || nextIndex < 0 || nextIndex >= activePreset.entries.length) return;
    const reordered = [...activePreset.entries];
    const [moved] = reordered.splice(currentIndex, 1);
    reordered.splice(nextIndex, 0, moved);
    movingEntryId = entryId;
    try {
      const entries = await reorderPresetEntries(selectedPreset, reordered.map((entry) => entry.id));
      activePreset = { ...activePreset, entries };
      toast.success("条目顺序已更新");
    } catch (error) {
      console.error("Failed to reorder preset entries:", error);
      toast.error("调整顺序失败", { description: formatError(error) });
    } finally {
      movingEntryId = null;
    }
  }
</script>

<PageShell>
  {#snippet header()}
    <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4" data-tauri-drag-region>
      <div class="flex items-center gap-2">
        <Layers3 size={16} class="text-[var(--brand)]" />
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.presets")}</h1>
      </div>
      <HeaderWindowGroup>
        {#snippet children()}
          <Button type="button" size="sm" onclick={openCreateDialog}>
            <Plus size={14} /> {i18n.t("presets.create")}
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
            <SearchField bind:value={searchQuery} placeholder={i18n.t("presets.search")} />
          </div>

          <div class="app-scrollbar flex-1 overflow-y-auto p-3">
            {#if loadingList}
              <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-4 py-8 text-center text-sm text-[var(--ink-muted)]">
                正在读取预设...
              </div>
            {:else if filteredPresets.length === 0}
              <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-4 py-8 text-center text-sm text-[var(--ink-faint)]">
                还没有预设，先创建一个。
              </div>
            {:else}
              <div class="space-y-2">
                {#each filteredPresets as preset (preset.id)}
                  <Button
                    type="button"
                    variant="ghost"
                    size="md"
                    className={cn(
                      "h-auto w-full justify-start rounded-[var(--radius-lg)] border p-3 text-left transition-all hover:border-[var(--border-medium)] hover:shadow-[var(--shadow-sm)]",
                      selectedPreset === preset.id
                        ? "border-[var(--brand)] bg-[var(--brand-soft)] text-[var(--ink-strong)] hover:bg-[var(--brand-soft)]"
                        : "border-[var(--border-soft)] bg-[var(--bg-surface)] text-[var(--ink-strong)]"
                    )}
                    onclick={() => void openPreset(preset.id)}
                  >
                    <div class="flex items-start gap-3">
                      <div class="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-amber-400 to-orange-500 text-white shadow-sm">
                        <Sparkles size={18} />
                      </div>
                      <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                          <p class="truncate text-sm font-semibold text-[var(--ink-strong)]">{preset.name}</p>
                          {#if !preset.enabled}
                            <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">已停用</span>
                          {/if}
                        </div>
                        <p class="mt-1 line-clamp-2 text-xs leading-relaxed text-[var(--ink-faint)]">{preset.description || "无描述"}</p>
                      </div>
                      <span class="text-[10px] text-[var(--ink-faint)]">#{preset.sort_order}</span>
                    </div>
                  </Button>
                {/each}
              </div>
            {/if}
          </div>
        </Card>

        <Card className="min-h-[38rem] overflow-hidden">
          {#if !selectedPreset}
            <div class="flex h-full min-h-[38rem] items-center justify-center px-6 py-10">
              <div class="max-w-sm text-center">
                <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-[var(--radius-lg)] bg-[var(--brand-soft)] text-[var(--brand)] shadow-sm">
                  <Layers3 size={24} />
                </div>
                <h2 class="mt-4 text-base font-semibold text-[var(--ink-strong)]">选择一个预设</h2>
                <p class="mt-1 text-sm leading-relaxed text-[var(--ink-muted)]">从左侧选择预设后，可在这里维护基础信息、条目与渠道绑定。</p>
              </div>
            </div>
          {:else if loadingDetail}
            <div class="flex h-full min-h-[38rem] items-center justify-center px-6 py-10 text-sm text-[var(--ink-muted)]">
              正在读取预设详情...
            </div>
          {:else if !activePreset}
            <div class="flex h-full min-h-[38rem] items-center justify-center px-6 py-10 text-sm text-[var(--ink-faint)]">
              无法读取预设详情
            </div>
          {:else}
            <div class="flex h-full min-h-[38rem] flex-col">
              <div class="flex items-center justify-between gap-3 border-b border-[var(--border-soft)] px-5 py-4">
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <h2 class="truncate text-base font-semibold text-[var(--ink-strong)]">{activePreset.preset.name}</h2>
                    {#if !activePreset.preset.enabled}
                      <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">已停用</span>
                    {/if}
                  </div>
                  <p class="mt-1 text-xs text-[var(--ink-muted)]">在右侧持续编辑当前预设，切换左侧条目不会离开当前页面。</p>
                </div>
                <div class="flex items-center gap-2">
                  <Button type="button" variant="secondary" size="sm" onclick={closeDetail}>
                    取消选择
                  </Button>
                  <Button type="button" size="sm" onclick={() => void submitSavePreset()} disabled={!detailDirty || savingPreset}>
                    <Save size={12} /> {savingPreset ? "保存中..." : "保存"}
                  </Button>
                  <Button type="button" variant="ghost" size="sm" className="h-8 w-8 px-0 text-[var(--ink-faint)] hover:text-[var(--danger)]" onclick={() => void removeCurrentPreset()} disabled={deletingPreset || savingPreset}>
                    <Trash2 size={14} />
                  </Button>
                </div>
              </div>

              <div class="flex gap-1 border-b border-[var(--border-soft)] px-4 py-2">
                {#each tabs as tab}
                  <Button type="button" variant={tab.id === activeTab ? "default" : "ghost"} size="sm" className={cn("rounded-[var(--radius-full)]", tab.id !== activeTab && "text-[var(--ink-muted)]")} onclick={() => (activeTab = tab.id)}>
                    {tab.label}
                  </Button>
                {/each}
              </div>

              <div class="app-scrollbar flex-1 overflow-y-auto p-5">
                <div class="mx-auto max-w-3xl space-y-6">
                  {#if activeTab === "overview"}
                    <div class="space-y-4">
                      <label class={labelClass}>
                        <span class={labelTextClass}>名称</span>
                        <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.name} />
                      </label>
                      <label class={labelClass}>
                        <span class={labelTextClass}>描述</span>
                        <textarea rows="5" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.description}></textarea>
                      </label>
                      <div class="grid gap-4 sm:grid-cols-2">
                        <label class={labelClass}>
                          <span class={labelTextClass}>排序值</span>
                          <input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.sortOrder} />
                        </label>
                        <label class="flex items-center gap-2 pt-6 text-sm text-[var(--ink-body)]">
                          <input type="checkbox" bind:checked={detailDraft.enabled} />
                          启用该预设
                        </label>
                      </div>
                    </div>
                  {:else if activeTab === "entries"}
                    <div class="space-y-3">
                      <div class="flex items-center justify-between">
                        <div>
                          <h3 class="text-sm font-semibold text-[var(--ink-strong)]">条目列表</h3>
                          <p class="mt-1 text-xs text-[var(--ink-muted)]">按顺序组织系统提示、角色设定或上下文片段。</p>
                        </div>
                        <Button type="button" variant="secondary" size="sm" className="h-8 gap-1 px-3" onclick={openCreateEntryDialog}>
                          <Plus size={12} /> 添加条目
                        </Button>
                      </div>
                      {#if activePreset.entries.length === 0}
                        <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-6 text-center text-xs text-[var(--ink-faint)]">暂无条目</div>
                      {:else}
                        {#each activePreset.entries as entry, idx (entry.id)}
                          <div class={cn("rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4", movingEntryId === entry.id && "opacity-70")}>
                            <div class="mb-2 flex items-start justify-between gap-3">
                              <div class="min-w-0 flex-1">
                                <div class="flex flex-wrap items-center gap-2">
                                  <span class="text-sm font-semibold text-[var(--ink-strong)]">{entry.name}</span>
                                  <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">{entry.role}</span>
                                  <span class="rounded-[var(--radius-full)] bg-[var(--brand-soft)] px-1.5 py-0.5 text-[10px] text-[var(--brand)]">{entry.position_type}</span>
                                </div>
                                <p class="mt-1 text-xs leading-relaxed text-[var(--ink-muted)]">{readContentText(entry.primary_content) || "无内容"}</p>
                              </div>
                              <div class="flex items-center gap-1">
                                <ActionIconButton title="上移" disabled={idx === 0 || movingEntryId === entry.id} onClick={() => void moveEntry(entry.id, -1)}>
                                  <ArrowUp size={13} />
                                </ActionIconButton>
                                <ActionIconButton title="下移" disabled={idx === activePreset.entries.length - 1 || movingEntryId === entry.id} onClick={() => void moveEntry(entry.id, 1)}>
                                  <ArrowDown size={13} />
                                </ActionIconButton>
                                <ActionIconButton title="编辑条目" onClick={() => openEditEntryDialog(entry)}>
                                  <Edit3 size={13} />
                                </ActionIconButton>
                                <ActionIconButton title="删除条目" tone="danger" disabled={deletingEntryId === entry.id} onClick={() => void removeEntry(entry)}>
                                  <Trash2 size={13} />
                                </ActionIconButton>
                              </div>
                            </div>
                            <div class="flex flex-wrap items-center gap-2 text-[11px] text-[var(--ink-faint)]">
                              <span class="inline-flex items-center gap-1"><GripVertical size={11} /> #{entry.list_order}</span>
                              {#if entry.depth !== null}<span>深度 {entry.depth}</span>{/if}
                              <span>{entry.enabled ? "已启用" : "已停用"}</span>
                              {#if entry.is_pinned}<span>已固定</span>{/if}
                            </div>
                          </div>
                        {/each}
                      {/if}
                    </div>
                  {:else}
                    <div class="space-y-4">
                      <div>
                        <div class="mb-2 flex items-center gap-2">
                          <Link size={14} class="text-[var(--ink-faint)]" />
                          <h3 class="text-sm font-semibold text-[var(--ink-strong)]">渠道与模型绑定</h3>
                        </div>
                        <p class="mb-4 text-xs text-[var(--ink-muted)]">当使用此预设时，推荐使用的模型组合。仅作为提示作用，不强制要求。</p>
                        
                        <div class="mb-4 flex gap-2">
                          <select class="rounded-md border border-[var(--border-medium)] bg-[var(--bg-surface)] px-3 py-1.5 text-xs text-[var(--ink-body)] outline-none" bind:value={bindChannelId} onchange={() => bindModelId = ""}>
                            <option value="">选取渠道...</option>
                            {#each availableChannels as ch}
                              <option value={ch.id}>{ch.name}</option>
                            {/each}
                          </select>
                          {#if bindChannelId}
                            <select class="rounded-md border border-[var(--border-medium)] bg-[var(--bg-surface)] px-3 py-1.5 text-xs text-[var(--ink-body)] outline-none" bind:value={bindModelId}>
                              <option value="">(任意模型)</option>
                              {#if allModels[bindChannelId]}
                                {#each allModels[bindChannelId] as mod}
                                  <option value={mod.model_id}>{mod.display_name || mod.model_id}</option>
                                {/each}
                              {/if}
                            </select>
                          {/if}
                          <Button size="sm" variant="secondary" className="px-3" onclick={handleAddChannelBinding} disabled={!bindChannelId}>
                            <Plus size={14} class="mr-1" /> 添加
                          </Button>
                        </div>

                        {#if activePreset.channel_bindings.length > 0}
                          <div class="space-y-2">
                            {#each activePreset.channel_bindings as binding (binding.id)}
                              <div class="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2.5 text-sm text-[var(--ink-body)]">
                                <div>
                                  <span class="font-medium text-[var(--ink-strong)]">{allChannels.find(c=>c.id === binding.channel_id)?.name || binding.channel_id}</span>
                                  {#if binding.channel_model_id}
                                    <span class="text-[var(--ink-muted)] px-1">/</span>
                                    <span class="text-xs text-[var(--ink-muted)]">{binding.channel_model_id}</span>
                                  {/if}
                                </div>
                                <ActionIconButton tone="danger" onClick={() => handleRemoveChannelBinding(binding.id)}><Trash2 size={14}/></ActionIconButton>
                              </div>
                            {/each}
                          </div>
                        {:else}
                          <div class="rounded-[var(--radius-sm)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-4 text-center text-xs text-[var(--ink-faint)]">暂无渠道绑定</div>
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
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">创建预设</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">先创建预设主体，后续再补充条目和绑定。</p>
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
              <span class={labelTextClass}>排序值</span>
              <input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.sortOrder} />
            </label>
            <label class="flex items-center gap-2 pt-6 text-sm text-[var(--ink-body)]">
              <input type="checkbox" bind:checked={createDraft.enabled} />
              创建后立即启用
            </label>
          </div>
        </div>
      </div>
      <div class="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-6 py-4">
        <Button type="button" variant="secondary" onclick={() => (createDialogOpen = false)} disabled={savingPreset}>取消</Button>
        <Button type="button" onclick={() => void submitCreatePreset()} disabled={savingPreset || !createDraft.name.trim()}>
          {savingPreset ? "创建中..." : "创建预设"}
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
          <p class="mt-1 text-xs text-[var(--ink-muted)]">支持 system、user、assistant、tool 四类条目，正文统一走内容层。</p>
        </div>
        <Button type="button" variant="ghost" size="sm" className="h-9 w-9 px-0" onclick={() => (entryDialogOpen = false)}>
          <X size={16} />
        </Button>
      </div>
      <div class="app-scrollbar max-h-[72dvh] overflow-y-auto px-6 py-5">
        <div class="grid gap-4 md:grid-cols-2">
          <label class={labelClass}>
            <span class={labelTextClass}>名称</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.name} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>角色</span>
            <select class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.role}>
              <option value="system">system</option>
              <option value="user">user</option>
              <option value="assistant">assistant</option>
              <option value="tool">tool</option>
            </select>
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>插入位置</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.positionType} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>排序值</span>
            <input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.listOrder} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>深度</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.depth} placeholder="留空表示无深度限制" />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>深度排序</span>
            <input type="number" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.depthOrder} />
          </label>
          <label class="flex items-center gap-2 pt-6 text-sm text-[var(--ink-body)]">
            <input type="checkbox" bind:checked={entryDraft.enabled} />
            启用该条目
          </label>
          <label class="flex items-center gap-2 pt-6 text-sm text-[var(--ink-body)]">
            <input type="checkbox" bind:checked={entryDraft.isPinned} />
            固定条目
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>正文</span>
            <textarea rows="9" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={entryDraft.content}></textarea>
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
