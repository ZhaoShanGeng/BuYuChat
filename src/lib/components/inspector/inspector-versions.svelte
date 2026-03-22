<script lang="ts">
  import { History, GitBranch } from "lucide-svelte";
  import Button from "$components/ui/button.svelte";
  let { conversationId = null }: { conversationId?: string | null } = $props();
  // Mock data for versions
  let versions = $state([
    { id: "v3", timestamp: "10:45 AM", isCurrent: true, msgCount: 24, label: "当前分支" },
    { id: "v2", timestamp: "10:30 AM", isCurrent: false, msgCount: 18, label: "重试生成的回复" },
    { id: "v1", timestamp: "10:15 AM", isCurrent: false, msgCount: 16, label: "由于偏离话题回退" }
  ]);
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-semibold text-[var(--ink-strong)]">对话树分支</h3>
  </div>

  <div class="space-y-3 relative before:absolute before:inset-y-2 before:left-[11px] before:w-[2px] before:bg-[var(--border-soft)]">
    {#each versions as v}
      <div class="relative flex gap-3 pl-8">
        <div class="absolute left-0 top-1.5 flex h-[24px] w-[24px] items-center justify-center rounded-full border-2 border-[var(--bg-app)] bg-[var(--brand)] text-white shadow-sm">
          <GitBranch size={12} />
        </div>
        <div class="flex-1 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-3 shadow-sm hover:border-[var(--brand)] cursor-pointer transition-colors">
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs font-semibold text-[var(--ink-strong)]">{v.label}</span>
            <span class="text-[10px] text-[var(--ink-faint)]">{v.timestamp}</span>
          </div>
          <div class="flex items-center justify-between text-[11px] text-[var(--ink-muted)]">
            <span>{v.msgCount} 条消息</span>
            {#if v.isCurrent}
              <span class="bg-[var(--brand-soft)] text-[var(--brand)] px-1.5 py-0.5 rounded-sm">Current</span>
            {:else}
              <Button size="sm" variant="ghost" className="h-5 px-1.5 text-[var(--ink-muted)]">切换并恢复</Button>
            {/if}
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>
