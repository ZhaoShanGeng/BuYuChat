<script lang="ts">
  /**
   * 工作台外壳 — 三栏布局：IconRail + 列表面板 + 主内容区。
   * 参考 LobeChat / CherryStudio 的成熟 AI 聊天布局。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Avatar from "$lib/components/ui/avatar/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import MessageSquarePlusIcon from "@lucide/svelte/icons/message-square-plus";
  import PinIcon from "@lucide/svelte/icons/pin";
  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import IconRail from "./IconRail.svelte";
  import ChatPanel from "./ChatPanel.svelte";
  import AgentSettingsPanel from "./AgentSettingsPanel.svelte";
  import ChannelSettings from "./ChannelSettings.svelte";
  import { createWorkspaceShellState, type ActiveSection } from "./workspace-shell.svelte.js";
  import { formatRelativeTime } from "./workspace-state";

  const ws = createWorkspaceShellState();

  /** 解析模型名称。 */
  function resolveModelName(): string {
    const c = ws.state.activeConversation;
    if (!c?.channelModelId) return "选择模型";
    const m = ws.activeChannelModels.find((m) => m.id === c.channelModelId);
    return m?.displayName ?? m?.modelId ?? "未知模型";
  }

  /** 解析 Agent 名称。 */
  function resolveAgentName(): string {
    const c = ws.state.activeConversation;
    if (!c?.agentId) return "";
    return ws.state.agents.find((a) => a.id === c.agentId)?.name ?? "";
  }
</script>

<div class="flex h-dvh bg-background text-foreground">
  <!-- 1. 图标导航栏 -->
  <IconRail active={ws.state.activeSection} onSwitch={ws.switchSection} />

  <!-- 2. 列表面板（仅 chat 和 agents 显示） -->
  {#if ws.state.activeSection !== "settings"}
    <div class="flex w-[280px] shrink-0 flex-col border-r bg-sidebar">
      {#if ws.state.activeSection === "chat"}
      <!-- 会话列表 -->
      <div class="flex h-12 items-center justify-between border-b px-4">
        <span class="text-sm font-semibold">对话</span>
        <Button class="size-7" onclick={ws.handleCreateConversation} size="icon" variant="ghost" title="新建会话">
          <MessageSquarePlusIcon class="size-4" />
        </Button>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto p-1.5">
        {#if ws.state.bootstrapping || ws.state.conversationsLoading}
          <div class="px-3 py-8 text-center text-xs text-muted-foreground">加载中...</div>
        {:else if ws.state.conversations.length === 0}
          <div class="px-3 py-8 text-center text-xs text-muted-foreground">还没有会话</div>
        {:else}
          {#each ws.state.conversations as conv (conv.id)}
            {@const isActive = conv.id === ws.state.activeConversationId}
            <div class="group relative">
              <button
                class={`flex w-full items-center rounded-lg px-3 py-2 text-left transition-colors ${
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
                    <span class="truncate text-[13px]">{conv.title}</span>
                  </div>
                  <span class="text-[11px] text-muted-foreground">{formatRelativeTime(conv.updatedAt)}</span>
                </div>
              </button>

              <!-- 右键/hover 操作 -->
              <div class="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 transition-opacity group-hover:opacity-100">
                <DropdownMenu.Root>
                  <DropdownMenu.Trigger>
                    {#snippet child({ props })}
                      <Button {...props} class="size-6" size="icon" variant="ghost">
                        <EllipsisIcon class="size-3.5" />
                      </Button>
                    {/snippet}
                  </DropdownMenu.Trigger>
                  <DropdownMenu.Content align="end" class="w-36">
                    <DropdownMenu.Item onclick={() => ws.handleTogglePin(conv)}>
                      <PinIcon class="text-muted-foreground" />
                      {conv.pinned ? "取消置顶" : "置顶"}
                    </DropdownMenu.Item>
                    <DropdownMenu.Item onclick={() => ws.handleToggleArchive(conv)}>
                      <ArchiveIcon class="text-muted-foreground" />
                      {conv.archived ? "取消归档" : "归档"}
                    </DropdownMenu.Item>
                    <DropdownMenu.Separator />
                    <DropdownMenu.Item onclick={() => ws.handleDeleteConversation(conv.id)}>
                      <Trash2Icon class="text-muted-foreground" />
                      删除
                    </DropdownMenu.Item>
                  </DropdownMenu.Content>
                </DropdownMenu.Root>
              </div>
            </div>
          {/each}
        {/if}
      </div>

    {:else if ws.state.activeSection === "agents"}
      <!-- Agent 列表 -->
      <div class="flex h-12 items-center justify-between border-b px-4">
        <span class="text-sm font-semibold">Agent</span>
        <Button class="size-7" onclick={ws.resetAgentForm} size="icon" variant="ghost" title="新建 Agent">
          <PlusIcon class="size-4" />
        </Button>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto p-1.5">
        {#if ws.state.agents.length === 0}
          <div class="px-3 py-8 text-center text-xs text-muted-foreground">还没有 Agent</div>
        {:else}
          {#each ws.state.agents as agent (agent.id)}
            {@const isActive = ws.state.agentEditingId === agent.id}
            <button
              class={`flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-left transition-colors ${
                isActive ? "bg-accent" : "hover:bg-accent/50"
              }`}
              onclick={() => ws.startEditAgent(agent)}
              type="button"
            >
              <Avatar.Root class="size-7 shrink-0 rounded-lg text-[11px] font-bold">
                <Avatar.Fallback class={`rounded-lg ${agent.enabled ? "bg-violet-600 text-white" : "bg-muted text-muted-foreground"}`}>
                  {agent.name.charAt(0)}
                </Avatar.Fallback>
              </Avatar.Root>
              <div class="min-w-0 flex-1">
                <span class="truncate text-[13px]">{agent.name}</span>
              </div>
              {#if agent.enabled}
                <Badge class="shrink-0 rounded-full bg-emerald-500/15 px-1.5 py-0 text-[10px] text-emerald-600" variant="secondary">ON</Badge>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

    {:else}
      <!-- 渠道列表（ChannelSettings 自带列表） -->
    {/if}
    </div>
  {/if}

  <!-- 3. 主内容区 -->
  <div class="flex min-h-0 flex-1 flex-col">
    {#if ws.state.activeSection === "chat"}
      <ChatPanel
        agentName={resolveAgentName()}
        agents={ws.state.agents}
        channels={ws.state.channels}
        composer={ws.state.composer}
        conversation={ws.state.activeConversation}
        dryRunSummary={ws.state.dryRunSummary}
        loading={ws.state.bootstrapping || ws.state.messagesLoading}
        messages={ws.activeMessages}
        modelName={resolveModelName()}
        models={ws.activeChannelModels}
        notice={ws.state.notice}
        onCancel={ws.handleCancelGeneration}
        onComposerChange={ws.setComposer}
        onDeleteVersion={ws.handleDeleteVersion}
        onDryRun={ws.handleDryRun}
        onEditMessage={ws.handleEditMessage}
        onOpenSettings={() => { ws.switchSection("settings"); }}
        onQuickAgentChange={ws.handleQuickAgentChange}
        onQuickChannelChange={ws.handleQuickChannelChange}
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
      <ChannelSettings onChanged={ws.handleChannelsChanged} />
    {/if}
  </div>
</div>
