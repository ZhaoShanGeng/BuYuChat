<script lang="ts">
  import { Tabs } from "bits-ui";
  import type { AgentSummary } from "$lib/api/agents";
  import type { ConversationDetail } from "$lib/api/conversations";
  import type { MessageVersionView } from "$lib/api/messages";
  import { cn } from "$lib/utils";
  import Card from "$components/ui/card.svelte";
  import Badge from "$components/ui/badge.svelte";
  import {
    Layers,
    GitBranch,
    FileText,
    Variable,
    Link,
    Workflow,
    X,
    Sparkles,
    MessagesSquare,
    SlidersHorizontal
  } from "lucide-svelte";
  import type { InspectorTabId } from "$lib/state/app-shell.svelte";
  import { i18n } from "$lib/i18n.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import InspectorBindingsTab from "$components/layout/inspector-bindings-tab.svelte";

  const tabIcons: Record<InspectorTabId, typeof Layers> = {
    context: Layers,
    versions: GitBranch,
    summaries: FileText,
    variables: Variable,
    bindings: Link,
    workflow: Workflow
  };

  const tabDescKeys: Record<InspectorTabId, string> = {
    context: "inspector.context_desc",
    versions: "inspector.versions_desc",
    summaries: "inspector.summaries_desc",
    variables: "inspector.variables_desc",
    bindings: "inspector.bindings_desc",
    workflow: "inspector.workflow_desc"
  };

  let {
    tabs = [],
    activeTab,
    conversationTitle = "",
    conversationDetail = null,
    availableAgents = [],
    messages = [],
    selectedMessage = null,
    selectedVersionCount = 0,
    onSelectTab,
    onClose = () => {}
  }: {
    tabs?: { id: InspectorTabId; label: string }[];
    activeTab: InspectorTabId;
    conversationTitle?: string;
    conversationDetail?: ConversationDetail | null;
    availableAgents?: AgentSummary[];
    messages?: MessageVersionView[];
    selectedMessage?: MessageVersionView | null;
    selectedVersionCount?: number;
    onSelectTab: (id: InspectorTabId) => void;
    onClose?: () => void;
  } = $props();

  const ActiveIcon = $derived(tabIcons[activeTab]);
  const activeTabLabel = $derived(tabs.find((tab) => tab.id === activeTab)?.label ?? "");
  const selectedMessageText = $derived(
    selectedMessage?.primary_content.text_content ?? selectedMessage?.primary_content.preview_text ?? ""
  );
  const selectedRoleLabel = $derived(
    selectedMessage
      ? selectedMessage.role === "assistant"
        ? "Assistant"
        : selectedMessage.role === "user"
          ? "User"
          : selectedMessage.role === "system"
            ? "System"
            : "Tool"
      : "None"
  );
  const messageCounts = $derived({
    all: messages.length,
    user: messages.filter((message) => message.role === "user").length,
    assistant: messages.filter((message) => message.role === "assistant").length
  });
  const bindingCounts = $derived({
    presets: conversationDetail?.preset_bindings.length ?? 0,
    lorebooks: conversationDetail?.lorebook_bindings.length ?? 0,
    profiles: conversationDetail?.user_profile_bindings.length ?? 0,
    channels: conversationDetail?.channel_bindings.length ?? 0
  });
  const participantCount = $derived(conversationDetail?.participants.length ?? 0);

  const tabSectionMeta = $derived.by((): Record<
    InspectorTabId,
    { eyebrow: string; title: string; items: { label: string; value: string }[] }
  > => ({
    context: {
      eyebrow: "Context graph",
      title: "上下文装配",
      items: [
        { label: "会话模式", value: conversationDetail?.summary.conversation_mode ?? "chat" },
        { label: "参与者", value: `${participantCount}` },
        { label: "可见消息", value: `${messageCounts.all}` }
      ]
    },
    versions: {
      eyebrow: "Version graph",
      title: "版本与分支",
      items: [
        { label: "当前节点", value: selectedMessage?.node_id ? "已选择" : "未选择" },
        { label: "活动版本", value: selectedMessage ? `v${selectedMessage.version_index + 1}` : "—" },
        { label: "版本数量", value: selectedVersionCount ? `${selectedVersionCount}` : "0" }
      ]
    },
    summaries: {
      eyebrow: "Compression",
      title: "摘要策略",
      items: [
        { label: "显示策略", value: "待接入" },
        { label: "发送策略", value: "待接入" },
        { label: "摘要版本", value: "0" }
      ]
    },
    variables: {
      eyebrow: "Runtime state",
      title: "变量状态",
      items: [
        { label: "会话变量", value: "待接入" },
        { label: "消息变量", value: selectedMessage ? "待接入" : "0" },
        { label: "锁定项", value: "待接入" }
      ]
    },
    bindings: {
      eyebrow: "Resource routing",
      title: "绑定资源",
      items: [
        { label: "预设", value: `${bindingCounts.presets}` },
        { label: "世界书", value: `${bindingCounts.lorebooks}` },
        { label: "用户设定", value: `${bindingCounts.profiles}` }
      ]
    },
    workflow: {
      eyebrow: "Execution graph",
      title: "工作流",
      items: [
        { label: "助手消息", value: `${messageCounts.assistant}` },
        { label: "用户消息", value: `${messageCounts.user}` },
        { label: "渠道绑定", value: `${bindingCounts.channels}` }
      ]
    }
  }));
</script>

<aside data-chat-inspector-panel class="flex h-full flex-col border-l border-[var(--border-soft)] bg-[var(--bg-sidebar)]">
  <div class="flex items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4 py-3" data-tauri-drag-region>
    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("inspector.title")}</h2>
    <HeaderWindowGroup>
      {#snippet children()}
        <ActionIconButton title="关闭检查器" className="inline-flex" onClick={onClose}>
          <X size={14} />
        </ActionIconButton>
      {/snippet}
    </HeaderWindowGroup>
  </div>

  <Tabs.Root
    value={activeTab}
    onValueChange={(value) => onSelectTab(value as InspectorTabId)}
    class="flex min-h-0 flex-1 flex-col"
  >
    <div class="border-b border-[var(--border-soft)] px-3 py-3">
      <Tabs.List class="grid grid-cols-2 gap-2">
        {#each tabs as tab}
          {@const Icon = tabIcons[tab.id]}
          <Tabs.Trigger
            value={tab.id}
            class={cn(
              "inline-flex h-10 items-center justify-start gap-2 rounded-[var(--radius-md)] px-3 text-sm font-medium transition-colors",
              "data-[state=active]:bg-[var(--brand)] data-[state=active]:text-white",
              "text-[var(--ink-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)]"
            )}
          >
            {#if Icon}<Icon size={14} />{/if}
            <span class="truncate">{tab.label}</span>
          </Tabs.Trigger>
        {/each}
      </Tabs.List>
    </div>

    <div class="app-scrollbar flex-1 overflow-y-auto p-4">
      {#each tabs as tab}
        {@const meta = tabSectionMeta[tab.id]}
        {@const Icon = tabIcons[tab.id]}
        <Tabs.Content value={tab.id} class="space-y-4 outline-none">
          <Card className="p-4">
            <div class="flex items-center gap-3">
              <div class="flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-[var(--brand-soft)]">
                {#if Icon}
                  <Icon size={18} class="text-[var(--brand)]" />
                {/if}
              </div>
              <div class="min-w-0">
                <p class="text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--ink-faint)]">{meta.eyebrow}</p>
                <h3 class="text-base font-semibold text-[var(--ink-strong)]">{meta.title}</h3>
              </div>
            </div>
            <p class="mt-3 text-sm leading-relaxed text-[var(--ink-muted)]">{i18n.t(tabDescKeys[tab.id])}</p>
          </Card>

          <Card className="p-4">
            <div class="mb-3 flex items-center justify-between gap-3">
              <div class="flex items-center gap-2">
                <Sparkles size={14} class="text-[var(--ink-faint)]" />
                <h4 class="text-sm font-semibold text-[var(--ink-strong)]">{activeTabLabel}</h4>
              </div>
              <Badge>{meta.items.length} 项</Badge>
            </div>
            <div class="grid gap-2">
              {#each meta.items as item}
                <div class="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-2">
                  <span class="text-xs font-medium text-[var(--ink-muted)]">{item.label}</span>
                  <span class="text-xs font-semibold text-[var(--ink-strong)]">{item.value}</span>
                </div>
              {/each}
            </div>
          </Card>

          <Card className="p-4">
            <div class="mb-3 flex items-center gap-2">
              <MessagesSquare size={14} class="text-[var(--ink-faint)]" />
              <h4 class="text-sm font-semibold text-[var(--ink-strong)]">当前选区</h4>
            </div>
            {#if selectedMessage}
              <div class="space-y-3 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] p-4">
                <div class="flex items-center justify-between gap-3">
                  <div>
                    <p class="text-xs font-medium text-[var(--ink-muted)]">{selectedRoleLabel}</p>
                    <p class="mt-1 text-sm font-semibold text-[var(--ink-strong)]">{conversationTitle || "当前会话"}</p>
                  </div>
                  <Badge>v{selectedMessage.version_index + 1}</Badge>
                </div>
                <p class="line-clamp-4 text-sm leading-relaxed text-[var(--ink-body)]">
                  {selectedMessageText || "当前消息没有可展示的正文。"}
                </p>
              </div>
            {:else}
              <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-4 py-8 text-center text-xs text-[var(--ink-faint)]">
                {i18n.t("inspector.select_msg")}
              </div>
            {/if}
          </Card>

          {#if tab.id === "bindings"}
            <Card className="p-4">
              <div class="mb-3 flex items-center gap-2">
                <SlidersHorizontal size={14} class="text-[var(--ink-faint)]" />
                <h4 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("inspector.tab.bindings")}</h4>
              </div>
              <InspectorBindingsTab {conversationDetail} {availableAgents} />
            </Card>
          {:else}
            <Card className="p-4">
              <div class="mb-3 flex items-center gap-2">
                <SlidersHorizontal size={14} class="text-[var(--ink-faint)]" />
                <h4 class="text-sm font-semibold text-[var(--ink-strong)]">后续动作</h4>
              </div>
              <div class="space-y-2">
                <div class="flex items-center justify-between rounded-[var(--radius-md)] bg-[var(--bg-app)] px-3 py-2">
                  <span class="text-xs text-[var(--ink-muted)]">面板模式</span>
                  <Badge className="bg-[var(--brand-soft)] text-[var(--brand)]">Inspector</Badge>
                </div>
                <div class="flex items-center justify-between rounded-[var(--radius-md)] bg-[var(--bg-app)] px-3 py-2">
                  <span class="text-xs text-[var(--ink-muted)]">内容状态</span>
                  <span class="text-xs font-medium text-[var(--ink-strong)]">逐步接入真实数据</span>
                </div>
              </div>
            </Card>
          {/if}
        </Tabs.Content>
      {/each}
    </div>
  </Tabs.Root>
</aside>
