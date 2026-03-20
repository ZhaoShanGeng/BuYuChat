<script lang="ts">
  import WindowControls from "./window-controls.svelte";

  let {
    sidebarOpen = false,
    inspectorOpen = false,
    onCloseSidebar = () => {},
    onCloseInspector = () => {},
    children,
    rail,
    sidebar,
    inspector,
    mobilebar
  }: {
    sidebarOpen?: boolean;
    inspectorOpen?: boolean;
    onCloseSidebar?: () => void;
    onCloseInspector?: () => void;
    children?: import("svelte").Snippet;
    rail?: import("svelte").Snippet;
    sidebar?: import("svelte").Snippet;
    inspector?: import("svelte").Snippet;
    mobilebar?: import("svelte").Snippet;
  } = $props();
</script>

<div class="app-shell text-[var(--ink-body)]">
  <!-- Custom drag region (titlebar area) -->
  <div class="app-titlebar" data-tauri-drag-region>
    <div class="app-titlebar-spacer"></div>
    <WindowControls />
  </div>

  {#if sidebarOpen || inspectorOpen}
    <button
      aria-label="Close mobile panels"
      class="app-mobile-backdrop lg:hidden"
      type="button"
      onclick={() => {
        onCloseSidebar();
        onCloseInspector();
      }}
    ></button>
  {/if}

  <div class="app-shell-grid">
    {#if rail}{@render rail()}{/if}

    <div class:sheet-open={sidebarOpen} class="app-sheet app-sheet-left">
      {#if sidebar}{@render sidebar()}{/if}
    </div>

    <div class="app-main-column">
      {#if children}{@render children()}{/if}
    </div>

    <div class:sheet-open={inspectorOpen} class="app-sheet app-sheet-right">
      {#if inspector}{@render inspector()}{/if}
    </div>
  </div>

  {#if mobilebar}{@render mobilebar()}{/if}
</div>
