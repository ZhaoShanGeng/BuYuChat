<script lang="ts">
  /**
   * 全页面设置 — CherryStudio 风格：左侧分类导航 + 右侧内容区。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import WaypointsIcon from "@lucide/svelte/icons/waypoints";
  import CpuIcon from "@lucide/svelte/icons/cpu";
  import BotIcon from "@lucide/svelte/icons/bot";
  import MessageCircleIcon from "@lucide/svelte/icons/message-circle";
  import type { Agent } from "../lib/transport/agents";
  import type { Channel } from "../lib/transport/channels";
  import type { Conversation } from "../lib/transport/conversations";
  import type { ChannelModel, RemoteModelInfo } from "../lib/transport/models";
  import type { SettingsTab, AgentFormState, ModelFormState, ConversationDraft } from "./workspace-shell.svelte.js";
  import ChannelSettings from "./ChannelSettings.svelte";
  import ModelSettingsPanel from "./ModelSettingsPanel.svelte";
  import AgentSettingsPanel from "./AgentSettingsPanel.svelte";
  import ConversationSettingsPanel from "./ConversationSettingsPanel.svelte";

  type Props = {
    settingsTab: SettingsTab;
    onSelectTab: (tab: SettingsTab) => void;
    onBack: () => void;
    /* Channel */
    channels: Channel[];
    onChannelsChanged: () => void | Promise<void>;
    /* Model */
    selectedModelChannelId: string;
    selectedChannelModels: ChannelModel[];
    remoteModels: RemoteModelInfo[];
    modelsLoadingChannelId: string | null;
    remoteModelsLoadingChannelId: string | null;
    modelEditingId: string | null;
    modelForm: ModelFormState;
    modelSaving: boolean;
    onSelectModelChannel: (channelId: string) => void | Promise<void>;
    onFetchRemoteModels: () => void | Promise<void>;
    onResetModelForm: () => void;
    onEditModel: (model: ChannelModel) => void;
    onDeleteModel: (id: string) => void | Promise<void>;
    onImportRemoteModel: (model: RemoteModelInfo) => void | Promise<void>;
    onModelFieldChange: (field: keyof ModelFormState, value: string) => void;
    onSubmitModel: (event: SubmitEvent) => void | Promise<void>;
    /* Agent */
    agents: Agent[];
    agentEditingId: string | null;
    agentForm: AgentFormState;
    agentSaving: boolean;
    onSetAgentName: (value: string) => void;
    onSetAgentSystemPrompt: (value: string) => void;
    onResetAgentForm: () => void;
    onEditAgent: (agent: Agent) => void;
    onDeleteAgent: (id: string) => void | Promise<void>;
    onSubmitAgent: (event: SubmitEvent) => void | Promise<void>;
    onToggleAgentEnabled: (agent: Agent) => void | Promise<void>;
    /* Conversation */
    activeConversation: Conversation | null;
    conversationDraft: ConversationDraft;
    draftedConversationModels: ChannelModel[];
    conversationSaving: boolean;
    onSetConversationTitleDraft: (value: string) => void;
    onSetConversationAgentDraft: (value: string) => void;
    onSetConversationChannelDraft: (value: string) => void;
    onSetConversationModelDraft: (value: string) => void;
    onSaveConversationSettings: (event: SubmitEvent) => void | Promise<void>;
  };

  const props: Props = $props();

  /** 左侧导航项。 */
  const navItems: Array<{ value: SettingsTab; label: string; icon: typeof WaypointsIcon }> = [
    { value: "channels", label: "渠道管理", icon: WaypointsIcon },
    { value: "models", label: "模型管理", icon: CpuIcon },
    { value: "agents", label: "Agent", icon: BotIcon },
    { value: "conversation", label: "会话设置", icon: MessageCircleIcon }
  ];
</script>

<div class="flex h-dvh bg-background">
  <!-- 左侧分类导航 -->
  <nav class="flex w-52 shrink-0 flex-col border-r bg-muted/30">
    <!-- 返回按钮 -->
    <div class="flex h-12 items-center gap-2 border-b px-4">
      <Button class="size-7" onclick={props.onBack} size="icon" variant="ghost">
        <ArrowLeftIcon class="size-4" />
      </Button>
      <span class="text-sm font-semibold">设置</span>
    </div>

    <!-- 导航列表 -->
    <div class="flex flex-1 flex-col gap-0.5 p-2">
      {#each navItems as item}
        {@const isActive = props.settingsTab === item.value}
        <button
          class={`flex items-center gap-3 rounded-lg px-3 py-2.5 text-left text-sm transition-colors ${
            isActive
              ? "bg-background font-medium text-foreground shadow-sm"
              : "text-muted-foreground hover:bg-background/60 hover:text-foreground"
          }`}
          onclick={() => props.onSelectTab(item.value)}
          type="button"
        >
          <item.icon class="size-4 shrink-0" />
          {item.label}
        </button>
      {/each}
    </div>
  </nav>

  <!-- 右侧内容区 -->
  <div class="min-h-0 flex-1 overflow-y-auto">
    {#if props.settingsTab === "channels"}
      <ChannelSettings onChanged={props.onChannelsChanged} />
    {:else if props.settingsTab === "models"}
      <ModelSettingsPanel
        channels={props.channels}
        editingId={props.modelEditingId}
        form={props.modelForm}
        models={props.selectedChannelModels}
        modelsLoadingChannelId={props.modelsLoadingChannelId}
        onDelete={props.onDeleteModel}
        onEdit={props.onEditModel}
        onFetchRemoteModels={props.onFetchRemoteModels}
        onFieldChange={props.onModelFieldChange}
        onImportRemoteModel={props.onImportRemoteModel}
        onReset={props.onResetModelForm}
        onSelectChannel={props.onSelectModelChannel}
        onSubmit={props.onSubmitModel}
        remoteModels={props.remoteModels}
        remoteModelsLoadingChannelId={props.remoteModelsLoadingChannelId}
        saving={props.modelSaving}
        selectedChannelId={props.selectedModelChannelId}
      />
    {:else if props.settingsTab === "agents"}
      <AgentSettingsPanel
        agents={props.agents}
        editingId={props.agentEditingId}
        form={props.agentForm}
        onDelete={props.onDeleteAgent}
        onEdit={props.onEditAgent}
        onNameChange={props.onSetAgentName}
        onReset={props.onResetAgentForm}
        onSubmit={props.onSubmitAgent}
        onSystemPromptChange={props.onSetAgentSystemPrompt}
        onToggleEnabled={props.onToggleAgentEnabled}
        saving={props.agentSaving}
      />
    {:else}
      <ConversationSettingsPanel
        agents={props.agents}
        channels={props.channels}
        conversation={props.activeConversation}
        draft={props.conversationDraft}
        models={props.draftedConversationModels}
        onAgentChange={props.onSetConversationAgentDraft}
        onChannelChange={props.onSetConversationChannelDraft}
        onModelChange={props.onSetConversationModelDraft}
        onSubmit={props.onSaveConversationSettings}
        onTitleChange={props.onSetConversationTitleDraft}
        saving={props.conversationSaving}
      />
    {/if}
  </div>
</div>
