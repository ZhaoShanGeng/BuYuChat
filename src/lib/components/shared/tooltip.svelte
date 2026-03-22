<script lang="ts">
  import { Tooltip as BitsTooltip } from "bits-ui";
  import { fly } from "svelte/transition";

  let {
    text = "",
    placement = "right",
    disabled = false,
    children
  }: {
    text?: string;
    placement?: "top" | "right" | "bottom" | "left";
    disabled?: boolean;
    children?: import("svelte").Snippet;
  } = $props();

  const axisDelta = $derived.by(() => {
    switch (placement) {
      case "left":
        return { x: 6, y: 0 };
      case "right":
        return { x: -6, y: 0 };
      case "bottom":
        return { x: 0, y: -6 };
      case "top":
      default:
        return { x: 0, y: 6 };
    }
  });
</script>

<BitsTooltip.Root disabled={disabled || !text} delayDuration={180}>
  <BitsTooltip.Trigger>
    {#snippet child({ props })}
      <span {...props} class="inline-flex">
        {#if children}
          {@render children()}
        {/if}
      </span>
    {/snippet}
  </BitsTooltip.Trigger>

  <BitsTooltip.Portal>
    <BitsTooltip.Content
      side={placement}
      sideOffset={8}
      collisionPadding={12}
      forceMount
    >
      {#snippet child({ wrapperProps, props, open })}
        {#if open}
          <div {...wrapperProps} class="app-tooltip-wrapper">
            <div
              {...props}
              class="app-tooltip-content"
              transition:fly={{ ...axisDelta, duration: 140 }}
            >
              {text}
              <BitsTooltip.Arrow class="app-tooltip-arrow" />
            </div>
          </div>
        {/if}
      {/snippet}
    </BitsTooltip.Content>
  </BitsTooltip.Portal>
</BitsTooltip.Root>
