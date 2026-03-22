<script lang="ts">
  import GlobalTopbarActions from "$components/layout/global-topbar-actions.svelte";
  import WindowControls from "$components/layout/window-controls.svelte";

  let {
    children,
    rail,
    mobilebar,
    workspaceLabel = "BuYu",
    onOpenSettings = () => {}
  }: {
    children?: import("svelte").Snippet;
    rail?: import("svelte").Snippet;
    mobilebar?: import("svelte").Snippet;
    workspaceLabel?: string;
    onOpenSettings?: () => void;
  } = $props();
</script>

<div class="app-shell text-[var(--ink-body)]">
  <div class="app-shell-grid">
    {#if rail}{@render rail()}{/if}

    <div class="app-main-column">
      <div class="app-window-strip" data-tauri-drag-region>
        <GlobalTopbarActions {workspaceLabel} {onOpenSettings} />
        <WindowControls />
      </div>

      <div class="app-workspace-column">
        {#if children}{@render children()}{/if}
      </div>
    </div>
  </div>

  {#if mobilebar}{@render mobilebar()}{/if}
</div>
