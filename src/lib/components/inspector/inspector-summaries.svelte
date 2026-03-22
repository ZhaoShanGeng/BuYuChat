<script lang="ts">
  import { FileText, AlignLeft } from "lucide-svelte";
  import Button from "$components/ui/button.svelte";
  import { onMount } from "svelte";
  import { listSummaries, generateManualSummary, type SummaryLog } from "$lib/api/summaries";

  let { conversationId = null }: { conversationId?: string | null } = $props();
  let summaries = $state<SummaryLog[]>([]);
  let loading = $state(false);

  onMount(loadData);

  async function loadData() {
    if (!conversationId) return;
    loading = true;
    try {
      summaries = await listSummaries(conversationId);
    } catch {
      summaries = [{ id: "mock-sum", conversation_id: conversationId, trigger_reason: "manual", range_start_index: 0, range_end_index: 10, original_token_count: 500, summary_token_count: 50, summary_content: "这是一个模拟生成的长对话历史摘要...", created_at: Date.now() }];
    } finally { loading = false; }
  }

  async function generate() {
    if (!conversationId) return;
    try {
      await generateManualSummary(conversationId);
      await loadData();
    } catch {}
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-semibold text-[var(--ink-strong)]">历史摘要</h3>
    <Button size="sm" variant="secondary" className="h-7 px-2" onclick={generate}><FileText size={14} class="mr-1" /> 手动生成</Button>
  </div>
  
  {#if loading}
    <div class="text-center text-xs text-[var(--ink-muted)] py-4">加载中...</div>
  {:else if summaries.length === 0}
    <div class="text-center text-xs text-[var(--ink-faint)] py-4 border border-dashed border-[var(--border-medium)] rounded-md">暂无摘要记录</div>
  {:else}
    <div class="space-y-3">
      {#each summaries as sum}
        <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3 shadow-sm">
          <div class="mb-2 flex items-center justify-between text-[11px] text-[var(--ink-muted)]">
            <span class="flex items-center gap-1"><AlignLeft size={12} /> 消息 {sum.range_start_index} - {sum.range_end_index}</span>
            <span>触发: {sum.trigger_reason}</span>
          </div>
          <p class="text-xs text-[var(--ink-body)] leading-relaxed">{sum.summary_content}</p>
          <div class="mt-2 text-[10px] text-[var(--ink-faint)]">原 Token: {sum.original_token_count} / 缩短至: {sum.summary_token_count}</div>
        </div>
      {/each}
    </div>
  {/if}
</div>
