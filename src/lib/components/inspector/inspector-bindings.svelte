<script lang="ts">
  import { FileBox, BookMarked } from "lucide-svelte";
  
  let { conversationId = null }: { conversationId?: string | null } = $props();
  // Mock data
  let bindings = $state([
    { type: "preset", name: "代码审查专家", enabled: true },
    { type: "lorebook", name: "Rust 项目规范", enabled: true },
    { type: "lorebook", name: "Tauri 开发指南", enabled: false }
  ]);
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-semibold text-[var(--ink-strong)]">会话级资源绑定</h3>
  </div>

  <div class="text-xs text-[var(--ink-muted)] leading-relaxed">
    您可以为当前对话单独开关智能体绑定的资源，或临时追加新的预设与世界书。
  </div>

  <div class="space-y-2">
    {#each bindings as b}
      <div class="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2.5 shadow-sm">
        <div class="flex items-center gap-2">
          {#if b.type === "preset"}
            <FileBox size={14} class="text-[var(--brand)]" />
          {:else}
            <BookMarked size={14} class="text-indigo-500" />
          {/if}
          <span class="text-xs font-medium text-[var(--ink-body)]">{b.name}</span>
        </div>
        <input type="checkbox" bind:checked={b.enabled} class="h-3 w-3" />
      </div>
    {/each}
  </div>

  <button class="w-full rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] py-3 text-center text-xs text-[var(--ink-muted)] hover:border-[var(--brand)] hover:text-[var(--brand)] transition-colors">
    + 添加临时绑定
  </button>
</div>
