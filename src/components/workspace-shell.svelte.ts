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
  editMessage,
  getVersionContent,
  listMessages,
  reroll,
  sendMessage,
  setActiveVersion,
  type GenerationEvent,
  type EditMessageInput,
  type EditMessageResult,
  type ImageAttachment,
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

export type MessageHistoryState = {
  hasOlder: boolean;
  loadingOlder: boolean;
  loadedCount: number;
  oldestOrderKey: string | null;
};

export type PendingComposerImage = ImageAttachment & {
  name: string;
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
    limit?: number,
    fromLatest?: boolean
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
  editMessage: (
    id: string,
    nodeId: string,
    input: EditMessageInput,
    onEvent?: (event: GenerationEvent) => void
  ) => Promise<EditMessageResult>;
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

const MESSAGE_PAGE_SIZE = 40;

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
  editMessage,
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
    /** 正在切换中的目标会话 ID，用于侧边栏即时高亮。 */
    pendingConversationId: null as string | null,
    messagesByConversation: {} as Record<string, MessageNode[]>,
    messageHistoryByConversation: {} as Record<string, MessageHistoryState>,
    modelsByChannel: {} as Record<string, ChannelModel[]>,
    remoteModelsByChannel: {} as Record<string, RemoteModelInfo[]>,
    notice: null as Notice | null,
    composer: "",
    pendingImages: [] as PendingComposerImage[],
    sending: false,
    dryRunSummary: null as string | null,
    renamingConversationId: null as string | null,
    renamingConversationTitle: "",
    agentEditingId: null as string | null,
    agentEditorMode: null as null | "create" | "edit",
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
  let noticeTimer: ReturnType<typeof setTimeout> | null = null;

  /**
   * 获取当前活跃会话对应的消息列表。
   */
  function getActiveMessages() {
    return state.activeConversationId
      ? state.messagesByConversation[state.activeConversationId] ?? []
      : [];
  }

  function getEmptyMessageHistory(): MessageHistoryState {
    return {
      hasOlder: false,
      loadingOlder: false,
      loadedCount: 0,
      oldestOrderKey: null
    };
  }

  function ensureMessageHistory(conversationId: string): MessageHistoryState {
    if (!state.messageHistoryByConversation[conversationId]) {
      state.messageHistoryByConversation[conversationId] = getEmptyMessageHistory();
    }

    return state.messageHistoryByConversation[conversationId];
  }

  function getActiveMessageHistory() {
    return state.activeConversationId
      ? ensureMessageHistory(state.activeConversationId)
      : getEmptyMessageHistory();
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
   * 判断当前渠道缓存里是否已经包含指定渠道。
   */
  function hasKnownChannel(channelId: string | null | undefined) {
    return !!channelId && state.channels.some((channel) => channel.id === channelId);
  }

  /**
   * 判断错误是否来自缺失渠道资源。
   */
  function isChannelNotFoundError(error: unknown) {
    const appError = toAppError(error);
    return appError.errorCode === "NOT_FOUND" && /channel/i.test(appError.message);
  }

  /**
   * 将错误写入顶部通知。
   */
  function setErrorNotice(error: unknown) {
    setNotice({
      kind: "error",
      text: humanizeWorkspaceError(toAppError(error))
    });
  }

  /**
   * 统一设置顶部通知，并在 3 秒后自动消失。
   */
  function setNotice(notice: Notice | null) {
    state.notice = notice;
    if (noticeTimer) {
      clearTimeout(noticeTimer);
      noticeTimer = null;
    }
    if (notice) {
      noticeTimer = setTimeout(() => {
        state.notice = null;
        noticeTimer = null;
      }, 3000);
    }
  }

  /**
   * 更新某个会话的消息缓存。
   */
  function setConversationMessages(
    conversationId: string,
    nodes: MessageNode[],
    historyPatch: Partial<MessageHistoryState> = {}
  ) {
    state.messagesByConversation[conversationId] = nodes;
    const history = ensureMessageHistory(conversationId);
    history.loadedCount = historyPatch.loadedCount ?? nodes.length;
    history.oldestOrderKey = nodes[0]?.orderKey ?? null;
    if (historyPatch.hasOlder !== undefined) {
      history.hasOlder = historyPatch.hasOlder;
    }
    if (historyPatch.loadingOlder !== undefined) {
      history.loadingOlder = historyPatch.loadingOlder;
    }
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
    content: string,
    images: PendingComposerImage[]
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
            thinkingContent: null,
            images: images.map(({ name: _, ...image }) => image),
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
            thinkingContent: null,
            images: [],
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
      if (event.reasoningDelta) {
        existing.reasoningDelta = `${existing.reasoningDelta ?? ""}${event.reasoningDelta}`;
      }
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
    if (event.reasoningDelta) {
      version.thinkingContent = `${version.thinkingContent ?? ""}${event.reasoningDelta}`;
    }
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
          setConversationMessages(
            event.conversationId,
            nodes.filter((node) => node.id !== event.nodeId)
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
   * 同步当前活跃会话依赖的最新渠道与模型绑定。
   *
   * - `force = false` 时，仅在本地缓存缺失当前绑定渠道时刷新渠道列表。
   * - `force = true` 时，会强制刷新渠道列表、会话详情和模型缓存，用于自愈陈旧状态。
   */
  async function syncLatestChannelBindings(force = false) {
    if (force) {
      await reloadChannels();
    }

    if (!state.activeConversationId) {
      return;
    }

    if (force || !state.activeConversation) {
      state.activeConversation = await deps.getConversation(state.activeConversationId);
      syncConversationDraft(state.activeConversation);
    }

    const channelId = state.activeConversation?.channelId;
    if (channelId && !hasKnownChannel(channelId)) {
      await reloadChannels();
    }

    if (channelId) {
      try {
        await ensureModelsLoaded(channelId, force);
      } catch {
        clearModelCache(channelId);
      }
    }
  }

  /**
   * 执行依赖渠道绑定的操作；若命中陈旧渠道缓存导致的 NOT_FOUND，则自动同步一次后重试。
   */
  async function runWithChannelBindingRecovery<T>(task: () => Promise<T>) {
    if (state.activeConversation?.channelId && !hasKnownChannel(state.activeConversation.channelId)) {
      await syncLatestChannelBindings(true);
    }

    try {
      return await task();
    } catch (error) {
      if (!isChannelNotFoundError(error)) {
        throw error;
      }

      await syncLatestChannelBindings(true);
      await reloadConversations();
      return task();
    }
  }

  /**
   * 刷新当前活跃会话详情。
   *
   * ensureModelsLoaded 的失败不会中断整体刷新流程，
   * 以避免渠道被删除后导致会话页面不可用。
   */
  async function refreshActiveConversation() {
    if (!state.activeConversationId) {
      return;
    }

    state.activeConversation = await deps.getConversation(state.activeConversationId);
    syncConversationDraft(state.activeConversation);
    await syncLatestChannelBindings();
  }

  /**
   * 读取某个会话的消息列表。
   */
  async function reloadMessages(conversationId: string, resetPage = false) {
    const reloadVersion = (messageReloadVersions.get(conversationId) ?? 0) + 1;
    messageReloadVersions.set(conversationId, reloadVersion);
    state.messagesLoading = true;
    try {
      const history = ensureMessageHistory(conversationId);
      const targetCount =
        resetPage || history.loadedCount === 0
          ? MESSAGE_PAGE_SIZE
          : Math.max(history.loadedCount, MESSAGE_PAGE_SIZE);
      const nodes = await deps.listMessages(
        conversationId,
        null,
        targetCount + 1,
        true
      );
      if (messageReloadVersions.get(conversationId) !== reloadVersion) {
        return;
      }

      const hasOlder = nodes.length > targetCount;
      const visibleNodes = hasOlder ? nodes.slice(nodes.length - targetCount) : nodes;

      setConversationMessages(conversationId, visibleNodes, {
        hasOlder,
        loadingOlder: false,
        loadedCount: visibleNodes.length
      });
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

  async function loadOlderMessages(conversationId: string) {
    const history = ensureMessageHistory(conversationId);
    if (
      state.messagesLoading ||
      history.loadingOlder ||
      !history.hasOlder ||
      !history.oldestOrderKey
    ) {
      return;
    }

    history.loadingOlder = true;
    try {
      const olderNodes = await deps.listMessages(
        conversationId,
        history.oldestOrderKey,
        MESSAGE_PAGE_SIZE + 1,
        false
      );
      const hasOlder = olderNodes.length > MESSAGE_PAGE_SIZE;
      const visibleOlderNodes = hasOlder
        ? olderNodes.slice(olderNodes.length - MESSAGE_PAGE_SIZE)
        : olderNodes;
      const currentNodes = state.messagesByConversation[conversationId] ?? [];

      setConversationMessages(conversationId, [...visibleOlderNodes, ...currentNodes], {
        hasOlder,
        loadingOlder: false,
        loadedCount: visibleOlderNodes.length + currentNodes.length
      });
    } catch (error) {
      history.loadingOlder = false;
      setErrorNotice(error);
    }
  }

  /**
   * 选择会话并加载详情与消息。
   *
   * 延迟切换 activeConversationId，确保旧消息保持显示直到新数据就绪，
   * 避免切换过程中出现空白闪烁。
   */
  async function selectConversation(id: string) {
    if (state.activeConversationId === id && state.activeConversation) {
      return;
    }

    state.pendingConversationId = id;
    state.conversationsLoading = true;

    try {
      const conversation = await deps.getConversation(id);
      state.activeConversationId = id;
      state.activeConversation = conversation;
      state.dryRunSummary = null;
      syncConversationDraft(conversation);

      if (conversation.channelId && !hasKnownChannel(conversation.channelId)) {
        await reloadChannels();
      }

      await Promise.all([
        reloadMessages(id, true),
        conversation.channelId ? ensureModelsLoaded(conversation.channelId) : Promise.resolve()
      ]);
    } catch (error) {
      setErrorNotice(error);
    } finally {
      state.pendingConversationId = null;
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
   *
   * 分两阶段：Phase 1 加载列表数据后立即解除 bootstrapping 显示侧边栏，
   * Phase 2 并行加载模型和首个会话详情。
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

      // Phase 1 完成：侧边栏可以立即渲染。
      state.bootstrapping = false;

      // Phase 2：并行加载模型缓存和首个会话详情。
      const phase2: Promise<void>[] = [];

      if (loadedChannels.length > 0) {
        state.selectedModelChannelId = loadedChannels[0].id;
        phase2.push(ensureModelsLoaded(loadedChannels[0].id));
      }

      if (loadedConversations.length > 0) {
        phase2.push(selectConversation(loadedConversations[0].id));
      }

      await Promise.all(phase2);
    } catch (error) {
      setErrorNotice(error);
      state.bootstrapping = false;
    }
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

  function setPendingImages(images: PendingComposerImage[]) {
    state.pendingImages = images;
  }

  /**
   * 创建一个新的空会话并自动切换过去。
   */
  async function handleCreateConversation() {
    try {
      const conversation = await deps.createConversation();
      setNotice({ kind: "success", text: "会话已创建" });
      await reloadConversations();
      await selectConversation(conversation.id);
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
      setNotice({
        kind: "info",
        text: conversation.archived ? "会话已恢复到主列表" : "会话已归档"
      });
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
      setNotice({ kind: "success", text: "会话已删除" });
      await reloadConversations();
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 开始重命名会话。
   */
  function startConversationRename(conversation: ConversationSummary) {
    state.renamingConversationId = conversation.id;
    state.renamingConversationTitle = conversation.title;
  }

  /**
   * 取消会话重命名。
   */
  function cancelConversationRename() {
    state.renamingConversationId = null;
    state.renamingConversationTitle = "";
  }

  /**
   * 提交会话重命名。
   */
  async function commitConversationRename() {
    if (!state.renamingConversationId || !state.renamingConversationTitle.trim()) {
      cancelConversationRename();
      return;
    }

    try {
      await deps.updateConversation(state.renamingConversationId, {
        title: state.renamingConversationTitle.trim()
      });
      await reloadConversations();
      if (state.activeConversationId === state.renamingConversationId) {
        await refreshActiveConversation();
      }
      cancelConversationRename();
    } catch (error) {
      setErrorNotice(error);
    }
  }

  /**
   * 发送消息并启动后台生成。
   */
  async function handleSendMessage() {
    if (
      !state.activeConversationId ||
      (state.composer.trim().length === 0 && state.pendingImages.length === 0)
    ) {
      return;
    }

    state.sending = true;
    state.dryRunSummary = null;
    const content = state.composer;
    const images = state.pendingImages.slice();

    try {
      state.composer = "";
      state.pendingImages = [];
      const response = await runWithChannelBindingRecovery(() =>
        deps.sendMessage(
          state.activeConversationId!,
          {
            content,
            images: images.map(({ name: _, ...image }) => image),
            stream: true,
            dryRun: false
          },
          handleGenerationEvent
        )
      );

      if (response.kind === "dryRun") {
        state.dryRunSummary = `模型 ${response.model}，估算 tokens ${response.totalTokensEstimate}`;
      } else {
        insertStartedMessageNodes(state.activeConversationId, response, content, images);
        setNotice({ kind: "info", text: "消息已发送，正在生成回复" });
      }
    } catch (error) {
      if (!state.composer.trim()) {
        state.composer = content;
      }
      if (state.pendingImages.length === 0) {
        state.pendingImages = images;
      }
      setErrorNotice(error);
    } finally {
      state.sending = false;
    }
  }

  /**
   * 预览本次 send_message 的最终 prompt。
   */
  async function handleDryRun() {
    if (
      !state.activeConversationId ||
      (state.composer.trim().length === 0 && state.pendingImages.length === 0)
    ) {
      return;
    }

    try {
      const response = await runWithChannelBindingRecovery(() =>
        deps.sendMessage(state.activeConversationId!, {
          content: state.composer,
          images: state.pendingImages.map(({ name: _, ...image }) => image),
          stream: false,
          dryRun: true
        })
      );

      if (response.kind === "dryRun") {
        state.dryRunSummary = `目标模型：${response.model}；估算 tokens：${response.totalTokensEstimate}；消息数：${response.messages.length}`;
        setNotice({ kind: "info", text: "已生成 prompt 预览" });
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
      setNotice({ kind: "info", text: "已请求取消当前生成" });
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
      const result = await runWithChannelBindingRecovery(() =>
        deps.reroll(
          state.activeConversationId!,
          nodeId,
          { stream: true },
          handleGenerationEvent
        )
      );
      setNotice({ kind: "info", text: "已开始重新生成" });
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
            thinkingContent: null,
            images: [],
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
   * 设置页数据变更后的统一刷新入口。
   */
  async function handleSettingsChanged() {
    await handleChannelsChanged();
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
    state.agentEditorMode = "edit";
    state.agentEditingId = agent.id;
    state.agentForm = {
      name: agent.name,
      systemPrompt: agent.systemPrompt ?? ""
    };
  }

  /**
   * 进入 Agent 新建模式。
   */
  function startCreateAgent() {
    state.agentEditorMode = "create";
    state.agentEditingId = null;
    state.agentForm = { ...EMPTY_AGENT_FORM };
  }

  /**
   * 重置 Agent 表单。
   */
  function resetAgentForm() {
    if (state.agentEditorMode === "edit" && state.agentEditingId) {
      const editingAgent = state.agents.find((agent) => agent.id === state.agentEditingId);
      if (editingAgent) {
        state.agentForm = {
          name: editingAgent.name,
          systemPrompt: editingAgent.systemPrompt ?? ""
        };
        return;
      }
    }

    if (state.agentEditorMode === "create") {
      state.agentForm = { ...EMPTY_AGENT_FORM };
      return;
    }

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
        setNotice({ kind: "success", text: "Agent 已更新" });
      } else {
        await deps.createAgent({
          name: state.agentForm.name,
          systemPrompt: state.agentForm.systemPrompt.trim()
            ? state.agentForm.systemPrompt
            : null
        });
        setNotice({ kind: "success", text: "Agent 已创建" });
      }

      await reloadAgents();
      if (state.activeConversationId) {
        await refreshActiveConversation();
      }
      state.agentEditorMode = null;
      state.agentEditingId = null;
      state.agentForm = { ...EMPTY_AGENT_FORM };
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
      setNotice({ kind: "success", text: "Agent 已删除" });
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
        setNotice({ kind: "success", text: "模型已更新" });
      } else {
        await deps.createModel(state.selectedModelChannelId, payload);
        setNotice({ kind: "success", text: "模型已创建" });
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
      setNotice({ kind: "success", text: "模型已删除" });
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
      setNotice({ kind: "info", text: "远程模型候选已刷新" });
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
      setNotice({ kind: "success", text: `已导入模型 ${model.modelId}` });
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
      setNotice({ kind: "success", text: "会话设置已保存" });
      await reloadConversations();
      await refreshActiveConversation();
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
   * 确保某个版本的正文已经加载到本地缓存。
   */
  async function ensureMessageVersionContent(nodeId: string, versionId: string) {
    if (!state.activeConversationId) {
      return "";
    }

    const currentNodes = state.messagesByConversation[state.activeConversationId] ?? [];
    const existingVersion = currentNodes
      .find((node) => node.id === nodeId)
      ?.versions.find((version) => version.id === versionId);
    if (existingVersion?.content !== null && existingVersion?.content !== undefined) {
      return existingVersion.content;
    }

    const content = await deps.getVersionContent(versionId);
    const nextNodes = withVersionContent(currentNodes, versionId, content.content);
    setConversationMessages(state.activeConversationId, nextNodes);
    return content.content;
  }

  /**
   * 在当前 node 下新建版本，并可选地重新发送。
   */
  async function handleEditMessage(
    nodeId: string,
    content: string,
    options?: { resend?: boolean }
  ) {
    if (!state.activeConversationId) {
      return;
    }

    try {
      await runWithChannelBindingRecovery(() =>
        deps.editMessage(
          state.activeConversationId!,
          nodeId,
          {
            content,
            resend: options?.resend ?? false,
            stream: options?.resend ?? false
          },
          handleGenerationEvent
        )
      );
      setNotice({
        kind: "info",
        text: options?.resend ? "已保存并重新发送" : "已保存新版本"
      });
      await reloadMessages(state.activeConversationId);
      void reloadConversations();
    } catch (error) {
      setErrorNotice(error);
    }
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
   *
   * 模型加载失败时降级处理，不阻塞渠道切换流程。
   */
  async function handleQuickChannelChange(channelId: string) {
    if (!state.activeConversationId) return;
    try {
      if (channelId && !hasKnownChannel(channelId)) {
        await reloadChannels();
      }

      try {
        await deps.updateConversation(state.activeConversationId, {
          channelId: channelId || null,
          channelModelId: null
        });
      } catch (error) {
        if (!channelId || !isChannelNotFoundError(error)) {
          throw error;
        }

        await reloadChannels();
        await deps.updateConversation(state.activeConversationId, {
          channelId: channelId,
          channelModelId: null
        });
      }

      await refreshActiveConversation();
      await reloadConversations();

      if (channelId) {
        try {
          await ensureModelsLoaded(channelId, true);
        } catch {
          clearModelCache(channelId);
          setNotice({ kind: "error", text: "无法加载该渠道的模型列表" });
        }
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
      if (state.renamingConversationId === state.activeConversationId) {
        cancelConversationRename();
      }
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
      if (noticeTimer) {
        clearTimeout(noticeTimer);
      }
      destroy();
    },
    get activeMessages() {
      return getActiveMessages();
    },
    get activeMessageHistory() {
      return getActiveMessageHistory();
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
    selectConversation,
    handleCreateConversation,
    startConversationRename,
    cancelConversationRename,
    commitConversationRename,
    handleTogglePin,
    handleToggleArchive,
    handleDeleteConversation,
    setComposer,
    setPendingImages,
    handleSendMessage,
    handleDryRun,
    handleCancelGeneration,
    handleReroll,
    handleSwitchVersion,
    handleChannelsChanged,
    handleSettingsChanged,
    setAgentName,
    setAgentSystemPrompt,
    startCreateAgent,
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
    syncLatestChannelBindings,
    reloadAgents,
    reloadMessages,
    loadOlderMessages,
    handleGenerationEvent,
    switchSection,
    handleDeleteVersion,
    ensureMessageVersionContent,
    handleEditMessage,
    handleQuickModelChange,
    handleQuickAgentChange,
    handleQuickChannelChange,
    handleQuickTitleChange
  };
}
