<script lang="ts">
  import { onMount } from "svelte";
  import {
    Plus,
    ChevronLeft,
    Play,
    Save,
    GitBranch,
    Clock3,
    CheckCircle2,
    Workflow,
    ArrowRight,
    Shuffle,
    Sparkles,
    Trash2,
    Settings
  } from "lucide-svelte";
  import { getWorkflowDetail, listWorkflows, createWorkflow, type WorkflowDetail, type WorkflowSummary, saveWorkflowGraph } from "$lib/api/workflows";
  import { toast } from "svelte-sonner";
  import { i18n } from "$lib/i18n.svelte";
  import { cn } from "$lib/utils";
  import Button from "$components/ui/button.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import PageShell from "$components/layout/page-shell.svelte";
  import SearchField from "$components/shared/search-field.svelte";

  let selectedWorkflow = $state<string | null>(null);
  let searchQuery = $state("");
  let workflows = $state<WorkflowSummary[]>([]);
  let activeWorkflowId = $state<string | null>(null);
  let activeWorkflow = $state<WorkflowDetail | null>(null);
  let loadingList = $state(true);
  let loadingDetail = $state(false);
  let saving = $state(false);
  
  let editingNodeId = $state<string | null>(null);

  type TabId = "graph" | "runs";
  let activeTab = $state<TabId>("graph");

  const filteredWorkflows = $derived(
    searchQuery
      ? workflows.filter((workflow) =>
          `${workflow.name} ${workflow.description ?? ""}`.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : workflows
  );

  const nodeColors: Record<string, string> = {
    input: "from-blue-500 to-cyan-500",
    agent: "from-violet-500 to-fuchsia-500",
    router: "from-amber-500 to-orange-500",
    merge: "from-emerald-500 to-teal-500",
    loop: "from-pink-500 to-rose-500",
    writeback: "from-sky-500 to-indigo-500",
    output: "from-green-500 to-emerald-500",
    subflow: "from-slate-500 to-slate-700",
    plugin: "from-purple-500 to-indigo-500",
    tool: "from-cyan-500 to-blue-500",
    rag: "from-teal-500 to-emerald-500",
    mcp: "from-orange-500 to-amber-500"
  };

  onMount(() => {
    void loadWorkflows();
  });

  async function loadWorkflows() {
    loadingList = true;
    try {
      workflows = await listWorkflows();
    } catch (error) {
      console.error("Failed to load workflows:", error);
      workflows = [];
    } finally {
      loadingList = false;
    }
  }

  async function openWorkflow(id: string) {
    selectedWorkflow = id;
    activeTab = "graph";
    loadingDetail = true;
    activeWorkflowId = id;
    try {
      activeWorkflow = await getWorkflowDetail(id);
    } catch (error) {
      console.error("Failed to load workflow detail:", error);
      activeWorkflow = null;
    } finally {
      loadingDetail = false;
    }
  }

  function shortNodeType(type: string) {
    return type.replaceAll("_", " ");
  }

  function gradientForNode(type: string) {
    return nodeColors[type] ?? "from-slate-500 to-slate-700";
  }

  function nodeDisplayName(node: WorkflowDetail["nodes"][number]) {
    return (node.data?.name as string) || node.type;
  }

  function entryNodeCount(detail: WorkflowDetail) {
    const inbound = new Set(detail.edges.map((edge: any) => edge.target));
    return detail.nodes.filter((node: any) => !inbound.has(node.id)).length;
  }

  function conditionalEdgeCount(detail: WorkflowDetail) {
    return detail.edges.filter((edge: any) => edge.source_handle).length;
  }

  function outputNodeCount(detail: WorkflowDetail) {
    return detail.nodes.filter((node: any) => node.type === "output").length;
  }

  function edgeLabel(edge: WorkflowDetail["edges"][number]) {
    return edge.source_handle || "default";
  }

  function nodeById(id: string) {
    return activeWorkflow?.nodes.find((node: any) => node.id === id) ?? null;
  }

  function resolveNodeName(id: string) {
    const node = nodeById(id);
    return node ? nodeDisplayName(node) : id;
  }

  async function handleSave() {
    if (!activeWorkflow) return;
    saving = true;
    try {
      await saveWorkflowGraph(activeWorkflow.summary.id, {
        nodes: activeWorkflow.nodes,
        edges: activeWorkflow.edges
      });
      toast.success("工作流图已保存");
    } catch {
      toast.error("保存失败");
    } finally { saving = false; }
  }

  async function handleCreateWorkflow() {
    try {
      const newWf = await createWorkflow({
        name: "新建工作流 " + new Date().toLocaleTimeString(),
        description: "",
        enabled: true,
        sort_order: 0,
        config_json: {}
      });
      await loadWorkflows();
      await openWorkflow(newWf.summary.id);
      toast.success("工作流已创建");
    } catch (e) {
      toast.error("创建失败", { description: String(e) });
    }
  }

  function handleAddNode() {
    if (!activeWorkflow) return;
    activeWorkflow.nodes = [...activeWorkflow.nodes, {
      id: "node-" + Date.now(),
      type: "agent",
      position: { x: 0, y: 0 },
      data: { name: "新建节点", node_key: "new_agent", node_type: "agent" }
    }];
  }

  function deleteNode(id: string) {
    if (!activeWorkflow) return;
    activeWorkflow.nodes = activeWorkflow.nodes.filter((n: any) => n.id !== id);
    activeWorkflow.edges = activeWorkflow.edges.filter((e: any) => e.source !== id && e.target !== id);
  }

  const MOCK_RUNS = [
    { id: "run-1", time: "10 mins ago", status: "success", dur: "1.2s", trig: "manual" },
    { id: "run-2", time: "1 hour ago", status: "success", dur: "4.5s", trig: "api" },
    { id: "run-3", time: "2 hours ago", status: "error", dur: "0.2s", trig: "manual" },
    { id: "run-4", time: "1 day ago", status: "success", dur: "1.0s", trig: "schedule" }
  ];
</script>

{#if !selectedWorkflow}
  <PageShell>
    {#snippet header()}
      <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4" data-tauri-drag-region>
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.workflows")}</h1>
        <HeaderWindowGroup>
          {#snippet children()}
            <Button type="button" size="sm" onclick={handleCreateWorkflow}>
              <Plus size={14} /> {i18n.t("workflows.create")}
            </Button>
          {/snippet}
        </HeaderWindowGroup>
      </header>
    {/snippet}

    {#snippet toolbar()}
      <div class="border-b border-[var(--border-soft)] px-4 py-3">
        <SearchField bind:value={searchQuery} placeholder="搜索工作流..." />
      </div>
    {/snippet}

    {#snippet body()}
      <div class="app-scrollbar h-full overflow-y-auto p-4">
        {#if loadingList}
          <div class="mx-auto max-w-3xl rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-10 text-center text-sm text-[var(--ink-muted)]">
            正在读取工作流...
          </div>
        {:else}
          <div class="mx-auto grid max-w-5xl gap-4 lg:grid-cols-2">
            {#each filteredWorkflows as workflow (workflow.id)}
              <button
                type="button"
                class="suggestion-card flex flex-col gap-4 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4 text-left transition-shadow hover:shadow-[var(--shadow-md)]"
                onclick={() => void openWorkflow(workflow.id)}
              >
                <div class="flex items-start gap-3">
                  <div class="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-fuchsia-500 to-violet-600 text-white shadow-sm">
                    <GitBranch size={18} />
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <h3 class="truncate text-sm font-semibold text-[var(--ink-strong)]">{workflow.name}</h3>
                      <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">
                        {workflow.enabled ? "启用" : "停用"}
                      </span>
                    </div>
                    <p class="mt-1 text-xs leading-relaxed text-[var(--ink-muted)]">
                      {workflow.description || "用于串联智能体、工具与写回流程的可执行工作流。"}
                    </p>
                  </div>
                </div>

                <div class="grid grid-cols-3 gap-2">
                  <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-2">
                    <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Sort</div>
                    <div class="mt-1 text-sm font-semibold text-[var(--ink-strong)]">#{workflow.sort_order}</div>
                  </div>
                  <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-2">
                    <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Config</div>
                    <div class="mt-1 text-sm font-semibold text-[var(--ink-strong)]">
                      {Object.keys(workflow.config_json ?? {}).length}
                    </div>
                  </div>
                  <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-2">
                    <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Updated</div>
                    <div class="mt-1 text-sm font-semibold text-[var(--ink-strong)]">
                      {new Date(workflow.updated_at).toLocaleDateString()}
                    </div>
                  </div>
                </div>
              </button>
            {/each}
            {#if filteredWorkflows.length === 0}
              <div class="col-span-1 flex flex-col items-center justify-center gap-3 rounded-[var(--radius-lg)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-surface)] px-4 py-16 text-center text-[var(--ink-muted)] lg:col-span-2">
                <Workflow size={32} class="opacity-40" />
                <div>
                  <h3 class="text-sm font-semibold text-[var(--ink-strong)]">暂无工作流</h3>
                  <p class="mt-1 text-xs text-[var(--ink-faint)]">你还没有创建任何工作流，请点击右上角的「新建工作流」开始创建。</p>
                </div>
                <Button type="button" size="sm" onclick={handleCreateWorkflow} className="mt-2 text-xs h-7 px-3">
                  <Plus size={14} class="mr-1" /> 立即创建
                </Button>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/snippet}
  </PageShell>
{:else}
  <PageShell>
    {#snippet header()}
      <header class="flex h-12 items-center gap-3 border-b border-[var(--border-soft)] px-4">
        <Button type="button" variant="ghost" size="sm" className="h-8 w-8 px-0" onclick={() => { selectedWorkflow = null; activeWorkflow = null; }}>
          <ChevronLeft size={18} />
        </Button>
        <Workflow size={16} class="text-[var(--brand)]" />
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{activeWorkflow?.summary.name ?? "工作流"}</h1>
        <div class="flex-1"></div>
        <div class="flex items-center gap-1">
          {#each [{ id: "graph" as const, label: "定义视图" }, { id: "runs" as const, label: "运行状态" }] as tab}
            <Button
              type="button"
              variant={tab.id === activeTab ? "default" : "ghost"}
              size="sm"
              className={cn("rounded-[var(--radius-full)]", tab.id !== activeTab && "text-[var(--ink-muted)]")}
              onclick={() => { activeTab = tab.id; }}
            >
              {tab.label}
            </Button>
          {/each}
        </div>
        <HeaderWindowGroup>
          {#snippet children()}
            <Button type="button" variant="secondary" size="sm" className="bg-[var(--success)] text-white hover:opacity-90">
              <Play size={12} /> 运行
            </Button>
            <Button type="button" size="sm" onclick={handleSave} disabled={saving}>
              <Save size={12} /> {saving ? "保存中" : "保存"}
            </Button>
          {/snippet}
        </HeaderWindowGroup>
      </header>
    {/snippet}

    {#snippet body()}
      <div class="app-scrollbar h-full overflow-y-auto p-4">
        {#if loadingDetail}
          <div class="mx-auto max-w-4xl rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-10 text-center text-sm text-[var(--ink-muted)]">
            正在读取工作流详情...
          </div>
        {:else if !activeWorkflow}
          <div class="mx-auto max-w-4xl rounded-[var(--radius-lg)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-surface)] px-4 py-10 text-center text-sm text-[var(--ink-faint)]">
            无法读取工作流详情
          </div>
        {:else if activeTab === "graph"}
          <div class="mx-auto max-w-6xl space-y-6">
            <div class="grid gap-3 md:grid-cols-4">
              <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-3">
                <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Nodes</div>
                <div class="mt-1 text-xl font-semibold text-[var(--ink-strong)]">{activeWorkflow.nodes.length}</div>
              </div>
              <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-3">
                <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Edges</div>
                <div class="mt-1 text-xl font-semibold text-[var(--ink-strong)]">{activeWorkflow.edges.length}</div>
              </div>
              <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-3">
                <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Entry Nodes</div>
                <div class="mt-1 text-xl font-semibold text-[var(--ink-strong)]">{entryNodeCount(activeWorkflow)}</div>
              </div>
              <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-3">
                <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Conditional</div>
                <div class="mt-1 text-xl font-semibold text-[var(--ink-strong)]">{conditionalEdgeCount(activeWorkflow)}</div>
              </div>
            </div>

            <div class="grid gap-6 xl:grid-cols-[minmax(0,1.5fr)_minmax(20rem,0.9fr)]">
              <section class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
                <div class="mb-4 flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <Sparkles size={14} class="text-[var(--ink-faint)]" />
                    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">节点定义</h2>
                  </div>
                  <Button size="sm" variant="secondary" className="h-7 px-2" onclick={handleAddNode}><Plus size={14} class="mr-1"/> 添加</Button>
                </div>
                <div class="grid gap-3 md:grid-cols-2">
                  {#each activeWorkflow.nodes as node (node.id)}
                    {@const nData = (node.data || node) as any}
                    <article class="relative group rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] p-3">
                      <div class="flex items-start gap-3">
                        <div class={`flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br ${gradientForNode(nData.node_type || "agent")} text-[10px] font-bold uppercase text-white shadow-sm`}>
                          {(nData.node_type || "A").slice(0, 3)}
                        </div>
                        <div class="min-w-0 flex-1">
                          <div class="flex flex-wrap items-center gap-2 pr-8">
                            <h3 class="truncate text-sm font-semibold text-[var(--ink-strong)]">{nData.name || nData.node_key || node.id}</h3>
                            <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">
                              {nData.node_type || node.type}
                            </span>
                          </div>
                          {#if editingNodeId === node.id}
                            <div class="mt-2 space-y-1">
                              <input class="w-full text-xs p-1 rounded border border-[var(--border-medium)] bg-[var(--bg-surface)]" bind:value={nData.name} placeholder="节点名称" />
                              <input class="w-full text-xs p-1 rounded border border-[var(--border-medium)] bg-[var(--bg-surface)]" bind:value={nData.node_key} placeholder="Key" />
                              <div class="flex justify-end gap-1"><Button size="sm" variant="ghost" className="h-6 px-1.5 text-xs" onclick={() => editingNodeId = null}>完成</Button></div>
                            </div>
                          {:else}
                            <div class="mt-2 text-[11px] font-mono text-[var(--ink-muted)] truncate">ID: {node.id}</div>
                          {/if}
                        </div>
                      </div>
                      <div class="absolute right-2 top-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-0.5">
                        <Button size="sm" variant="ghost" className="h-6 w-6 px-0" onclick={() => editingNodeId = node.id}><Settings size={12} class="text-[var(--ink-muted)]"/></Button>
                        <Button size="sm" variant="ghost" className="h-6 w-6 px-0" onclick={() => deleteNode(node.id)}><Trash2 size={12} class="text-[var(--danger)]"/></Button>
                      </div>
                    </article>
                  {/each}
                  {#if activeWorkflow.nodes.length === 0}
                     <div class="col-span-2 py-4 text-center text-xs text-[var(--ink-faint)] border border-dashed rounded-md">暂无节点</div>
                  {/if}
                </div>
              </section>

              <section class="space-y-4">
                <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
                  <div class="mb-4 flex items-center gap-2">
                    <Shuffle size={14} class="text-[var(--ink-faint)]" />
                    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">连接关系</h2>
                  </div>
                  <div class="space-y-2">
                    {#if activeWorkflow.edges.length === 0}
                      <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-6 text-center text-xs text-[var(--ink-faint)]">
                        当前工作流还没有边定义
                      </div>
                    {:else}
                      {#each activeWorkflow.edges as edge (edge.id)}
                        <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-2.5">
                          <div class="flex items-center gap-2 text-xs font-medium text-[var(--ink-strong)]">
                            <span class="truncate">{resolveNodeName(edge.source)}</span>
                            <ArrowRight size={12} class="text-[var(--ink-faint)]" />
                            <span class="truncate">{resolveNodeName(edge.target)}</span>
                          </div>
                          <div class="mt-1.5 flex items-center justify-between text-[11px] text-[var(--ink-muted)]">
                            <span>{edgeLabel(edge)}</span>
                          </div>
                        </div>
                      {/each}
                    {/if}
                  </div>
                </div>

                <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
                  <div class="mb-4 flex items-center gap-2">
                    <CheckCircle2 size={14} class="text-[var(--ink-faint)]" />
                    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">定义状态</h2>
                  </div>
                  <div class="space-y-2 text-sm text-[var(--ink-muted)]">
                    <div class="flex items-center justify-between rounded-[var(--radius-md)] bg-[var(--bg-app)] px-3 py-2">
                      <span>启用状态</span>
                      <span class="font-medium text-[var(--ink-strong)]">{activeWorkflow.summary.enabled ? "已启用" : "已停用"}</span>
                    </div>
                    <div class="flex items-center justify-between rounded-[var(--radius-md)] bg-[var(--bg-app)] px-3 py-2">
                      <span>输出节点</span>
                      <span class="font-medium text-[var(--ink-strong)]">{outputNodeCount(activeWorkflow)}</span>
                    </div>
                    <div class="flex items-center justify-between rounded-[var(--radius-md)] bg-[var(--bg-app)] px-3 py-2">
                      <span>最近更新时间</span>
                      <span class="font-medium text-[var(--ink-strong)]">{new Date(activeWorkflow.summary.updated_at).toLocaleString()}</span>
                    </div>
                  </div>
                </div>
              </section>
            </div>
          </div>
        {:else}
          <div class="mx-auto max-w-4xl space-y-6">
            <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
              <div class="flex items-center gap-2">
                <Clock3 size={14} class="text-[var(--ink-faint)]" />
                <h2 class="text-sm font-semibold text-[var(--ink-strong)]">运行历史</h2>
              </div>
              <p class="mt-2 text-xs leading-relaxed text-[var(--ink-muted)]">
                工作流的最近运行日志和执行状态，可用于调试追踪。
              </p>
            </div>

            <div class="space-y-2">
              {#each MOCK_RUNS as run}
                <div class="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4 shadow-sm">
                  <div class="flex items-center gap-4">
                    <div class={cn("flex h-8 w-8 items-center justify-center rounded-full", run.status === "success" ? "bg-emerald-100 text-emerald-600" : "bg-red-100 text-red-600")}>
                      <Workflow size={14} />
                    </div>
                    <div>
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-semibold text-[var(--ink-strong)]">{run.id}</span>
                        <span class={cn("text-[10px] px-1.5 py-0.5 rounded-sm", run.status === "success" ? "bg-emerald-50 text-emerald-600" : "bg-red-50 text-red-600")}>{run.status.toUpperCase()}</span>
                      </div>
                      <div class="text-[11px] text-[var(--ink-faint)] mt-0.5">{run.time} · 耗时 {run.dur}</div>
                    </div>
                  </div>
                  <div class="flex items-center gap-4">
                    <span class="text-[11px] text-[var(--ink-muted)]">触发源: {run.trig}</span>
                    <Button size="sm" variant="secondary" className="h-7 px-3 text-xs">查看详情</Button>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/snippet}
  </PageShell>
{/if}
