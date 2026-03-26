<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import MinusIcon from "@lucide/svelte/icons/minus";
  import SquareIcon from "@lucide/svelte/icons/square";
  import XIcon from "@lucide/svelte/icons/x";

  type Props = {
    compact?: boolean;
  };

  const { compact = false }: Props = $props();
  const currentWindow = getCurrentWindow();

  async function handleMinimize() {
    await currentWindow.minimize();
  }

  async function handleToggleMaximize() {
    await currentWindow.toggleMaximize();
  }

  async function handleClose() {
    await currentWindow.close();
  }
</script>

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
