<script lang="ts">
  import { Tooltip } from "bits-ui";
  import { onMount } from "svelte";
  import { Toaster } from "svelte-sonner";
  import { toast } from "svelte-sonner";
  import AppFrame from "$components/layout/app-frame.svelte";
  import MobileTabBar from "$components/layout/mobile-tab-bar.svelte";
  import NavRail from "$components/layout/nav-rail.svelte";
  import WorkspaceContent from "$components/layout/workspace-content.svelte";
  import { listAgents, type AgentSummary } from "$lib/api/agents";
  import {
    listApiChannelModels,
    listApiChannels,
    type ApiChannel,
    type ApiChannelModel
  } from "$lib/api/api-channels";
  import {
    appShell,
    navItems,
    type InspectorTabId,
    type WorkspaceId
  } from "$lib/state/app-shell.svelte";
  import { conversationsState } from "$lib/state/conversations.svelte";
  import { workspaceSidebarItems } from "$lib/fixtures/workspaces";
  import {
    createConversation,
    deleteConversation,
    replaceConversationChannels,
    renameConversation,
    updateConversationMeta,
    type ConversationDetail
  } from "$lib/api/conversations";
  import type { MessageVersionView } from "$lib/api/messages";
  import { listenGenerationStream } from "$lib/events/generation-stream";
  import { listenIncrementalPatches } from "$lib/events/patch-bus";
  import {
    buildConversationChatConfigFromParticipants,
    mergeConversationChatConfig
  } from "$lib/chat/conversation-preferences";
  import { i18n } from "$lib/i18n.svelte";
  import { generationJobsState } from "$lib/state/generation-jobs.svelte";
  import { theme } from "$lib/theme.svelte";

  let availableAgents = $state<AgentSummary[]>([]);

  const activeSidebarItems = $derived(
    appShell.activeWorkspace === "chat"
      ? conversationsState.summaries.map((item) => ({
          id: item.id,
          title: item.title,
          updatedAt: item.updated_at,
          busyCount: generationJobsState.inFlightCountForConversation(item.id),
          unreadCount: generationJobsState.unreadCountForConversation(item.id)
        }))
      : workspaceSidebarItems[appShell.activeWorkspace]
  );

  const activeTitle = $derived(
    appShell.activeWorkspace === "chat"
      ? conversationsState.activeSummary?.title ?? i18n.t("chat.new_conversation")
      : activeSidebarItems.find((item: { id: string; title: string }) => item.id === appShell.activeSidebarItemId)?.title ?? "BuYu"
  );
  const workspaceLabel = $derived(i18n.t(`nav.${appShell.activeWorkspace}`));
  const isChatWorkspace = $derived(appShell.activeWorkspace === "chat");
  const localizedInspectorTabs = $derived<{ id: InspectorTabId; label: string }[]>([
    { id: "context", label: i18n.t("inspector.tab.context") },
    { id: "versions", label: i18n.t("inspector.tab.versions") },
    { id: "summaries", label: i18n.t("inspector.tab.summaries") },
    { id: "variables", label: i18n.t("inspector.tab.variables") },
    { id: "bindings", label: i18n.t("inspector.tab.bindings") },
    { id: "workflow", label: i18n.t("inspector.tab.workflow") }
  ]);

  $effect(() => {
    generationJobsState.setActiveConversation(conversationsState.activeConversationId);
  });

  onMount(() => {
    // Apply theme on mount
    theme.apply();

    let unlistenPatches: (() => void) | undefined;
    let unlistenGeneration: (() => void) | undefined;

    void (async () => {
      await loadAgents();
      await conversationsState.bootstrap();

      if (conversationsState.activeConversationId) {
        appShell.setSidebarItem(conversationsState.activeConversationId);
      }

      unlistenPatches = await listenIncrementalPatches((event) => {
        conversationsState.applyPatch(event);

        if (
          event.resource_kind === "message_version" &&
          event.op === "upsert" &&
          event.data &&
          typeof event.data === "object"
        ) {
          generationJobsState.resolveMessage(event.data as MessageVersionView);
        }
      });
      unlistenGeneration = await listenGenerationStream((event) => {
        generationJobsState.applyEvent(event);
      });
    })();

    return () => {
      unlistenPatches?.();
      unlistenGeneration?.();
    };
  });

  async function loadAgents() {
    try {
      const items = await listAgents();
      availableAgents = [...items]
        .filter((agent) => agent.enabled)
        .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
    } catch (error) {
      console.error("Failed to load agents:", error);
      availableAgents = [];
    }
  }

  function buildConversationParticipants(agentId: string) {
    return [
      {
        agent_id: null,
        display_name: i18n.t("chat.user_label"),
        participant_type: "human",
        enabled: true,
        sort_order: 0,
        config_json: {}
      },
      {
        agent_id: agentId,
        display_name: null,
        participant_type: "agent",
        enabled: true,
        sort_order: 1,
        config_json: {}
      }
    ];
  }

  async function persistConversationResponderConfig(
    detail: ConversationDetail,
    primaryAgentId: string,
    preferredAgentIds: string[]
  ) {
    const chatConfig = buildConversationChatConfigFromParticipants(
      detail.participants,
      primaryAgentId,
      preferredAgentIds
    );

    return updateConversationMeta(detail.summary.id, {
      title: detail.summary.title,
      description: detail.summary.description,
      archived: detail.summary.archived,
      pinned: detail.summary.pinned,
      config_json: mergeConversationChatConfig(detail.summary, chatConfig)
    });
  }

  function hasActiveConversationChannelBinding(detail: ConversationDetail | null | undefined) {
    return !!detail?.channel_bindings.some(
      (binding) => binding.enabled && binding.binding_type === "active"
    );
  }

  function sortChannels(items: ApiChannel[]) {
    return [...items].sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
  }

  function sortChannelModels(items: ApiChannelModel[]) {
    return [...items].sort(
      (a, b) =>
        a.sort_order - b.sort_order ||
        (a.display_name ?? a.model_id).localeCompare(b.display_name ?? b.model_id)
    );
  }

  async function resolveDefaultChannelSelection() {
    const channels = sortChannels((await listApiChannels()).filter((channel) => channel.enabled));

    for (const channel of channels) {
      const models = sortChannelModels(await listApiChannelModels(channel.id));
      const model = models[0];
      if (model) {
        return {
          channelId: channel.id,
          channelModelId: model.id
        };
      }
    }

    return null;
  }

  async function ensureConversationChannelBinding(
    conversationId: string,
    detail: ConversationDetail | null | undefined
  ) {
    if (hasActiveConversationChannelBinding(detail)) {
      return detail ?? null;
    }

    const selection = await resolveDefaultChannelSelection();
    if (!selection) {
      return detail ?? null;
    }

    await replaceConversationChannels(conversationId, [
      {
        channel_id: selection.channelId,
        channel_model_id: selection.channelModelId,
        binding_type: "active",
        enabled: true,
        sort_order: 0
      }
    ]);

    await conversationsState.loadConversation(conversationId);
    return conversationsState.detailsById[conversationId] ?? detail ?? null;
  }

  function getDefaultAgentId() {
    return availableAgents[0]?.id ?? null;
  }

  function hasChatParticipants() {
    const detail = conversationsState.activeDetail;
    if (!detail) return false;

    const hasHuman = detail.participants.some(
      (participant) => participant.enabled && participant.participant_type === "human"
    );
    const hasAgent = detail.participants.some(
      (participant) =>
        participant.enabled &&
        participant.participant_type === "agent" &&
        !!participant.agent_id
    );

    return hasHuman && hasAgent;
  }

  async function createConversationForAgent(agentId: string, title?: string) {
    let detail = await createConversation({
      title: title ?? i18n.t("chat.new_conversation"),
      conversation_mode: "chat",
      participants: buildConversationParticipants(agentId)
    });

    detail = await persistConversationResponderConfig(detail, agentId, [agentId]);
    detail = (await ensureConversationChannelBinding(detail.summary.id, detail)) ?? detail;

    if (!hasActiveConversationChannelBinding(detail)) {
      toast.error(i18n.t("chat.send_failed"), {
        description: i18n.t("chat.no_channel_desc")
      });
    }

    await conversationsState.loadList();
    appShell.setSidebarItem(detail.summary.id);
    await conversationsState.selectConversation(detail.summary.id);
    return conversationsState.activeDetail ?? detail;
  }

  async function handleWorkspaceSelect(id: WorkspaceId) {
    appShell.setWorkspace(id);

    if (id === "chat") {
      await loadAgents();
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
    const agentId = getDefaultAgentId();
    if (!agentId) {
      toast.error(i18n.t("chat.no_agent_title"), {
        description: i18n.t("chat.no_agent_desc")
      });
      return;
    }

    try {
      await createConversationForAgent(agentId);
    } catch (err) {
      console.error("Failed to create conversation:", err);
      toast.error(i18n.t("chat.create_conversation_failed"), {
        description: err instanceof Error ? err.message : i18n.t("chat.generic_error")
      });
    }
  }

  async function ensureConversation(preferredAgentId?: string): Promise<ConversationDetail | null> {
    if (conversationsState.activeConversationId && hasChatParticipants() && conversationsState.activeDetail) {
      return await ensureConversationChannelBinding(
        conversationsState.activeConversationId,
        conversationsState.activeDetail
      );
    }

    const agentId = preferredAgentId ?? getDefaultAgentId();
    if (!agentId) {
      return null;
    }

    try {
      return await createConversationForAgent(agentId);
    } catch (err) {
      console.error("Failed to ensure conversation:", err);
      return null;
    }
  }

  async function handleStartConversationWithAgent(agentId: string) {
    try {
      const detail = await createConversationForAgent(agentId);
      return detail.summary.id;
    } catch (error) {
      console.error("Failed to start conversation with agent:", error);
      toast.error(i18n.t("chat.create_conversation_failed"), {
        description: error instanceof Error ? error.message : i18n.t("chat.generic_error")
      });
      return null;
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

<Tooltip.Provider delayDuration={180} skipDelayDuration={80}>
  <AppFrame
    {workspaceLabel}
    onOpenSettings={() => void handleWorkspaceSelect("settings")}
  >
    {#snippet rail()}
      <NavRail items={navItems} active={appShell.activeWorkspace} onSelect={(id) => void handleWorkspaceSelect(id)} />
    {/snippet}

    <WorkspaceContent
      workspace={appShell.activeWorkspace}
      conversationTitle={activeTitle}
      conversationId={conversationsState.activeConversationId ?? ""}
      conversationDetail={conversationsState.activeDetail}
      loading={conversationsState.loadingConversation || conversationsState.loadingList}
      messages={conversationsState.activeMessages}
      editable={!!conversationsState.activeConversationId}
      onEnsureConversation={ensureConversation}
      {availableAgents}
      onStartConversationWithAgent={handleStartConversationWithAgent}
      desktopWide={appShell.desktopWide}
      sidebarItems={isChatWorkspace ? activeSidebarItems : []}
      activeSidebarId={isChatWorkspace ? appShell.activeSidebarItemId : ""}
      sidebarOpen={appShell.mobileSidebarOpen}
      inspectorVisible={appShell.inspectorVisible}
      inspectorOpen={appShell.mobileInspectorOpen}
      inspectorTabs={localizedInspectorTabs}
      activeInspectorTab={appShell.activeInspectorTab}
      onRename={handleRenameTitle}
      onSelectSidebar={(id) => void handleSidebarSelect(id)}
      onCreateSidebarItem={isChatWorkspace ? () => void handleCreateConversation() : undefined}
      onRenameSidebarItem={isChatWorkspace ? (id, title) => void handleRenameConversation(id, title) : undefined}
      onDeleteSidebarItem={isChatWorkspace ? (id) => void handleDeleteConversation(id) : undefined}
      onToggleSidebar={() => appShell.toggleMobileSidebar()}
      onToggleInspector={() => appShell.toggleInspector()}
      onOpenInspector={() => appShell.openInspector()}
      onCloseSidebar={() => appShell.closeMobileSidebar()}
      onCloseInspector={() => appShell.closeInspector()}
      onSelectInspectorTab={(id) => appShell.setInspectorTab(id)}
    />

    {#snippet mobilebar()}
      <MobileTabBar items={navItems} active={appShell.activeWorkspace} onSelect={(id) => void handleWorkspaceSelect(id)} />
    {/snippet}
  </AppFrame>
  <Toaster theme={theme.resolved} richColors closeButton position="top-right" />
</Tooltip.Provider>
