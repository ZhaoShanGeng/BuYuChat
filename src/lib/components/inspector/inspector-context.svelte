<script lang="ts">
  import { Layers, Database, MessageSquare } from "lucide-svelte";
  let { conversationId = null }: { conversationId?: string | null } = $props();
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-semibold text-[var(--ink-strong)]">上下文装配预览</h3>
  </div>

  <div class="flex items-center justify-between rounded-[var(--radius-sm)] bg-[var(--bg-sunken)] px-3 py-2 text-xs">
    <span class="text-[var(--ink-muted)]">总 Token 占用：</span>
    <span class="font-mono font-medium text-[var(--ink-strong)]">3,245 / 8,192</span>
  </div>

  <div class="space-y-3">
    <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-1.5 text-xs font-semibold text-[var(--ink-strong)]">
          <Layers size={14} class="text-purple-500" /> 系统提示与人设
        </div>
        <span class="text-[10px] font-mono text-[var(--ink-faint)]">1,024 t</span>
      </div>
      <div class="space-y-1 pl-5">
        <div class="text-[11px] text-[var(--ink-muted)] flex justify-between"><span>基础人设</span><span class="font-mono">800 t</span></div>
        <div class="text-[11px] text-[var(--ink-muted)] flex justify-between"><span>代码审查专家 (Preset)</span><span class="font-mono">224 t</span></div>
      </div>
    </div>

    <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-1.5 text-xs font-semibold text-[var(--ink-strong)]">
          <Database size={14} class="text-blue-500" /> 知识与记忆片段
        </div>
        <span class="text-[10px] font-mono text-[var(--ink-faint)]">451 t</span>
      </div>
      <div class="space-y-1 pl-5">
        <div class="text-[11px] text-[var(--ink-muted)] flex justify-between"><span>Rust 错误处理规范</span><span class="font-mono">300 t</span></div>
        <div class="text-[11px] text-[var(--ink-muted)] flex justify-between"><span>Tauri IPC 机制</span><span class="font-mono">151 t</span></div>
      </div>
    </div>

    <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-1.5 text-xs font-semibold text-[var(--ink-strong)]">
          <MessageSquare size={14} class="text-emerald-500" /> 历史消息
        </div>
        <span class="text-[10px] font-mono text-[var(--ink-faint)]">1,770 t</span>
      </div>
      <div class="text-[11px] text-[var(--ink-muted)] pl-5">
        包含最近的 12 条对话消息。
      </div>
    </div>
  </div>
</div>
