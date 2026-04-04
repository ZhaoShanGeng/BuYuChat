<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import MinusIcon from "@lucide/svelte/icons/minus";
  import SquareIcon from "@lucide/svelte/icons/square";
  import XIcon from "@lucide/svelte/icons/x";
  import { getOptionalCurrentWindow } from "../lib/tauri-window";

  type Props = {
    compact?: boolean;
  };

  const { compact = false }: Props = $props();
  const currentWindow = getOptionalCurrentWindow();

  async function handleMinimize() {
    if (!currentWindow) {
      return;
    }

    await currentWindow.minimize();
  }

  async function handleToggleMaximize() {
    if (!currentWindow) {
      return;
    }

    await currentWindow.toggleMaximize();
  }

  async function handleClose() {
    if (!currentWindow) {
      return;
    }

    await currentWindow.close();
  }
</script>

{#if currentWindow}
  <div class={`flex items-center ${compact ? "gap-1" : "gap-1.5"}`}>
    <Button
      class={`rounded-lg text-muted-foreground ${compact ? "size-7" : "size-8"}`}
      onclick={handleMinimize}
      size="icon"
      variant="ghost"
      title="最小化"
    >
      <MinusIcon class="size-4" />
    </Button>
    <Button
      class={`rounded-lg text-muted-foreground ${compact ? "size-7" : "size-8"}`}
      onclick={handleToggleMaximize}
      size="icon"
      variant="ghost"
      title="最大化"
    >
      <SquareIcon class="size-3.5" />
    </Button>
    <Button
      class={`rounded-lg text-muted-foreground hover:bg-destructive hover:text-destructive-foreground ${compact ? "size-7" : "size-8"}`}
      onclick={handleClose}
      size="icon"
      variant="ghost"
      title="关闭"
    >
      <XIcon class="size-4" />
    </Button>
  </div>
{/if}
