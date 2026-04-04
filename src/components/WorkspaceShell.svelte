<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { IsMobile } from "../lib/hooks/is-mobile.svelte";
  import { getOptionalCurrentWindow } from "../lib/tauri-window";
  import AgentSettingsPanel from "./AgentSettingsPanel.svelte";
  import AgentSidebarPanel from "./AgentSidebarPanel.svelte";
  import ChatPanel from "./ChatPanel.svelte";
  import ConversationSidebarPanel from "./ConversationSidebarPanel.svelte";
  import IconRail from "./IconRail.svelte";
  import SettingsChannelSidebar from "./SettingsChannelSidebar.svelte";
  import SettingsPage from "./SettingsPage.svelte";
  import { createSettingsPageState } from "./settings-page-state.svelte.js";
  import { createWorkspaceShellState } from "./workspace-shell.svelte.js";
  import type { ActiveSection } from "./workspace-shell.svelte.js";
  import MessageSquareIcon from "@lucide/svelte/icons/message-square";
  import BotIcon from "@lucide/svelte/icons/bot";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import MenuIcon from "@lucide/svelte/icons/menu";

  const ws = createWorkspaceShellState();
  const currentWindow = getOptionalCurrentWindow();
  const isMobile = new IsMobile();

  const settings = createSettingsPageState({
    onChanged: ws.handleSettingsChanged
  });

  let drawerOpen = $state(false);
  let drawerClosing = $state(false);

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

  /** bootstrap 完成后通知 splash 退场。 */
  $effect(() => {
    if (!ws.state.bootstrapping) {
      window.dispatchEvent(new Event("buyu:ready"));
    }
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

    if (!currentWindow) {
      return;
    }

    await currentWindow.startDragging();
  }

  function openDrawer() {
    drawerOpen = true;
    drawerClosing = false;
  }

  function closeDrawer() {
    if (!drawerOpen || drawerClosing) return;
    drawerClosing = true;
    setTimeout(() => {
      drawerOpen = false;
      drawerClosing = false;
    }, 200);
  }

  function handleDrawerSectionSwitch(section: ActiveSection) {
    ws.switchSection(section);
    if (section === "settings") {
      settings.init();
    }
  }

  function handleSwitchSection(section: ActiveSection) {
    ws.switchSection(section);
    if (section === "settings") {
      settings.init();
    }
  }

  function handleDrawerSelectConversation(id: string) {
    void ws.selectConversation(id);
    closeDrawer();
  }

  async function handleDrawerSelectChannel(channel: import("../lib/transport/channels").Channel) {
    await settings.selectChannel(channel);
    closeDrawer();
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

  const DRAWER_SECTIONS: Array<{ section: ActiveSection; icon: typeof MessageSquareIcon; label: string }> = [
    { section: "chat", icon: MessageSquareIcon, label: "对话" },
    { section: "agents", icon: BotIcon, label: "Agent" },
    { section: "settings", icon: Settings2Icon, label: "设置" }
  ];
</script>

<div class="workspace-shell flex h-dvh min-h-0 w-full overflow-hidden bg-background text-foreground" data-ui="workspace-shell">
  <!-- 桌面端：IconRail -->
  {#if !isMobile.current}
    <IconRail active={ws.state.activeSection} onSwitch={handleSwitchSection} />
  {/if}

  <!-- 桌面端：统一侧边栏 -->
  {#if !isMobile.current}
    <aside class="workspace-shell__context flex h-full shrink-0 flex-col overflow-hidden border-r" data-section={ws.state.activeSection} data-ui="workspace-context-sidebar">
      {#if ws.state.activeSection === "chat"}
        <ConversationSidebarPanel
          activeConversationId={ws.state.activeConversationId}
          agents={ws.state.agents}
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
      {:else if ws.state.activeSection === "agents"}
        <AgentSidebarPanel
          agents={ws.state.agents}
          editingId={ws.state.agentEditorMode === "edit" ? ws.state.agentEditingId : null}
          onCreate={ws.startCreateAgent}
          onHeaderMouseDown={handleHeaderMouseDown}
          onSelect={ws.startEditAgent}
        />
      {:else if ws.state.activeSection === "settings"}
        <SettingsChannelSidebar
          channels={settings.filteredChannels}
          loading={settings.state.loading}
          notice={settings.state.notice}
          onCreate={settings.startCreateChannel}
          onSelect={settings.selectChannel}
          search={settings.state.search}
          selectedChannelEnabled={settings.state.selectedChannelId ? settings.state.form.enabled : false}
          selectedChannelId={settings.state.selectedChannelId}
        />
      {/if}
    </aside>
  {/if}

  <!-- 移动端：Drawer 抽屉 -->
  {#if isMobile.current && drawerOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="drawer-overlay"
      data-closing={drawerClosing || undefined}
      onclick={closeDrawer}
      onkeydown={(e) => e.key === "Escape" && closeDrawer()}
    ></div>
    <div class="drawer-panel" data-closing={drawerClosing || undefined}>
      <!-- Drawer 内的 section 切换 tabs -->
      <div class="flex shrink-0 items-center gap-1 border-b px-3 py-2.5">
        {#each DRAWER_SECTIONS as item}
          {@const isActive = ws.state.activeSection === item.section}
          <button
            class={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium transition-colors ${
              isActive
                ? "bg-primary text-primary-foreground"
                : "text-muted-foreground hover:bg-accent hover:text-foreground"
            }`}
            onclick={() => handleDrawerSectionSwitch(item.section)}
            type="button"
          >
            <item.icon class="size-4" />
            {item.label}
          </button>
        {/each}
      </div>

      <!-- Drawer 内容区：各 section 共用的侧边栏内容 -->
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
        {#if ws.state.activeSection === "chat"}
          <ConversationSidebarPanel
            activeConversationId={ws.state.activeConversationId}
            agents={ws.state.agents}
            bootstrapping={ws.state.bootstrapping}
            conversations={ws.state.conversations}
            onCancelRename={ws.cancelConversationRename}
            onCommitRename={ws.commitConversationRename}
            onCreate={ws.handleCreateConversation}
            onDelete={ws.handleDeleteConversation}
            onHeaderMouseDown={handleHeaderMouseDown}
            onRenameTitleChange={(value) => (ws.state.renamingConversationTitle = value)}
            onSelect={handleDrawerSelectConversation}
            onStartRename={ws.startConversationRename}
            onToggleArchive={ws.handleToggleArchive}
            onTogglePin={ws.handleTogglePin}
            pendingConversationId={ws.state.pendingConversationId}
            renamingConversationId={ws.state.renamingConversationId}
            renamingConversationTitle={ws.state.renamingConversationTitle}
          />
        {:else if ws.state.activeSection === "agents"}
          <AgentSidebarPanel
            agents={ws.state.agents}
            editingId={ws.state.agentEditorMode === "edit" ? ws.state.agentEditingId : null}
            onCreate={ws.startCreateAgent}
            onHeaderMouseDown={handleHeaderMouseDown}
            onSelect={(agent) => { ws.startEditAgent(agent); closeDrawer(); }}
          />
        {:else if ws.state.activeSection === "settings"}
          <SettingsChannelSidebar
            channels={settings.filteredChannels}
            loading={settings.state.loading}
            notice={settings.state.notice}
            onCreate={settings.startCreateChannel}
            onSelect={handleDrawerSelectChannel}
            search={settings.state.search}
            selectedChannelEnabled={settings.state.selectedChannelId ? settings.state.form.enabled : false}
            selectedChannelId={settings.state.selectedChannelId}
          />
        {/if}
      </div>
    </div>
  {/if}

  <!-- 主内容区 -->
  <div class="workspace-shell__content flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden" data-section={ws.state.activeSection} data-ui="workspace-main-content">
    <!-- 移动端：非 chat 页面的统一顶栏 -->
    {#if isMobile.current && ws.state.activeSection !== "chat"}
      <div class="flex min-h-12 shrink-0 items-center gap-2 border-b bg-background px-3 py-2">
        <button
          class="flex size-9 shrink-0 items-center justify-center rounded-xl text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          onclick={openDrawer}
          title="菜单"
          type="button"
        >
          <MenuIcon class="size-5" />
        </button>
        <span class="text-sm font-medium text-foreground">
          {ws.state.activeSection === "agents" ? "Agent" : "设置"}
        </span>
      </div>
    {/if}

    <div class:hidden={ws.state.activeSection !== "chat"} class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <ChatPanel
        agentName={activeAgentName}
        agents={ws.state.agents}
        channelName={activeChannelName}
        channels={ws.state.channels}
        composer={ws.state.composer}
        conversation={ws.state.activeConversation}
        dryRunSummary={ws.state.dryRunSummary}
        enabledTools={ws.state.conversationDraft.enabledTools}
        isMobile={isMobile.current}
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
        onEnabledToolsChange={ws.handleEnabledToolsChange}
        onLoadOlderMessages={() =>
          ws.state.activeConversationId
            ? ws.loadOlderMessages(ws.state.activeConversationId)
            : Promise.resolve()
        }
        onLoadVersionContent={ws.ensureMessageVersionContent}
        onMenuToggle={openDrawer}
        onPendingFilesChange={ws.setPendingFiles}
        onPendingImagesChange={ws.setPendingImages}
        onQuickAgentChange={ws.handleQuickAgentChange}
        onQuickChannelChange={ws.handleQuickChannelChange}
        onQuickChannelMenuOpen={() => ws.syncLatestChannelBindings(true)}
        onQuickModelChange={ws.handleQuickModelChange}
        onQuickTitleChange={ws.handleQuickTitleChange}
        onReroll={ws.handleReroll}
        onSend={ws.handleSendMessage}
        onSwitchVersion={ws.handleSwitchVersion}
        pendingImages={ws.state.pendingImages}
        pendingFiles={ws.state.pendingFiles}
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
      <SettingsPage {settings} />
    {/if}
  </div>
</div>
