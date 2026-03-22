<script lang="ts">
  import { Network, Play, CheckCircle2, Clock } from "lucide-svelte";
  import Button from "$components/ui/button.svelte";

  let { conversationId = null }: { conversationId?: string | null } = $props();
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-semibold text-[var(--ink-strong)]">当前流程执行</h3>
    <Button size="sm" variant="ghost" className="h-7 px-2"><Network size={14} class="mr-1" />图例模式</Button>
  </div>

  <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3">
    <div class="mb-3 flex items-center gap-2">
      <div class="flex h-6 w-6 items-center justify-center rounded-sm bg-blue-100 text-blue-600"><Play size={12} /></div>
      <div>
        <div class="text-xs font-medium text-[var(--ink-strong)]">Deep Research Pipeline</div>
        <div class="text-[10px] text-[var(--ink-faint)]">正在执行中...</div>
      </div>
    </div>

    <div class="space-y-2 pl-3">
      <div class="flex items-center gap-2 text-xs">
        <CheckCircle2 size={14} class="text-emerald-500" />
        <span class="text-[var(--ink-muted)] line-through">解析用户意图</span>
      </div>
      <div class="flex items-center gap-2 text-xs">
        <CheckCircle2 size={14} class="text-emerald-500" />
        <span class="text-[var(--ink-muted)] line-through">提取搜索关键词</span>
      </div>
      <div class="flex items-center gap-2 text-xs">
        <div class="relative flex h-3.5 w-3.5 items-center justify-center">
          <div class="absolute inset-0 rounded-full border-2 border-r-transparent border-blue-500 animate-spin"></div>
        </div>
        <span class="font-medium text-[var(--brand)]">执行网络检索并抓取</span>
        <span class="text-[10px] text-[var(--ink-faint)]">(3/5 URLs)</span>
      </div>
      <div class="flex items-center gap-2 text-xs">
        <Clock size={14} class="text-[var(--ink-faint)]" />
        <span class="text-[var(--ink-faint)]">总结研报并生成</span>
      </div>
    </div>
  </div>
</div>
