<script lang="ts">
  import { Dialog } from "bits-ui";
  import { Pane, PaneGroup, PaneResizer, type PaneAPI } from "paneforge";
  import { fade, fly } from "svelte/transition";
  import type { Snippet } from "svelte";

  let {
    desktopWide = true,
    sidebarOpen = false,
    inspectorVisible = true,
    inspectorOpen = false,
    onCloseSidebar = () => {},
    onCloseInspector = () => {},
    onOpenInspector = () => {},
    rail,
    header,
    body,
    composer,
    inspector
  }: {
    desktopWide?: boolean;
    sidebarOpen?: boolean;
    inspectorVisible?: boolean;
    inspectorOpen?: boolean;
    onCloseSidebar?: () => void;
    onCloseInspector?: () => void;
    onOpenInspector?: () => void;
    rail: Snippet;
    header: Snippet;
    body: Snippet;
    composer: Snippet;
    inspector: Snippet;
  } = $props();

  let inspectorPane = $state<PaneAPI | null>(null);

  $effect(() => {
    if (!desktopWide || !inspectorPane) return;

    if (inspectorVisible) {
      if (inspectorPane.isCollapsed()) {
        inspectorPane.expand();
      }
      return;
    }

    if (!inspectorPane.isCollapsed()) {
      inspectorPane.collapse();
    }
  });
</script>

{#if desktopWide}
  <PaneGroup
    direction="horizontal"
    autoSaveId="buyu-chat-layout-v1"
    class="chat-shell chat-shell--desktop"
  >
    <Pane defaultSize={24} minSize={18} maxSize={30} class="chat-shell__pane">
      <aside class="chat-shell__sidebar chat-shell__sidebar--desktop">
        {@render rail()}
      </aside>
    </Pane>

    <PaneResizer class="chat-shell__resizer" />

    <Pane defaultSize={54} minSize={40} class="chat-shell__pane">
      <section class="chat-shell__main">
        <div class="chat-shell__header">
          {@render header()}
        </div>

        <div class="chat-shell__body">
          <div class="chat-shell__body-content">
            {@render body()}
          </div>
        </div>

        <div class="chat-shell__composer">
          {@render composer()}
        </div>
      </section>
    </Pane>

    <PaneResizer class={`chat-shell__resizer ${!inspectorVisible ? "hidden" : ""}`} />

    <Pane
      bind:this={inspectorPane}
      defaultSize={22}
      minSize={18}
      maxSize={32}
      collapsible={true}
      collapsedSize={0}
      class="chat-shell__pane chat-shell__pane--inspector"
      onCollapse={() => {
        if (inspectorVisible) {
          onCloseInspector();
        }
      }}
      onExpand={() => {
        if (!inspectorVisible) {
          onOpenInspector();
        }
      }}
    >
      <aside class="chat-shell__inspector-dock">
        <div class="chat-shell__inspector-inner">
          {@render inspector()}
        </div>
      </aside>
    </Pane>
  </PaneGroup>
{:else}
  <div class="chat-shell chat-shell--mobile">
    {#if sidebarOpen}
      <button
        type="button"
        aria-label="关闭聊天侧栏"
        class="chat-shell__mobile-backdrop"
        onclick={onCloseSidebar}
      ></button>
    {/if}

    <aside class:sheet-open={sidebarOpen} class="chat-shell__sidebar chat-shell__sidebar--mobile">
      {@render rail()}
    </aside>

    <section class="chat-shell__main">
      <div class="chat-shell__header">
        {@render header()}
      </div>

      <div class="chat-shell__body">
        <div class="chat-shell__body-content">
          {@render body()}
        </div>
      </div>

      <div class="chat-shell__composer">
        {@render composer()}
      </div>
    </section>

    <Dialog.Root open={inspectorOpen} onOpenChange={(open) => {
      if (open) {
        onOpenInspector();
      } else {
        onCloseInspector();
      }
    }}>
      <Dialog.Portal>
        <Dialog.Overlay class="chat-shell__dialog-overlay" />
        <Dialog.Content
          class="chat-shell__dialog-content"
          interactOutsideBehavior="close"
        >
          <div class="chat-shell__dialog-panel" in:fly={{ x: 24, duration: 220 }} out:fade={{ duration: 140 }}>
            {@render inspector()}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  </div>
{/if}

<style>
  .chat-shell {
    position: relative;
    display: flex;
    height: 100%;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
  }

  :global(.chat-shell--desktop) {
    background: var(--bg-app);
  }

  .chat-shell__sidebar {
    height: 100%;
    min-height: 0;
    overflow: hidden;
    background: var(--bg-sidebar);
  }

  .chat-shell__sidebar--desktop {
    border-right: 1px solid var(--border-soft);
  }

  .chat-shell__main {
    display: flex;
    height: 100%;
    min-height: 0;
    min-width: 0;
    flex-direction: column;
    background: var(--bg-surface);
  }

  .chat-shell__header {
    position: relative;
    z-index: 3;
    flex: 0 0 auto;
  }

  .chat-shell__body {
    min-width: 0;
    min-height: 0;
    flex: 1 1 auto;
    overflow: hidden;
  }

  .chat-shell__body-content {
    display: flex;
    height: 100%;
    min-height: 0;
    min-width: 0;
    flex-direction: column;
  }

  .chat-shell__composer {
    position: relative;
    z-index: 2;
    flex: 0 0 auto;
  }

  .chat-shell__inspector-dock {
    height: 100%;
    min-height: 0;
    overflow: hidden;
    border-left: 1px solid var(--border-soft);
    background: var(--bg-sidebar);
  }

  .chat-shell__inspector-inner {
    height: 100%;
    min-height: 0;
    transform: translateX(0);
    opacity: 1;
    transition:
      opacity 180ms ease,
      transform 220ms cubic-bezier(0.22, 1, 0.36, 1);
    will-change: opacity, transform;
  }

  :global(.chat-shell__resizer) {
    position: relative;
    width: 1px;
    min-width: 1px;
    background: var(--border-soft);
    transition:
      background-color 120ms ease,
      opacity 160ms ease;
  }

  :global(.chat-shell__resizer::after) {
    position: absolute;
    top: 0;
    left: 50%;
    width: 10px;
    height: 100%;
    transform: translateX(-50%);
    content: "";
  }

  :global(.chat-shell__resizer:hover),
  :global(.chat-shell__resizer[data-active]) {
    background: color-mix(in srgb, var(--brand) 40%, var(--border-soft));
  }

  :global(.chat-shell__resizer.hidden) {
    width: 0;
    min-width: 0;
    opacity: 0;
    pointer-events: none;
  }

  :global(.chat-shell__pane--inspector[data-pane]) {
    overflow: hidden;
    transition:
      flex-basis 220ms cubic-bezier(0.22, 1, 0.36, 1),
      width 220ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  :global(.chat-shell__pane--inspector[data-pane-state="collapsed"]) .chat-shell__inspector-inner {
    opacity: 0;
    transform: translateX(12px);
    pointer-events: none;
  }

  :global(.chat-shell__pane--inspector[data-pane-state="expanded"]) .chat-shell__inspector-inner {
    opacity: 1;
    transform: translateX(0);
  }

  .chat-shell__mobile-backdrop {
    position: fixed;
    inset: var(--window-strip-height, 40px) 0 0;
    z-index: 45;
    border: 0;
    background: rgba(15, 23, 42, 0.18);
    backdrop-filter: blur(2px);
  }

  .chat-shell__sidebar--mobile {
    position: fixed;
    top: var(--window-strip-height, 40px);
    left: 0;
    z-index: 50;
    width: min(var(--sidebar-width), calc(100vw - 20px));
    height: calc(100dvh - var(--window-strip-height, 40px));
    transform: translateX(-100%);
    transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
    box-shadow: var(--shadow-lg);
  }

  .chat-shell__sidebar--mobile.sheet-open {
    transform: translateX(0);
  }

  :global(.chat-shell__dialog-overlay) {
    position: fixed;
    inset: var(--window-strip-height, 40px) 0 0;
    z-index: 55;
    background: rgba(15, 23, 42, 0.18);
    backdrop-filter: blur(2px);
  }

  :global(.chat-shell__dialog-content) {
    position: fixed;
    top: var(--window-strip-height, 40px);
    right: 0;
    z-index: 60;
    width: min(var(--inspector-width), calc(100vw - 12px));
    height: calc(100dvh - var(--window-strip-height, 40px));
    overflow: hidden;
    background: var(--bg-sidebar);
    box-shadow: var(--shadow-lg);
    outline: none;
  }

  .chat-shell__dialog-panel {
    height: 100%;
    min-height: 0;
  }
</style>
