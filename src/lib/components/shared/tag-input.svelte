<script lang="ts">
  import { X } from "lucide-svelte";
  import { cn } from "$lib/utils";

  let {
    values = [],
    placeholder = "Add a tag...",
    disabled = false,
    className = "",
    onChange
  }: {
    values: string[];
    placeholder?: string;
    disabled?: boolean;
    className?: string;
    onChange: (values: string[]) => void;
  } = $props();

  let inputValue = $state("");
  let inputElement = $state<HTMLInputElement | undefined>(undefined);
  let isFocused = $state(false);

  function addTag(tag: string) {
    const trimmed = tag.trim();
    if (!trimmed) return;
    
    // Prevent duplicates
    if (!values.includes(trimmed)) {
      onChange([...values, trimmed]);
    }
    inputValue = "";
  }

  function removeTag(indexToRemove: number) {
    if (disabled) return;
    onChange(values.filter((_, index) => index !== indexToRemove));
  }

  function handleKeydown(event: KeyboardEvent) {
    if (disabled) return;
    
    if (event.key === "Enter" || event.key === ",") {
      event.preventDefault();
      addTag(inputValue);
    } else if (event.key === "Backspace" && inputValue === "" && values.length > 0) {
      removeTag(values.length - 1);
    }
  }

  function handleBlur() {
    isFocused = false;
    if (inputValue.trim()) {
      addTag(inputValue);
    }
  }
</script>

<div 
  class={cn(
    "flex min-h-[36px] flex-wrap items-center gap-1.5 rounded-[var(--radius-sm)] border bg-[var(--bg-surface)] px-2 py-1.5 transition-colors",
    isFocused ? "border-[var(--brand)] shadow-[0_0_0_2px_var(--brand-glow)]" : "border-[var(--border-medium)]",
    disabled && "opacity-60 cursor-not-allowed bg-[var(--bg-sunken)]",
    className
  )}
  onclick={() => inputElement?.focus()}
  role="presentation"
>
  {#each values as value, idx}
    <span class="inline-flex items-center gap-1 rounded-[var(--radius-full)] bg-[var(--bg-sunken)] px-2.5 py-0.5 text-xs font-medium text-[var(--ink-body)] border border-[var(--border-soft)]">
      {value}
      <button
        type="button"
        class="inline-flex h-3.5 w-3.5 items-center justify-center rounded-full text-[var(--ink-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--danger)] focus:outline-none"
        onclick={(e) => { e.stopPropagation(); removeTag(idx); }}
        disabled={disabled}
      >
        <span class="sr-only">Remove {value}</span>
        <X size={10} />
      </button>
    </span>
  {/each}

  <input
    bind:this={inputElement}
    bind:value={inputValue}
    type="text"
    class="flex-1 min-w-[80px] bg-transparent text-sm text-[var(--ink-strong)] outline-none placeholder:text-[var(--ink-faint)] disabled:cursor-not-allowed"
    {placeholder}
    {disabled}
    onkeydown={handleKeydown}
    onfocus={() => isFocused = true}
    onblur={handleBlur}
  />
</div>
