<script lang="ts">
  import { onMount } from "svelte";
  import AppFrame from "$components/layout/app-frame.svelte";
  import ContextBar from "$components/layout/context-bar.svelte";
  import InspectorPanel from "$components/layout/inspector-panel.svelte";
  import MobileTabBar from "$components/layout/mobile-tab-bar.svelte";
  import NavRail from "$components/layout/nav-rail.svelte";
  import ResourceSidebar from "$components/layout/resource-sidebar.svelte";
  import ChatWorkspace from "$components/chat/chat-workspace.svelte";
  import WorkspacePlaceholder from "$components/shared/workspace-placeholder.svelte";
  import {
    appShell,
    inspectorTabs,
    navItems,
    workspaceSidebarItems,
    type WorkspaceId
  } from "$lib/state/app-shell.svelte";
  import { conversationsState } from "$lib/state/conversations.svelte";
  import { listenIncrementalPatches } from "$lib/events/patch-bus";

  const workspaceMeta: Record<WorkspaceId, string[]> = {
    chat: ["Agent-aware", "Patch-first", "Streaming replies"],
    agents: ["Role cards", "Bindings", "Greetings"],
    presets: ["Prompt entries", "Ordering", "Channel preferences"],
    lorebooks: ["Entry rules", "Match keys", "Insert strategy"],
    workflows: ["Execution graph", "Writeback", "Run traces"],
    settings: ["API channels", "Plugins", "Diagnostics"]
  };

  const activeSidebarItems = $derived(
    appShell.activeWorkspace === "chat"
      ? conversationsState.summaries.map((item) => ({
          id: item.id,
          title: item.title,
          meta: item.description ?? `${item.conversation_mode} · ${new Date(item.updated_at).toLocaleDateString()}`
        }))
      : workspaceSidebarItems[appShell.activeWorkspace]
  );

  const activeTitle = $derived(
    appShell.activeWorkspace === "chat"
      ? conversationsState.activeSummary?.title ?? "Chat"
      : activeSidebarItems.find((item) => item.id === appShell.activeSidebarItemId)?.title ?? "BuYu"
  );

  onMount(() => {
    let unlisten: (() => void) | undefined;

    void (async () => {
      await conversationsState.bootstrap();

      if (conversationsState.activeConversationId) {
        appShell.setSidebarItem(conversationsState.activeConversationId);
      }

      unlisten = await listenIncrementalPatches((event) => {
        conversationsState.applyPatch(event);
      });
    })();

    return () => {
      unlisten?.();
    };
  });

  async function handleWorkspaceSelect(id: WorkspaceId) {
    appShell.setWorkspace(id);

    if (id === "chat") {
      await conversationsState.bootstrap();
      if (conversationsState.activeConversationId) {
        appShell.setSidebarItem(conversationsState.activeConversationId);
      }
    }
  }

  async function handleSidebarSelect(id: string) {
    appShell.setSidebarItem(id);
    if (appShell.activeWorkspace === "chat") {
      await conversationsState.selectConversation(id);
    }
  }
</script>

<svelte:head>
  <title>BuYu</title>
</svelte:head>

<AppFrame sidebarOpen={appShell.mobileSidebarOpen} inspectorOpen={appShell.mobileInspectorOpen}>
  <svelte:fragment slot="rail">
    <NavRail items={navItems} active={appShell.activeWorkspace} onSelect={(id) => void handleWorkspaceSelect(id)} />
  </svelte:fragment>

  <svelte:fragment slot="sidebar">
    <ResourceSidebar
      workspace={appShell.activeWorkspace}
      items={activeSidebarItems}
      activeId={appShell.activeSidebarItemId}
      onSelect={(id) => void handleSidebarSelect(id)}
    />
  </svelte:fragment>

  <svelte:fragment slot="topbar">
    <ContextBar
      title={activeTitle}
      subtitle={appShell.activeWorkspace}
      meta={workspaceMeta[appShell.activeWorkspace]}
      onToggleSidebar={() => appShell.toggleMobileSidebar()}
      onToggleInspector={() => appShell.toggleMobileInspector()}
    />
  </svelte:fragment>

  {#if appShell.activeWorkspace === "chat"}
    <ChatWorkspace
      conversationTitle={conversationsState.activeSummary?.title ?? "Chat"}
      loading={conversationsState.loadingConversation || conversationsState.loadingList}
      messages={conversationsState.activeMessages}
    />
  {:else if appShell.activeWorkspace === "agents"}
    <WorkspacePlaceholder
      eyebrow="Agents"
      title="Role cards with bindings and greetings"
      description="Agents should feel like editable working personas instead of flat settings rows. This workspace will own card content, media, greetings and default bindings."
      bullets={[
        "Card editor with avatar, description, personality and scenario.",
        "Greeting management with active selection and version-safe updates.",
        "Preset, lorebook, user profile and channel bindings in the inspector."
      ]}
      cta="Create agent"
    />
  {:else if appShell.activeWorkspace === "presets"}
    <WorkspacePlaceholder
      eyebrow="Presets"
      title="Prompt choreography, not a flat parameter form"
      description="Presets are modeled as ordered prompt entries with role, depth, position and enabled state. The frontend should make this sequencing obvious and easy to edit."
      bullets={[
        "Entry list with drag reorder and active state.",
        "Role and position editor for each entry.",
        "Optional channel/model preference bindings."
      ]}
      cta="Create preset"
    />
  {:else if appShell.activeWorkspace === "lorebooks"}
    <WorkspacePlaceholder
      eyebrow="Lorebooks"
      title="Knowledge rules with visible matching behavior"
      description="Lorebooks need list, search and detail editing without hiding how keys and insertion rules influence context assembly."
      bullets={[
        "Entry browser with filters and key count overview.",
        "Dedicated detail editor for text, regex and insertion settings.",
        "Inspector-side preview for matching and trigger behavior."
      ]}
      cta="Create lorebook"
    />
  {:else if appShell.activeWorkspace === "workflows"}
    <WorkspacePlaceholder
      eyebrow="Workflows"
      title="Execution graph with readable runtime traces"
      description="Desktop users should be able to design and inspect workflows visually, while mobile users still get a functional list and execution timeline."
      bullets={[
        "Canvas-based graph editor on desktop.",
        "Node list and execution traces on narrow screens.",
        "Runtime writeback, outputs and result messages in one inspector."
      ]}
      cta="Create workflow"
    />
  {:else}
    <WorkspacePlaceholder
      eyebrow="Settings"
      title="Global channels, plugins and diagnostics"
      description="Settings stays narrow in scope: global integration setup, diagnostics and appearance, not every editable resource in the app."
      bullets={[
        "API channel management with model lists.",
        "Plugin registry and capability views.",
        "Appearance, diagnostics and debug surfaces."
      ]}
      cta="Open settings"
    />
  {/if}

  <svelte:fragment slot="inspector">
    <InspectorPanel
      tabs={inspectorTabs}
      activeTab={appShell.activeInspectorTab}
      onSelectTab={(id) => appShell.setInspectorTab(id)}
    />
  </svelte:fragment>

  <svelte:fragment slot="mobilebar">
    <MobileTabBar items={navItems} active={appShell.activeWorkspace} onSelect={(id) => void handleWorkspaceSelect(id)} />
  </svelte:fragment>
</AppFrame>
