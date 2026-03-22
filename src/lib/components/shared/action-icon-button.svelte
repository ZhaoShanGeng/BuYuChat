<script lang="ts">
  import Button from "$components/ui/button.svelte";
  import Tooltip from "$components/shared/tooltip.svelte";

  import type { Snippet } from "svelte";

  let {
    title = "",
    onClick = undefined,
    disabled = false,
    tone = "default",
    className = "",
    dataInspectorToggle = false,
    children
  }: {
    title?: string;
    onClick?: ((event: MouseEvent) => void) | undefined;
    disabled?: boolean;
    tone?: "default" | "danger";
    className?: string;
    dataInspectorToggle?: boolean;
    children?: Snippet;
  } = $props();
</script>

<Tooltip text={title} disabled={!title}>
  {#snippet children()}
    <Button
      variant="ghost"
      size="sm"
      type="button"
      aria-label={title}
      disabled={disabled}
      data-inspector-toggle={dataInspectorToggle ? "true" : undefined}
      className={`h-8 w-8 px-0 text-[var(--ink-faint)] ${tone === "danger" ? "hover:text-[var(--danger)]" : "hover:text-[var(--ink-muted)]"} ${className}`}
      onclick={onClick}
    >
      {#if children}{@render children()}{/if}
    </Button>
  {/snippet}
</Tooltip>
