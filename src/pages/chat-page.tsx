import { useEffect, useState, useTransition } from "react";

import {
  createConversation,
  createCustomChannel,
  deleteCustomChannel,
  deleteConversation,
  deleteMessage,
  forkConversationFromMessage,
  getCustomChannelApiKey,
  listConversations,
  listCustomChannels,
  listMessageBundle,
  listModels,
  regenerateMessage,
  refreshCustomChannelModels,
  saveCustomChannelModels,
  saveMessageEdit,
  sendMessage,
  switchMessageVersion,
  testProviderConnection,
  type ConversationRow,
  type CustomChannelRow,
  type MessageRow,
  type ModelInfo,
  updateConversationModel,
  updateConversationTitle,
  updateConversationSystemPrompt,
  updateCustomChannel,
} from "../api/tauri";

const DEFAULT_BASE_URL = "https://api.wataruu.me/v1";

function formatTimestamp(value: number) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  const pad = (input: number, size = 2) => input.toString().padStart(size, "0");
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(
    date.getHours()
  )}:${pad(date.getMinutes())}:${pad(date.getSeconds())}.${pad(
    date.getMilliseconds(),
    3
  )}`;
}

export function ChatPage() {
  const [channelName, setChannelName] = useState("默认渠道");
  const [channelType, setChannelType] = useState("openai");
  const [baseUrl, setBaseUrl] = useState(DEFAULT_BASE_URL);
  const [modelsPath, setModelsPath] = useState("/models");
  const [chatPath, setChatPath] = useState("/chat/completions");
  const [streamPath, setStreamPath] = useState("/chat/completions");
  const [apiKey, setApiKey] = useState("");
  const [showApiKey, setShowApiKey] = useState(false);
  const [channels, setChannels] = useState<CustomChannelRow[]>([]);
  const [selectedChannelId, setSelectedChannelId] = useState<string | null>(null);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [selectedModel, setSelectedModel] = useState("gpt-5.1");
  const [conversations, setConversations] = useState<ConversationRow[]>([]);
  const [activeConversationId, setActiveConversationId] = useState<string | null>(null);
  const [messages, setMessages] = useState<MessageRow[]>([]);
  const [messageVersions, setMessageVersions] = useState<Record<string, MessageRow[]>>({});
  const [editingMessageId, setEditingMessageId] = useState<string | null>(null);
  const [editingContent, setEditingContent] = useState("");
  const [systemPromptDraft, setSystemPromptDraft] = useState("");
  const [titleDraft, setTitleDraft] = useState("");
  const [modelDraft, setModelDraft] = useState("");
  const [draft, setDraft] = useState("");
  const [status, setStatus] = useState("准备就绪");
  const [error, setError] = useState<string | null>(null);
  const [isBusy, startTransition] = useTransition();

  const activeConversation =
    conversations.find((item) => item.id === activeConversationId) ?? null;
  const selectedChannel =
    channels.find((item) => item.id === selectedChannelId) ?? null;

  useEffect(() => {
    void bootstrap();
  }, []);

  async function bootstrap() {
    setError(null);
    try {
      const [savedChannels, convoPage] = await Promise.all([
        listCustomChannels(),
        listConversations(),
      ]);
      setChannels(savedChannels);
      if (savedChannels.length > 0) {
        await selectChannel(savedChannels[0]);
      }

      setConversations(convoPage.items);
      if (convoPage.items.length > 0) {
        const first = convoPage.items[0];
        setActiveConversationId(first.id);
        setTitleDraft(first.title);
        setSelectedModel(first.model_id);
        setSystemPromptDraft(first.system_prompt ?? "");
        await refreshMessages(first.id);
      }
    } catch (err) {
      setError(toMessage(err));
    }
  }

  async function refreshMessages(convId: string) {
    const bundle = await listMessageBundle(convId);
    setMessages(bundle.active_messages);
    setMessageVersions(bundle.versions_by_group);
  }

  async function selectChannel(channel: CustomChannelRow) {
    setSelectedChannelId(channel.id);
    setChannelName(channel.name);
    setChannelType(channel.channel_type);
    setBaseUrl(channel.base_url);
    try {
      const endpoints = JSON.parse(channel.endpoints_json) as {
        models?: string;
        chat?: string;
        stream?: string;
      };
      setModelsPath(endpoints.models ?? "/models");
      setChatPath(endpoints.chat ?? "/chat/completions");
      setStreamPath(endpoints.stream ?? endpoints.chat ?? "/chat/completions");
    } catch {
      setModelsPath("/models");
      setChatPath("/chat/completions");
      setStreamPath("/chat/completions");
    }
    const savedApiKey = await getCustomChannelApiKey(channel.id);
    setApiKey(savedApiKey ?? "");
    try {
      const savedModels = JSON.parse(channel.models_json) as ModelInfo[];
      setModels(savedModels);
      if (savedModels.length > 0) {
        setSelectedModel(savedModels[0].id);
      }
    } catch {
      setModels([]);
    }
  }

  async function handleSaveChannel() {
    if (!channelName.trim()) {
      setError("渠道名称不能为空");
      return;
    }
    if (!baseUrl.trim()) {
      setError("Base URL 不能为空");
      return;
    }
    if (!modelsPath.trim() || !chatPath.trim() || !streamPath.trim()) {
      setError("请求路径不能为空");
      return;
    }

    setStatus(selectedChannelId ? "正在更新渠道..." : "正在创建渠道...");
    setError(null);
    try {
      const channel = selectedChannelId
        ? await updateCustomChannel({
            id: selectedChannelId,
            name: channelName.trim(),
            channelType,
            baseUrl: baseUrl.trim(),
            modelsPath: modelsPath.trim(),
            chatPath: chatPath.trim(),
            streamPath: streamPath.trim(),
            apiKey,
          })
        : await createCustomChannel({
            name: channelName.trim(),
            channelType,
            baseUrl: baseUrl.trim(),
            modelsPath: modelsPath.trim(),
            chatPath: chatPath.trim(),
            streamPath: streamPath.trim(),
            apiKey,
          });

      const nextChannels = await listCustomChannels();
      setChannels(nextChannels);
      await selectChannel(channel);

      if (apiKey.trim()) {
        const nextModels = await listModels(`custom:${channel.id}`);
        setModels(nextModels);
        if (nextModels.length > 0) {
          setSelectedModel((current) => {
            if (nextModels.some((item) => item.id === current)) {
              return current;
            }
            return nextModels[0].id;
          });
        }
        setStatus("渠道已保存，模型列表已刷新");
      } else {
        setModels([]);
        setStatus("渠道已保存，当前未设置 API Key");
      }
    } catch (err) {
      setError(toMessage(err));
      setStatus("保存渠道失败");
    }
  }

  async function handleTestChannel() {
    if (!selectedChannelId) {
      setError("请先选择或保存渠道");
      return;
    }
    setStatus("正在测试连接...");
    setError(null);
    try {
      const result = await testProviderConnection(`custom:${selectedChannelId}`);
      setStatus(result.message);
    } catch (err) {
      setError(toMessage(err));
      setStatus("连接失败");
    }
  }

  async function handleRefreshModels() {
    if (!selectedChannelId) {
      setError("请先选择或保存渠道");
      return;
    }
    setStatus("正在拉取模型...");
    setError(null);
    try {
      const nextModels = await listModels(`custom:${selectedChannelId}`);
      setModels(nextModels);
      if (nextModels.length > 0 && !nextModels.some((item) => item.id === selectedModel)) {
        setSelectedModel(nextModels[0].id);
      }
      setStatus(`已拉取 ${nextModels.length} 个模型`);
    } catch (err) {
      setError(toMessage(err));
      setStatus("拉取模型失败");
    }
  }

  async function handleRefreshAndSaveModels() {
    if (!selectedChannelId) {
      setError("请先选择渠道");
      return;
    }

    setStatus("正在拉取并保存模型...");
    setError(null);
    try {
      const nextModels = await refreshCustomChannelModels(selectedChannelId);
      setModels(nextModels);
      if (nextModels.length > 0) {
        setSelectedModel(nextModels[0].id);
      }
      const refreshedChannels = await listCustomChannels();
      setChannels(refreshedChannels);
      setStatus(`已保存 ${nextModels.length} 个模型`);
    } catch (err) {
      setError(toMessage(err));
      setStatus("保存模型失败");
    }
  }

  async function handleAddModel() {
    if (!selectedChannelId) {
      setError("请先选择渠道");
      return;
    }
    const modelId = modelDraft.trim();
    if (!modelId) {
      return;
    }

    const nextModels = dedupeModels([
      ...models,
      {
        id: modelId,
        name: modelId,
        context_length: null,
        supports_vision: false,
        supports_function_calling: true,
      },
    ]);

    setStatus("正在保存模型列表...");
    setError(null);
    try {
      await saveCustomChannelModels({ id: selectedChannelId, models: nextModels });
      setModels(nextModels);
      setModelDraft("");
      const refreshedChannels = await listCustomChannels();
      setChannels(refreshedChannels);
      setStatus("模型列表已更新");
    } catch (err) {
      setError(toMessage(err));
      setStatus("模型列表保存失败");
    }
  }

  async function handleRemoveModel(modelId: string) {
    if (!selectedChannelId) {
      return;
    }

    const nextModels = models.filter((item) => item.id !== modelId);
    setStatus("正在移除模型...");
    setError(null);
    try {
      await saveCustomChannelModels({ id: selectedChannelId, models: nextModels });
      setModels(nextModels);
      if (selectedModel === modelId) {
        setSelectedModel(nextModels[0]?.id ?? "");
      }
      const refreshedChannels = await listCustomChannels();
      setChannels(refreshedChannels);
      setStatus("模型已移除");
    } catch (err) {
      setError(toMessage(err));
      setStatus("移除模型失败");
    }
  }

  async function handleCreateConversation() {
    if (!selectedModel) {
      setError("请先选择模型");
      return;
    }
    if (!selectedChannelId) {
      setError("请先保存并选择渠道");
      return;
    }

    setStatus("正在创建会话...");
    setError(null);
    try {
      const conversation = await createConversation({
        modelId: selectedModel,
        provider: `custom:${selectedChannelId}`,
      });

      setConversations((current) => [conversation, ...current]);
      setActiveConversationId(conversation.id);
      setTitleDraft(conversation.title);
      setSystemPromptDraft(conversation.system_prompt ?? "");
      setMessages([]);
      setMessageVersions({});
      setStatus("会话已创建");
    } catch (err) {
      setError(toMessage(err));
      setStatus("创建会话失败");
    }
  }

  async function handleSelectConversation(convId: string) {
    setActiveConversationId(convId);
    const selected = conversations.find((item) => item.id === convId);
    if (selected) {
      setTitleDraft(selected.title);
      setSelectedModel(selected.model_id);
      setSystemPromptDraft(selected.system_prompt ?? "");
    }

    setStatus("正在加载消息...");
    setError(null);
    try {
      await refreshMessages(convId);
      setStatus("消息已加载");
    } catch (err) {
      setError(toMessage(err));
      setStatus("加载消息失败");
    }
  }

  async function ensureActiveConversationProvider() {
    if (!activeConversation || !selectedChannelId || !selectedModel) {
      return;
    }

    const expectedProvider = `custom:${selectedChannelId}`;
    if (
      activeConversation.provider === expectedProvider &&
      activeConversation.model_id === selectedModel
    ) {
      return;
    }

    await updateConversationModel({
      id: activeConversation.id,
      modelId: selectedModel,
      provider: expectedProvider,
    });

    setConversations((current) =>
      current.map((item) =>
        item.id === activeConversation.id
          ? {
              ...item,
              provider: expectedProvider,
              model_id: selectedModel,
            }
          : item
      )
    );
  }

  function handleSendMessage() {
    if (!draft.trim()) {
      return;
    }
    if (!activeConversationId) {
      setError("请先创建会话");
      return;
    }

    startTransition(() => {
      void (async () => {
        setStatus("正在发送消息...");
        setError(null);
        try {
          await ensureActiveConversationProvider();
          await sendMessage({
            convId: activeConversationId,
            content: draft.trim(),
            overrideModel: selectedModel || undefined,
          });
          setDraft("");
          await Promise.all([refreshMessages(activeConversationId), reloadConversations()]);
          setStatus("消息发送完成");
        } catch (err) {
          setError(toMessage(err));
          setStatus("发送失败");
        }
      })();
    });
  }

  function handleRegenerate() {
    if (!activeConversationId) {
      setError("请先选择会话");
      return;
    }

    startTransition(() => {
      void (async () => {
        setStatus("正在重新生成...");
        setError(null);
        try {
          await ensureActiveConversationProvider();
          await regenerateMessage({ convId: activeConversationId });
          await Promise.all([refreshMessages(activeConversationId), reloadConversations()]);
          setStatus("已生成新版本回复");
        } catch (err) {
          setError(toMessage(err));
          setStatus("重新生成失败");
        }
      })();
    });
  }

  async function reloadConversations() {
    const convoPage = await listConversations();
    setConversations(convoPage.items);
  }

  async function handleDeleteMessage(messageId: string) {
    if (!activeConversationId) {
      return;
    }

    setStatus("正在删除消息...");
    setError(null);
    try {
      await deleteMessage({ convId: activeConversationId, messageId });
      await Promise.all([refreshMessages(activeConversationId), reloadConversations()]);
      setStatus("消息已删除");
    } catch (err) {
      setError(toMessage(err));
      setStatus("删除消息失败");
    }
  }

  async function handleSaveEdit(messageId: string) {
    if (!activeConversationId || !editingContent.trim()) {
      return;
    }

    setStatus("正在保存编辑...");
    setError(null);
    try {
      await saveMessageEdit({
        convId: activeConversationId,
        messageId,
        newContent: editingContent.trim(),
      });
      setEditingMessageId(null);
      setEditingContent("");
      await Promise.all([refreshMessages(activeConversationId), reloadConversations()]);
      setStatus("消息已更新，后续消息已保留在当前会话");
    } catch (err) {
      setError(toMessage(err));
      setStatus("编辑消息失败");
    }
  }

  async function handleRegenerateMessage(messageId: string) {
    if (!activeConversationId) {
      return;
    }

    setStatus("正在重新生成该消息...");
    setError(null);
    try {
      await ensureActiveConversationProvider();
      await regenerateMessage({ convId: activeConversationId, messageId });
      await Promise.all([refreshMessages(activeConversationId), reloadConversations()]);
      setStatus("消息已重新生成");
    } catch (err) {
      setError(toMessage(err));
      setStatus("消息重新生成失败");
    }
  }

  async function handleSaveSystemPrompt() {
    if (!activeConversation) {
      setError("请先选择会话");
      return;
    }

    setStatus("正在保存系统消息...");
    setError(null);
    try {
      await updateConversationSystemPrompt({
        id: activeConversation.id,
        systemPrompt: systemPromptDraft.trim() || null,
      });
      setConversations((current) =>
        current.map((item) =>
          item.id === activeConversation.id
            ? { ...item, system_prompt: systemPromptDraft.trim() || null }
            : item
        )
      );
      setStatus("系统消息已保存");
    } catch (err) {
      setError(toMessage(err));
      setStatus("保存系统消息失败");
    }
  }

  async function handleSwitchVersion(versionGroupId: string, targetIndex: number) {
    if (!activeConversationId) {
      return;
    }

    setStatus("正在切换消息分支...");
    setError(null);
    try {
      await switchMessageVersion({ versionGroupId, targetIndex });
      await Promise.all([refreshMessages(activeConversationId), reloadConversations()]);
      setStatus("已切换消息分支");
    } catch (err) {
      setError(toMessage(err));
      setStatus("切换消息分支失败");
    }
  }

  function getVersionPosition(item: MessageRow) {
    const versions = messageVersions[item.version_group_id] ?? [];
    return versions.findIndex((version) => version.id === item.id);
  }

  async function handleDeleteChannel() {
    if (!selectedChannelId) {
      setError("请先选择渠道");
      return;
    }

    setStatus("正在删除渠道...");
    setError(null);
    try {
      await deleteCustomChannel(selectedChannelId);
      const nextChannels = await listCustomChannels();
      setChannels(nextChannels);
      setModels([]);
      if (nextChannels.length > 0) {
        await selectChannel(nextChannels[0]);
      } else {
        setSelectedChannelId(null);
        setChannelName("默认渠道");
        setBaseUrl(DEFAULT_BASE_URL);
        setApiKey("");
      }
      setStatus("渠道已删除");
    } catch (err) {
      setError(toMessage(err));
      setStatus("删除渠道失败");
    }
  }

  function handleNewChannel() {
    setSelectedChannelId(null);
    setChannelName("新渠道");
    setChannelType("openai");
    setBaseUrl(DEFAULT_BASE_URL);
    setModelsPath("/models");
    setChatPath("/chat/completions");
    setStreamPath("/chat/completions");
    setApiKey("");
    setModels([]);
    setStatus("正在创建新渠道");
    setError(null);
  }

  async function handleForkConversation(messageId: string) {
    if (!activeConversationId) {
      return;
    }

    setStatus("正在创建分支会话...");
    setError(null);
    try {
      const forked = await forkConversationFromMessage({
        convId: activeConversationId,
        messageId,
      });
      const convoPage = await listConversations();
      setConversations(convoPage.items);
      setActiveConversationId(forked.id);
      setTitleDraft(forked.title);
      setSelectedModel(forked.model_id);
      setSystemPromptDraft(forked.system_prompt ?? "");
      await refreshMessages(forked.id);
      setStatus("已创建分支会话，后续修改将与原会话独立");
    } catch (err) {
      setError(toMessage(err));
      setStatus("创建分支会话失败");
    }
  }

  const lastAssistant = [...messages].reverse().find((item) => item.role === "assistant");

  async function handleRenameConversation() {
    if (!activeConversation || !titleDraft.trim()) {
      return;
    }

    setStatus("正在重命名会话...");
    setError(null);
    try {
      await updateConversationTitle({
        id: activeConversation.id,
        title: titleDraft.trim(),
      });
      setConversations((current) =>
        current.map((item) =>
          item.id === activeConversation.id ? { ...item, title: titleDraft.trim() } : item
        )
      );
      setStatus("会话名称已更新");
    } catch (err) {
      setError(toMessage(err));
      setStatus("重命名会话失败");
    }
  }

  async function handleDeleteConversation() {
    if (!activeConversation) {
      return;
    }
    if (!globalThis.confirm("确认删除当前会话？此操作不可恢复。")) {
      return;
    }

    setStatus("正在删除会话...");
    setError(null);
    try {
      await deleteConversation(activeConversation.id);
      const convoPage = await listConversations();
      setConversations(convoPage.items);
      const next = convoPage.items[0] ?? null;
      setActiveConversationId(next?.id ?? null);
      setTitleDraft(next?.title ?? "");
      setSelectedModel(next?.model_id ?? selectedModel);
      setSystemPromptDraft(next?.system_prompt ?? "");
      if (next) {
        await refreshMessages(next.id);
      } else {
        setMessages([]);
        setMessageVersions({});
      }
      setStatus("会话已删除");
    } catch (err) {
      setError(toMessage(err));
      setStatus("删除会话失败");
    }
  }

  return (
    <main className="workspace">
      <aside className="sidebar">
        <section className="panel brand-panel">
          <p className="eyebrow">OmniChat</p>
          <h1>本地优先 AI 客户端</h1>
          <p className="muted">
            当前页面已切到多渠道模式：渠道管理、真实模型拉取、建会话、发消息、重生成。
          </p>
        </section>

        <section className="panel">
          <div className="section-head">
            <h2>API 渠道</h2>
            <span className="tag">{selectedChannel ? "已选择" : "未保存"}</span>
          </div>

          <label className="field">
            <span>渠道名称</span>
            <input
              value={channelName}
              onChange={(event) => setChannelName(event.target.value)}
              placeholder="例如：Wataru / OpenAI / 本地 vLLM"
            />
          </label>

          <label className="field">
            <span>渠道类型</span>
            <select
              value={channelType}
              onChange={(event) => setChannelType(event.target.value)}
            >
              <option value="openai">openai</option>
              <option value="openai-response">openai-response</option>
            </select>
          </label>

          <label className="field">
            <span>Base URL</span>
            <input
              value={baseUrl}
              onChange={(event) => setBaseUrl(event.target.value)}
              placeholder="https://api.wataruu.me/v1"
            />
          </label>

          <label className="field">
            <span>Models Path</span>
            <input
              value={modelsPath}
              onChange={(event) => setModelsPath(event.target.value)}
              placeholder="/models 或 models"
            />
          </label>

          <label className="field">
            <span>Chat Path</span>
            <input
              value={chatPath}
              onChange={(event) => setChatPath(event.target.value)}
              placeholder="/v1/chat/completions 或 chat/completions"
            />
          </label>

          <label className="field">
            <span>Stream Path</span>
            <input
              value={streamPath}
              onChange={(event) => setStreamPath(event.target.value)}
              placeholder="/v1/chat/completions 或 chat/completions"
            />
          </label>

          <label className="field">
            <span>API Key</span>
            <div className="input-with-action">
              <input
                value={apiKey}
                onChange={(event) => setApiKey(event.target.value)}
                placeholder="留空表示不设置或清除当前 Key"
                type={showApiKey ? "text" : "password"}
              />
              <button
                className="button-secondary small-button"
                onClick={() => setShowApiKey((current) => !current)}
                type="button"
              >
                {showApiKey ? "隐藏" : "显示"}
              </button>
            </div>
          </label>

          <div className="button-row">
            <button onClick={handleSaveChannel} type="button">
              {selectedChannelId ? "保存修改" : "创建渠道"}
            </button>
            <button className="button-secondary" onClick={handleTestChannel} type="button">
              测试连接
            </button>
          </div>

          <div className="button-row">
            <button className="button-secondary" onClick={handleNewChannel} type="button">
              新渠道
            </button>
            <button className="button-secondary" onClick={handleRefreshModels} type="button">
              临时拉取
            </button>
            <button
              className="button-secondary"
              disabled={!selectedChannelId}
              onClick={handleRefreshAndSaveModels}
              type="button"
            >
              拉取并保存
            </button>
            <button
              className="button-secondary"
              disabled={!selectedChannelId}
              onClick={handleDeleteChannel}
              type="button"
            >
              删除渠道
            </button>
          </div>

          <label className="field">
            <span>模型</span>
            <select
              value={selectedModel}
              onChange={(event) => setSelectedModel(event.target.value)}
            >
              {models.length === 0 ? <option value={selectedModel}>{selectedModel}</option> : null}
              {models.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.id}
                </option>
              ))}
            </select>
          </label>

          <div className="field">
            <span>模型库</span>
            <div className="input-with-action">
              <input
                value={modelDraft}
                onChange={(event) => setModelDraft(event.target.value)}
                placeholder="手动添加模型 ID"
              />
              <button className="button-secondary small-button" onClick={() => void handleAddModel()} type="button">
                添加
              </button>
            </div>
          </div>

          <div className="model-list">
            {models.map((item) => (
              <button
                key={item.id}
                className={item.id === selectedModel ? "model-chip active" : "model-chip"}
                onClick={() => setSelectedModel(item.id)}
                type="button"
              >
                <span>{item.id}</span>
                <strong
                  onClick={(event) => {
                    event.stopPropagation();
                    void handleRemoveModel(item.id);
                  }}
                >
                  ×
                </strong>
              </button>
            ))}
          </div>

          <div className="conversation-list">
            {channels.length === 0 ? (
              <p className="empty-text">还没有保存的渠道。</p>
            ) : (
              channels.map((item) => (
                <button
                  key={item.id}
                  className={item.id === selectedChannelId ? "conversation-item active" : "conversation-item"}
                  onClick={() => void selectChannel(item)}
                  type="button"
                >
                  <strong>{item.name}</strong>
                  <span>{item.channel_type} · {item.base_url}</span>
                  <time>{formatTimestamp(item.updated_at)}</time>
                </button>
              ))
            )}
          </div>
        </section>

        <section className="panel conversations-panel">
          <div className="section-head">
            <h2>会话</h2>
            <button className="button-secondary small-button" onClick={handleCreateConversation} type="button">
              新建
            </button>
          </div>

          <div className="conversation-list">
            {conversations.length === 0 ? (
              <p className="empty-text">还没有会话，先创建一个。</p>
            ) : (
              conversations.map((item) => (
                <button
                  key={item.id}
                  className={item.id === activeConversationId ? "conversation-item active" : "conversation-item"}
                  onClick={() => void handleSelectConversation(item.id)}
                  type="button"
                >
                  <strong>{item.title}</strong>
                  <span>{item.model_id}</span>
                  <time>{formatTimestamp(item.updated_at)}</time>
                </button>
              ))
            )}
          </div>
        </section>
      </aside>

      <section className="chat-stage">
        <header className="panel chat-header">
          <div>
            <p className="eyebrow">Active Conversation</p>
            <div className="input-with-action">
              <input
                value={titleDraft}
                onChange={(event) => setTitleDraft(event.target.value)}
                placeholder="输入会话名称"
              />
              <button
                className="button-secondary small-button"
                disabled={!activeConversation}
                onClick={() => void handleRenameConversation()}
                type="button"
              >
                重命名
              </button>
              <button
                className="button-secondary small-button"
                disabled={!activeConversation}
                onClick={() => void handleDeleteConversation()}
                type="button"
              >
                删除会话
              </button>
            </div>
            <p className="muted">
              {activeConversation
                ? `${activeConversation.provider} / ${activeConversation.model_id}`
                : "先保存渠道并创建会话"}
            </p>
          </div>
          <div className="chat-actions">
            <button
              className="button-secondary"
              disabled={!lastAssistant || isBusy}
              onClick={handleRegenerate}
              type="button"
            >
              重新生成
            </button>
          </div>
        </header>

        <section className="panel composer-panel">
          <label className="field">
            <span>系统消息</span>
            <textarea
              value={systemPromptDraft}
              onChange={(event) => setSystemPromptDraft(event.target.value)}
              placeholder="为当前会话设置系统消息。"
              rows={4}
            />
          </label>
          <div className="composer-actions">
            <span className="muted">保存后会作用于后续生成</span>
            <button
              className="button-secondary"
              disabled={!activeConversation}
              onClick={() => void handleSaveSystemPrompt()}
              type="button"
            >
              保存系统消息
            </button>
          </div>
        </section>

        <section className="panel status-panel">
          <span className={error ? "status-text error" : "status-text"}>{error ?? status}</span>
        </section>

        <section className="panel messages-panel">
          {messages.length === 0 ? (
            <div className="empty-state">
              <h3>消息还没有开始</h3>
              <p>保存配置、拉取模型、创建会话之后，就可以发送第一条消息。</p>
            </div>
          ) : (
            messages.map((item, index) => (
              <article
                key={item.id}
                className={item.role === "user" ? "message-bubble user" : "message-bubble assistant"}
              >
                <div className="message-meta">
                  <strong>
                    #{index + 1} · {item.role}
                  </strong>
                  <span>
                    v{getVersionPosition(item) + 1 || 1}
                    {messageVersions[item.version_group_id]?.length > 1
                      ? ` / ${messageVersions[item.version_group_id]?.length}`
                      : ""}
                  </span>
                  <time>{formatTimestamp(item.created_at)}</time>
                  {messageVersions[item.version_group_id]?.length > 1 ? (
                    <span className="version-switch">
                      {(() => {
                        const versions = messageVersions[item.version_group_id] ?? [];
                        const position = getVersionPosition(item);
                        const prev = position > 0 ? versions[position - 1] : null;
                        const next =
                          position >= 0 && position + 1 < versions.length
                            ? versions[position + 1]
                            : null;

                        return (
                          <>
                      <button
                        className="button-secondary small-button"
                        disabled={!prev}
                        onClick={() =>
                          prev
                            ? void handleSwitchVersion(
                                item.version_group_id,
                                prev.version_index
                              )
                            : undefined
                        }
                        type="button"
                      >
                        ←
                      </button>
                      <button
                        className="button-secondary small-button"
                        disabled={!next}
                        onClick={() =>
                          next
                            ? void handleSwitchVersion(
                                item.version_group_id,
                                next.version_index
                              )
                            : undefined
                        }
                        type="button"
                      >
                        →
                      </button>
                          </>
                        );
                      })()}
                    </span>
                  ) : null}
                </div>
                {editingMessageId === item.id ? (
                  <>
                    <textarea
                      value={editingContent}
                      onChange={(event) => setEditingContent(event.target.value)}
                      rows={4}
                    />
                    <div className="message-actions">
                      <button onClick={() => void handleSaveEdit(item.id)} type="button">
                        保存编辑
                      </button>
                      <button
                        className="button-secondary"
                        onClick={() => {
                          setEditingMessageId(null);
                          setEditingContent("");
                        }}
                        type="button"
                      >
                        取消
                      </button>
                    </div>
                  </>
                ) : (
                  <p>{item.content ?? ""}</p>
                )}
                {item.role === "assistant" ? (
                  <div className="message-meta">
                    <span>{item.provider ?? activeConversation?.provider ?? "unknown"}</span>
                    <span>{item.model_id ?? activeConversation?.model_id ?? "unknown"}</span>
                    <span>tokens: {item.tokens_used ?? 0}</span>
                  </div>
                ) : null}
                <div className="message-actions">
                  <button
                    className="button-secondary small-button"
                    onClick={() => {
                      setEditingMessageId(item.id);
                      setEditingContent(item.content ?? "");
                    }}
                    type="button"
                  >
                    编辑
                  </button>
                  <button
                    className="button-secondary small-button"
                    onClick={() => void handleRegenerateMessage(item.id)}
                    type="button"
                  >
                    重新生成
                  </button>
                  <button
                    className="button-secondary small-button"
                    onClick={() => void handleDeleteMessage(item.id)}
                    type="button"
                  >
                    删除消息
                  </button>
                  <button
                    className="button-secondary small-button"
                    onClick={() => void handleForkConversation(item.id)}
                    type="button"
                  >
                    分支会话
                  </button>
                </div>
              </article>
            ))
          )}
        </section>

        <footer className="panel composer-panel">
          <label className="field">
            <span>输入消息</span>
            <textarea
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder="输入你的问题，然后发送。"
              rows={5}
            />
          </label>

          <div className="composer-actions">
            <span className="muted">
              {activeConversation ? `当前模型：${selectedModel}` : "还没有会话"}
            </span>
            <button disabled={!activeConversationId || isBusy} onClick={handleSendMessage} type="button">
              发送消息
            </button>
          </div>
        </footer>
      </section>
    </main>
  );
}

function toMessage(error: unknown) {
  if (typeof error === "string") {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return "发生了未知错误";
}

function dedupeModels(items: ModelInfo[]) {
  return Object.values(
    items.reduce<Record<string, ModelInfo>>((acc, item) => {
      acc[item.id] = item;
      return acc;
    }, {})
  );
}
