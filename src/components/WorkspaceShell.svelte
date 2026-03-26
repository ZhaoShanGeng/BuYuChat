<script lang="ts">
  import * as Avatar from "$lib/components/ui/avatar/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onDestroy, onMount } from "svelte";
  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import MessageSquarePlusIcon from "@lucide/svelte/icons/message-square-plus";
  import PenLineIcon from "@lucide/svelte/icons/pen-line";
  import PinIcon from "@lucide/svelte/icons/pin";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import AgentSettingsPanel from "./AgentSettingsPanel.svelte";
  import ChatPanel from "./ChatPanel.svelte";
  import IconRail from "./IconRail.svelte";
  import SettingsPage from "./SettingsPage.svelte";
  import { createWorkspaceShellState } from "./workspace-shell.svelte.js";
  import { formatRelativeTime } from "./workspace-state";

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

  function resolveModelName(): string {
    const conversation = ws.state.activeConversation;
    if (!conversation?.channelModelId) return "选择模型";
    const model = ws.activeChannelModels.find((item) => item.id === conversation.channelModelId);
    return model?.displayName ?? model?.modelId ?? "未知模型";
  }

  function resolveAgentName(): string {
    const conversation = ws.state.activeConversation;
    if (!conversation?.agentId) return "";
    return ws.state.agents.find((agent) => agent.id === conversation.agentId)?.name ?? "";
  }

  function resolveChannelName(): string {
    const conversation = ws.state.activeConversation;
    if (!conversation?.channelId) return "";
    return ws.state.channels.find((channel) => channel.id === conversation.channelId)?.name ?? "";
  }
</script>

<div class="flex h-dvh min-h-0 w-full overflow-hidden bg-background text-foreground">
  <IconRail active={ws.state.activeSection} onSwitch={ws.switchSection} />

  {#if ws.state.activeSection !== "settings"}
    <div class="flex h-full w-[clamp(14rem,22vw,18rem)] shrink-0 flex-col overflow-hidden border-r">
      {#if ws.state.activeSection === "chat"}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="flex h-12 items-center justify-between border-b px-4" onmousedown={handleHeaderMouseDown}>
          <div class="min-w-0">
            <span class="text-sm font-semibold">对话</span>
          </div>
          <div class="min-w-4 flex-1"></div>
          <Button class="size-8 rounded-xl" onclick={ws.handleCreateConversation} size="icon" variant="ghost" title="新建会话">
            <MessageSquarePlusIcon class="size-4" />
          </Button>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-2">
          {#if ws.state.bootstrapping}
            <div class="px-3 py-8 text-center text-xs text-muted-foreground">加载中...</div>
          {:else if ws.state.conversations.length === 0}
            <div class="px-3 py-8 text-center text-xs text-muted-foreground">还没有会话</div>
          {:else}
            {#each ws.state.conversations as conv (conv.id)}
              {@const isActive = conv.id === (ws.state.pendingConversationId ?? ws.state.activeConversationId)}
              <ContextMenu.Root>
                <ContextMenu.Trigger>
                  <div class="group relative">
                    <button
                      class={`flex w-full items-center rounded-2xl px-3 py-2.5 text-left transition-colors ${
                        isActive ? "bg-accent" : "hover:bg-accent/50"
                      }`}
                      onclick={() => ws.selectConversation(conv.id)}
                      type="button"
                    >
                      <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-1.5">
                          {#if conv.pinned}
                            <PinIcon class="size-3 shrink-0 text-muted-foreground" />
                          {/if}
                          {#if ws.state.renamingConversationId === conv.id}
                            <Input
                              bind:value={ws.state.renamingConversationTitle}
                              class="h-8 rounded-xl border-transparent bg-background shadow-none"
                              onblur={ws.commitConversationRename}
                              onkeydown={(event) => {
                                if (event.key === "Enter") void ws.commitConversationRename();
                                if (event.key === "Escape") ws.cancelConversationRename();
                              }}
                            />
                          {:else}
                            <span class="truncate text-[13px] font-medium">{conv.title}</span>
                          {/if}
                        </div>
                        <span class="text-[11px] text-muted-foreground">{formatRelativeTime(conv.updatedAt)}</span>
                      </div>
                    </button>

                    <div class="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 transition-opacity group-hover:opacity-100">
                      <DropdownMenu.Root>
                        <DropdownMenu.Trigger>
                          {#snippet child({ props })}
                            <Button {...props} class="size-7 rounded-xl" size="icon" variant="ghost">
                              <EllipsisIcon class="size-3.5" />
                            </Button>
                          {/snippet}
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content align="end" class="w-40">
                          <DropdownMenu.Item onclick={() => ws.startConversationRename(conv)}>
                            <PenLineIcon class="text-muted-foreground" />
                            重命名
                          </DropdownMenu.Item>
                          <DropdownMenu.Item onclick={() => ws.handleTogglePin(conv)}>
                            <PinIcon class="text-muted-foreground" />
                            {conv.pinned ? "取消置顶" : "置顶"}
                          </DropdownMenu.Item>
                          <DropdownMenu.Item onclick={() => ws.handleToggleArchive(conv)}>
                            <ArchiveIcon class="text-muted-foreground" />
                            {conv.archived ? "取消归档" : "归档"}
                          </DropdownMenu.Item>
                          <DropdownMenu.Separator />
                          <DropdownMenu.Item onclick={() => ws.handleDeleteConversation(conv.id)} variant="destructive">
                            <Trash2Icon />
                            删除
                          </DropdownMenu.Item>
                        </DropdownMenu.Content>
                      </DropdownMenu.Root>
                    </div>
                  </div>
                </ContextMenu.Trigger>
                <ContextMenu.Content class="w-40">
                  <ContextMenu.Item onclick={() => ws.startConversationRename(conv)}>
                    <PenLineIcon class="text-muted-foreground" />
                    重命名
                  </ContextMenu.Item>
                  <ContextMenu.Item onclick={() => ws.handleTogglePin(conv)}>
                    <PinIcon class="text-muted-foreground" />
                    {conv.pinned ? "取消置顶" : "置顶"}
                  </ContextMenu.Item>
                  <ContextMenu.Item onclick={() => ws.handleToggleArchive(conv)}>
                    <ArchiveIcon class="text-muted-foreground" />
                    {conv.archived ? "取消归档" : "归档"}
                  </ContextMenu.Item>
                  <ContextMenu.Separator />
                  <ContextMenu.Item onclick={() => ws.handleDeleteConversation(conv.id)} variant="destructive">
                    <Trash2Icon />
                    删除
                  </ContextMenu.Item>
                </ContextMenu.Content>
              </ContextMenu.Root>
            {/each}
          {/if}
        </div>
      {:else}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="flex h-12 items-center justify-between border-b px-4" onmousedown={handleHeaderMouseDown}>
          <div class="min-w-0">
            <span class="text-sm font-semibold">Agent</span>
          </div>
          <div class="min-w-4 flex-1"></div>
          <Button class="size-8 rounded-xl" onclick={ws.startCreateAgent} size="icon" variant="ghost" title="新建 Agent">
            <PlusIcon class="size-4" />
          </Button>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-2">
          {#if ws.state.agents.length === 0}
            <div class="px-3 py-8 text-center text-xs text-muted-foreground">还没有 Agent</div>
          {:else}
            {#each ws.state.agents as agent (agent.id)}
              {@const isActive = ws.state.agentEditingId === agent.id && ws.state.agentEditorMode === "edit"}
              <button
                class={`flex w-full items-center gap-2.5 rounded-2xl px-3 py-2.5 text-left transition-colors ${
                  isActive ? "bg-accent" : "hover:bg-accent/50"
                }`}
                onclick={() => ws.startEditAgent(agent)}
                type="button"
              >
                <Avatar.Root class="size-8 shrink-0 rounded-xl text-[11px] font-bold">
                  <Avatar.Fallback class={`rounded-xl ${agent.enabled ? "bg-violet-600 text-white" : "bg-muted text-muted-foreground"}`}>
                    {agent.name.charAt(0)}
                  </Avatar.Fallback>
                </Avatar.Root>
                <div class="min-w-0 flex-1">
                  <span class="truncate text-[13px] font-medium">{agent.name}</span>
                </div>
                <Badge class={`rounded-full border px-2 py-0.5 text-[10px] ${agent.enabled ? "border-emerald-200 bg-emerald-50 text-emerald-700" : "border-slate-200 bg-slate-100 text-slate-500"}`} variant="outline">
                  {agent.enabled ? "启用" : "禁用"}
                </Badge>
              </button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  <div class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
    {#if ws.state.activeSection === "chat"}
      <ChatPanel
        agentName={resolveAgentName()}
        agents={ws.state.agents}
        channelName={resolveChannelName()}
        channels={ws.state.channels}
        composer={ws.state.composer}
        conversation={ws.state.activeConversation}
        dryRunSummary={ws.state.dryRunSummary}
        loading={ws.state.messagesLoading}
        messageHistory={ws.activeMessageHistory}
        messages={ws.activeMessages}
        modelName={resolveModelName()}
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
        onReroll={ws.handleReroll}
        onSend={ws.handleSendMessage}
        onSwitchVersion={ws.handleSwitchVersion}
        sending={ws.state.sending}
      />
    {:else if ws.state.activeSection === "agents"}
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
    {:else}
      <SettingsPage onChanged={ws.handleSettingsChanged} />
    {/if}
  </div>
</div>
