<script lang="ts">
  export let sidebarOpen = false;
  export let inspectorOpen = false;
</script>

<div class="min-h-screen bg-[var(--bg-app)] text-[var(--fg-primary)]">
  <div class="app-shell-grid">
    <slot name="rail" />

    <div class:sheet-open={sidebarOpen} class="app-sheet app-sheet-left">
      <slot name="sidebar" />
    </div>

    <div class="app-main-column">
      <slot name="topbar" />
      <slot />
    </div>

    <div class:sheet-open={inspectorOpen} class="app-sheet app-sheet-right">
      <slot name="inspector" />
    </div>
  </div>

  <slot name="mobilebar" />
</div>
