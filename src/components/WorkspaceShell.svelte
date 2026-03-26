<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onDestroy, onMount } from "svelte";
  import AgentSettingsPanel from "./AgentSettingsPanel.svelte";
  import AgentSidebarPanel from "./AgentSidebarPanel.svelte";
  import ChatPanel from "./ChatPanel.svelte";
  import ConversationSidebarPanel from "./ConversationSidebarPanel.svelte";
  import IconRail from "./IconRail.svelte";
  import SettingsPage from "./SettingsPage.svelte";
  import { createWorkspaceShellState } from "./workspace-shell.svelte.js";

  const ws = createWorkspaceShellState();
  const currentWindow = getCurrentWindow();

  /** resize 期间抑制所有 transition/animation 以避免卡顿。 */
  let resizeTimer: ReturnType<typeof setTimeout> | null = null;

  function handleWindowResize() {
    document.documentElement.classList.add("resize-active");
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
      document.documentElement.classList.remove("resize-active");
    }, 150);
  }

  onMount(() => {
    window.addEventListener("resize", handleWindowResize);
    return () => {
      window.removeEventListener("resize", handleWindowResize);
      if (resizeTimer) clearTimeout(resizeTimer);
    };
  });

  onDestroy(() => {
    ws.destroy();
  });

  async function handleHeaderMouseDown(event: MouseEvent) {
    const target = event.target as HTMLElement | null;
    if (
      event.button !== 0 ||
      target?.closest("button, input, textarea, select, a, [role='button'], [data-no-drag]")
    ) {
      return;
    }

    await currentWindow.startDragging();
  }

  let activeAgentName = $derived.by(() => {
    const conversation = ws.state.activeConversation;
    if (!conversation?.agentId) return "";
    return ws.state.agents.find((agent) => agent.id === conversation.agentId)?.name ?? "";
  });

  let activeChannelName = $derived.by(() => {
    const conversation = ws.state.activeConversation;
    if (!conversation?.channelId) return "";
    return ws.state.channels.find((channel) => channel.id === conversation.channelId)?.name ?? "";
  });

  let activeModelName = $derived.by(() => {
    const conversation = ws.state.activeConversation;
    if (!conversation?.channelModelId) return "选择模型";
    const model = ws.activeChannelModels.find((item) => item.id === conversation.channelModelId);
    return model?.displayName ?? model?.modelId ?? "未知模型";
  });
</script>

<div class="workspace-shell flex h-dvh min-h-0 w-full overflow-hidden bg-background text-foreground" data-ui="workspace-shell">
  <IconRail active={ws.state.activeSection} onSwitch={ws.switchSection} />

  {#if ws.state.activeSection !== "settings"}
    <aside class="workspace-shell__context flex h-full shrink-0 flex-col overflow-hidden border-r" data-section={ws.state.activeSection} data-ui="workspace-context-sidebar">
      {#if ws.state.activeSection === "chat"}
        <ConversationSidebarPanel
          activeConversationId={ws.state.activeConversationId}
          bootstrapping={ws.state.bootstrapping}
          conversations={ws.state.conversations}
          onCancelRename={ws.cancelConversationRename}
          onCommitRename={ws.commitConversationRename}
          onCreate={ws.handleCreateConversation}
          onDelete={ws.handleDeleteConversation}
          onHeaderMouseDown={handleHeaderMouseDown}
          onRenameTitleChange={(value) => (ws.state.renamingConversationTitle = value)}
          onSelect={ws.selectConversation}
          onStartRename={ws.startConversationRename}
          onToggleArchive={ws.handleToggleArchive}
          onTogglePin={ws.handleTogglePin}
          pendingConversationId={ws.state.pendingConversationId}
          renamingConversationId={ws.state.renamingConversationId}
          renamingConversationTitle={ws.state.renamingConversationTitle}
        />
      {:else}
        <AgentSidebarPanel
          agents={ws.state.agents}
          editingId={ws.state.agentEditorMode === "edit" ? ws.state.agentEditingId : null}
          onCreate={ws.startCreateAgent}
          onHeaderMouseDown={handleHeaderMouseDown}
          onSelect={ws.startEditAgent}
        />
      {/if}
    </aside>
  {/if}

  <div class="workspace-shell__content flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden" data-section={ws.state.activeSection} data-ui="workspace-main-content">
    <div class:hidden={ws.state.activeSection !== "chat"} class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <ChatPanel
        agentName={activeAgentName}
        agents={ws.state.agents}
        channelName={activeChannelName}
        channels={ws.state.channels}
        composer={ws.state.composer}
        conversation={ws.state.activeConversation}
        dryRunSummary={ws.state.dryRunSummary}
        loading={ws.state.messagesLoading}
        messageHistory={ws.activeMessageHistory}
        messages={ws.activeMessages}
        modelName={activeModelName}
        models={ws.activeChannelModels}
        notice={ws.state.notice}
        onCancel={ws.handleCancelGeneration}
        onComposerChange={ws.setComposer}
        onDeleteVersion={ws.handleDeleteVersion}
        onDryRun={ws.handleDryRun}
        onEditMessage={ws.handleEditMessage}
        onLoadOlderMessages={() =>
          ws.state.activeConversationId
            ? ws.loadOlderMessages(ws.state.activeConversationId)
            : Promise.resolve()
        }
        onLoadVersionContent={ws.ensureMessageVersionContent}
        onQuickAgentChange={ws.handleQuickAgentChange}
        onQuickChannelChange={ws.handleQuickChannelChange}
        onQuickChannelMenuOpen={() => ws.syncLatestChannelBindings(true)}
        onQuickModelChange={ws.handleQuickModelChange}
        onQuickTitleChange={ws.handleQuickTitleChange}
        onPendingImagesChange={ws.setPendingImages}
        onReroll={ws.handleReroll}
        onSend={ws.handleSendMessage}
        onSwitchVersion={ws.handleSwitchVersion}
        pendingImages={ws.state.pendingImages}
        sending={ws.state.sending}
      />
    </div>

    {#if ws.state.activeSection === "agents"}
      <AgentSettingsPanel
        agents={ws.state.agents}
        editingId={ws.state.agentEditingId}
        form={ws.state.agentForm}
        isCreating={ws.state.agentEditorMode === "create"}
        onDelete={ws.handleDeleteAgent}
        onEdit={ws.startEditAgent}
        onNameChange={ws.setAgentName}
        onReset={ws.resetAgentForm}
        onSubmit={ws.handleSubmitAgent}
        onSystemPromptChange={ws.setAgentSystemPrompt}
        onToggleEnabled={ws.handleToggleAgentEnabled}
        saving={ws.state.agentSaving}
      />
    {:else if ws.state.activeSection === "settings"}
      <SettingsPage onChanged={ws.handleSettingsChanged} />
    {/if}
  </div>
</div>
