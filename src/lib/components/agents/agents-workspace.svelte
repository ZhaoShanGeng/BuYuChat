<script lang="ts">
  import { Plus, Search, Bot, MessageSquare, Image, Link, Trash2, Save, ChevronLeft, Loader2, Edit3, Sparkles } from "lucide-svelte";
  import { i18n } from "$lib/i18n.svelte";

  // Mock data for design preview
  let selectedAgent = $state<string | null>(null);
  let searchQuery = $state("");

  const mockAgents = [
    { id: "1", name: "小助手", description: "通用聊天助手，擅长回答各种问题", greetings: 2, media: 0, presets: 1, color: "from-blue-400 to-blue-600" },
    { id: "2", name: "代码专家", description: "专注于编程和技术问题的AI助手", greetings: 1, media: 0, presets: 2, color: "from-violet-400 to-violet-600" },
    { id: "3", name: "创意写手", description: "擅长创意写作、故事生成和文案创作", greetings: 3, media: 1, presets: 1, color: "from-emerald-400 to-emerald-600" },
  ];

  // Tabs for agent detail
  type TabId = "profile" | "greetings" | "bindings";
  let activeTab = $state<TabId>("profile");

  const tabs: { id: TabId; label: string }[] = [
    { id: "profile", label: "角色设定" },
    { id: "greetings", label: "问候语" },
    { id: "bindings", label: "资源绑定" },
  ];

  const activeAgent = $derived(mockAgents.find(a => a.id === selectedAgent));
</script>

<div class="flex h-full flex-1">
  {#if !selectedAgent}
    <!-- Agent list view -->
    <div class="flex flex-1 flex-col">
      <!-- Header -->
      <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4">
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.agents")}</h1>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
          <Plus size={14} /> 新建智能体
        </button>
      </header>

      <!-- Search -->
      <div class="border-b border-[var(--border-soft)] px-4 py-3">
        <label class="search-box flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2">
          <Search size={14} class="flex-shrink-0 text-[var(--ink-faint)]" />
          <input class="w-full bg-transparent text-sm text-[var(--ink-body)] outline-none placeholder:text-[var(--ink-faint)]" placeholder="搜索智能体…" bind:value={searchQuery} />
        </label>
      </div>

      <!-- Grid -->
      <div class="app-scrollbar flex-1 overflow-y-auto p-4">
        <div class="mx-auto grid max-w-4xl gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {#each mockAgents as agent (agent.id)}
            <button
              type="button"
              class="suggestion-card flex flex-col gap-3 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4 text-left transition-shadow hover:shadow-[var(--shadow-md)]"
              onclick={() => { selectedAgent = agent.id; }}
            >
              <div class="flex items-center gap-3">
                <div class="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br {agent.color} text-lg font-bold text-white shadow-sm">
                  {agent.name.charAt(0)}
                </div>
                <div class="min-w-0">
                  <h3 class="truncate text-sm font-semibold text-[var(--ink-strong)]">{agent.name}</h3>
                  <p class="mt-0.5 text-xs text-[var(--ink-faint)]">{agent.greetings} 条问候语 · {agent.presets} 个预设</p>
                </div>
              </div>
              <p class="line-clamp-2 text-xs leading-relaxed text-[var(--ink-muted)]">{agent.description}</p>
            </button>
          {/each}

          <!-- Create new card -->
          <button type="button" class="flex flex-col items-center justify-center gap-2 rounded-[var(--radius-lg)] border-2 border-dashed border-[var(--border-medium)] bg-transparent px-4 py-8 text-center transition-colors hover:border-[var(--brand)] hover:bg-[var(--brand-soft)]">
            <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[var(--bg-hover)]">
              <Plus size={20} class="text-[var(--ink-faint)]" />
            </div>
            <span class="text-xs font-medium text-[var(--ink-muted)]">创建新智能体</span>
          </button>
        </div>
      </div>
    </div>
  {:else if activeAgent}
    <!-- Agent detail view -->
    <div class="flex flex-1 flex-col">
      <!-- Header with back -->
      <header class="flex h-12 items-center gap-3 border-b border-[var(--border-soft)] px-4">
        <button type="button" class="icon-hover flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]" onclick={() => { selectedAgent = null; }}>
          <ChevronLeft size={18} />
        </button>
        <div class="flex items-center gap-2">
          <div class="flex h-7 w-7 items-center justify-center rounded-[var(--radius-sm)] bg-gradient-to-br {activeAgent.color} text-xs font-bold text-white">
            {activeAgent.name.charAt(0)}
          </div>
          <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{activeAgent.name}</h1>
        </div>
        <div class="flex-1"></div>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
          <Save size={12} /> 保存
        </button>
        <button type="button" class="inline-flex h-8 w-8 items-center justify-center rounded-[var(--radius-md)] text-[var(--ink-faint)] hover:bg-[var(--bg-hover)] hover:text-[var(--danger)]">
          <Trash2 size={14} />
        </button>
      </header>

      <!-- Tabs -->
      <div class="flex gap-1 border-b border-[var(--border-soft)] px-4 py-2">
        {#each tabs as tab}
          <button
            type="button"
            class="rounded-[var(--radius-full)] px-3 py-1 text-xs font-medium transition-colors {tab.id === activeTab ? 'bg-[var(--ink-strong)] text-white' : 'text-[var(--ink-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)]'}"
            onclick={() => { activeTab = tab.id; }}
          >
            {tab.label}
          </button>
        {/each}
      </div>

      <!-- Content -->
      <div class="app-scrollbar flex-1 overflow-y-auto p-6">
        <div class="mx-auto max-w-2xl space-y-6">
          {#if activeTab === "profile"}
            <!-- Profile form -->
            <div class="space-y-4">
              <div>
                <label for="agent-name" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">名称</label>
                <input id="agent-name" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" value={activeAgent.name} />
              </div>
              <div>
                <label for="agent-desc" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">描述</label>
                <input id="agent-desc" class="w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" value={activeAgent.description} />
              </div>
              <div>
                <label for="agent-system" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">系统提示 (System Prompt)</label>
                <textarea id="agent-system" rows="8" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" placeholder="你是一个有用的AI助手…"></textarea>
              </div>
              <div>
                <label for="agent-persona" class="mb-1 block text-xs font-medium text-[var(--ink-muted)]">角色性格 (Persona)</label>
                <textarea id="agent-persona" rows="4" class="w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2 text-sm leading-relaxed text-[var(--ink-body)] outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]" placeholder="描述角色的性格特点、说话方式…"></textarea>
              </div>
            </div>
          {:else if activeTab === "greetings"}
            <!-- Greetings -->
            <div class="space-y-3">
              <div class="flex items-center justify-between">
                <h3 class="text-sm font-semibold text-[var(--ink-strong)]">问候语列表</h3>
                <button type="button" class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--brand-soft)] px-2 text-xs font-medium text-[var(--brand)] hover:bg-[var(--brand)]  hover:text-white">
                  <Plus size={12} /> 添加
                </button>
              </div>
              {#each [1, 2] as idx}
                <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4">
                  <div class="mb-2 flex items-center justify-between">
                    <span class="text-xs font-medium text-[var(--ink-faint)]">问候语 #{idx}</span>
                    <div class="flex items-center gap-1">
                      <button type="button" class="msg-action-btn"><Edit3 size={13} /></button>
                      <button type="button" class="msg-action-btn hover:!text-[var(--danger)]"><Trash2 size={13} /></button>
                    </div>
                  </div>
                  <p class="text-sm leading-relaxed text-[var(--ink-body)]">
                    {idx === 1 ? "你好！我是你的AI助手，有什么可以帮你的吗？" : "嗨！很高兴见到你，让我们开始今天的对话吧。"}
                  </p>
                </div>
              {/each}
            </div>
          {:else}
            <!-- Bindings -->
            <div class="space-y-6">
              {#each [{ title: "预设绑定", icon: Sparkles, items: ["默认预设"] }, { title: "世界书绑定", icon: Link, items: [] }, { title: "API 渠道", icon: Bot, items: ["OpenAI GPT-4o"] }] as section}
                <div>
                  <div class="mb-2 flex items-center justify-between">
                    <div class="flex items-center gap-2">
                      <section.icon size={14} class="text-[var(--ink-faint)]" />
                      <h3 class="text-sm font-semibold text-[var(--ink-strong)]">{section.title}</h3>
                    </div>
                    <button type="button" class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--brand-soft)] px-2 text-xs font-medium text-[var(--brand)] hover:bg-[var(--brand)] hover:text-white">
                      <Plus size={12} /> 添加
                    </button>
                  </div>
                  {#if section.items.length > 0}
                    {#each section.items as item}
                      <div class="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2.5">
                        <span class="text-sm text-[var(--ink-body)]">{item}</span>
                        <button type="button" class="msg-action-btn hover:!text-[var(--danger)]"><Trash2 size={13} /></button>
                      </div>
                    {/each}
                  {:else}
                    <div class="rounded-[var(--radius-sm)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-3 py-4 text-center text-xs text-[var(--ink-faint)]">
                      暂无绑定
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .msg-action-btn {
    display: inline-flex; height: 24px; width: 24px; align-items: center; justify-content: center;
    border-radius: var(--radius-sm); color: var(--ink-faint); transition: all 120ms ease; cursor: pointer;
  }
  .msg-action-btn:hover { background: var(--bg-hover); color: var(--ink-muted); }
</style>
