<script lang="ts">
  import { Plus, Trash2, Code } from "lucide-svelte";
  import Button from "$components/ui/button.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import { onMount } from "svelte";
  import { listVariables, updateVariable, deleteVariable, type ConversationVariable } from "$lib/api/variables";
  import { cn } from "$lib/utils";

  let { conversationId = null }: { conversationId?: string | null } = $props();
  let vars = $state<ConversationVariable[]>([]);
  let loading = $state(false);

  let newKey = $state("");
  let newVal = $state("");

  onMount(loadData);

  async function loadData() {
    if (!conversationId) return;
    loading = true;
    try {
      vars = await listVariables(conversationId);
    } catch {
      vars = [{ key: "user_name", value: "Alice", source: "user", updated_at: Date.now() }];
    } finally { loading = false; }
  }

  async function addVar() {
    if (!conversationId || !newKey.trim()) return;
    try {
      await updateVariable(conversationId, newKey.trim(), newVal);
      newKey = ""; newVal = "";
      await loadData();
    } catch {}
  }

  async function removeVar(key: string) {
    if (!conversationId) return;
    try {
      await deleteVariable(conversationId, key);
      await loadData();
    } catch {}
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-semibold text-[var(--ink-strong)]">运行时变量</h3>
  </div>
  
  <div class="grid gap-2 grid-cols-[1fr_1fr_auto]">
    <input class="w-full rounded-[var(--radius-sm)] border border-[var(--border-medium)] bg-[var(--bg-surface)] px-2 py-1 text-xs outline-none focus:border-[var(--brand)]" placeholder="Key..." bind:value={newKey} />
    <input class="w-full rounded-[var(--radius-sm)] border border-[var(--border-medium)] bg-[var(--bg-surface)] px-2 py-1 text-xs outline-none focus:border-[var(--brand)]" placeholder="Value..." bind:value={newVal} />
    <Button size="sm" variant="secondary" className="h-7 px-2" onclick={addVar} disabled={!newKey.trim()}><Plus size={14} /></Button>
  </div>

  {#if loading}
    <div class="text-center text-xs text-[var(--ink-muted)] py-4">加载中...</div>
  {:else if vars.length === 0}
    <div class="text-center text-xs text-[var(--ink-faint)] py-4 border border-dashed border-[var(--border-medium)] rounded-md">暂无变量</div>
  {:else}
    <div class="space-y-2">
      {#each vars as v}
        <div class="flex items-center justify-between rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--bg-sunken)] px-3 py-2 text-xs">
          <div class="flex items-center gap-2">
            <span class="font-mono font-medium text-[var(--brand)]">{v.key}</span>
            <span class="text-[var(--ink-muted)]">=</span>
            <span class="max-w-[120px] truncate font-mono text-[var(--ink-body)]" title={v.value}>{v.value}</span>
            <span class="text-[9px] bg-[var(--bg-surface)] px-1 rounded-sm border border-[var(--border-soft)] text-[var(--ink-faint)]">{v.source}</span>
          </div>
          <ActionIconButton tone="danger" onClick={()=>removeVar(v.key)}><Trash2 size={12} /></ActionIconButton>
        </div>
      {/each}
    </div>
  {/if}
</div>
