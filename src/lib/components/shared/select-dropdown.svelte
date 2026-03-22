<script lang="ts" generics="T">
  import { ChevronDown, Check } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import { clickOutside } from "$lib/actions/click-outside";

  let {
    value,
    options = [],
    placeholder = "请选择...",
    disabled = false,
    className = "",
    onChange
  }: {
    value?: T;
    options: { value: T; label: string; description?: string }[];
    placeholder?: string;
    disabled?: boolean;
    className?: string;
    onChange: (value: T) => void;
  } = $props();

  let isOpen = $state(false);
  let triggerRef = $state<HTMLButtonElement | undefined>(undefined);

  const selectedOption = $derived(options.find(opt => opt.value === value));

  function toggleOpen() {
    if (disabled) return;
    isOpen = !isOpen;
  }

  function selectOption(val: T) {
    if (disabled) return;
    onChange(val);
    isOpen = false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (disabled) return;
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      toggleOpen();
    } else if (event.key === "Escape") {
      isOpen = false;
    }
  }

  function handleClickOutside() {
    isOpen = false;
  }
</script>

<!-- Note: In a real world scenario, a full-featured portal component like bits-ui Select should be used.
     For this specific project, this implements the core visual styling we defined. -->
<div class={cn("relative z-10", className)}>
  <button
    bind:this={triggerRef}
    type="button"
    class={cn(
      "flex h-9 w-full items-center justify-between gap-2 border bg-[var(--bg-surface)] px-3 py-2 text-sm transition-colors",
      "rounded-[var(--radius-sm)] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--brand)]",
      isOpen ? "border-[var(--brand)] shadow-[0_0_0_2px_var(--brand-glow)]" : "border-[var(--border-medium)]",
      disabled ? "cursor-not-allowed opacity-60 bg-[var(--bg-sunken)]" : "cursor-pointer hover:bg-[var(--bg-hover)]"
    )}
    onclick={toggleOpen}
    onkeydown={handleKeydown}
    aria-haspopup="listbox"
    aria-expanded={isOpen}
    {disabled}
  >
    <span class={cn("truncate", !selectedOption && "text-[var(--ink-faint)]")}>
      {selectedOption ? selectedOption.label : placeholder}
    </span>
    <ChevronDown size={14} class={cn("flex-shrink-0 text-[var(--ink-muted)] transition-transform duration-200", isOpen && "rotate-180")} />
  </button>

  {#if isOpen}
    <div
      class="absolute top-full left-0 mt-1 max-h-[300px] w-full min-w-[max-content] overflow-auto rounded-[var(--radius-md)] border border-[var(--border-strong)] bg-[var(--bg-surface)] p-1 shadow-[var(--shadow-md)] animate-in fade-in slide-in-from-top-2 z-50 app-scrollbar"
      use:clickOutside={handleClickOutside}
      role="listbox"
    >
      {#if options.length === 0}
        <div class="px-3 py-2 text-center text-xs text-[var(--ink-faint)]">
          无可用选项
        </div>
      {:else}
        {#each options as option}
          <button
            type="button"
            class={cn(
              "flex w-full cursor-pointer items-center justify-between gap-2 rounded-sm px-2 py-1.5 text-left text-sm transition-colors hover:bg-[var(--bg-hover)]",
              option.value === value ? "bg-[var(--bg-active)] text-[var(--brand)]" : "text-[var(--ink-body)]"
            )}
            role="option"
            aria-selected={option.value === value}
            onclick={() => selectOption(option.value)}
          >
            <div class="flex flex-col">
              <span class={cn("truncate font-medium", option.value === value && "font-semibold")}>
                {option.label}
              </span>
              {#if option.description}
                <span class="truncate text-xs text-[var(--ink-faint)]">{option.description}</span>
              {/if}
            </div>
            {#if option.value === value}
              <Check size={14} class="flex-shrink-0" />
            {/if}
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>
