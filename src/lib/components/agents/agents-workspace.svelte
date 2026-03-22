<script lang="ts">
  import { onMount } from "svelte";
  import { Dialog } from "bits-ui";
  import { toast } from "svelte-sonner";
  import { Bot, ChevronLeft, ChevronDown, Edit3, Link, Plus, Save, Sparkles, Trash2, X, Settings2 } from "lucide-svelte";
  import {
    createAgent, createAgentGreeting, deleteAgent, deleteAgentGreeting, getAgentDetail, listAgents,
    updateAgent, updateAgentGreeting, replaceAgentPresets, replaceAgentLorebooks, replaceAgentChannels, replaceAgentUserProfiles,
    type AgentDetail, type AgentGreetingDetail, type AgentSummary, type ContentWriteInput,
    type CreateAgentInput, type UpdateAgentInput, type CreateAgentGreetingInput
  } from "$lib/api/agents";
  import { listPresets, type PresetSummary } from "$lib/api/presets";
  import { listLorebooks, type LorebookSummary } from "$lib/api/lorebooks";
  import { listApiChannels, listApiChannelModels, type ApiChannelModel } from "$lib/api/api-channels";
  import { i18n } from "$lib/i18n.svelte";
  import { cn } from "$lib/utils";
  import SearchField from "$components/shared/search-field.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import Button from "$components/ui/button.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import PageShell from "$components/layout/page-shell.svelte";
  import SelectDropdown from "$components/shared/select-dropdown.svelte";

  type TabId = "profile" | "persona" | "greetings" | "bindings" | "advanced";

  const tabs: { id: TabId; label: string }[] = [
    { id: "profile", label: "基础信息" },
    { id: "persona", label: "人设内容" },
    { id: "greetings", label: "问候语" },
    { id: "bindings", label: "绑定关系" },
    { id: "advanced", label: "高级设置" }
  ];

  let selectedAgent = $state<string | null>(null);
  let agents = $state<AgentSummary[]>([]);
  let activeAgent = $state<AgentDetail | null>(null);
  let searchQuery = $state("");
  let loadingList = $state(true);
  let loadingDetail = $state(false);
  let savingAgent = $state(false);
  let activeTab = $state<TabId>("profile");
  let createDialogOpen = $state(false);
  
  // Available resources for bindings
  let presetOptions = $state<{value: string; label: string}[]>([]);
  let lorebookOptions = $state<{value: string; label: string}[]>([]);
  let channelOptions = $state<{value: string; label: string}[]>([]);
  let selectedPresetToAdd = $state<string | undefined>();
  let selectedLorebookToAdd = $state<string | undefined>();
  let selectedChannelToAdd = $state<string | undefined>();

  const filteredAgents = $derived(
    searchQuery ? agents.filter(a => `${a.name} ${a.title ?? ""}`.toLowerCase().includes(searchQuery.toLowerCase())) : agents
  );

  type AgentDraft = {
    name: string; title: string; avatarUri: string; creatorName: string; characterVersion: string; talkativeness: number; enabled: boolean; sortOrder: number;
    description: string; personality: string; scenario: string; exampleMessages: string; mainPrompt: string; postHistory: string; characterNote: string; creatorNotes: string;
    characterNoteDepth: number | null; characterNoteRole: string | null; configJsonStr: string;
  };
  let detailDraft = $state<AgentDraft>(emptyAgentDraft());
  let createDraft = $state<AgentDraft>(emptyAgentDraft());

  let greetingDraft = $state({ greetingType: "default", name: "", text: "", enabled: true, sortOrder: 0 });
  let greetingDialogOpen = $state(false);
  let editingGreetingId = $state<string | null>(null);

  // 折叠状态控制
  let expandedSections = $state<Record<string, boolean>>({
    description: true, personality: false, scenario: false, exampleMessages: false,
    mainPrompt: false, postHistory: false, characterNote: false, creatorNotes: false
  });

  const detailDirty = $derived(activeAgent ? JSON.stringify(mapAgentToDraft(activeAgent)) !== JSON.stringify(detailDraft) : false);

  function emptyAgentDraft(sortOrder = 0): AgentDraft {
    return { name: "", title: "", avatarUri: "", creatorName: "", characterVersion: "", talkativeness: 50, enabled: true, sortOrder,
      description: "", personality: "", scenario: "", exampleMessages: "", mainPrompt: "", postHistory: "", characterNote: "", creatorNotes: "",
      characterNoteDepth: null, characterNoteRole: "system", configJsonStr: "{}" };
  }

  function readContentText(c?: { text_content: string | null; preview_text: string | null } | null) {
    return c?.text_content?.trim() || c?.preview_text?.trim() || "";
  }
  function toContent(text: string): ContentWriteInput | null {
    const t = text.trim(); if (!t) return null;
    return { content_type: "text", mime_type: "text/plain", text_content: t, source_file_path: null, primary_storage_uri: null, size_bytes_hint: null, preview_text: t.slice(0, 160), config_json: {} };
  }

  function mapAgentToDraft(a: AgentDetail): AgentDraft {
    return {
      name: a.summary.name, title: a.summary.title ?? "", avatarUri: a.summary.avatar_uri ?? "", creatorName: a.creator_name ?? "", characterVersion: a.character_version ?? "", talkativeness: a.talkativeness, enabled: a.summary.enabled, sortOrder: a.summary.sort_order,
      description: readContentText(a.description_content), personality: readContentText(a.personality_content), scenario: readContentText(a.scenario_content), exampleMessages: readContentText(a.example_messages_content), mainPrompt: readContentText(a.main_prompt_override_content), postHistory: readContentText(a.post_history_instructions_content), characterNote: readContentText(a.character_note_content), creatorNotes: readContentText(a.creator_notes_content),
      characterNoteDepth: a.character_note_depth, characterNoteRole: a.character_note_role || "system", configJsonStr: JSON.stringify(a.config_json || {}, null, 2)
    };
  }

  function buildAgentInput(draft: AgentDraft, src?: AgentDetail | null): CreateAgentInput {
    let cfg = {}; try { cfg = JSON.parse(draft.configJsonStr); } catch(e) {}
    return {
      name: draft.name.trim() || "未命名", title: draft.title.trim() || null, avatar_uri: draft.avatarUri.trim() || null, creator_name: draft.creatorName.trim() || null, character_version: draft.characterVersion.trim() || null, talkativeness: Number(draft.talkativeness) || 50, enabled: draft.enabled, sort_order: Number(draft.sortOrder) || 0,
      description_content: toContent(draft.description), personality_content: toContent(draft.personality), scenario_content: toContent(draft.scenario), example_messages_content: toContent(draft.exampleMessages), main_prompt_override_content: toContent(draft.mainPrompt), post_history_instructions_content: toContent(draft.postHistory), character_note_content: toContent(draft.characterNote), creator_notes_content: toContent(draft.creatorNotes),
      character_note_depth: draft.characterNoteDepth ? Number(draft.characterNoteDepth) : null, character_note_role: draft.characterNoteRole || null, config_json: cfg
    };
  }

  onMount(async () => {
    await loadAgents();
    // Load options for bindings
    Promise.all([listPresets(), listLorebooks(), listApiChannels()]).then(async ([ps, ls, cs]) => {
      presetOptions = ps.map(p => ({ value: p.id, label: p.name }));
      lorebookOptions = ls.map(l => ({ value: l.id, label: l.name }));
      let allChanModels: {value: string, label: string}[] = [];
      for (const c of cs) {
        allChanModels.push({ value: `${c.id}|`, label: `${c.name} (默认)` });
        try {
          const models = await listApiChannelModels(c.id);
          for (const m of models) allChanModels.push({ value: `${c.id}|${m.model_id}`, label: `${c.name} / ${m.display_name || m.model_id}` });
        } catch(e) {}
      }
      channelOptions = allChanModels;
    });
  });

  async function loadAgents(sel?: string | null) {
    loadingList = true;
    try {
      const items = await listAgents();
      agents = items.sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
      if (sel !== undefined) selectedAgent = sel;
    } catch (e) { toast.error("加载列表失败"); } finally { loadingList = false; }
  }

  async function openAgent(id: string) {
    selectedAgent = id; activeTab = "profile"; loadingDetail = true;
    try {
      activeAgent = await getAgentDetail(id);
      detailDraft = mapAgentToDraft(activeAgent);
    } catch (e) { toast.error("读取失败"); activeAgent = null; } finally { loadingDetail = false; }
  }

  async function saveAgent() {
    if (!selectedAgent || !activeAgent) return;
    savingAgent = true;
    try {
      activeAgent = await updateAgent(selectedAgent, buildAgentInput(detailDraft, activeAgent));
      detailDraft = mapAgentToDraft(activeAgent);
      await loadAgents(selectedAgent);
      toast.success("保存成功");
    } catch (e) { toast.error("保存失败"); } finally { savingAgent = false; }
  }

  async function createNewAgent() {
    savingAgent = true;
    try {
      const created = await createAgent(buildAgentInput(createDraft));
      createDialogOpen = false;
      await loadAgents(created.summary.id);
      await openAgent(created.summary.id);
      toast.success("创建成功");
    } catch (e) { toast.error("创建失败"); } finally { savingAgent = false; }
  }

  async function removeAgent() {
    if (!selectedAgent || !confirm("确定删除？")) return;
    try {
      await deleteAgent(selectedAgent);
      selectedAgent = null; activeAgent = null;
      await loadAgents(null);
      toast.success("已删除");
    } catch(e) { toast.error("删除失败"); }
  }

  async function saveGreeting() {
    if (!selectedAgent) return;
    try {
      const inp = { greeting_type: greetingDraft.greetingType || "default", name: greetingDraft.name || null, primary_content: toContent(greetingDraft.text) || {content_type:"text", mime_type:"text/plain",text_content:"",source_file_path:null,primary_storage_uri:null,size_bytes_hint:null,preview_text:null,config_json:{}}, enabled: greetingDraft.enabled, sort_order: Number(greetingDraft.sortOrder)||0, config_json:{} };
      if (editingGreetingId) await updateAgentGreeting(editingGreetingId, inp);
      else await createAgentGreeting(selectedAgent, inp);
      toast.success("问候语保存成功");
      greetingDialogOpen = false;
      await openAgent(selectedAgent);
    } catch(e) { toast.error("保存失败"); }
  }

  async function removeGreeting(gid: string) {
    if(!confirm("确定删除该问候语？")) return;
    try { await deleteAgentGreeting(gid); toast.success("已删除"); await openAgent(selectedAgent!); } catch(e) { toast.error("删除失败"); }
  }

  function getColor(name: string) {
    const colors = ["from-blue-500 to-indigo-500", "from-violet-500 to-fuchsia-500", "from-emerald-500 to-teal-500", "from-amber-500 to-orange-500", "from-rose-500 to-pink-500"];
    return colors[name.charCodeAt(0) % colors.length || 0];
  }

  // --- Binding Handlers ---
  async function addPresetBinding() {
    if (!selectedPresetToAdd || !activeAgent) return;
    try {
      const exist = activeAgent.preset_bindings.map(b => ({resource_id: b.resource_id, binding_type: b.binding_type, enabled: b.enabled, sort_order: b.sort_order}));
      if (exist.some(e => e.resource_id === selectedPresetToAdd)) return;
      await replaceAgentPresets(activeAgent.summary.id, [...exist, {resource_id: selectedPresetToAdd, binding_type: "preset", enabled: true, sort_order: exist.length}]);
      toast.success("添加成功"); selectedPresetToAdd = undefined; await openAgent(activeAgent.summary.id);
    } catch(e) { toast.error("添加失败"); }
  }
  async function removePresetBinding(rid: string) {
    if (!activeAgent) return;
    try {
      const remain = activeAgent.preset_bindings.filter(b => b.resource_id !== rid).map(b => ({resource_id: b.resource_id, binding_type: b.binding_type, enabled: b.enabled, sort_order: b.sort_order}));
      await replaceAgentPresets(activeAgent.summary.id, remain); await openAgent(activeAgent.summary.id);
    } catch(e) { toast.error("移除失败"); }
  }
  async function addLorebookBinding() {
    if (!selectedLorebookToAdd || !activeAgent) return;
    try {
      const exist = activeAgent.lorebook_bindings.map(b => ({resource_id: b.resource_id, binding_type: b.binding_type, enabled: b.enabled, sort_order: b.sort_order}));
      if (exist.some(e => e.resource_id === selectedLorebookToAdd)) return;
      await replaceAgentLorebooks(activeAgent.summary.id, [...exist, {resource_id: selectedLorebookToAdd, binding_type: "lorebook", enabled: true, sort_order: exist.length}]);
      toast.success("添加成功"); selectedLorebookToAdd = undefined; await openAgent(activeAgent.summary.id);
    } catch(e) { toast.error("添加失败"); }
  }
  async function removeLorebookBinding(rid: string) {
    if (!activeAgent) return;
    try {
      const remain = activeAgent.lorebook_bindings.filter(b => b.resource_id !== rid).map(b => ({resource_id: b.resource_id, binding_type: b.binding_type, enabled: b.enabled, sort_order: b.sort_order}));
      await replaceAgentLorebooks(activeAgent.summary.id, remain); await openAgent(activeAgent.summary.id);
    } catch(e) { toast.error("移除失败"); }
  }
  async function addChannelBinding() {
    if (!selectedChannelToAdd || !activeAgent) return;
    const [cid, cmid] = selectedChannelToAdd.split("|");
    try {
      const exist = activeAgent.channel_bindings.map(b => ({channel_id: b.channel_id, channel_model_id: b.channel_model_id, binding_type: b.binding_type, enabled: b.enabled, sort_order: b.sort_order}));
      if (exist.some(e => e.channel_id === cid && e.channel_model_id === (cmid||null))) return;
      await replaceAgentChannels(activeAgent.summary.id, [...exist, {channel_id: cid, channel_model_id: cmid||null, binding_type: "agent", enabled: true, sort_order: exist.length}]);
      toast.success("添加成功"); selectedChannelToAdd = undefined; await openAgent(activeAgent.summary.id);
    } catch(e) { toast.error("添加失败"); }
  }
  async function removeChannelBinding(cid: string, cmid: string|null) {
    if (!activeAgent) return;
    try {
      const remain = activeAgent.channel_bindings.filter(b => !(b.channel_id === cid && b.channel_model_id === cmid)).map(b => ({channel_id: b.channel_id, channel_model_id: b.channel_model_id, binding_type: b.binding_type, enabled: b.enabled, sort_order: b.sort_order}));
      await replaceAgentChannels(activeAgent.summary.id, remain); await openAgent(activeAgent.summary.id);
    } catch(e) { toast.error("移除失败"); }
  }

  const labelCls = "mb-1 block text-[13px] font-medium text-[var(--ink-muted)]";
  const inputCls = "w-full rounded-[var(--radius-sm)] border border-[var(--border-medium)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]";
</script>

{#if !selectedAgent}
  <PageShell>
    {#snippet header()}
      <header class="flex h-12 items-center justify-between border-b border-[var(--border-soft)] px-4" data-tauri-drag-region>
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">AI Agents</h1>
        <HeaderWindowGroup>
          {#snippet children()}
            <Button type="button" size="sm" onclick={()=>{createDraft=emptyAgentDraft(agents.length); createDialogOpen=true;}}>
              <Plus size={14} /> 新建
            </Button>
          {/snippet}
        </HeaderWindowGroup>
      </header>
    {/snippet}
    {#snippet toolbar()}
      <div class="border-b border-[var(--border-soft)] px-4 py-3"><SearchField bind:value={searchQuery} placeholder="搜索 Agents..." /></div>
    {/snippet}
    {#snippet body()}
      <div class="app-scrollbar h-full overflow-y-auto p-4">
        {#if loadingList}
          <div class="text-center text-sm text-[var(--ink-muted)] mt-10">加载中...</div>
        {:else}
          <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {#each filteredAgents as a}
              <button class="flex items-center gap-3 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3 text-left hover:bg-[var(--bg-hover)]" onclick={()=>openAgent(a.id)}>
                <div class={`flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-br ${getColor(a.name)} text-white font-bold`}>{a.name.charAt(0)}</div>
                <div class="min-w-0 flex-1">
                  <h3 class="truncate text-sm font-semibold text-[var(--ink-strong)]">{a.name}</h3>
                  <p class="truncate text-xs text-[var(--ink-faint)]">{a.title || '无标题'}</p>
                </div>
                {#if !a.enabled}<div class="text-[10px] text-[var(--ink-faint)] bg-[var(--bg-sunken)] px-1.5 py-0.5 rounded-sm">已停用</div>{/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/snippet}
  </PageShell>
{:else}
  <PageShell>
    {#snippet header()}
      <header class="flex h-12 items-center gap-3 border-b border-[var(--border-soft)] px-4">
        <Button type="button" variant="ghost" size="sm" className="h-8 w-8 px-0" onclick={()=>selectedAgent=null}><ChevronLeft size={18} /></Button>
        <div class="flex items-center gap-2">
          <div class={`flex h-6 w-6 items-center justify-center rounded-full bg-gradient-to-br ${getColor(activeAgent?.summary.name||'')} text-[10px] font-bold text-white`}>{activeAgent?.summary.name.charAt(0)}</div>
          <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{activeAgent?.summary.name}</h1>
        </div>
        <div class="flex-1"></div>
        <HeaderWindowGroup>
          {#snippet children()}
            <Button type="button" size="sm" disabled={!detailDirty || savingAgent} onclick={saveAgent}><Save size={12}/> 保存</Button>
            <Button type="button" variant="ghost" size="sm" className="h-8 w-8 px-0 text-[var(--ink-faint)] hover:text-[var(--danger)]" onclick={removeAgent}><Trash2 size={14}/></Button>
          {/snippet}
        </HeaderWindowGroup>
      </header>
    {/snippet}
    {#snippet toolbar()}
      <div class="flex gap-2 border-b border-[var(--border-soft)] px-4 py-2">
        {#each tabs as t}<button class={cn("rounded-full px-3 py-1.5 text-xs font-semibold", activeTab===t.id?"bg-[var(--bg-active)] text-[var(--brand)]":"text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]")} onclick={()=>activeTab=t.id}>{t.label}</button>{/each}
      </div>
    {/snippet}
    {#snippet body()}
      <div class="app-scrollbar h-full overflow-y-auto p-6 bg-[var(--bg-app)]">
        <div class="mx-auto max-w-3xl">
          {#if loadingDetail}
            <div class="text-center text-sm text-[var(--ink-muted)]">加载详情...</div>
          {:else if activeAgent}
            
            {#if activeTab === "profile"}
              <div class="space-y-5 rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-6">
                <div class="grid gap-5 sm:grid-cols-2">
                  <div><label class={labelCls}>名称 *</label><input class={inputCls} bind:value={detailDraft.name} /></div>
                  <div><label class={labelCls}>标题</label><input class={inputCls} bind:value={detailDraft.title} placeholder="例如：全能编程助手" /></div>
                  <div><label class={labelCls}>头像 URL</label><input class={inputCls} bind:value={detailDraft.avatarUri} /></div>
                  <div><label class={labelCls}>创作者</label><input class={inputCls} bind:value={detailDraft.creatorName} /></div>
                  <div><label class={labelCls}>版本号</label><input class={inputCls} bind:value={detailDraft.characterVersion} /></div>
                  <div><label class={labelCls}>排序值</label><input type="number" class={inputCls} bind:value={detailDraft.sortOrder} /></div>
                  <div class="sm:col-span-2">
                    <label class={labelCls}>健谈度 ({detailDraft.talkativeness})</label>
                    <input type="range" class="w-full" min="0" max="100" bind:value={detailDraft.talkativeness} />
                  </div>
                  <div class="sm:col-span-2 flex items-center gap-2">
                    <input type="checkbox" bind:checked={detailDraft.enabled} class="h-4 w-4" />
                    <span class="text-sm font-medium text-[var(--ink-body)]">启用该智能体</span>
                  </div>
                </div>
              </div>
            {/if}

            {#if activeTab === "persona"}
              <div class="space-y-3">
                {#each [
                  {id: 'description', label: '描述 (Description)', bindTo: 'description', h: '80px'},
                  {id: 'personality', label: '性格 (Personality)', bindTo: 'personality', h: '80px'},
                  {id: 'scenario', label: '场景设定 (Scenario)', bindTo: 'scenario', h: '80px'},
                  {id: 'exampleMessages', label: '示例消息 (Example Messages)', bindTo: 'exampleMessages', h: '120px'},
                  {id: 'mainPrompt', label: '系统主提示覆盖 (Main Prompt Override)', bindTo: 'mainPrompt', h: '160px'},
                  {id: 'postHistory', label: '历史后指令 (Post History)', bindTo: 'postHistory', h: '80px'},
                  {id: 'characterNote', label: '角色备注 (Character Note)', bindTo: 'characterNote', h: '80px', hasDepth: true},
                  {id: 'creatorNotes', label: '创作者备注 (Creator Notes)', bindTo: 'creatorNotes', h: '80px'}
                ] as field}
                  <div class="rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] overflow-hidden">
                    <button class="flex w-full items-center justify-between bg-[var(--bg-sunken)] px-4 py-2.5 hover:bg-[var(--bg-hover)]" onclick={() => expandedSections[field.id] = !expandedSections[field.id]}>
                      <span class="text-sm font-semibold text-[var(--ink-strong)]">{field.label}</span>
                      <ChevronDown size={14} class={cn("text-[var(--ink-muted)] transition-transform", expandedSections[field.id] && "rotate-180")} />
                    </button>
                    {#if expandedSections[field.id]}
                      <div class="p-4 border-t border-[var(--border-soft)]">
                        <textarea class={cn(inputCls, "resize-y font-mono text-[13px]")} style={`min-height: ${field.h}`} bind:value={detailDraft[field.bindTo as keyof AgentDraft]} placeholder={`输入 ${field.label}...`}></textarea>
                        {#if field.hasDepth}
                          <div class="mt-3 flex gap-4">
                            <div class="flex-1"><label class={labelCls}>插入深度 (Depth)</label><input type="number" class={inputCls} bind:value={detailDraft.characterNoteDepth} placeholder="为空表示默认" /></div>
                            <div class="flex-1"><label class={labelCls}>插入角色 (Role)</label><select class={inputCls} bind:value={detailDraft.characterNoteRole}><option value="system">system</option><option value="assistant">assistant</option><option value="user">user</option></select></div>
                          </div>
                        {/if}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}

            {#if activeTab === "greetings"}
              <div class="space-y-3">
                {#each activeAgent.greetings as g}
                  <div class="rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-4">
                    <div class="mb-2 flex items-center justify-between">
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-semibold text-[var(--ink-strong)]">{g.name || '默认问候'}</span>
                        {#if !g.enabled}<span class="text-[10px] bg-[var(--bg-sunken)] px-1 py-0.5 rounded-sm">已停用</span>{/if}
                      </div>
                      <div class="flex gap-1">
                        <ActionIconButton onClick={()=>{greetingDraft={greetingType:g.greeting_type, name:g.name||'', text:readContentText(g.primary_content), enabled:g.enabled, sortOrder:g.sort_order}; editingGreetingId=g.id; greetingDialogOpen=true;}}><Edit3 size={14}/></ActionIconButton>
                        <ActionIconButton tone="danger" onClick={()=>removeGreeting(g.id)}><Trash2 size={14}/></ActionIconButton>
                      </div>
                    </div>
                    <p class="text-[13px] text-[var(--ink-body)] line-clamp-3">{readContentText(g.primary_content)}</p>
                  </div>
                {/each}
                <Button variant="secondary" className="w-full border-dashed text-[var(--ink-muted)]" onclick={()=>{greetingDraft={greetingType:'default', name:'', text:'', enabled:true, sortOrder:0}; editingGreetingId=null; greetingDialogOpen=true;}}>
                  <Plus size={14} class="mr-2" /> 添加问候语
                </Button>
              </div>
            {/if}

            {#if activeTab === "bindings"}
              <div class="grid gap-6 sm:grid-cols-2">
                <!-- Presets -->
                <div class="rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-4">
                  <h3 class="mb-3 text-sm font-semibold text-[var(--ink-strong)]">预设 (Presets)</h3>
                  <div class="flex flex-wrap gap-2 mb-3">
                    {#each activeAgent.preset_bindings as b}
                      <span class="inline-flex items-center gap-1 rounded-full border border-[var(--border-soft)] bg-[var(--bg-sunken)] px-2.5 py-1 text-xs">
                        {presetOptions.find(o=>o.value===b.resource_id)?.label || b.resource_id}
                        <button class="ml-1 text-[var(--ink-faint)] hover:text-[var(--danger)]" onclick={()=>removePresetBinding(b.resource_id)}><X size={12}/></button>
                      </span>
                    {/each}
                    {#if activeAgent.preset_bindings.length===0}<span class="text-xs text-[var(--ink-faint)]">暂无绑定的预设</span>{/if}
                  </div>
                  <div class="flex gap-2">
                    <SelectDropdown options={presetOptions} value={selectedPresetToAdd} onChange={(val) => selectedPresetToAdd = val} placeholder="选择预设..." className="flex-1" />
                    <Button size="sm" onclick={addPresetBinding} disabled={!selectedPresetToAdd}>添加</Button>
                  </div>
                </div>

                <!-- Lorebooks -->
                <div class="rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-4">
                  <h3 class="mb-3 text-sm font-semibold text-[var(--ink-strong)]">世界书 (Lorebooks)</h3>
                  <div class="flex flex-wrap gap-2 mb-3">
                    {#each activeAgent.lorebook_bindings as b}
                      <span class="inline-flex items-center gap-1 rounded-full border border-[var(--border-soft)] bg-[var(--bg-sunken)] px-2.5 py-1 text-xs">
                        {lorebookOptions.find(o=>o.value===b.resource_id)?.label || b.resource_id}
                        <button class="ml-1 text-[var(--ink-faint)] hover:text-[var(--danger)]" onclick={()=>removeLorebookBinding(b.resource_id)}><X size={12}/></button>
                      </span>
                    {/each}
                    {#if activeAgent.lorebook_bindings.length===0}<span class="text-xs text-[var(--ink-faint)]">暂无绑定的世界书</span>{/if}
                  </div>
                  <div class="flex gap-2">
                    <SelectDropdown options={lorebookOptions} value={selectedLorebookToAdd} onChange={(val) => selectedLorebookToAdd = val} placeholder="选择世界书..." className="flex-1" />
                    <Button size="sm" onclick={addLorebookBinding} disabled={!selectedLorebookToAdd}>添加</Button>
                  </div>
                </div>

                <!-- Channels -->
                <div class="rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-4 sm:col-span-2">
                  <h3 class="mb-3 text-sm font-semibold text-[var(--ink-strong)]">API 渠道 (Channels)</h3>
                  <div class="space-y-2 mb-3">
                    {#each activeAgent.channel_bindings as b}
                      <div class="flex items-center justify-between rounded-sm border border-[var(--border-soft)] bg-[var(--bg-sunken)] px-3 py-2 text-sm">
                        <span>
                          {channelOptions.find(o=>o.value===`${b.channel_id}|${b.channel_model_id||''}`)?.label || `${b.channel_id} / ${b.channel_model_id||'默认'}`}
                        </span>
                        <ActionIconButton tone="danger" onClick={()=>removeChannelBinding(b.channel_id, b.channel_model_id)}><X size={14}/></ActionIconButton>
                      </div>
                    {/each}
                    {#if activeAgent.channel_bindings.length===0}<span class="text-xs text-[var(--ink-faint)]">暂无绑定的 API 渠道</span>{/if}
                  </div>
                  <div class="flex gap-2 w-full max-w-sm">
                    <SelectDropdown options={channelOptions} value={selectedChannelToAdd} onChange={(val) => selectedChannelToAdd = val} placeholder="选择渠道 / 模型..." className="flex-1" />
                    <Button size="sm" onclick={addChannelBinding} disabled={!selectedChannelToAdd}>添加渠道</Button>
                  </div>
                </div>
              </div>
            {/if}

            {#if activeTab === "advanced"}
              <div class="space-y-4 rounded-[var(--radius-md)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-6">
                <div>
                  <div class="mb-2 flex items-center justify-between">
                    <label class={labelCls}>Config JSON (高级配置)</label>
                  </div>
                  <textarea class={cn(inputCls, "font-mono text-xs")} rows="10" bind:value={detailDraft.configJsonStr}></textarea>
                </div>
                <div class="pt-4 border-t border-[var(--border-soft)]">
                  <h3 class="mb-2 text-sm font-semibold text-[var(--ink-strong)]">多媒体附件 (Media)</h3>
                  <div class="rounded-[var(--radius-sm)] border border-dashed border-[var(--border-medium)] px-4 py-8 text-center text-xs text-[var(--ink-faint)]">
                    媒体管理功能开发中...
                  </div>
                </div>
              </div>
            {/if}

          {/if}
        </div>
      </div>
    {/snippet}
  </PageShell>
{/if}

<!-- Create Dialog -->
<Dialog.Root bind:open={createDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/40 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(500px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-6 shadow-[var(--shadow-lg)] outline-none">
      <h2 class="text-lg font-semibold text-[var(--ink-strong)] mb-4">新建 Agent</h2>
      <div class="space-y-4">
        <div><label class={labelCls}>名称 *</label><input class={inputCls} bind:value={createDraft.name} /></div>
        <div><label class={labelCls}>系统主提示词</label><textarea class={inputCls} rows="4" bind:value={createDraft.mainPrompt}></textarea></div>
      </div>
      <div class="mt-6 flex justify-end gap-2 text-sm">
        <Button variant="secondary" onclick={()=>createDialogOpen=false}>取消</Button>
        <Button onclick={createNewAgent} disabled={savingAgent||!createDraft.name.trim()}>{savingAgent?'创建中':'创建'}</Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<!-- Greeting Dialog -->
<Dialog.Root bind:open={greetingDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/40 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(500px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-medium)] bg-[var(--bg-surface)] p-6 shadow-[var(--shadow-lg)] outline-none">
      <h2 class="text-lg font-semibold text-[var(--ink-strong)] mb-4">{editingGreetingId ? '编辑问候语' : '添加问候语'}</h2>
      <div class="space-y-4">
        <div class="flex gap-4">
          <div class="flex-1"><label class={labelCls}>名称</label><input class={inputCls} bind:value={greetingDraft.name} /></div>
          <div class="w-24"><label class={labelCls}>排序值</label><input type="number" class={inputCls} bind:value={greetingDraft.sortOrder} /></div>
        </div>
        <div><label class={labelCls}>内容 *</label><textarea class={cn(inputCls,"resize-y")} rows="5" bind:value={greetingDraft.text}></textarea></div>
        <div class="flex items-center gap-2"><input type="checkbox" bind:checked={greetingDraft.enabled} /><span class="text-sm font-medium">启用</span></div>
      </div>
      <div class="mt-6 flex justify-end gap-2 text-sm">
        <Button variant="secondary" onclick={()=>greetingDialogOpen=false}>取消</Button>
        <Button onclick={saveGreeting} disabled={!greetingDraft.text.trim()}>保存</Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
