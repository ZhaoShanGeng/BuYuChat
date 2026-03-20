<script lang="ts">
  import { Cable, Plus, Trash2, Edit3, Save, Check, X, Eye, EyeOff, Globe, Palette, Languages, Monitor, Moon, Sun, Plug, Database, Bell, Shield, Key, Server, ChevronRight } from "lucide-svelte";
  import { i18n } from "$lib/i18n.svelte";
  import { theme } from "$lib/theme.svelte";

  type SectionId = "channels" | "appearance" | "general";
  let activeSection = $state<SectionId>("channels");

  // Channel management
  let editingChannel = $state<string | null>(null);

  const mockChannels = [
    { id: "ch1", name: "OpenAI", provider: "openai", baseUrl: "https://api.openai.com/v1", apiKey: "sk-***...***abc", models: ["gpt-4o", "gpt-4o-mini", "o1-preview"], enabled: true },
    { id: "ch2", name: "Anthropic", provider: "anthropic", baseUrl: "https://api.anthropic.com", apiKey: "sk-ant-***...***xyz", models: ["claude-3.5-sonnet", "claude-3-haiku"], enabled: true },
    { id: "ch3", name: "本地 Ollama", provider: "ollama", baseUrl: "http://localhost:11434", apiKey: "", models: ["llama3", "qwen2.5"], enabled: false },
  ];

  const sections: { id: SectionId; label: string; icon: typeof Cable }[] = [
    { id: "channels", label: "API 渠道", icon: Cable },
    { id: "appearance", label: "外观", icon: Palette },
    { id: "general", label: "通用", icon: Monitor },
  ];
</script>

<div class="flex h-full flex-1">
  <!-- Left nav -->
  <div class="hidden w-48 flex-shrink-0 border-r border-[var(--border-soft)] bg-[var(--bg-sidebar)] py-3 md:block">
    <nav class="space-y-0.5 px-2">
      {#each sections as section}
        <button
          type="button"
          class="flex w-full items-center gap-2.5 rounded-[var(--radius-md)] px-3 py-2 text-left text-sm transition-colors {section.id === activeSection ? 'bg-[var(--bg-active)] font-medium text-[var(--ink-strong)]' : 'text-[var(--ink-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)]'}"
          onclick={() => { activeSection = section.id; }}
        >
          <section.icon size={16} />
          {section.label}
        </button>
      {/each}
    </nav>
  </div>

  <!-- Content -->
  <div class="flex flex-1 flex-col">
    <header class="flex h-12 items-center gap-3 border-b border-[var(--border-soft)] px-4 pr-[140px]" data-tauri-drag-region>
      <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{sections.find(s => s.id === activeSection)?.label ?? i18n.t("nav.settings")}</h1>
    </header>

    <div class="app-scrollbar flex-1 overflow-y-auto p-6">
      <div class="mx-auto max-w-2xl">
        {#if activeSection === "channels"}
          <!-- API Channels -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h2 class="text-sm font-semibold text-[var(--ink-strong)]">API 渠道管理</h2>
                <p class="mt-0.5 text-xs text-[var(--ink-muted)]">配置和管理 AI 服务的 API 连接</p>
              </div>
              <button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm hover:bg-[var(--brand-strong)]">
                <Plus size={14} /> 添加渠道
              </button>
            </div>

            {#each mockChannels as channel (channel.id)}
              <div class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-4 transition-shadow hover:shadow-[var(--shadow-sm)]">
                <div class="flex items-center gap-3">
                  <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-cyan-400 to-cyan-600 text-sm font-bold text-white shadow-sm">
                    {channel.provider.charAt(0).toUpperCase()}
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <h3 class="text-sm font-semibold text-[var(--ink-strong)]">{channel.name}</h3>
                      <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--ink-faint)]">{channel.provider}</span>
                      {#if !channel.enabled}
                        <span class="rounded-[var(--radius-full)] bg-[var(--danger)]/10 px-1.5 py-0.5 text-[10px] text-[var(--danger)]">已禁用</span>
                      {/if}
                    </div>
                    <p class="mt-0.5 text-xs text-[var(--ink-faint)]">{channel.baseUrl}</p>
                  </div>
                  <div class="flex items-center gap-1">
                    <button type="button" class="icon-hover flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] hover:bg-[var(--bg-hover)] hover:text-[var(--ink-muted)]">
                      <Edit3 size={14} />
                    </button>
                    <button type="button" class="icon-hover flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] hover:bg-[var(--bg-hover)] hover:text-[var(--danger)]">
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
                <!-- Models -->
                <div class="mt-3 flex flex-wrap gap-1.5">
                  {#each channel.models as model}
                    <span class="inline-flex items-center gap-1 rounded-[var(--radius-full)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-2 py-0.5 text-[10px] font-medium text-[var(--ink-muted)]">
                      <Server size={9} />{model}
                    </span>
                  {/each}
                  <button type="button" class="inline-flex items-center gap-0.5 rounded-[var(--radius-full)] border border-dashed border-[var(--border-medium)] px-2 py-0.5 text-[10px] text-[var(--ink-faint)] transition-colors hover:border-[var(--brand)] hover:text-[var(--brand)]">
                    <Plus size={9} /> 添加模型
                  </button>
                </div>
                <!-- API key preview -->
                <div class="mt-2 flex items-center gap-2">
                  <Key size={11} class="text-[var(--ink-faint)]" />
                  <span class="font-mono text-[10px] text-[var(--ink-faint)]">{channel.apiKey || "未设置"}</span>
                </div>
              </div>
            {/each}
          </div>

        {:else if activeSection === "appearance"}
          <!-- Appearance settings -->
          <div class="space-y-6">
            <div>
              <h2 class="text-sm font-semibold text-[var(--ink-strong)]">主题</h2>
              <p class="mt-0.5 text-xs text-[var(--ink-muted)]">选择你喜欢的颜色主题</p>
              <div class="mt-3 grid grid-cols-2 gap-3">
                <button type="button" class="flex items-center gap-3 rounded-[var(--radius-md)] border-2 px-4 py-3 transition-colors {!theme.isDark ? 'border-[var(--brand)] bg-[var(--brand-soft)]' : 'border-[var(--border-soft)] bg-[var(--bg-surface)] hover:border-[var(--border-medium)]'}" onclick={() => theme.set("light")}>
                  <Sun size={20} class="{!theme.isDark ? 'text-[var(--brand)]' : 'text-[var(--ink-faint)]'}" />
                  <div class="text-left">
                    <span class="text-sm font-medium text-[var(--ink-strong)]">{i18n.t("theme.light")}</span>
                  </div>
                  {#if !theme.isDark}<Check size={16} class="ml-auto text-[var(--brand)]" />{/if}
                </button>
                <button type="button" class="flex items-center gap-3 rounded-[var(--radius-md)] border-2 px-4 py-3 transition-colors {theme.isDark ? 'border-[var(--brand)] bg-[var(--brand-soft)]' : 'border-[var(--border-soft)] bg-[var(--bg-surface)] hover:border-[var(--border-medium)]'}" onclick={() => theme.set("dark")}>
                  <Moon size={20} class="{theme.isDark ? 'text-[var(--brand)]' : 'text-[var(--ink-faint)]'}" />
                  <div class="text-left">
                    <span class="text-sm font-medium text-[var(--ink-strong)]">{i18n.t("theme.dark")}</span>
                  </div>
                  {#if theme.isDark}<Check size={16} class="ml-auto text-[var(--brand)]" />{/if}
                </button>
              </div>
            </div>

            <hr class="border-[var(--border-soft)]" />

            <div>
              <h2 class="text-sm font-semibold text-[var(--ink-strong)]">语言</h2>
              <p class="mt-0.5 text-xs text-[var(--ink-muted)]">切换界面显示语言</p>
              <div class="mt-3 grid grid-cols-2 gap-3">
                <button type="button" class="flex items-center gap-3 rounded-[var(--radius-md)] border-2 px-4 py-3 transition-colors {i18n.locale === 'zh-CN' ? 'border-[var(--brand)] bg-[var(--brand-soft)]' : 'border-[var(--border-soft)] bg-[var(--bg-surface)] hover:border-[var(--border-medium)]'}" onclick={() => i18n.setLocale("zh-CN")}>
                  <span class="text-lg">🇨🇳</span>
                  <span class="text-sm font-medium text-[var(--ink-strong)]">简体中文</span>
                  {#if i18n.locale === "zh-CN"}<Check size={16} class="ml-auto text-[var(--brand)]" />{/if}
                </button>
                <button type="button" class="flex items-center gap-3 rounded-[var(--radius-md)] border-2 px-4 py-3 transition-colors {i18n.locale === 'en' ? 'border-[var(--brand)] bg-[var(--brand-soft)]' : 'border-[var(--border-soft)] bg-[var(--bg-surface)] hover:border-[var(--border-medium)]'}" onclick={() => i18n.setLocale("en")}>
                  <span class="text-lg">🇺🇸</span>
                  <span class="text-sm font-medium text-[var(--ink-strong)]">English</span>
                  {#if i18n.locale === "en"}<Check size={16} class="ml-auto text-[var(--brand)]" />{/if}
                </button>
              </div>
            </div>
          </div>

        {:else}
          <!-- General settings -->
          <div class="space-y-6">
            {#each [
              { title: "数据管理", desc: "管理应用数据、导入导出和备份", icon: Database, items: ["导出所有数据", "导入数据", "清除缓存"] },
              { title: "通知", desc: "管理应用通知和提醒", icon: Bell, items: ["生成完成通知", "错误通知", "更新提醒"] },
              { title: "安全", desc: "安全和隐私设置", icon: Shield, items: ["API 密钥加密", "对话历史保留期", "匿名使用统计"] },
              { title: "插件", desc: "管理已安装的插件和扩展", icon: Plug, items: ["MCP 服务器", "RAG 检索引擎", "TTS 语音合成"] },
            ] as section}
              <div>
                <div class="flex items-center gap-2">
                  <section.icon size={16} class="text-[var(--ink-faint)]" />
                  <div>
                    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">{section.title}</h2>
                    <p class="text-xs text-[var(--ink-muted)]">{section.desc}</p>
                  </div>
                </div>
                <div class="mt-3 space-y-1">
                  {#each section.items as item}
                    <button type="button" class="flex w-full items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2.5 text-sm text-[var(--ink-body)] transition-colors hover:bg-[var(--bg-hover)]">
                      {item}
                      <ChevronRight size={14} class="text-[var(--ink-faint)]" />
                    </button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
