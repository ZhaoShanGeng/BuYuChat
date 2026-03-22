<script lang="ts">
  import AgentsWorkspace from "$components/agents/agents-workspace.svelte";
  import ChatWorkspace from "$components/chat/chat-workspace.svelte";
  import LorebooksWorkspace from "$components/lorebooks/lorebooks-workspace.svelte";
  import PresetsWorkspace from "$components/presets/presets-workspace.svelte";
  import SettingsWorkspace from "$components/settings/settings-workspace.svelte";
  import WorkflowsWorkspace from "$components/workflows/workflows-workspace.svelte";
  import type { AgentSummary } from "$lib/api/agents";
  import type { ConversationDetail } from "$lib/api/conversations";
  import type { MessageVersionView } from "$lib/api/messages";
  import type { InspectorTabId, SidebarItem, WorkspaceId } from "$lib/state/app-shell.svelte";

  let {
    workspace,
    conversationTitle = "Conversation",
    conversationId = "",
    conversationDetail = null,
    loading = false,
    messages = [],
    editable = false,
    onEnsureConversation = undefined,
    availableAgents = [],
    onStartConversationWithAgent = undefined,
    desktopWide = true,
    sidebarItems = [],
    activeSidebarId = "",
    sidebarOpen = false,
    inspectorVisible = true,
    inspectorOpen = false,
    inspectorTabs = [],
    activeInspectorTab = "context" as InspectorTabId,
    onRename = undefined,
    onSelectSidebar = () => {},
    onCreateSidebarItem = undefined,
    onRenameSidebarItem = undefined,
    onDeleteSidebarItem = undefined,
    onToggleSidebar = () => {},
    onToggleInspector = () => {},
    onOpenInspector = () => {},
    onCloseSidebar = () => {},
    onCloseInspector = () => {},
    onSelectInspectorTab = () => {}
  }: {
    workspace: WorkspaceId;
    conversationTitle?: string;
    conversationId?: string;
    conversationDetail?: ConversationDetail | null;
    loading?: boolean;
    messages?: MessageVersionView[];
    editable?: boolean;
    onEnsureConversation?: ((preferredAgentId?: string) => Promise<ConversationDetail | null>) | undefined;
    availableAgents?: AgentSummary[];
    onStartConversationWithAgent?: ((agentId: string) => Promise<string | null>) | undefined;
    desktopWide?: boolean;
    sidebarItems?: SidebarItem[];
    activeSidebarId?: string;
    sidebarOpen?: boolean;
    inspectorVisible?: boolean;
    inspectorOpen?: boolean;
    inspectorTabs?: { id: InspectorTabId; label: string }[];
    activeInspectorTab?: InspectorTabId;
    onRename?: ((title: string) => void) | undefined;
    onSelectSidebar?: (id: string) => void;
    onCreateSidebarItem?: (() => void) | undefined;
    onRenameSidebarItem?: ((id: string, title: string) => void) | undefined;
    onDeleteSidebarItem?: ((id: string) => void) | undefined;
    onToggleSidebar?: () => void;
    onToggleInspector?: () => void;
    onOpenInspector?: () => void;
    onCloseSidebar?: () => void;
    onCloseInspector?: () => void;
    onSelectInspectorTab?: (id: InspectorTabId) => void;
  } = $props();
</script>

{#key workspace}
  <div class="workspace-stage">
    {#if workspace === "chat"}
      <ChatWorkspace
        {conversationTitle}
        {conversationId}
        {conversationDetail}
        {loading}
        {messages}
        {editable}
        {onEnsureConversation}
        {availableAgents}
        {onStartConversationWithAgent}
        {desktopWide}
        {sidebarItems}
        {activeSidebarId}
        {sidebarOpen}
        {inspectorVisible}
        {inspectorOpen}
        {inspectorTabs}
        {activeInspectorTab}
        {onRename}
        {onSelectSidebar}
        {onCreateSidebarItem}
        {onRenameSidebarItem}
        {onDeleteSidebarItem}
        {onToggleSidebar}
        {onToggleInspector}
        {onOpenInspector}
        {onCloseSidebar}
        {onCloseInspector}
        {onSelectInspectorTab}
      />
    {:else if workspace === "agents"}
      <AgentsWorkspace />
    {:else if workspace === "presets"}
      <PresetsWorkspace />
    {:else if workspace === "lorebooks"}
      <LorebooksWorkspace />
    {:else if workspace === "workflows"}
      <WorkflowsWorkspace />
    {:else}
      <SettingsWorkspace />
    {/if}
  </div>
{/key}
