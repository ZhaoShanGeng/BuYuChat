<script lang="ts">
  import { Minus, Square, X, Copy } from "lucide-svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  let isMaximized = $state(false);
  const appWindow = getCurrentWindow();

  onMount(() => {
    // Check initial maximized state
    void appWindow.isMaximized().then(v => { isMaximized = v; });
  });

  async function handleMinimize() {
    await appWindow.minimize();
  }

  async function handleMaximize() {
    await appWindow.toggleMaximize();
    isMaximized = await appWindow.isMaximized();
  }

  async function handleClose() {
    await appWindow.close();
  }
</script>

<div class="window-controls">
  <button
    type="button"
    class="window-btn window-btn-minimize"
    title="最小化"
    onclick={() => void handleMinimize()}
  >
    <Minus size={12} />
  </button>
  <button
    type="button"
    class="window-btn window-btn-maximize"
    title={isMaximized ? "向下还原" : "最大化"}
    onclick={() => void handleMaximize()}
  >
    {#if isMaximized}
      <Copy size={10} />
    {:else}
      <Square size={10} />
    {/if}
  </button>
  <button
    type="button"
    class="window-btn window-btn-close"
    title="关闭"
    onclick={() => void handleClose()}
  >
    <X size={14} />
  </button>
</div>

<style>
  .window-controls {
    display: flex;
    align-items: center;
    gap: 0;
    -webkit-app-region: no-drag;
  }

  .window-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--ink-muted);
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease;
  }

  .window-btn:hover {
    background: var(--bg-hover);
    color: var(--ink-strong);
  }

  .window-btn-close:hover {
    background: #e81123;
    color: #ffffff;
  }
</style>
