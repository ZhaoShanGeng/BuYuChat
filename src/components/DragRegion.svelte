<script lang="ts">
  import { getOptionalCurrentWindow } from "../lib/tauri-window";

  type Props = {
    class?: string;
  };

  const { class: className = "" }: Props = $props();
  const currentWindow = getOptionalCurrentWindow();

  async function handleMouseDown(event: MouseEvent) {
    if (event.button !== 0) {
      return;
    }

    if (!currentWindow) {
      return;
    }

    await currentWindow.startDragging();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div aria-hidden="true" class={`select-none ${className}`} onmousedown={handleMouseDown}></div>
