<script lang="ts">
  import { Search } from "lucide-svelte";

  let {
    value = $bindable(""),
    placeholder = ""
  }: {
    value?: string;
    placeholder?: string;
  } = $props();
</script>

<label class="search-box flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-3 py-2">
  <Search size={14} class="flex-shrink-0 text-[var(--ink-faint)]" />
  <input
    class="w-full bg-transparent text-sm text-[var(--ink-body)] outline-none placeholder:text-[var(--ink-faint)]"
    bind:value
    {placeholder}
  />
</label>
