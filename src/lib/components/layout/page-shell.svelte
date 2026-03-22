<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    sidebar,
    header,
    mobileTabs,
    toolbar,
    body
  }: {
    sidebar?: Snippet;
    header: Snippet;
    mobileTabs?: Snippet;
    toolbar?: Snippet;
    body: Snippet;
  } = $props();
</script>

<div class="page-shell">
  {#if sidebar}
    <aside class="page-shell__sidebar">
      {@render sidebar()}
    </aside>
  {/if}

  <section class="page-shell__main">
    <div class="page-shell__header">
      {@render header()}
    </div>

    {#if mobileTabs}
      <div class="page-shell__mobile-tabs">
        {@render mobileTabs()}
      </div>
    {/if}

    {#if toolbar}
      <div class="page-shell__toolbar">
        {@render toolbar()}
      </div>
    {/if}

    <div class="page-shell__body">
      {@render body()}
    </div>
  </section>
</div>

<style>
  .page-shell {
    display: flex;
    height: 100%;
    min-height: 0;
    min-width: 0;
    flex: 1 1 auto;
    overflow: hidden;
  }

  .page-shell__sidebar {
    width: var(--page-sidebar-width);
    flex: 0 0 var(--page-sidebar-width);
    min-width: 0;
    overflow: hidden;
    border-right: 1px solid var(--border-soft);
    background: var(--bg-sidebar);
  }

  .page-shell__main {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex: 1 1 auto;
    flex-direction: column;
  }

  .page-shell__header,
  .page-shell__mobile-tabs,
  .page-shell__toolbar {
    flex: 0 0 auto;
  }

  .page-shell__body {
    min-width: 0;
    min-height: 0;
    flex: 1 1 auto;
    overflow: hidden;
  }

  @media (max-width: 1023px) {
    .page-shell__sidebar {
      display: none;
    }
  }
</style>
