<script lang="ts">
  import { onMount } from "svelte";
  import AppFrame from "$components/layout/app-frame.svelte";
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
  import { createConversation, renameConversation, deleteConversation } from "$lib/api/conversations";
  import { listenIncrementalPatches } from "$lib/events/patch-bus";
  import { i18n } from "$lib/i18n.svelte";
  import { theme } from "$lib/theme.svelte";

  const activeSidebarItems = $derived(
    appShell.activeWorkspace === "chat"
      ? conversationsState.summaries.map((item) => ({
          id: item.id,
          title: item.title,
          meta: new Date(item.updated_at).toLocaleDateString()
        }))
      : workspaceSidebarItems[appShell.activeWorkspace]
  );

  const activeTitle = $derived(
    appShell.activeWorkspace === "chat"
      ? conversationsState.activeSummary?.title ?? i18n.t("chat.new_conversation")
      : activeSidebarItems.find((item) => item.id === appShell.activeSidebarItemId)?.title ?? "BuYu"
  );

  onMount(() => {
    // Apply theme on mount
    theme.apply();

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

  async function handleCreateConversation() {
    try {
      const detail = await createConversation({
        title: i18n.t("chat.new_conversation"),
        conversation_mode: "chat"
      });
      await conversationsState.loadList();
      appShell.setSidebarItem(detail.summary.id);
      await conversationsState.selectConversation(detail.summary.id);
    } catch (err) {
      console.error("Failed to create conversation:", err);
    }
  }

  async function handleRenameConversation(id: string, title: string) {
    try {
      await renameConversation(id, title);
      await conversationsState.loadList();
    } catch (err) {
      console.error("Failed to rename conversation:", err);
    }
  }

  async function handleDeleteConversation(id: string) {
    try {
      await deleteConversation(id);
      await conversationsState.loadList();
      if (conversationsState.activeConversationId === id) {
        const next = conversationsState.summaries[0];
        if (next) {
          appShell.setSidebarItem(next.id);
          await conversationsState.selectConversation(next.id);
        }
      }
    } catch (err) {
      console.error("Failed to delete conversation:", err);
    }
  }

  function handleRenameTitle(title: string) {
    if (conversationsState.activeConversationId) {
      void handleRenameConversation(conversationsState.activeConversationId, title);
    }
  }
</script>

<svelte:head>
  <title>BuYu</title>
</svelte:head>

<AppFrame
  sidebarOpen={appShell.mobileSidebarOpen}
  inspectorOpen={appShell.mobileInspectorOpen}
  onCloseSidebar={() => appShell.closeMobileSidebar()}
  onCloseInspector={() => appShell.closeMobileInspector()}
>
  {#snippet rail()}
    <NavRail items={navItems} active={appShell.activeWorkspace} onSelect={(id) => void handleWorkspaceSelect(id)} />
  {/snippet}

  {#snippet sidebar()}
    <ResourceSidebar
      workspace={appShell.activeWorkspace}
      items={activeSidebarItems}
      activeId={appShell.activeSidebarItemId}
      onSelect={(id) => void handleSidebarSelect(id)}
      onCreateNew={appShell.activeWorkspace === "chat" ? () => void handleCreateConversation() : undefined}
      onRename={appShell.activeWorkspace === "chat" ? (id, title) => void handleRenameConversation(id, title) : undefined}
      onDelete={appShell.activeWorkspace === "chat" ? (id) => void handleDeleteConversation(id) : undefined}
    />
  {/snippet}

  {#if appShell.activeWorkspace === "chat"}
    <ChatWorkspace
      conversationTitle={activeTitle}
      conversationId={conversationsState.activeConversationId ?? ""}
      loading={conversationsState.loadingConversation || conversationsState.loadingList}
      messages={conversationsState.activeMessages}
      editable={!!conversationsState.activeConversationId}
      onRename={handleRenameTitle}
      onToggleSidebar={() => appShell.toggleMobileSidebar()}
      onToggleInspector={() => appShell.toggleMobileInspector()}
    />
  {:else if appShell.activeWorkspace === "agents"}
    <WorkspacePlaceholder
      eyebrow={i18n.t("nav.agents")}
      title={i18n.t("ws.agents.title")}
      description={i18n.t("ws.agents.desc")}
      bullets={["角色卡编辑器", "问候语管理", "预设与世界书绑定"]}
      cta="创建智能体"
    />
  {:else if appShell.activeWorkspace === "presets"}
    <WorkspacePlaceholder
      eyebrow={i18n.t("nav.presets")}
      title={i18n.t("ws.presets.title")}
      description={i18n.t("ws.presets.desc")}
      bullets={["条目排序", "角色与位置", "渠道绑定"]}
      cta="创建预设"
    />
  {:else if appShell.activeWorkspace === "lorebooks"}
    <WorkspacePlaceholder
      eyebrow={i18n.t("nav.lorebooks")}
      title={i18n.t("ws.lorebooks.title")}
      description={i18n.t("ws.lorebooks.desc")}
      bullets={["条目浏览", "关键词匹配", "插入策略"]}
      cta="创建世界书"
    />
  {:else if appShell.activeWorkspace === "workflows"}
    <WorkspacePlaceholder
      eyebrow={i18n.t("nav.workflows")}
      title={i18n.t("ws.workflows.title")}
      description={i18n.t("ws.workflows.desc")}
      bullets={["图编辑器", "执行记录", "结果写回"]}
      cta="创建工作流"
    />
  {:else}
    <WorkspacePlaceholder
      eyebrow={i18n.t("nav.settings")}
      title={i18n.t("ws.settings.title")}
      description={i18n.t("ws.settings.desc")}
      bullets={["API 渠道", "插件管理", "外观偏好"]}
      cta="打开设置"
    />
  {/if}

  {#snippet inspector()}
    <InspectorPanel
      tabs={inspectorTabs}
      activeTab={appShell.activeInspectorTab}
      onSelectTab={(id) => appShell.setInspectorTab(id)}
    />
  {/snippet}

  {#snippet mobilebar()}
    <MobileTabBar items={navItems} active={appShell.activeWorkspace} onSelect={(id) => void handleWorkspaceSelect(id)} />
  {/snippet}
</AppFrame>
