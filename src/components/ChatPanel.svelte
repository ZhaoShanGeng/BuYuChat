<script lang="ts">
  /**
   * 聊天面板 — ChatHeader + MessageTimeline + ChatComposer。
   */
  import type { Agent } from "../lib/transport/agents";
  import type { Channel } from "../lib/transport/channels";
  import type { Conversation } from "../lib/transport/conversations";
  import type { ChannelModel } from "../lib/transport/models";
  import type { MessageNode } from "../lib/transport/messages";
  import { parseThinkingTagsConfig } from "../lib/thinking-tags";
  import type {
    MessageHistoryState,
    PendingComposerImage
  } from "./workspace-shell.svelte.js";
  import type { Notice } from "./workspace-state";
  import { isNodeGenerating } from "./workspace-state";
  import ChatHeader from "./ChatHeader.svelte";
  import ChatComposer from "./ChatComposer.svelte";
  import MessageTimeline from "./MessageTimeline.svelte";

  type Props = {
    conversation: Conversation | null;
    messages: MessageNode[];
    messageHistory: MessageHistoryState;
    agents: Agent[];
    channels: Channel[];
    models: ChannelModel[];
    loading: boolean;
    sending: boolean;
    composer: string;
    pendingImages: PendingComposerImage[];
    notice: Notice | null;
    dryRunSummary: string | null;
    agentName: string;
    channelName: string;
    modelName: string;
    onComposerChange: (value: string) => void;
    onPendingImagesChange: (images: PendingComposerImage[]) => void;
    onSend: () => void | Promise<void>;
    onDryRun: () => void | Promise<void>;
    onCancel: (versionId: string) => void | Promise<void>;
    onReroll: (nodeId: string) => void | Promise<void>;
    onSwitchVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onDeleteVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onEditMessage: (
      nodeId: string,
      content: string,
      options?: { resend?: boolean }
    ) => void | Promise<void>;
    onLoadVersionContent: (nodeId: string, versionId: string) => Promise<string>;
    onLoadOlderMessages: () => void | Promise<void>;
    onQuickModelChange: (modelId: string) => void | Promise<void>;
    onQuickAgentChange: (agentId: string) => void | Promise<void>;
    onQuickChannelChange: (channelId: string) => void | Promise<void>;
    onQuickChannelMenuOpen: () => void | Promise<void>;
    onQuickTitleChange: (title: string) => void | Promise<void>;
    enabledTools: string[];
    onEnabledToolsChange: (tools: string[]) => void | Promise<void>;
    isMobile?: boolean;
    onMenuToggle?: () => void;
  };

  const props: Props = $props();

  /** 当前是否有正在生成的版本。 */
  let generatingVersionId = $derived.by(() => {
    for (const node of props.messages) {
      if (isNodeGenerating(node)) {
        const v = node.versions.find((v) => v.status === "generating");
        if (v) return v.id;
      }
    }
    return null;
  });

  let thinkingTags = $derived.by(() => {
    const channel = props.channels.find((item) => item.id === props.conversation?.channelId);
    return parseThinkingTagsConfig(channel?.thinkingTags);
  });
</script>

<section class="flex h-full min-h-0 min-w-0 flex-col overflow-hidden">
  <ChatHeader
    agentName={props.agentName}
    agents={props.agents}
    channelName={props.channelName}
    channels={props.channels}
    conversation={props.conversation}
    isMobile={props.isMobile}
    modelName={props.modelName}
    models={props.models}
    onMenuToggle={props.onMenuToggle}
    onQuickAgentChange={props.onQuickAgentChange}
    onQuickChannelChange={props.onQuickChannelChange}
    onQuickChannelMenuOpen={props.onQuickChannelMenuOpen}
    onQuickModelChange={props.onQuickModelChange}
    onQuickTitleChange={props.onQuickTitleChange}
  />

  <MessageTimeline
    conversation={props.conversation}
    dryRunSummary={props.dryRunSummary}
    hasOlderMessages={props.messageHistory.hasOlder}
    loading={props.loading}
    loadingOlderMessages={props.messageHistory.loadingOlder}
    messages={props.messages}
    notice={props.notice}
    {thinkingTags}
    onCancel={props.onCancel}
    onDeleteVersion={props.onDeleteVersion}
    onEditMessage={props.onEditMessage}
    onLoadVersionContent={props.onLoadVersionContent}
    onLoadOlderMessages={props.onLoadOlderMessages}
    onReroll={props.onReroll}
    onSwitchVersion={props.onSwitchVersion}
  />

  <ChatComposer
    composer={props.composer}
    conversation={props.conversation}
    enabledTools={props.enabledTools}
    {generatingVersionId}
    onCancel={props.onCancel}
    onComposerChange={props.onComposerChange}
    onEnabledToolsChange={props.onEnabledToolsChange}
    onPendingImagesChange={props.onPendingImagesChange}
    onDryRun={props.onDryRun}
    onSend={props.onSend}
    pendingImages={props.pendingImages}
    sending={props.sending}
  />
</section>
