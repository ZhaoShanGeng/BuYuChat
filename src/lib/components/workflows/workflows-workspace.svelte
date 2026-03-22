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
    Sparkles
  } from "lucide-svelte";
  import { getWorkflowDefDetail, listWorkflowDefs, type WorkflowDefDetail, type WorkflowDefSummary } from "$lib/api/workflows";
  import { i18n } from "$lib/i18n.svelte";
  import { cn } from "$lib/utils";
  import Button from "$components/ui/button.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import PageShell from "$components/layout/page-shell.svelte";
  import SearchField from "$components/shared/search-field.svelte";

  let selectedWorkflow = $state<string | null>(null);
  let searchQuery = $state("");
  let workflows = $state<WorkflowDefSummary[]>([]);
  let activeWorkflow = $state<WorkflowDefDetail | null>(null);
  let loadingList = $state(true);
  let loadingDetail = $state(false);

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
      workflows = await listWorkflowDefs();
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
    try {
      activeWorkflow = await getWorkflowDefDetail(id);
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

  function nodeDisplayName(node: WorkflowDefDetail["nodes"][number]) {
    return node.name?.trim() || node.node_key;
  }

  function entryNodeCount(detail: WorkflowDefDetail) {
    const inbound = new Set(detail.edges.filter((edge) => edge.enabled).map((edge) => edge.to_node_id));
    return detail.nodes.filter((node) => !inbound.has(node.id)).length;
  }

  function conditionalEdgeCount(detail: WorkflowDefDetail) {
    return detail.edges.filter((edge) => edge.enabled && (edge.condition_expr || edge.edge_type !== "default")).length;
  }

  function outputNodeCount(detail: WorkflowDefDetail) {
    return detail.nodes.filter((node) => node.node_type === "output").length;
  }

  function edgeLabel(edge: WorkflowDefDetail["edges"][number]) {
    if (edge.label?.trim()) return edge.label;
    if (edge.condition_expr?.trim()) return edge.condition_expr;
    return edge.edge_type;
  }

  function nodeById(id: string) {
    return activeWorkflow?.nodes.find((node) => node.id === id) ?? null;
  }

  function resolveNodeName(id: string) {
    const node = nodeById(id);
    return node ? nodeDisplayName(node) : id;
  }
</script>

{#if !selectedWorkflow}
  <PageShell>
    {#snippet header()}
      <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4" data-tauri-drag-region>
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.workflows")}</h1>
        <HeaderWindowGroup>
          {#snippet children()}
            <Button type="button" size="sm">
              <Plus size={14} /> {i18n.t("workflows.create")}
            </Button>
          {/snippet}
        </HeaderWindowGroup>
      </header>
    {/snippet}

    {#snippet toolbar()}
      <div class="border-b border-[var(--border-soft)] px-4 py-3">
        <SearchField bind:value={searchQuery} placeholder={i18n.t("workflows.search")} />
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
            <Button type="button" size="sm">
              <Save size={12} /> 保存
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
                <div class="mb-4 flex items-center gap-2">
                  <Sparkles size={14} class="text-[var(--ink-faint)]" />
                  <h2 class="text-sm font-semibold text-[var(--ink-strong)]">节点定义</h2>
                </div>
                <div class="grid gap-3 md:grid-cols-2">
                  {#each activeWorkflow.nodes as node (node.id)}
                    <article class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] p-3">
                      <div class="flex items-start gap-3">
                        <div class={`flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br ${gradientForNode(node.node_type)} text-xs font-bold uppercase text-white shadow-sm`}>
                          {node.node_type.slice(0, 1)}
                        </div>
                        <div class="min-w-0 flex-1">
                          <div class="flex flex-wrap items-center gap-2">
                            <h3 class="truncate text-sm font-semibold text-[var(--ink-strong)]">{nodeDisplayName(node)}</h3>
                            <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">
                              {shortNodeType(node.node_type)}
                            </span>
                          </div>
                          <div class="mt-2 grid gap-1 text-[11px] text-[var(--ink-muted)]">
                            <div>workspace: <span class="font-medium text-[var(--ink-strong)]">{node.workspace_mode}</span></div>
                            <div>history: <span class="font-medium text-[var(--ink-strong)]">{node.history_read_mode}</span></div>
                            <div>output: <span class="font-medium text-[var(--ink-strong)]">{node.visible_output_mode}</span></div>
                          </div>
                        </div>
                      </div>
                    </article>
                  {/each}
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
                            <span class="truncate">{resolveNodeName(edge.from_node_id)}</span>
                            <ArrowRight size={12} class="text-[var(--ink-faint)]" />
                            <span class="truncate">{resolveNodeName(edge.to_node_id)}</span>
                          </div>
                          <div class="mt-1 flex items-center justify-between text-[11px] text-[var(--ink-faint)]">
                            <span>{edgeLabel(edge)}</span>
                            <span>priority {edge.priority}</span>
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
          <div class="mx-auto max-w-3xl space-y-4">
            <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
              <div class="flex items-center gap-2">
                <Clock3 size={14} class="text-[var(--ink-faint)]" />
                <h2 class="text-sm font-semibold text-[var(--ink-strong)]">运行记录</h2>
              </div>
              <p class="mt-3 text-sm leading-relaxed text-[var(--ink-muted)]">
                当前前端已接入真实的工作流定义数据。运行记录列表接口后端还未提供，这里暂时不再展示假数据。
              </p>
            </div>

            <div class="grid gap-3 md:grid-cols-3">
              <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-3">
                <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Ready To Run</div>
                <div class="mt-1 text-xl font-semibold text-[var(--ink-strong)]">{activeWorkflow.summary.enabled ? "Yes" : "No"}</div>
              </div>
              <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-3">
                <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Node Types</div>
                <div class="mt-1 text-xl font-semibold text-[var(--ink-strong)]">{new Set(activeWorkflow.nodes.map((node) => node.node_type)).size}</div>
              </div>
              <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-3">
                <div class="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--ink-faint)]">Flow Complexity</div>
                <div class="mt-1 text-xl font-semibold text-[var(--ink-strong)]">{activeWorkflow.edges.length > activeWorkflow.nodes.length ? "Dense" : "Lean"}</div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/snippet}
  </PageShell>
{/if}
