<script lang="ts">
  import { Plus, Search, ChevronLeft, Play, Save, Trash2, GitBranch, Clock, CheckCircle, XCircle, Circle, ArrowRight } from "lucide-svelte";
  import { i18n } from "$lib/i18n.svelte";

  let selectedWorkflow = $state<string | null>(null);

  type TabId = "graph" | "runs";
  let activeTab = $state<TabId>("graph");

  const mockWorkflows = [
    { id: "1", name: "多模型辩论", description: "让两个AI模型对同一问题进行辩论，合成最终回答", nodes: 5, edges: 4, lastRun: "2分钟前", status: "success" as const },
    { id: "2", name: "RAG 增强生成", description: "先检索相关文档，再基于上下文生成回答", nodes: 4, edges: 3, lastRun: "1小时前", status: "success" as const },
    { id: "3", name: "自动摘要链", description: "将长文本拆分后逐段摘要，最终合成摘要", nodes: 3, edges: 2, lastRun: "未执行", status: "idle" as const },
  ];

  const mockNodes = [
    { id: "n1", type: "input", label: "用户输入", x: 50, y: 100 },
    { id: "n2", type: "llm", label: "正方 (GPT-4o)", x: 250, y: 50 },
    { id: "n3", type: "llm", label: "反方 (Claude)", x: 250, y: 150 },
    { id: "n4", type: "merge", label: "观点合并", x: 450, y: 100 },
    { id: "n5", type: "output", label: "最终回答", x: 650, y: 100 },
  ];

  const mockRuns = [
    { id: "r1", status: "success" as const, duration: "12.3s", startedAt: "2分钟前", tokens: 2450 },
    { id: "r2", status: "failed" as const, duration: "3.1s", startedAt: "1小时前", tokens: 820 },
    { id: "r3", status: "success" as const, duration: "8.7s", startedAt: "3小时前", tokens: 1890 },
  ];

  const activeWorkflow = $derived(mockWorkflows.find(w => w.id === selectedWorkflow));

  const nodeColors: Record<string, string> = {
    input: "from-blue-400 to-blue-600",
    llm: "from-violet-400 to-violet-600",
    merge: "from-amber-400 to-amber-600",
    output: "from-emerald-400 to-emerald-600",
  };

  const statusIcons = { success: CheckCircle, failed: XCircle, idle: Circle };
  const statusColors = { success: "text-[var(--success)]", failed: "text-[var(--danger)]", idle: "text-[var(--ink-faint)]" };
</script>

<div class="flex h-full flex-1">
  {#if !selectedWorkflow}
    <div class="flex flex-1 flex-col">
      <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4">
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.workflows")}</h1>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
          <Plus size={14} /> 新建工作流
        </button>
      </header>

      <div class="app-scrollbar flex-1 overflow-y-auto p-4">
        <div class="mx-auto grid max-w-4xl gap-4 md:grid-cols-2">
          {#each mockWorkflows as wf (wf.id)}
            {@const StatusIcon = statusIcons[wf.status]}
            <button
              type="button"
              class="suggestion-card flex flex-col gap-3 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4 text-left transition-shadow hover:shadow-[var(--shadow-md)]"
              onclick={() => { selectedWorkflow = wf.id; }}
            >
              <div class="flex items-center gap-3">
                <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-pink-400 to-pink-600 text-white shadow-sm">
                  <GitBranch size={18} />
                </div>
                <div class="min-w-0 flex-1">
                  <h3 class="text-sm font-semibold text-[var(--ink-strong)]">{wf.name}</h3>
                  <p class="mt-0.5 text-xs text-[var(--ink-faint)]">{wf.nodes} 节点 · {wf.edges} 边</p>
                </div>
                <StatusIcon size={16} class={statusColors[wf.status]} />
              </div>
              <p class="text-xs leading-relaxed text-[var(--ink-muted)]">{wf.description}</p>
              <div class="flex items-center gap-1.5 text-[10px] text-[var(--ink-faint)]">
                <Clock size={10} />
                {wf.lastRun}
              </div>
            </button>
          {/each}
        </div>
      </div>
    </div>
  {:else if activeWorkflow}
    <div class="flex flex-1 flex-col">
      <header class="flex h-12 items-center gap-3 border-b border-[var(--border-soft)] px-4">
        <button type="button" class="icon-hover flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]" onclick={() => { selectedWorkflow = null; }}>
          <ChevronLeft size={18} />
        </button>
        <GitBranch size={16} class="text-[var(--brand)]" />
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{activeWorkflow.name}</h1>
        <div class="flex-1"></div>
        <!-- Tabs -->
        {#each [{ id: "graph" as const, label: "节点图" }, { id: "runs" as const, label: "运行记录" }] as tab}
          <button type="button" class="rounded-[var(--radius-full)] px-3 py-1 text-xs font-medium transition-colors {tab.id === activeTab ? 'bg-[var(--ink-strong)] text-white' : 'text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]'}" onclick={() => { activeTab = tab.id; }}>
            {tab.label}
          </button>
        {/each}
        <div class="mx-1 h-5 w-px bg-[var(--border-soft)]"></div>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--success)] px-3 text-xs font-medium text-white shadow-sm hover:opacity-90">
          <Play size={12} /> 运行
        </button>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
          <Save size={12} /> 保存
        </button>
      </header>

      {#if activeTab === "graph"}
        <!-- Node graph mockup -->
        <div class="flex-1 overflow-auto bg-[var(--bg-app)] p-6">
          <div class="relative mx-auto" style="width: 800px; height: 250px;">
            <!-- Edges (simplified lines) -->
            <svg class="pointer-events-none absolute inset-0" width="800" height="250">
              <defs><marker id="arrowhead" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto"><polygon points="0 0, 8 3, 0 6" fill="var(--ink-faint)" /></marker></defs>
              <line x1="160" y1="110" x2="230" y2="70" stroke="var(--border-medium)" stroke-width="2" marker-end="url(#arrowhead)" />
              <line x1="160" y1="110" x2="230" y2="160" stroke="var(--border-medium)" stroke-width="2" marker-end="url(#arrowhead)" />
              <line x1="370" y1="70" x2="430" y2="110" stroke="var(--border-medium)" stroke-width="2" marker-end="url(#arrowhead)" />
              <line x1="370" y1="160" x2="430" y2="110" stroke="var(--border-medium)" stroke-width="2" marker-end="url(#arrowhead)" />
              <line x1="560" y1="110" x2="630" y2="110" stroke="var(--border-medium)" stroke-width="2" marker-end="url(#arrowhead)" />
            </svg>
            <!-- Nodes -->
            {#each mockNodes as node}
              <div
                class="absolute flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 shadow-[var(--shadow-sm)] transition-shadow hover:shadow-[var(--shadow-md)]"
                style="left: {node.x}px; top: {node.y - 15}px;"
              >
                <div class="flex h-6 w-6 items-center justify-center rounded-full bg-gradient-to-br {nodeColors[node.type]} text-[10px] font-bold text-white">
                  {node.type.charAt(0).toUpperCase()}
                </div>
                <span class="text-xs font-medium text-[var(--ink-strong)]">{node.label}</span>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <!-- Run history -->
        <div class="app-scrollbar flex-1 overflow-y-auto p-4">
          <div class="mx-auto max-w-2xl space-y-2">
            {#each mockRuns as run (run.id)}
              {@const StatusIcon = statusIcons[run.status]}
              <div class="flex items-center gap-3 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-3">
                <StatusIcon size={16} class={statusColors[run.status]} />
                <div class="min-w-0 flex-1">
                  <span class="text-sm font-medium text-[var(--ink-strong)]">{run.id}</span>
                  <span class="ml-2 text-xs text-[var(--ink-faint)]">{run.startedAt}</span>
                </div>
                <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-2 py-0.5 text-[10px] text-[var(--ink-faint)]">{run.duration}</span>
                <span class="text-xs text-[var(--ink-faint)]">{run.tokens} tokens</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
