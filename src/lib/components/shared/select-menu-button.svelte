<script lang="ts">
  import { onMount } from "svelte";
  import Button from "$components/ui/button.svelte";
  import Tooltip from "$components/shared/tooltip.svelte";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  type MenuOption = {
    id: string;
    label: string;
    active?: boolean;
    onSelect: () => void;
  };

  let {
    title = "",
    options = [],
    className = "",
    menuClassName = "",
    placement = "top",
    children
  }: {
    title?: string;
    options?: MenuOption[];
    className?: string;
    menuClassName?: string;
    placement?: "top" | "bottom";
    children?: Snippet;
  } = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement | undefined>(undefined);

  function toggle() {
    open = !open;
  }

  function close() {
    open = false;
  }

  function handleDocumentPointerDown(event: PointerEvent) {
    const target = event.target;
    if (!(target instanceof Node) || !root?.contains(target)) {
      close();
    }
  }

  function handleDocumentKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      close();
    }
  }

  onMount(() => {
    document.addEventListener("pointerdown", handleDocumentPointerDown);
    document.addEventListener("keydown", handleDocumentKeydown);

    return () => {
      document.removeEventListener("pointerdown", handleDocumentPointerDown);
      document.removeEventListener("keydown", handleDocumentKeydown);
    };
  });
</script>

<div bind:this={root} class="relative" data-no-drag>
  <Tooltip text={title} disabled={!title || open} placement={placement === "top" ? "left" : "bottom"}>
    {#snippet children()}
      <Button
        variant="ghost"
        size="sm"
        type="button"
        aria-label={title}
        aria-haspopup="menu"
        aria-expanded={open}
        className={cn("h-9 w-9 px-0 text-[var(--ink-faint)] hover:text-[var(--ink-strong)]", className)}
        onclick={toggle}
      >
        {#if children}
          {@render children()}
        {/if}
      </Button>
    {/snippet}
  </Tooltip>

  {#if open}
    <div
      class={cn(
        "absolute z-[950] min-w-[156px] rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-surface)] p-1.5 shadow-[var(--shadow-lg)]",
        placement === "top" ? "bottom-[calc(100%+10px)] left-1/2 -translate-x-1/2" : "top-[calc(100%+10px)] right-0",
        menuClassName
      )}
      role="menu"
    >
      {#each options as option}
        <button
          type="button"
          class={cn(
            "flex w-full items-center justify-between rounded-[calc(var(--radius-sm)-2px)] px-3 py-2 text-left text-xs font-medium transition-colors",
            option.active
              ? "bg-[var(--bg-active)] text-[var(--brand)]"
              : "text-[var(--ink-body)] hover:bg-[var(--bg-hover)]"
          )}
          role="menuitemradio"
          aria-checked={option.active}
          onclick={() => {
            option.onSelect();
            close();
          }}
        >
          <span>{option.label}</span>
          {#if option.active}
            <span class="text-[10px]">●</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
