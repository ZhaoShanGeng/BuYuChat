<script lang="ts">
  import { onMount } from "svelte";
  import { Dialog } from "bits-ui";
  import { toast } from "svelte-sonner";
  import { Bot, ChevronLeft, Edit3, Link, Plus, Save, Sparkles, Trash2, X } from "lucide-svelte";
  import {
    createAgent,
    createAgentGreeting,
    deleteAgent,
    deleteAgentGreeting,
    getAgentDetail,
    listAgents,
    updateAgent,
    updateAgentGreeting,
    type AgentDetail,
    type AgentGreetingDetail,
    type AgentSummary,
    type ContentWriteInput,
    type CreateAgentGreetingInput,
    type CreateAgentInput,
    type UpdateAgentInput
  } from "$lib/api/agents";
  import { i18n } from "$lib/i18n.svelte";
  import { cn } from "$lib/utils";
  import SearchField from "$components/shared/search-field.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import Button from "$components/ui/button.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import PageShell from "$components/layout/page-shell.svelte";

  type TabId = "profile" | "greetings" | "bindings";
  type AgentDraft = {
    name: string;
    title: string;
    description: string;
    personality: string;
    mainPrompt: string;
    enabled: boolean;
    sortOrder: number;
    talkativeness: number;
    avatarUri: string;
    creatorName: string;
    characterVersion: string;
  };
  type GreetingDraft = {
    greetingType: string;
    name: string;
    text: string;
    enabled: boolean;
    sortOrder: number;
  };

  let selectedAgent = $state<string | null>(null);
  let searchQuery = $state("");
  let agents = $state<AgentSummary[]>([]);
  let activeAgent = $state<AgentDetail | null>(null);
  let detailDraft = $state<AgentDraft>(emptyAgentDraft());
  let createDraft = $state<AgentDraft>(emptyAgentDraft());
  let greetingDraft = $state<GreetingDraft>(emptyGreetingDraft());
  let loadingList = $state(true);
  let loadingDetail = $state(false);
  let savingAgent = $state(false);
  let deletingAgent = $state(false);
  let savingGreeting = $state(false);
  let deletingGreetingId = $state<string | null>(null);
  let createDialogOpen = $state(false);
  let greetingDialogOpen = $state(false);
  let editingGreetingId = $state<string | null>(null);
  let activeTab = $state<TabId>("profile");

  const tabs: { id: TabId; label: string }[] = [
    { id: "profile", label: "角色设定" },
    { id: "greetings", label: "问候语" },
    { id: "bindings", label: "资源绑定" }
  ];

  const filteredAgents = $derived(
    searchQuery
      ? agents.filter((agent) =>
          `${agent.name} ${agent.title ?? ""}`.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : agents
  );

  const detailDirty = $derived(
    activeAgent ? JSON.stringify(mapAgentToDraft(activeAgent)) !== JSON.stringify(detailDraft) : false
  );

  const agentPalette = [
    "from-blue-500 to-indigo-500",
    "from-violet-500 to-fuchsia-500",
    "from-emerald-500 to-teal-500",
    "from-amber-500 to-orange-500"
  ];
  const labelClass = "space-y-1";
  const labelTextClass = "text-xs font-medium text-[var(--ink-muted)]";

  onMount(() => {
    void loadAgents();
  });

  function emptyAgentDraft(sortOrder = 0): AgentDraft {
    return {
      name: "",
      title: "",
      description: "",
      personality: "",
      mainPrompt: "",
      enabled: true,
      sortOrder,
      talkativeness: 50,
      avatarUri: "",
      creatorName: "",
      characterVersion: ""
    };
  }

  function emptyGreetingDraft(sortOrder = 0): GreetingDraft {
    return {
      greetingType: "default",
      name: "",
      text: "",
      enabled: true,
      sortOrder
    };
  }

  function normalizeNullable(value: string) {
    const trimmed = value.trim();
    return trimmed ? trimmed : null;
  }

  function readContentText(content?: { text_content: string | null; preview_text: string | null } | null) {
    return content?.text_content?.trim() || content?.preview_text?.trim() || "";
  }

  function toContentInput(text: string): ContentWriteInput | null {
    const trimmed = text.trim();
    if (!trimmed) return null;
    return {
      content_type: "text",
      mime_type: "text/plain",
      text_content: trimmed,
      source_file_path: null,
      primary_storage_uri: null,
      size_bytes_hint: null,
      preview_text: trimmed.slice(0, 160),
      config_json: {}
    };
  }

  function shortId(value: string) {
    return value.length > 12 ? `${value.slice(0, 6)}…${value.slice(-4)}` : value;
  }

  function formatError(error: unknown) {
    if (error instanceof Error && error.message) return error.message;
    if (typeof error === "string") return error;
    return "请求失败，请稍后重试";
  }

  function getAgentColor(id: string) {
    const seed = Array.from(id).reduce((sum, ch) => sum + ch.charCodeAt(0), 0);
    return agentPalette[seed % agentPalette.length];
  }

  function mapAgentToDraft(agent: AgentDetail): AgentDraft {
    return {
      name: agent.summary.name,
      title: agent.summary.title ?? "",
      description: readContentText(agent.description_content),
      personality: readContentText(agent.personality_content),
      mainPrompt: readContentText(agent.main_prompt_override_content),
      enabled: agent.summary.enabled,
      sortOrder: agent.summary.sort_order,
      talkativeness: agent.talkativeness,
      avatarUri: agent.summary.avatar_uri ?? "",
      creatorName: agent.creator_name ?? "",
      characterVersion: agent.character_version ?? ""
    };
  }

  function mapGreetingToDraft(greeting: AgentGreetingDetail): GreetingDraft {
    return {
      greetingType: greeting.greeting_type,
      name: greeting.name ?? "",
      text: readContentText(greeting.primary_content),
      enabled: greeting.enabled,
      sortOrder: greeting.sort_order
    };
  }

  function buildAgentInput(draft: AgentDraft, source?: AgentDetail | null): CreateAgentInput | UpdateAgentInput {
    return {
      name: draft.name.trim() || "未命名智能体",
      title: normalizeNullable(draft.title),
      description_content: toContentInput(draft.description),
      personality_content: toContentInput(draft.personality),
      scenario_content: source ? toContentInput(readContentText(source.scenario_content)) : null,
      example_messages_content: source ? toContentInput(readContentText(source.example_messages_content)) : null,
      main_prompt_override_content: toContentInput(draft.mainPrompt),
      post_history_instructions_content: source ? toContentInput(readContentText(source.post_history_instructions_content)) : null,
      character_note_content: source ? toContentInput(readContentText(source.character_note_content)) : null,
      creator_notes_content: source ? toContentInput(readContentText(source.creator_notes_content)) : null,
      character_note_depth: source?.character_note_depth ?? null,
      character_note_role: source?.character_note_role ?? null,
      talkativeness: Number.isFinite(draft.talkativeness) ? draft.talkativeness : 50,
      avatar_uri: normalizeNullable(draft.avatarUri),
      creator_name: normalizeNullable(draft.creatorName),
      character_version: normalizeNullable(draft.characterVersion),
      enabled: draft.enabled,
      sort_order: Number.isFinite(draft.sortOrder) ? draft.sortOrder : 0,
      config_json: source?.config_json ?? {}
    };
  }

  function buildGreetingInput(draft: GreetingDraft): CreateAgentGreetingInput {
    return {
      greeting_type: draft.greetingType.trim() || "default",
      name: normalizeNullable(draft.name),
      primary_content: toContentInput(draft.text) ?? {
        content_type: "text",
        mime_type: "text/plain",
        text_content: "",
        source_file_path: null,
        primary_storage_uri: null,
        size_bytes_hint: null,
        preview_text: null,
        config_json: {}
      },
      enabled: draft.enabled,
      sort_order: Number.isFinite(draft.sortOrder) ? draft.sortOrder : 0,
      config_json: {}
    };
  }

  async function loadAgents(preferredSelection?: string | null) {
    loadingList = true;
    try {
      const items = await listAgents();
      agents = [...items].sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
      if (preferredSelection !== undefined) selectedAgent = preferredSelection;
    } catch (error) {
      console.error("Failed to load agents:", error);
      agents = [];
    } finally {
      loadingList = false;
    }
  }

  async function openAgent(id: string) {
    selectedAgent = id;
    activeTab = "profile";
    loadingDetail = true;
    try {
      activeAgent = await getAgentDetail(id);
      detailDraft = mapAgentToDraft(activeAgent);
    } catch (error) {
      console.error("Failed to load agent detail:", error);
      activeAgent = null;
      toast.error("读取智能体失败", { description: formatError(error) });
    } finally {
      loadingDetail = false;
    }
  }

  function closeDetail() {
    selectedAgent = null;
    activeAgent = null;
    detailDraft = emptyAgentDraft();
    activeTab = "profile";
  }

  function openCreateDialog() {
    createDraft = emptyAgentDraft(agents.length);
    createDialogOpen = true;
  }

  async function submitCreateAgent() {
    savingAgent = true;
    try {
      const created = await createAgent(buildAgentInput(createDraft));
      createDialogOpen = false;
      await loadAgents(created.summary.id);
      await openAgent(created.summary.id);
      toast.success("智能体已创建", { description: created.summary.name });
    } catch (error) {
      console.error("Failed to create agent:", error);
      toast.error("创建智能体失败", { description: formatError(error) });
    } finally {
      savingAgent = false;
    }
  }

  async function submitSaveAgent() {
    if (!selectedAgent || !activeAgent) return;
    savingAgent = true;
    try {
      const updated = await updateAgent(selectedAgent, buildAgentInput(detailDraft, activeAgent));
      activeAgent = updated;
      detailDraft = mapAgentToDraft(updated);
      await loadAgents(selectedAgent);
      toast.success("智能体已保存", { description: updated.summary.name });
    } catch (error) {
      console.error("Failed to update agent:", error);
      toast.error("保存智能体失败", { description: formatError(error) });
    } finally {
      savingAgent = false;
    }
  }

  async function removeCurrentAgent() {
    if (!selectedAgent || !activeAgent) return;
    if (!confirm(`确定删除智能体“${activeAgent.summary.name}”吗？`)) return;
    deletingAgent = true;
    try {
      await deleteAgent(selectedAgent);
      toast.success("智能体已删除", { description: activeAgent.summary.name });
      closeDetail();
      await loadAgents(null);
    } catch (error) {
      console.error("Failed to delete agent:", error);
      toast.error("删除智能体失败", { description: formatError(error) });
    } finally {
      deletingAgent = false;
    }
  }

  function openCreateGreetingDialog() {
    if (!activeAgent) return;
    editingGreetingId = null;
    greetingDraft = emptyGreetingDraft(activeAgent.greetings.length);
    greetingDialogOpen = true;
  }

  function openEditGreetingDialog(greeting: AgentGreetingDetail) {
    editingGreetingId = greeting.id;
    greetingDraft = mapGreetingToDraft(greeting);
    greetingDialogOpen = true;
  }

  async function submitGreeting() {
    if (!selectedAgent) return;
    savingGreeting = true;
    try {
      if (editingGreetingId) {
        await updateAgentGreeting(editingGreetingId, buildGreetingInput(greetingDraft));
        toast.success("问候语已保存");
      } else {
        await createAgentGreeting(selectedAgent, buildGreetingInput(greetingDraft));
        toast.success("问候语已添加");
      }
      greetingDialogOpen = false;
      await openAgent(selectedAgent);
    } catch (error) {
      console.error("Failed to save greeting:", error);
      toast.error("保存问候语失败", { description: formatError(error) });
    } finally {
      savingGreeting = false;
    }
  }

  async function removeGreeting(greeting: AgentGreetingDetail) {
    if (!confirm(`确定删除问候语“${greeting.name || greeting.id}”吗？`)) return;
    deletingGreetingId = greeting.id;
    try {
      await deleteAgentGreeting(greeting.id);
      toast.success("问候语已删除");
      if (selectedAgent) await openAgent(selectedAgent);
    } catch (error) {
      console.error("Failed to delete greeting:", error);
      toast.error("删除问候语失败", { description: formatError(error) });
    } finally {
      deletingGreetingId = null;
    }
  }
</script>

{#if !selectedAgent}
  <PageShell>
    {#snippet header()}
      <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4" data-tauri-drag-region>
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.agents")}</h1>
        <HeaderWindowGroup>
          {#snippet children()}
            <Button type="button" size="sm" onclick={openCreateDialog}>
              <Plus size={14} /> {i18n.t("agents.create")}
            </Button>
          {/snippet}
        </HeaderWindowGroup>
      </header>
    {/snippet}

    {#snippet toolbar()}
      <div class="border-b border-[var(--border-soft)] px-4 py-3">
        <SearchField bind:value={searchQuery} placeholder={i18n.t("agents.search")} />
      </div>
    {/snippet}

    {#snippet body()}
      <div class="app-scrollbar h-full overflow-y-auto p-4">
        {#if loadingList}
          <div class="mx-auto max-w-3xl rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-10 text-center text-sm text-[var(--ink-muted)]">
            正在读取智能体...
          </div>
        {:else}
          <div class="mx-auto grid max-w-4xl gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {#each filteredAgents as agent (agent.id)}
              <button
                type="button"
                class="suggestion-card flex flex-col gap-3 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4 text-left transition-shadow hover:shadow-[var(--shadow-md)]"
                onclick={() => void openAgent(agent.id)}
              >
                <div class="flex items-center gap-3">
                  <div class={`flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br ${getAgentColor(agent.id)} text-lg font-bold text-white shadow-sm`}>
                    {agent.name.charAt(0)}
                  </div>
                  <div class="min-w-0">
                    <h3 class="truncate text-sm font-semibold text-[var(--ink-strong)]">{agent.name}</h3>
                    <p class="mt-0.5 text-xs text-[var(--ink-faint)]">{agent.title || "未设置标题"}</p>
                  </div>
                </div>
                <div class="flex items-center gap-2 text-[11px] text-[var(--ink-faint)]">
                  <span>{agent.enabled ? "已启用" : "已停用"}</span>
                  <span>·</span>
                  <span>#{agent.sort_order}</span>
                </div>
              </button>
            {/each}

            <Button
              type="button"
              variant="ghost"
              size="lg"
              className="h-auto flex-col gap-2 rounded-[var(--radius-lg)] border-2 border-dashed border-[var(--border-medium)] bg-transparent px-4 py-8 text-center hover:border-[var(--brand)] hover:bg-[var(--brand-soft)]"
              onclick={openCreateDialog}
            >
              <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[var(--bg-hover)]">
                <Plus size={20} class="text-[var(--ink-faint)]" />
              </div>
              <span class="text-xs font-medium text-[var(--ink-muted)]">{i18n.t("agents.create_card")}</span>
            </Button>
          </div>
        {/if}
      </div>
    {/snippet}
  </PageShell>
{:else}
  <PageShell>
    {#snippet header()}
      <header class="flex h-12 items-center gap-3 border-b border-[var(--border-soft)] px-4">
        <Button type="button" variant="ghost" size="sm" className="h-8 w-8 px-0" onclick={closeDetail}>
          <ChevronLeft size={18} />
        </Button>
        <div class="flex items-center gap-2">
          <div class={`flex h-7 w-7 items-center justify-center rounded-[var(--radius-sm)] bg-gradient-to-br ${selectedAgent ? getAgentColor(selectedAgent) : agentPalette[0]} text-xs font-bold text-white`}>
            {activeAgent?.summary.name.charAt(0) ?? "A"}
          </div>
          <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{activeAgent?.summary.name ?? "智能体"}</h1>
        </div>
        <div class="flex-1"></div>
        <HeaderWindowGroup>
          {#snippet children()}
            <Button type="button" size="sm" onclick={() => void submitSaveAgent()} disabled={!detailDirty || savingAgent || loadingDetail}>
              <Save size={12} /> {savingAgent ? "保存中..." : "保存"}
            </Button>
            <Button type="button" variant="ghost" size="sm" className="h-8 w-8 px-0 text-[var(--ink-faint)] hover:text-[var(--danger)]" onclick={() => void removeCurrentAgent()} disabled={deletingAgent || savingAgent}>
              <Trash2 size={14} />
            </Button>
          {/snippet}
        </HeaderWindowGroup>
      </header>
    {/snippet}

    {#snippet toolbar()}
      <div class="flex gap-1 border-b border-[var(--border-soft)] px-4 py-2">
        {#each tabs as tab}
          <Button type="button" variant={tab.id === activeTab ? "default" : "ghost"} size="sm" className={cn("rounded-[var(--radius-full)]", tab.id !== activeTab && "text-[var(--ink-muted)]")} onclick={() => { activeTab = tab.id; }}>
            {tab.label}
          </Button>
        {/each}
      </div>
    {/snippet}

    {#snippet body()}
      <div class="app-scrollbar h-full overflow-y-auto p-6">
        <div class="mx-auto max-w-2xl space-y-6">
          {#if loadingDetail}
            <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-10 text-center text-sm text-[var(--ink-muted)]">正在读取智能体详情...</div>
          {:else if !activeAgent}
            <div class="rounded-[var(--radius-lg)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-surface)] px-4 py-10 text-center text-sm text-[var(--ink-faint)]">无法读取智能体详情</div>
          {:else if activeTab === "profile"}
            <div class="space-y-4">
              <div>
                <label for="agent-name" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">名称</label>
                <input id="agent-name" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" bind:value={detailDraft.name} />
              </div>
              <div>
                <label for="agent-title" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">标题</label>
                <input id="agent-title" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" bind:value={detailDraft.title} />
              </div>
              <div>
                <label for="agent-description" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">描述</label>
                <textarea id="agent-description" rows="5" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" bind:value={detailDraft.description}></textarea>
              </div>
              <div>
                <label for="agent-main-prompt" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">系统提示</label>
                <textarea id="agent-main-prompt" rows="6" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" bind:value={detailDraft.mainPrompt}></textarea>
              </div>
              <div>
                <label for="agent-personality" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">角色性格</label>
                <textarea id="agent-personality" rows="4" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" bind:value={detailDraft.personality}></textarea>
              </div>
              <div class="grid gap-4 sm:grid-cols-2">
                <label class="space-y-1">
                  <span class="text-xs font-medium text-[var(--ink-muted)]">创作者</span>
                  <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.creatorName} />
                </label>
                <label class="space-y-1">
                  <span class="text-xs font-medium text-[var(--ink-muted)]">角色版本</span>
                  <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.characterVersion} />
                </label>
                <label class="space-y-1">
                  <span class="text-xs font-medium text-[var(--ink-muted)]">头像 URI</span>
                  <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={detailDraft.avatarUri} />
                </label>
                <label class="space-y-1">
                  <span class="text-xs font-medium text-[var(--ink-muted)]">排序值</span>
                  <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" type="number" bind:value={detailDraft.sortOrder} />
                </label>
              </div>
              <label class="flex items-center gap-2 text-sm text-[var(--ink-body)]">
                <input type="checkbox" bind:checked={detailDraft.enabled} />
                启用该智能体
              </label>
            </div>
          {:else if activeTab === "greetings"}
            <div class="space-y-3">
              <div class="flex items-center justify-between">
                <h3 class="text-sm font-semibold text-[var(--ink-strong)]">问候语列表</h3>
                <Button type="button" variant="secondary" size="sm" className="h-7 gap-1 px-2 text-[var(--brand)]" onclick={openCreateGreetingDialog}>
                  <Plus size={12} /> 添加
                </Button>
              </div>
              {#if activeAgent.greetings.length === 0}
                <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-6 text-center text-xs text-[var(--ink-faint)]">暂无问候语</div>
              {:else}
                {#each activeAgent.greetings as greeting, idx (greeting.id)}
                  <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
                    <div class="mb-2 flex items-center justify-between">
                      <span class="text-xs font-medium text-[var(--ink-faint)]">{greeting.name || `问候语 #${idx + 1}`}</span>
                      <div class="flex items-center gap-1">
                        <ActionIconButton title="编辑问候语" onClick={() => openEditGreetingDialog(greeting)}>
                          <Edit3 size={13} />
                        </ActionIconButton>
                        <ActionIconButton title="删除问候语" tone="danger" disabled={deletingGreetingId === greeting.id} onClick={() => void removeGreeting(greeting)}>
                          <Trash2 size={13} />
                        </ActionIconButton>
                      </div>
                    </div>
                    <p class="text-sm leading-relaxed text-[var(--ink-body)]">{readContentText(greeting.primary_content) || "无内容"}</p>
                  </div>
                {/each}
              {/if}
            </div>
          {:else}
            <div class="space-y-6">
              {#each [
                { title: "预设绑定", icon: Sparkles, items: activeAgent.preset_bindings.map((item) => shortId(item.resource_id)) },
                { title: "世界书绑定", icon: Link, items: activeAgent.lorebook_bindings.map((item) => shortId(item.resource_id)) },
                { title: "API 渠道", icon: Bot, items: activeAgent.channel_bindings.map((item) => shortId(item.channel_model_id ?? item.channel_id)) }
              ] as section}
                <div>
                  <div class="mb-2 flex items-center gap-2">
                    <section.icon size={14} class="text-[var(--ink-faint)]" />
                    <h3 class="text-sm font-semibold text-[var(--ink-strong)]">{section.title}</h3>
                  </div>
                  {#if section.items.length > 0}
                    <div class="space-y-2">
                      {#each section.items as item}
                        <div class="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2.5">
                          <span class="text-sm text-[var(--ink-body)]">{item}</span>
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <div class="rounded-[var(--radius-sm)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-4 text-center text-xs text-[var(--ink-faint)]">暂无绑定</div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/snippet}
  </PageShell>
{/if}

<Dialog.Root bind:open={createDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/20 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(760px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-lg)] outline-none">
      <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-4">
        <div>
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">创建智能体</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">使用真实后端创建角色卡主体信息，后续可继续补充绑定资源与媒体。</p>
        </div>
        <Button type="button" variant="ghost" size="sm" className="h-9 w-9 px-0" onclick={() => (createDialogOpen = false)}>
          <X size={16} />
        </Button>
      </div>

      <div class="app-scrollbar max-h-[72dvh] overflow-y-auto px-6 py-5">
        <div class="grid gap-4 md:grid-cols-2">
          <label class={labelClass}>
            <span class={labelTextClass}>名称</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.name} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>标题</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.title} />
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>描述</span>
            <textarea rows="4" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.description}></textarea>
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>系统提示</span>
            <textarea rows="5" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.mainPrompt}></textarea>
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>角色性格</span>
            <textarea rows="4" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.personality}></textarea>
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>创作者</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.creatorName} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>角色版本</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.characterVersion} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>头像 URI</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={createDraft.avatarUri} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>排序值</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" type="number" bind:value={createDraft.sortOrder} />
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>话痨程度</span>
            <input class="w-full" type="range" min="0" max="100" bind:value={createDraft.talkativeness} />
            <div class="mt-1 text-xs text-[var(--ink-faint)]">{createDraft.talkativeness}</div>
          </label>
          <label class={`${labelClass} flex items-center gap-2 md:col-span-2`}>
            <input type="checkbox" bind:checked={createDraft.enabled} />
            <span class="text-sm text-[var(--ink-body)]">创建后立即启用</span>
          </label>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-6 py-4">
        <Button type="button" variant="secondary" onclick={() => (createDialogOpen = false)} disabled={savingAgent}>取消</Button>
        <Button type="button" onclick={() => void submitCreateAgent()} disabled={savingAgent || !createDraft.name.trim()}>
          {savingAgent ? "创建中..." : "创建智能体"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<Dialog.Root bind:open={greetingDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/20 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(720px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-lg)] outline-none">
      <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-4">
        <div>
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">{editingGreetingId ? "编辑问候语" : "添加问候语"}</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">支持为智能体配置多条开场白，后续可按类型与排序值控制生效顺序。</p>
        </div>
        <Button type="button" variant="ghost" size="sm" className="h-9 w-9 px-0" onclick={() => (greetingDialogOpen = false)}>
          <X size={16} />
        </Button>
      </div>

      <div class="app-scrollbar max-h-[72dvh] overflow-y-auto px-6 py-5">
        <div class="grid gap-4 md:grid-cols-2">
          <label class={labelClass}>
            <span class={labelTextClass}>问候语类型</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={greetingDraft.greetingType} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>名称</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={greetingDraft.name} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>排序值</span>
            <input class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" type="number" bind:value={greetingDraft.sortOrder} />
          </label>
          <label class={`${labelClass} flex items-center gap-2 pt-6`}>
            <input type="checkbox" bind:checked={greetingDraft.enabled} />
            <span class="text-sm text-[var(--ink-body)]">启用这条问候语</span>
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>内容</span>
            <textarea rows="8" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)]" bind:value={greetingDraft.text}></textarea>
          </label>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-6 py-4">
        <Button type="button" variant="secondary" onclick={() => (greetingDialogOpen = false)} disabled={savingGreeting}>取消</Button>
        <Button type="button" onclick={() => void submitGreeting()} disabled={savingGreeting}>
          {savingGreeting ? "保存中..." : editingGreetingId ? "保存问候语" : "添加问候语"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
