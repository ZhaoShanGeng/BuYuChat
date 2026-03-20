<script lang="ts">
  import { Plus, Search, GripVertical, Trash2, Save, ChevronLeft, Edit3, ChevronDown, ChevronUp, ToggleLeft, ToggleRight, Link } from "lucide-svelte";
  import { i18n } from "$lib/i18n.svelte";

  let selectedPreset = $state<string | null>(null);
  let searchQuery = $state("");

  const mockPresets = [
    { id: "1", name: "默认预设", description: "基础对话预设，包含标准系统提示", entries: 5, channels: 1 },
    { id: "2", name: "角色扮演", description: "适用于角色扮演场景的提示词编排", entries: 8, channels: 2 },
    { id: "3", name: "代码助手", description: "优化了代码生成和分析能力的预设", entries: 4, channels: 1 },
  ];

  const mockEntries = [
    { id: "e1", role: "system", position: "before_chat", label: "系统提示", enabled: true, text: "你是一个有用的AI助手。" },
    { id: "e2", role: "system", position: "after_char", label: "角色增强", enabled: true, text: "请保持角色一致性。" },
    { id: "e3", role: "user", position: "depth_4", label: "上下文提醒", enabled: false, text: "[重要：请记住以上设定]" },
    { id: "e4", role: "system", position: "before_chat", label: "输出格式", enabled: true, text: "请使用Markdown格式回复。" },
    { id: "e5", role: "system", position: "after_chat", label: "安全提示", enabled: true, text: "请遵守内容安全规范。" },
  ];

  const activePreset = $derived(mockPresets.find(p => p.id === selectedPreset));

  const roleColors: Record<string, string> = {
    system: "bg-blue-100 text-blue-700",
    user: "bg-green-100 text-green-700",
    assistant: "bg-purple-100 text-purple-700",
  };

  const positionLabels: Record<string, string> = {
    before_chat: "对话前",
    after_char: "角色卡后",
    after_chat: "对话后",
    depth_4: "深度 4",
  };
</script>

<div class="flex h-full flex-1">
  {#if !selectedPreset}
    <div class="flex flex-1 flex-col">
      <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4">
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.presets")}</h1>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
          <Plus size={14} /> 新建预设
        </button>
      </header>

      <div class="border-b border-[var(--border-soft)] px-4 py-3">
        <label class="search-box flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2">
          <Search size={14} class="flex-shrink-0 text-[var(--ink-faint)]" />
          <input class="w-full bg-transparent text-sm text-[var(--ink-body)] outline-none placeholder:text-[var(--ink-faint)]" placeholder="搜索预设…" bind:value={searchQuery} />
        </label>
      </div>

      <div class="app-scrollbar flex-1 overflow-y-auto p-4">
        <div class="mx-auto max-w-3xl space-y-3">
          {#each mockPresets as preset (preset.id)}
            <button
              type="button"
              class="suggestion-card flex w-full items-center gap-4 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4 text-left transition-shadow hover:shadow-[var(--shadow-md)]"
              onclick={() => { selectedPreset = preset.id; }}
            >
              <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-amber-400 to-amber-600 text-sm font-bold text-white shadow-sm">
                P
              </div>
              <div class="min-w-0 flex-1">
                <h3 class="text-sm font-semibold text-[var(--ink-strong)]">{preset.name}</h3>
                <p class="mt-0.5 text-xs text-[var(--ink-muted)]">{preset.description}</p>
              </div>
              <div class="flex items-center gap-3 text-xs text-[var(--ink-faint)]">
                <span>{preset.entries} 条目</span>
                <span>{preset.channels} 渠道</span>
              </div>
            </button>
          {/each}
        </div>
      </div>
    </div>
  {:else if activePreset}
    <div class="flex flex-1 flex-col">
      <header class="flex h-12 items-center gap-3 border-b border-[var(--border-soft)] px-4">
        <button type="button" class="icon-hover flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]" onclick={() => { selectedPreset = null; }}>
          <ChevronLeft size={18} />
        </button>
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{activePreset.name}</h1>
        <div class="flex-1"></div>
        <button type="button" class="inline-flex h-8 items-center gap-1 rounded-[var(--radius-md)] bg-[var(--brand-soft)] px-2.5 text-xs font-medium text-[var(--brand)] hover:bg-[var(--brand)] hover:text-white">
          <Plus size={12} /> 添加条目
        </button>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
          <Save size={12} /> 保存
        </button>
      </header>

      <!-- Preset entries list -->
      <div class="app-scrollbar flex-1 overflow-y-auto p-4">
        <div class="mx-auto max-w-2xl space-y-2">
          {#each mockEntries as entry, idx (entry.id)}
            <div class="group flex items-start gap-2 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3 transition-shadow hover:shadow-[var(--shadow-sm)]">
              <!-- Drag handle -->
              <div class="mt-1 cursor-grab text-[var(--ink-faint)] opacity-0 transition-opacity group-hover:opacity-100">
                <GripVertical size={14} />
              </div>

              <!-- Order number -->
              <span class="mt-1 min-w-[20px] text-center text-[10px] font-bold text-[var(--ink-faint)]">{idx + 1}</span>

              <!-- Content -->
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-[var(--ink-strong)]">{entry.label}</span>
                  <span class="rounded-[var(--radius-full)] px-1.5 py-0.5 text-[10px] font-medium {roleColors[entry.role] ?? 'bg-gray-100 text-gray-600'}">{entry.role}</span>
                  <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">{positionLabels[entry.position] ?? entry.position}</span>
                </div>
                <p class="mt-1 line-clamp-2 text-xs leading-relaxed text-[var(--ink-muted)]">{entry.text}</p>
              </div>

              <!-- Toggle + actions -->
              <div class="flex items-center gap-1">
                <button type="button" class="text-[var(--ink-faint)] transition-colors" title={entry.enabled ? "启用" : "禁用"}>
                  {#if entry.enabled}
                    <ToggleRight size={20} class="text-[var(--brand)]" />
                  {:else}
                    <ToggleLeft size={20} />
                  {/if}
                </button>
                <button type="button" class="msg-action-btn opacity-0 group-hover:opacity-100"><Edit3 size={13} /></button>
                <button type="button" class="msg-action-btn opacity-0 group-hover:opacity-100 hover:!text-[var(--danger)]"><Trash2 size={13} /></button>
              </div>
            </div>
          {/each}

          <!-- Channel bindings section -->
          <div class="mt-6">
            <div class="mb-2 flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Link size={14} class="text-[var(--ink-faint)]" />
                <h3 class="text-sm font-semibold text-[var(--ink-strong)]">渠道绑定</h3>
              </div>
              <button type="button" class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--brand-soft)] px-2 text-xs font-medium text-[var(--brand)] hover:bg-[var(--brand)] hover:text-white">
                <Plus size={12} /> 绑定渠道
              </button>
            </div>
            <div class="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2.5">
              <div>
                <span class="text-sm text-[var(--ink-body)]">OpenAI</span>
                <span class="ml-2 text-xs text-[var(--ink-faint)]">gpt-4o</span>
              </div>
              <button type="button" class="msg-action-btn hover:!text-[var(--danger)]"><Trash2 size={13} /></button>
            </div>
          </div>
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
