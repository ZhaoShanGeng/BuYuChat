/**
 * 聊天工作台的 Svelte 5 runes 状态工厂。
 *
 * 这个文件集中承接页面级状态：
 * 1. 首屏 bootstrap、会话切换、模型缓存与设置草稿同步。
 * 2. send_message / reroll / cancel_generation / version switch 的异步流程。
 * 3. 设置抽屉内的 Agent、模型、会话绑定表单状态。
 */

import {
  createAgent,
  deleteAgent,
  listAgents,
  updateAgent,
  type Agent,
  type AgentInput,
  type AgentPatch
} from "../lib/transport/agents";
import { listChannels, type Channel } from "../lib/transport/channels";
import {
  createConversation,
  deleteConversation,
  getConversation,
  listConversations,
  updateConversation,
  type Conversation,
  type ConversationInput,
  type ConversationPatch,
  type ConversationSummary
} from "../lib/transport/conversations";
import {
  deleteModel,
  fetchRemoteModels,
  listModels,
  createModel,
  updateModel,
  type ChannelModel,
  type ModelInput,
  type ModelPatch,
  type RemoteModelInfo
} from "../lib/transport/models";
import {
  cancelGeneration,
  getVersionContent,
  listMessages,
  reroll,
  sendMessage,
  setActiveVersion,
  type GenerationEvent,
  type MessageNode,
  type RerollInput,
  type RerollResult,
  type SendMessageInput,
  type SendMessageResponse,
  type VersionContent
} from "../lib/transport/messages";
import { toAppError } from "../lib/transport/common";
import {
  humanizeWorkspaceError,
  withActiveVersion,
  withVersionContent,
  type Notice
} from "./workspace-state";

/**
 * 设置抽屉的页签类型。
 */
export type SettingsTab = "channels" | "models" | "agents" | "conversation";

/**
 * 顶层区域类型。
 */
export type ActiveSection = "chat" | "agents" | "settings";

/**
 * Agent 表单状态。
 */
export type AgentFormState = {
  name: string;
  systemPrompt: string;
};

/**
 * 模型表单状态。
 */
export type ModelFormState = {
  modelId: string;
  displayName: string;
  contextWindow: string;
  maxOutputTokens: string;
};

/**
 * 会话设置草稿。
 */
export type ConversationDraft = {
  title: string;
  agentId: string;
  channelId: string;
  modelId: string;
};

/**
 * 工作台依赖的 transport 能力集合。
 */
export type WorkspaceShellDeps = {
  listChannels: (includeDisabled?: boolean) => Promise<Channel[]>;
  listAgents: (includeDisabled?: boolean) => Promise<Agent[]>;
  createAgent: (input: AgentInput) => Promise<Agent>;
  updateAgent: (id: string, input: AgentPatch) => Promise<Agent>;
  deleteAgent: (id: string) => Promise<void>;
  listConversations: (archived?: boolean) => Promise<ConversationSummary[]>;
  getConversation: (id: string) => Promise<Conversation>;
  createConversation: (input?: ConversationInput) => Promise<Conversation>;
  updateConversation: (id: string, input: ConversationPatch) => Promise<Conversation>;
  deleteConversation: (id: string) => Promise<void>;
  listModels: (channelId: string) => Promise<ChannelModel[]>;
  createModel: (channelId: string, input: ModelInput) => Promise<ChannelModel>;
  updateModel: (channelId: string, id: string, input: ModelPatch) => Promise<ChannelModel>;
  deleteModel: (channelId: string, id: string) => Promise<void>;
  fetchRemoteModels: (channelId: string) => Promise<RemoteModelInfo[]>;
  listMessages: (
    id: string,
    beforeOrderKey?: string | null,
    limit?: number
  ) => Promise<MessageNode[]>;
  getVersionContent: (versionId: string) => Promise<VersionContent>;
  setActiveVersion: (
    conversationId: string,
    nodeId: string,
    versionId: string
  ) => Promise<void>;
  sendMessage: (
    id: string,
    input: SendMessageInput,
    onEvent?: (event: GenerationEvent) => void
  ) => Promise<SendMessageResponse>;
  reroll: (
    id: string,
    nodeId: string,
    input?: RerollInput,
    onEvent?: (event: GenerationEvent) => void
  ) => Promise<RerollResult>;
  cancelGeneration: (versionId: string) => Promise<void>;
};

/**
 * 新建 Agent 时使用的空白表单。
 */
const EMPTY_AGENT_FORM: AgentFormState = {
  name: "",
  systemPrompt: ""
};

/**
 * 新建模型时使用的空白表单。
 */
const EMPTY_MODEL_FORM: ModelFormState = {
  modelId: "",
  displayName: "",
  contextWindow: "",
  maxOutputTokens: ""
};

/**
 * 空会话草稿的默认值。
 */
const EMPTY_CONVERSATION_DRAFT: ConversationDraft = {
  title: "",
  agentId: "",
  channelId: "",
  modelId: ""
};

/**
 * 工作台状态使用的默认 transport 依赖。
 */
const defaultDeps: WorkspaceShellDeps = {
  listChannels,
  listAgents,
  createAgent,
  updateAgent,
  deleteAgent,
  listConversations,
  getConversation,
  createConversation,
  updateConversation,
  deleteConversation,
  listModels,
  createModel,
  updateModel,
  deleteModel,
  fetchRemoteModels,
  listMessages,
  getVersionContent,
  setActiveVersion,
  sendMessage,
  reroll,
  cancelGeneration
};

/**
 * 创建工作台的响应式状态和交互行为。
 */
export function createWorkspaceShellState(overrides: Partial<WorkspaceShellDeps> = {}) {
  const deps = {
    ...defaultDeps,
    ...overrides
  };

  const state = $state({
    activeSection: "chat" as ActiveSection,
    bootstrapping: true,
    conversationsLoading: false,
    messagesLoading: false,
    channels: [] as Channel[],
    agents: [] as Agent[],
    conversations: [] as ConversationSummary[],
    activeConversationId: null as string | null,
    activeConversation: null as Conversation | null,
    messagesByConversation: {} as Record<string, MessageNode[]>,
    modelsByChannel: {} as Record<string, ChannelModel[]>,
    remoteModelsByChannel: {} as Record<string, RemoteModelInfo[]>,
    settingsOpen: false,
    settingsTab: "conversation" as SettingsTab,
    notice: null as Notice | null,
    composer: "",
    sending: false,
    dryRunSummary: null as string | null,
    agentEditingId: null as string | null,
    agentForm: { ...EMPTY_AGENT_FORM },
    agentSaving: false,
    selectedModelChannelId: "",
    modelEditingId: null as string | null,
    modelForm: { ...EMPTY_MODEL_FORM },
    modelSaving: false,
    modelsLoadingChannelId: null as string | null,
    remoteModelsLoadingChannelId: null as string | null,
    conversationDraft: { ...EMPTY_CONVERSATION_DRAFT },
    conversationSaving: false
  });

  let initialized = false;
  let lastConversationDraftChannelId = "";
  const pendingChunkEvents = new Map<
    string,
    Map<string, Extract<GenerationEvent, { type: "chunk" }>>
  >();
  const messageReloadVersions = new Map<string, number>();

  /**
   * 获取当前活跃会话对应的消息列表。
   */
  function getActiveMessages() {
    return state.activeConversationId
      ? state.messagesByConversation[state.activeConversationId] ?? []
      : [];
  }

  /**
   * 获取当前会话绑定渠道下的模型列表。
   */
  function getActiveChannelModels() {
    return state.activeConversation?.channelId
      ? state.modelsByChannel[state.activeConversation.channelId] ?? []
      : [];
  }

  /**
   * 获取模型设置页当前选中渠道下的模型列表。
   */
  function getSelectedChannelModels() {
    return state.selectedModelChannelId
      ? state.modelsByChannel[state.selectedModelChannelId] ?? []
      : [];
  }

  /**
   * 获取模型设置页当前选中渠道下的远程候选模型列表。
   */
  function getSelectedRemoteModels() {
    return state.selectedModelChannelId
      ? state.remoteModelsByChannel[state.selectedModelChannelId] ?? []
      : [];
  }

  /**
   * 获取会话设置草稿所选渠道下可用的模型列表。
   */
  function getDraftedConversationModels() {
    return state.conversationDraft.channelId
      ? state.modelsByChannel[state.conversationDraft.channelId] ?? []
      : [];
  }

  /**
   * 将错误写入顶部通知。
   */
  function setErrorNotice(error: unknown) {
    state.notice = {
      kind: "error",
      text: humanizeWorkspaceError(toAppError(error))
    };
  }

  /**
   * 更新某个会话的消息缓存。
   */
  function setConversationMessages(conversationId: string, nodes: MessageNode[]) {
    state.messagesByConversation[conversationId] = nodes;
  }

  /**
   * 更新某个渠道的模型缓存。
   */
  function setChannelModels(channelId: string, models: ChannelModel[]) {
    state.modelsByChannel[channelId] = models;
  }

  /**
   * 更新某个渠道的远程模型候选缓存。
   */
  function setRemoteModels(channelId: string, models: RemoteModelInfo[]) {
    state.remoteModelsByChannel[channelId] = models;
  }

  /**
   * 清理某个渠道相关的本地模型缓存。
   */
  function clearModelCache(channelId?: string | null) {
    if (!channelId) {
      return;
    }

    delete state.modelsByChannel[channelId];
    delete state.remoteModelsByChannel[channelId];
  }

  /**
   * 在当前消息缓存中查找某个版本。
   */
  function findVersion(
    nodes: MessageNode[],
    versionId: string,
    nodeId?: string
  ) {
    if (nodeId) {
      const node = nodes.find((item) => item.id === nodeId);
      return node?.versions.find((version) => version.id === versionId) ?? null;
    }

    for (const node of nodes) {
      const version = node.versions.find((item) => item.id === versionId);
      if (version) {
        return version;
      }
    }

    return null;
  }

  /**
   * 在当前消息缓存里定位某个楼层。
   */
  function findNode(nodes: MessageNode[], nodeId: string) {
    return nodes.find((node) => node.id === nodeId) ?? null;
  }

  /**
   * 把刚启动的 send_message 结果先插入本地消息缓存。
   *
   * 这样即使第一批 chunk 早于首轮 `reloadMessages` 返回，流式事件也有命中的目标版本。
   */
  function insertStartedMessageNodes(
    conversationId: string,
    response: Extract<SendMessageResponse, { kind: "started" }>,
    content: string
  ) {
    const currentNodes = state.messagesByConversation[conversationId] ?? [];
    if (
      currentNodes.some((node) => node.id === response.userNodeId) ||
      currentNodes.some((node) => node.id === response.assistantNodeId)
    ) {
      return;
    }

    const createdAt = Date.now();
    const activeAgentId = state.activeConversation?.agentId ?? null;
    setConversationMessages(conversationId, [
      ...currentNodes,
      {
        id: response.userNodeId,
        conversationId,
        authorAgentId: null,
        role: "user",
        orderKey: `local-user-${createdAt}`,
        activeVersionId: response.userVersionId,
        versions: [
          {
            id: response.userVersionId,
            nodeId: response.userNodeId,
            content,
            status: "committed",
            modelName: null,
            promptTokens: null,
            completionTokens: null,
            finishReason: null,
            createdAt
          }
        ],
        createdAt
      },
      {
        id: response.assistantNodeId,
        conversationId,
        authorAgentId: activeAgentId,
        role: "assistant",
        orderKey: `local-assistant-${createdAt}`,
        activeVersionId: response.assistantVersionId,
        versions: [
          {
            id: response.assistantVersionId,
            nodeId: response.assistantNodeId,
            content: "",
            status: "generating",
            modelName: null,
            promptTokens: null,
            completionTokens: null,
            finishReason: null,
            createdAt
          }
        ],
        createdAt
      }
    ]);
    replayPendingChunkEvents(conversationId);
  }

  /**
   * 暂存尚未找到目标版本的早到 chunk。
   */
  function stashPendingChunkEvent(event: Extract<GenerationEvent, { type: "chunk" }>) {
    let conversationBuffer = pendingChunkEvents.get(event.conversationId);
    if (!conversationBuffer) {
      conversationBuffer = new Map();
      pendingChunkEvents.set(event.conversationId, conversationBuffer);
    }

    const existing = conversationBuffer.get(event.versionId);
    if (existing) {
      existing.delta += event.delta;
    } else {
      conversationBuffer.set(event.versionId, { ...event });
    }
  }

  /**
   * 直接把 chunk 落到当前版本，避免额外中转和延迟。
   */
  function applyChunkEventImmediately(event: Extract<GenerationEvent, { type: "chunk" }>) {
    const nodes = state.messagesByConversation[event.conversationId] ?? [];
    const version = findVersion(nodes, event.versionId, event.nodeId);
    if (!version || version.status !== "generating") {
      return false;
    }

    version.content = `${version.content ?? ""}${event.delta}`;
    version.status = "generating";
    return true;
  }

  /**
   * 在节点或版本已经就绪后，回放之前暂存的早到 chunk。
   */
  function replayPendingChunkEvents(conversationId: string) {
    const conversationBuffer = pendingChunkEvents.get(conversationId);
    if (!conversationBuffer || conversationBuffer.size === 0) {
      return;
    }

    const remainingEvents = new Map<string, Extract<GenerationEvent, { type: "chunk" }>>();
    for (const bufferedEvent of conversationBuffer.values()) {
      if (!applyChunkEventImmediately(bufferedEvent)) {
        remainingEvents.set(bufferedEvent.versionId, bufferedEvent);
      }
    }

    if (remainingEvents.size > 0) {
      pendingChunkEvents.set(conversationId, remainingEvents);
    } else {
      pendingChunkEvents.delete(conversationId);
    }
  }

  /**
   * 直接应用终态事件，避免终态前后都依赖额外列表刷新。
   */
  function applyTerminalEventImmediately(event: Exclude<GenerationEvent, { type: "chunk" }>) {
    const nodes = state.messagesByConversation[event.conversationId] ?? [];

    switch (event.type) {
      case "completed": {
        const version = findVersion(nodes, event.versionId, event.nodeId);
        if (!version) {
          return;
        }
        version.status = "committed";
        version.promptTokens = event.promptTokens;
        version.completionTokens = event.completionTokens;
        version.finishReason = event.finishReason;
        version.modelName = event.model;
        return;
      }
      case "failed": {
        const version = findVersion(nodes, event.versionId, event.nodeId);
        if (!version) {
          return;
        }
        version.status = "failed";
        return;
      }
      case "cancelled": {
        const version = findVersion(nodes, event.versionId, event.nodeId);
        if (!version) {
          return;
        }
        version.status = "cancelled";
        return;
      }
      case "empty_rollback": {
        if (event.nodeDeleted) {
          state.messagesByConversation[event.conversationId] = nodes.filter(
            (node) => node.id !== event.nodeId
          );
          return;
        }

        const node = findNode(nodes, event.nodeId);
        if (!node) {
          return;
        }

        node.activeVersionId = event.fallbackVersionId;
        node.versions = node.versions.filter((version) => version.status !== "generating");
      }
    }
  }

  /**
   * 用当前活跃会话同步会话设置草稿。
   */
  function syncConversationDraft(conversation: Conversation | null) {
    if (!conversation) {
      state.conversationDraft = { ...EMPTY_CONVERSATION_DRAFT };
      lastConversationDraftChannelId = "";
      return;
    }

    state.conversationDraft = {
      title: conversation.title,
      agentId: conversation.agentId ?? "",
      channelId: conversation.channelId ?? "",
      modelId: conversation.channelModelId ?? ""
    };
    lastConversationDraftChannelId = conversation.channelId ?? "";
  }

  /**
   * 读取渠道列表。
   */
  async function reloadChannels() {
    state.channels = await deps.listChannels(true);
  }

  /**
   * 读取 Agent 列表。
   */
  async function reloadAgents() {
    state.agents = await deps.listAgents(true);
  }

  /**
   * 读取会话列表，并在当前选中项失效时自动回退。
   */
  async function reloadConversations() {
    const next = await deps.listConversations(false);
    state.conversations = next;

    if (next.length === 0) {
      state.activeConversationId = null;
      state.activeConversation = null;
      syncConversationDraft(null);
      return;
    }

    const hasActiveConversation = state.activeConversationId
      ? next.some((conversation) => conversation.id === state.activeConversationId)
      : false;

    if (!hasActiveConversation) {
      await selectConversation(next[0].id);
    }
  }

  /**
   * 按需拉取某个渠道下的模型列表。
   */
  async function ensureModelsLoaded(channelId: string, force = false) {
    if (!force && state.modelsByChannel[channelId]) {
      return;
    }

    state.modelsLoadingChannelId = channelId;
    try {
      setChannelModels(channelId, await deps.listModels(channelId));
    } finally {
      state.modelsLoadingChannelId = null;
    }
  }

  /**
   * 刷新当前活跃会话详情。
   */
  async function refreshActiveConversation() {
    if (!state.activeConversationId) {
      return;
    }

    state.activeConversation = await deps.getConversation(state.activeConversationId);
    syncConversationDraft(state.activeConversation);

    if (state.activeConversation.channelId) {
      await ensureModelsLoaded(state.activeConversation.channelId);
    }
  }

  /**
   * 读取某个会话的消息列表。
   */
  async function reloadMessages(conversationId: string) {
    const reloadVersion = (messageReloadVersions.get(conversationId) ?? 0) + 1;
    messageReloadVersions.set(conversationId, reloadVersion);
    state.messagesLoading = true;
    try {
      const nodes = await deps.listMessages(conversationId);
      if (messageReloadVersions.get(conversationId) !== reloadVersion) {
        return;
      }

      setConversationMessages(conversationId, nodes);
      if (pendingChunkEvents.has(conversationId)) {
        replayPendingChunkEvents(conversationId);
      }
    } catch (error) {
      setErrorNotice(error);
    } finally {
      if (messageReloadVersions.get(conversationId) === reloadVersion) {
        state.messagesLoading = false;
      }
    }
  }

  /**
   * 选择会话并加载详情与消息。
   */
  async function selectConversation(id: string) {
    state.conversationsLoading = true;
    state.activeConversationId = id;
    state.dryRunSummary = null;

    try {
      state.activeConversation = await deps.getConversation(id);
      syncConversationDraft(state.activeConversation);
      if (state.activeConversation.channelId) {
        await ensureModelsLoaded(state.activeConversation.channelId);
      }
      await reloadMessages(id);
    } catch (error) {
      setErrorNotice(error);
    } finally {
      state.conversationsLoading = false;
    }
  }

  /**
   * 处理后台生成事件，并在终态后触发一次收敛刷新。
   */
  function handleGenerationEvent(event: GenerationEvent) {
    if (event.type === "chunk") {
      if (!applyChunkEventImmediately(event)) {
        stashPendingChunkEvent(event);
      }
      return;
    }

    replayPendingChunkEvents(event.conversationId);
    applyTerminalEventImmediately(event);

    if (event.type !== "completed") {
      void reloadMessages(event.conversationId);
    }
    void reloadConversations();
  }

  /**
   * 初始化页面所需的基础数据。
   */
  async function bootstrap() {
    state.bootstrapping = true;
    try {
      const [loadedChannels, loadedAgents, loadedConversations] = await Promise.all([
        deps.listChannels(true),
        deps.listAgents(true),
        deps.listConversations(false)
      ]);

      state.channels = loadedChannels;
      state.agents = loadedAgents;
      state.conversations = loadedConversations;

      if (loadedChannels.length > 0) {
        state.selectedModelChannelId = loadedChannels[0].id;
        await ensureModelsLoaded(loadedChannels[0].id);
      }

      if (loadedConversations.length > 0) {
        await selectConversation(loadedConversations[0].id);
      }
    } catch (error) {
      setErrorNotice(error);
    } finally {
      state.bootstrapping = false;
    }
  }

  /**
   * 打开设置抽屉并切到指定页签。
   */
  function openSettings(tab: SettingsTab) {
    state.settingsTab = tab;
    state.settingsOpen = true;
  }

  /**
   * 关闭设置抽屉。
   */
  function closeSettings() {
    state.settingsOpen = false;
  }

  /**
   * 切换设置抽屉页签。
   */
  function selectSettingsTab(tab: SettingsTab) {
    state.settingsTab = tab;
  }

  /**
   * 切换顶层区域（聊天 / Agent / 设置）。
   */
  function switchSection(section: ActiveSection) {
    state.activeSection = section;
  }

  /**
   * 更新消息输入框内容。
   */
  function setComposer(value: string) {
    state.composer = value;
  }

  /**
   * 创建一个新的空会话并自动切换过去。
   */
  async function handleCreateConversation() {
    try {
      const conversation = await deps.createConversation();
      state.notice = { kind: "success", text: "会话已创建" };
      await reloadConversations();
      await selectConversation(conversation.id);
      openSettings("conversation");
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 切换会话置顶状态。
   */
  async function handleTogglePin(conversation: ConversationSummary) {
    try {
      await deps.updateConversation(conversation.id, { pinned: !conversation.pinned });
      await reloadConversations();
      if (state.activeConversationId === conversation.id) {
        await refreshActiveConversation();
      }
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 切换会话归档状态。
   */
  async function handleToggleArchive(conversation: ConversationSummary) {
    try {
      await deps.updateConversation(conversation.id, { archived: !conversation.archived });
      state.notice = {
        kind: "info",
        text: conversation.archived ? "会话已恢复到主列表" : "会话已归档"
      };
      await reloadConversations();
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 删除会话。
   */
  async function handleDeleteConversation(id: string) {
    try {
      await deps.deleteConversation(id);
      state.notice = { kind: "success", text: "会话已删除" };
      await reloadConversations();
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 发送消息并启动后台生成。
   */
  async function handleSendMessage() {
    if (!state.activeConversationId || state.composer.trim().length === 0) {
      return;
    }

    state.sending = true;
    state.dryRunSummary = null;

    try {
      const content = state.composer;
      state.composer = "";
      const response = await deps.sendMessage(
        state.activeConversationId,
        {
          content,
          stream: true,
          dryRun: false
        },
        handleGenerationEvent
      );

      if (response.kind === "dryRun") {
        state.dryRunSummary = `模型 ${response.model}，估算 tokens ${response.totalTokensEstimate}`;
      } else {
        insertStartedMessageNodes(state.activeConversationId, response, content);
        state.notice = { kind: "info", text: "消息已发送，正在生成回复" };
      }
    } catch (error) {
      setErrorNotice(error);
    } finally {
      state.sending = false;
    }
  }

  /**
   * 预览本次 send_message 的最终 prompt。
   */
  async function handleDryRun() {
    if (!state.activeConversationId || state.composer.trim().length === 0) {
      return;
    }

    try {
      const response = await deps.sendMessage(state.activeConversationId, {
        content: state.composer,
        stream: false,
        dryRun: true
      });

      if (response.kind === "dryRun") {
        state.dryRunSummary = `目标模型：${response.model}；估算 tokens：${response.totalTokensEstimate}；消息数：${response.messages.length}`;
        state.notice = { kind: "info", text: "已生成 prompt 预览" };
      }
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 取消某个生成中的版本。
   */
  async function handleCancelGeneration(versionId: string) {
    try {
      await deps.cancelGeneration(versionId);
      state.notice = { kind: "info", text: "已请求取消当前生成" };
      if (state.activeConversationId) {
        await reloadMessages(state.activeConversationId);
      }
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 对指定楼层执行 reroll。
   */
  async function handleReroll(nodeId: string) {
    if (!state.activeConversationId) {
      return;
    }

    try {
      const result = await deps.reroll(
        state.activeConversationId,
        nodeId,
        { stream: true },
        handleGenerationEvent
      );
      state.notice = { kind: "info", text: "已开始重新生成" };
      void reloadConversations();

      const currentNodes = state.messagesByConversation[state.activeConversationId] ?? [];
      const targetNode = findNode(currentNodes, nodeId);
      const now = Date.now();

      if (targetNode?.role === "assistant" && targetNode.activeVersionId !== result.assistantVersionId) {
        targetNode.activeVersionId = result.assistantVersionId;
        targetNode.versions = [
          ...targetNode.versions,
          {
            id: result.assistantVersionId,
            nodeId,
            content: "",
            status: "generating",
            modelName: null,
            promptTokens: null,
            completionTokens: null,
            finishReason: null,
            createdAt: now
          }
        ];
        replayPendingChunkEvents(state.activeConversationId);
        return;
      }

      await reloadMessages(state.activeConversationId);
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 切换楼层的 active version，并在必要时按需拉取正文。
   */
  async function handleSwitchVersion(nodeId: string, versionId: string) {
    if (!state.activeConversationId) {
      return;
    }

    try {
      await deps.setActiveVersion(state.activeConversationId, nodeId, versionId);

      const currentNodes = state.messagesByConversation[state.activeConversationId] ?? [];
      let nextNodes = withActiveVersion(currentNodes, nodeId, versionId);
      const targetVersion = nextNodes
        .find((node) => node.id === nodeId)
        ?.versions.find((version) => version.id === versionId);

      if (targetVersion && targetVersion.content === null) {
        const content = await deps.getVersionContent(versionId);
        nextNodes = withVersionContent(nextNodes, versionId, content.content);
      }

      setConversationMessages(state.activeConversationId, nextNodes);
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 处理渠道设置变更后的父级刷新。
   */
  async function handleChannelsChanged() {
    try {
      await reloadChannels();
      if (
        state.selectedModelChannelId &&
        !state.channels.some((channel) => channel.id === state.selectedModelChannelId)
      ) {
        state.selectedModelChannelId = state.channels[0]?.id ?? "";
      }

      if (state.activeConversation?.channelId) {
        clearModelCache(state.activeConversation.channelId);
        await ensureModelsLoaded(state.activeConversation.channelId, true);
      }

      await reloadConversations();
      if (state.activeConversationId) {
        await refreshActiveConversation();
      }
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 更新 Agent 表单名称。
   */
  function setAgentName(value: string) {
    state.agentForm.name = value;
  }

  /**
   * 更新 Agent 表单系统提示词。
   */
  function setAgentSystemPrompt(value: string) {
    state.agentForm.systemPrompt = value;
  }

  /**
   * 进入 Agent 编辑模式。
   */
  function startEditAgent(agent: Agent) {
    state.agentEditingId = agent.id;
    state.agentForm = {
      name: agent.name,
      systemPrompt: agent.systemPrompt ?? ""
    };
  }

  /**
   * 重置 Agent 表单。
   */
  function resetAgentForm() {
    state.agentEditingId = null;
    state.agentForm = { ...EMPTY_AGENT_FORM };
  }

  /**
   * 提交 Agent 表单。
   */
  async function handleSubmitAgent(event: SubmitEvent) {
    event.preventDefault();
    state.agentSaving = true;

    try {
      if (state.agentEditingId) {
        await deps.updateAgent(state.agentEditingId, {
          name: state.agentForm.name,
          systemPrompt: state.agentForm.systemPrompt.trim()
            ? state.agentForm.systemPrompt
            : null
        });
        state.notice = { kind: "success", text: "Agent 已更新" };
      } else {
        await deps.createAgent({
          name: state.agentForm.name,
          systemPrompt: state.agentForm.systemPrompt.trim()
            ? state.agentForm.systemPrompt
            : null
        });
        state.notice = { kind: "success", text: "Agent 已创建" };
      }

      await reloadAgents();
      if (state.activeConversationId) {
        await refreshActiveConversation();
      }
      resetAgentForm();
    } catch (error) {
      setErrorNotice(error);
    } finally {
      state.agentSaving = false;
    }
  }

  /**
   * 切换 Agent 启用状态。
   */
  async function handleToggleAgentEnabled(agent: Agent) {
    try {
      await deps.updateAgent(agent.id, { enabled: !agent.enabled });
      await reloadAgents();
      if (state.activeConversationId) {
        await refreshActiveConversation();
      }
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 删除 Agent。
   */
  async function handleDeleteAgent(id: string) {
    try {
      await deps.deleteAgent(id);
      state.notice = { kind: "success", text: "Agent 已删除" };
      await reloadAgents();
      await reloadConversations();
      if (state.activeConversationId) {
        await refreshActiveConversation();
      }
      if (state.agentEditingId === id) {
        resetAgentForm();
      }
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 更新模型表单字段。
   */
  function setModelField(field: keyof ModelFormState, value: string) {
    state.modelForm[field] = value;
  }

  /**
   * 重置模型表单。
   */
  function resetModelForm() {
    state.modelEditingId = null;
    state.modelForm = { ...EMPTY_MODEL_FORM };
  }

  /**
   * 进入模型编辑模式。
   */
  function startEditModel(model: ChannelModel) {
    state.modelEditingId = model.id;
    state.selectedModelChannelId = model.channelId;
    state.modelForm = {
      modelId: model.modelId,
      displayName: model.displayName ?? "",
      contextWindow: model.contextWindow?.toString() ?? "",
      maxOutputTokens: model.maxOutputTokens?.toString() ?? ""
    };
  }

  /**
   * 将模型表单转换为 transport 载荷。
   */
  function buildModelPayload() {
    return {
      modelId: state.modelForm.modelId,
      displayName: state.modelForm.displayName.trim() || null,
      contextWindow: state.modelForm.contextWindow.trim()
        ? Number(state.modelForm.contextWindow)
        : null,
      maxOutputTokens: state.modelForm.maxOutputTokens.trim()
        ? Number(state.modelForm.maxOutputTokens)
        : null
    };
  }

  /**
   * 切换模型编辑页当前选中的渠道。
   */
  async function handleSelectModelChannel(channelId: string) {
    state.selectedModelChannelId = channelId;
    clearModelCache(channelId);
    await ensureModelsLoaded(channelId, true);
  }

  /**
   * 提交模型表单。
   */
  async function handleSubmitModel(event: SubmitEvent) {
    event.preventDefault();
    if (!state.selectedModelChannelId) {
      return;
    }

    state.modelSaving = true;
    try {
      const payload = buildModelPayload();
      if (state.modelEditingId) {
        await deps.updateModel(state.selectedModelChannelId, state.modelEditingId, {
          displayName: payload.displayName,
          contextWindow: payload.contextWindow,
          maxOutputTokens: payload.maxOutputTokens
        });
        state.notice = { kind: "success", text: "模型已更新" };
      } else {
        await deps.createModel(state.selectedModelChannelId, payload);
        state.notice = { kind: "success", text: "模型已创建" };
      }

      clearModelCache(state.selectedModelChannelId);
      await ensureModelsLoaded(state.selectedModelChannelId, true);
      if (state.activeConversation?.channelId === state.selectedModelChannelId) {
        await refreshActiveConversation();
      }
      resetModelForm();
    } catch (error) {
      setErrorNotice(error);
    } finally {
      state.modelSaving = false;
    }
  }

  /**
   * 删除模型。
   */
  async function handleDeleteModel(id: string) {
    if (!state.selectedModelChannelId) {
      return;
    }

    try {
      await deps.deleteModel(state.selectedModelChannelId, id);
      state.notice = { kind: "success", text: "模型已删除" };
      clearModelCache(state.selectedModelChannelId);
      await ensureModelsLoaded(state.selectedModelChannelId, true);
      if (state.activeConversation?.channelId === state.selectedModelChannelId) {
        await refreshActiveConversation();
      }
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 从远程渠道拉取模型候选。
   */
  async function handleFetchRemoteModels() {
    if (!state.selectedModelChannelId) {
      return;
    }

    state.remoteModelsLoadingChannelId = state.selectedModelChannelId;
    try {
      setRemoteModels(
        state.selectedModelChannelId,
        await deps.fetchRemoteModels(state.selectedModelChannelId)
      );
      state.notice = { kind: "info", text: "远程模型候选已刷新" };
    } catch (error) {
      setErrorNotice(error);
    } finally {
      state.remoteModelsLoadingChannelId = null;
    }
  }

  /**
   * 将远程模型候选导入本地模型库。
   */
  async function handleImportRemoteModel(model: RemoteModelInfo) {
    if (!state.selectedModelChannelId) {
      return;
    }

    try {
      await deps.createModel(state.selectedModelChannelId, {
        modelId: model.modelId,
        displayName: model.displayName,
        contextWindow: model.contextWindow,
        maxOutputTokens: null
      });
      state.notice = { kind: "success", text: `已导入模型 ${model.modelId}` };
      clearModelCache(state.selectedModelChannelId);
      await ensureModelsLoaded(state.selectedModelChannelId, true);
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 更新会话标题草稿。
   */
  function setConversationTitleDraft(value: string) {
    state.conversationDraft.title = value;
  }

  /**
   * 更新会话 Agent 绑定草稿。
   */
  function setConversationAgentDraft(value: string) {
    state.conversationDraft.agentId = value;
  }

  /**
   * 更新会话渠道绑定草稿。
   */
  function setConversationChannelDraft(value: string) {
    state.conversationDraft.channelId = value;
  }

  /**
   * 更新会话模型绑定草稿。
   */
  function setConversationModelDraft(value: string) {
    state.conversationDraft.modelId = value;
  }

  /**
   * 保存当前会话的绑定关系与标题。
   */
  async function handleSaveConversationSettings(event: SubmitEvent) {
    event.preventDefault();
    if (!state.activeConversationId) {
      return;
    }

    state.conversationSaving = true;
    try {
      await deps.updateConversation(state.activeConversationId, {
        title: state.conversationDraft.title.trim() || "新会话",
        agentId: state.conversationDraft.agentId || null,
        channelId: state.conversationDraft.channelId || null,
        channelModelId: state.conversationDraft.modelId || null
      });
      state.notice = { kind: "success", text: "会话设置已保存" };
      await reloadConversations();
      await refreshActiveConversation();
      closeSettings();
    } catch (error) {
      setErrorNotice(error);
    } finally {
      state.conversationSaving = false;
    }
  }

  /**
   * 删除消息版本（从当前活跃消息中移除该版本）。
   */
  async function handleDeleteVersion(nodeId: string, versionId: string) {
    if (!state.activeConversationId) return;
    try {
      await deps.cancelGeneration(versionId).catch(() => {});
      // deleteVersion 在 transport 层已有
      const { deleteVersion: del } = await import("../lib/transport/messages");
      await del(state.activeConversationId, nodeId, versionId);
      await reloadMessages(state.activeConversationId);
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * "编辑"消息：复制内容到 composer，删除当前版本。
   */
  async function handleEditMessage(nodeId: string, versionId: string, content: string) {
    state.composer = content;
    await handleDeleteVersion(nodeId, versionId);
  }

  /**
   * 快速切换当前会话的模型绑定。
   */
  async function handleQuickModelChange(modelId: string) {
    if (!state.activeConversationId) return;
    try {
      await deps.updateConversation(state.activeConversationId, {
        channelModelId: modelId || null
      });
      await refreshActiveConversation();
      await reloadConversations();
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 快速切换当前会话的 Agent 绑定。
   */
  async function handleQuickAgentChange(agentId: string) {
    if (!state.activeConversationId) return;
    try {
      await deps.updateConversation(state.activeConversationId, {
        agentId: agentId || null
      });
      await refreshActiveConversation();
      await reloadConversations();
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 快速切换当前会话的渠道绑定。
   */
  async function handleQuickChannelChange(channelId: string) {
    if (!state.activeConversationId) return;
    try {
      await deps.updateConversation(state.activeConversationId, {
        channelId: channelId || null,
        channelModelId: null
      });
      await refreshActiveConversation();
      await reloadConversations();
      if (channelId) {
        await ensureModelsLoaded(channelId, true);
      }
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 快速编辑会话标题。
   */
  async function handleQuickTitleChange(title: string) {
    if (!state.activeConversationId || !title.trim()) return;
    try {
      await deps.updateConversation(state.activeConversationId, { title: title.trim() });
      await reloadConversations();
      await refreshActiveConversation();
    } catch (error) {
      setErrorNotice(error);
    }
  }

  const destroy = $effect.root(() => {
    /**
     * 首次挂载时完成页面初始化。
     */
    $effect(() => {
      if (initialized) {
        return;
      }

      initialized = true;
      void bootstrap();
    });

    /**
     * 当活跃会话变化时，同步会话设置草稿。
     */
    $effect(() => {
      syncConversationDraft(state.activeConversation);
    });

    /**
     * 当会话设置里切换渠道草稿时，按需加载模型列表。
     */
    $effect(() => {
      const channelId = state.conversationDraft.channelId;
      if (channelId === lastConversationDraftChannelId) {
        return;
      }

      lastConversationDraftChannelId = channelId;
      state.conversationDraft.modelId = "";
      if (!channelId) {
        return;
      }

      void ensureModelsLoaded(channelId);
    });
  });

  return {
    state,
    destroy: () => {
      pendingChunkEvents.clear();
      destroy();
    },
    get activeMessages() {
      return getActiveMessages();
    },
    get activeChannelModels() {
      return getActiveChannelModels();
    },
    get selectedChannelModels() {
      return getSelectedChannelModels();
    },
    get selectedRemoteModels() {
      return getSelectedRemoteModels();
    },
    get draftedConversationModels() {
      return getDraftedConversationModels();
    },
    openSettings,
    closeSettings,
    selectSettingsTab,
    selectConversation,
    handleCreateConversation,
    handleTogglePin,
    handleToggleArchive,
    handleDeleteConversation,
    setComposer,
    handleSendMessage,
    handleDryRun,
    handleCancelGeneration,
    handleReroll,
    handleSwitchVersion,
    handleChannelsChanged,
    setAgentName,
    setAgentSystemPrompt,
    startEditAgent,
    resetAgentForm,
    handleSubmitAgent,
    handleToggleAgentEnabled,
    handleDeleteAgent,
    setModelField,
    resetModelForm,
    startEditModel,
    handleSelectModelChannel,
    handleSubmitModel,
    handleDeleteModel,
    handleFetchRemoteModels,
    handleImportRemoteModel,
    setConversationTitleDraft,
    setConversationAgentDraft,
    setConversationChannelDraft,
    setConversationModelDraft,
    handleSaveConversationSettings,
    refreshActiveConversation,
    reloadConversations,
    reloadChannels,
    reloadAgents,
    reloadMessages,
    handleGenerationEvent,
    switchSection,
    handleDeleteVersion,
    handleEditMessage,
    handleQuickModelChange,
    handleQuickAgentChange,
    handleQuickChannelChange,
    handleQuickTitleChange
  };
}
