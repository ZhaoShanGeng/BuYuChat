<script lang="ts">
  import { Plus, Search, ChevronLeft, Save, Trash2, Edit3, Tag, BookOpen, Hash, ToggleLeft, ToggleRight } from "lucide-svelte";
  import { i18n } from "$lib/i18n.svelte";

  let selectedLorebook = $state<string | null>(null);
  let searchQuery = $state("");

  const mockLorebooks = [
    { id: "1", name: "世界观设定", description: "包含世界的基础背景、规则和历史", entries: 12, enabled: true },
    { id: "2", name: "角色百科", description: "所有角色的详细信息和关系", entries: 25, enabled: true },
    { id: "3", name: "地理位置", description: "世界中的重要地点和场景描述", entries: 8, enabled: false },
  ];

  const mockEntries = [
    { id: "e1", name: "魔法体系", keys: ["魔法", "法术", "元素"], enabled: true, text: "这个世界的魔法基于五种元素：火、水、风、土、雷。", position: "before_char", depth: 4 },
    { id: "e2", name: "精灵族", keys: ["精灵", "精灵族", "艾达"], enabled: true, text: "精灵族是世界上最古老的种族之一，拥有超长的寿命和与自然的亲和力。", position: "after_char", depth: 4 },
    { id: "e3", name: "黑暗森林", keys: ["黑暗森林", "禁地"], enabled: false, text: "黑暗森林是大陆中央被诅咒的区域，充满了危险的魔物。", position: "before_char", depth: 8 },
    { id: "e4", name: "王国历史", keys: ["王国", "历史", "建国"], enabled: true, text: "莱恩王国由第一代国王亚瑟于500年前建立，经历了三次大战。", position: "before_char", depth: 4 },
  ];

  const activeLorebook = $derived(mockLorebooks.find(l => l.id === selectedLorebook));
</script>

<div class="flex h-full flex-1">
  {#if !selectedLorebook}
    <div class="flex flex-1 flex-col">
      <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4">
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("nav.lorebooks")}</h1>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
          <Plus size={14} /> 新建世界书
        </button>
      </header>

      <div class="border-b border-[var(--border-soft)] px-4 py-3">
        <label class="search-box flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2">
          <Search size={14} class="flex-shrink-0 text-[var(--ink-faint)]" />
          <input class="w-full bg-transparent text-sm text-[var(--ink-body)] outline-none placeholder:text-[var(--ink-faint)]" placeholder="搜索世界书…" bind:value={searchQuery} />
        </label>
      </div>

      <div class="app-scrollbar flex-1 overflow-y-auto p-4">
        <div class="mx-auto max-w-3xl space-y-3">
          {#each mockLorebooks as book (book.id)}
            <button
              type="button"
              class="suggestion-card flex w-full items-center gap-4 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4 text-left transition-shadow hover:shadow-[var(--shadow-md)]"
              onclick={() => { selectedLorebook = book.id; }}
            >
              <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-emerald-400 to-emerald-600 text-sm font-bold text-white shadow-sm">
                <BookOpen size={18} />
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <h3 class="text-sm font-semibold text-[var(--ink-strong)]">{book.name}</h3>
                  {#if !book.enabled}
                    <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">已禁用</span>
                  {/if}
                </div>
                <p class="mt-0.5 text-xs text-[var(--ink-muted)]">{book.description}</p>
              </div>
              <span class="text-xs text-[var(--ink-faint)]">{book.entries} 条目</span>
            </button>
          {/each}
        </div>
      </div>
    </div>
  {:else if activeLorebook}
    <div class="flex flex-1 flex-col">
      <header class="flex h-12 items-center gap-3 border-b border-[var(--border-soft)] px-4">
        <button type="button" class="icon-hover flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]" onclick={() => { selectedLorebook = null; }}>
          <ChevronLeft size={18} />
        </button>
        <BookOpen size={16} class="text-[var(--brand)]" />
        <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{activeLorebook.name}</h1>
        <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-2 py-0.5 text-[10px] text-[var(--ink-faint)]">{mockEntries.length} 条目</span>
        <div class="flex-1"></div>
        <button type="button" class="inline-flex h-8 items-center gap-1 rounded-[var(--radius-md)] bg-[var(--brand-soft)] px-2.5 text-xs font-medium text-[var(--brand)] hover:bg-[var(--brand)] hover:text-white">
          <Plus size={12} /> 添加条目
        </button>
        <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
          <Save size={12} /> 保存
        </button>
      </header>

      <!-- Entry list -->
      <div class="app-scrollbar flex-1 overflow-y-auto p-4">
        <div class="mx-auto max-w-2xl space-y-2">
          {#each mockEntries as entry (entry.id)}
            <div class="group rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3 transition-shadow hover:shadow-[var(--shadow-sm)] {!entry.enabled ? 'opacity-50' : ''}">
              <div class="flex items-start gap-3">
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-2">
                    <span class="text-sm font-semibold text-[var(--ink-strong)]">{entry.name}</span>
                    <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">{entry.position} / D{entry.depth}</span>
                  </div>
                  <!-- Keywords -->
                  <div class="mt-1.5 flex flex-wrap gap-1">
                    {#each entry.keys as key}
                      <span class="inline-flex items-center gap-0.5 rounded-[var(--radius-full)] bg-[var(--brand-soft)] px-2 py-0.5 text-[10px] font-medium text-[var(--brand)]">
                        <Hash size={9} />{key}
                      </span>
                    {/each}
                  </div>
                  <p class="mt-1.5 line-clamp-2 text-xs leading-relaxed text-[var(--ink-muted)]">{entry.text}</p>
                </div>
                <div class="flex items-center gap-1">
                  <button type="button" class="text-[var(--ink-faint)]">
                    {#if entry.enabled}<ToggleRight size={20} class="text-[var(--brand)]" />{:else}<ToggleLeft size={20} />{/if}
                  </button>
                  <button type="button" class="msg-action-btn opacity-0 group-hover:opacity-100"><Edit3 size={13} /></button>
                  <button type="button" class="msg-action-btn opacity-0 group-hover:opacity-100 hover:!text-[var(--danger)]"><Trash2 size={13} /></button>
                </div>
              </div>
            </div>
          {/each}
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
