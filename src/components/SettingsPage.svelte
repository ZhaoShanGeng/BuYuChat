<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    parseThinkingTagsConfig,
    serializeThinkingTagsInput
  } from "../lib/thinking-tags";
  import type { AppError } from "../lib/transport/common";
  import { toAppError } from "../lib/transport/common";
  import type { Channel, ChannelInput } from "../lib/transport/channels";
  import {
    createChannel,
    deleteChannel,
    listChannels,
    testChannel,
    updateChannel
  } from "../lib/transport/channels";
  import type { ChannelModel, ModelInput, RemoteModelInfo } from "../lib/transport/models";
  import {
    createModel,
    deleteModel,
    fetchRemoteModels,
    listModels
  } from "../lib/transport/models";
  import SettingsChannelEditor from "./SettingsChannelEditor.svelte";
  import SettingsChannelSidebar from "./SettingsChannelSidebar.svelte";
  import SettingsModelManager from "./SettingsModelManager.svelte";
  import SettingsPageHeader from "./SettingsPageHeader.svelte";
  import type {
    ChannelFormState,
    Notice,
    SelectOption
  } from "./settings-page.types";

  type Props = {
    onChanged?: () => void | Promise<void>;
  };

  const { onChanged = async () => undefined }: Props = $props();
  const currentWindow = getCurrentWindow();

  const EMPTY_FORM: ChannelFormState = {
    name: "",
    baseUrl: "https://api.openai.com",
    apiKey: "",
    authType: "bearer",
    modelsEndpoint: "/v1/models",
    chatEndpoint: "/v1/chat/completions",
    streamEndpoint: "/v1/chat/completions",
    thinkingTagsInput: "",
    channelType: "openai_compatible",
    enabled: true
  };

  const CHANNEL_TYPE_OPTIONS: SelectOption[] = [
    { value: "openai_compatible", label: "OpenAI Compatible" }
  ];

  const AUTH_TYPE_OPTIONS: SelectOption[] = [
    { value: "bearer", label: "Bearer Token" },
    { value: "x_api_key", label: "X-API-Key" },
    { value: "none", label: "无鉴权" }
  ];

  let initialized = false;
  let search = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let testingId = $state<string | null>(null);
  let notice = $state<Notice | null>(null);
  let channels = $state<Channel[]>([]);
  let selectedChannelId = $state<string | null>(null);
  let form = $state<ChannelFormState>({ ...EMPTY_FORM });
  let models = $state<ChannelModel[]>([]);
  let remoteModels = $state<RemoteModelInfo[]>([]);
  let loadingModels = $state(false);
  let loadingRemoteModels = $state(false);
  let managingModels = $state(false);
  let addingModel = $state(false);
  let newModelId = $state("");
  let newModelDisplayName = $state("");

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

  function setNotice(kind: Notice["kind"], text: string) {
    notice = { kind, text };
  }

  function humanizeError(error: AppError): string {
    switch (error.errorCode) {
      case "INVALID_URL":
        return "请输入有效的 API 地址";
      case "NAME_EMPTY":
        return "名称不能为空";
      case "VALIDATION_ERROR":
        return "输入不合法，请检查后重试";
      case "CHANNEL_UNREACHABLE":
        return "无法连接到该渠道，请检查地址和密钥";
      case "MODEL_ID_CONFLICT":
        return "该渠道下已存在相同模型 ID";
      case "NOT_FOUND":
        return "目标资源不存在，列表已为你刷新";
      default:
        return error.message || "操作失败，请稍后重试";
    }
  }

  function buildFormFromChannel(channel: Channel): ChannelFormState {
    return {
      name: channel.name,
      baseUrl: channel.baseUrl,
      apiKey: channel.apiKey ?? "",
      authType: channel.authType ?? "bearer",
      modelsEndpoint: channel.modelsEndpoint ?? "/v1/models",
      chatEndpoint: channel.chatEndpoint ?? "/v1/chat/completions",
      streamEndpoint: channel.streamEndpoint ?? "/v1/chat/completions",
      thinkingTagsInput: parseThinkingTagsConfig(channel.thinkingTags).join(", "),
      channelType: channel.channelType || "openai_compatible",
      enabled: channel.enabled
    };
  }

  function clearModelDraft() {
    addingModel = false;
    managingModels = false;
    newModelId = "";
    newModelDisplayName = "";
    remoteModels = [];
  }

  function normalizeChannelType(value: string): string | null {
    const normalized = value.trim().toLowerCase().replace(/[\s-]+/g, "_");
    return normalized || null;
  }

  function normalizeAuthType(value: string): string | null {
    const normalized = value.trim().toLowerCase().replace(/[\s-]+/g, "_");
    if (!normalized) {
      return null;
    }
    if (normalized === "bearer_token") {
      return "bearer";
    }
    return normalized;
  }

  const filteredChannels = $derived.by(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return channels;
    }

    return channels.filter((channel) =>
      [channel.name, channel.baseUrl, channel.channelType].some((value) =>
        value.toLowerCase().includes(keyword)
      )
    );
  });

  const selectedChannel = $derived.by(() =>
    selectedChannelId ? channels.find((channel) => channel.id === selectedChannelId) ?? null : null
  );

  const groupedModels = $derived.by(() => {
    const groups = new Map<string, ChannelModel[]>();
    for (const model of models) {
      const key = resolveModelGroup(model.modelId);
      const bucket = groups.get(key) ?? [];
      bucket.push(model);
      groups.set(key, bucket);
    }

    return [...groups.entries()]
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([group, items]) => [
        group,
        [...items].sort((a, b) =>
          (a.displayName ?? a.modelId).localeCompare(b.displayName ?? b.modelId)
        )
      ] as const);
  });

  function resolveModelGroup(modelId: string) {
    const normalized = modelId.trim().toLowerCase();
    if (!normalized) {
      return "其他";
    }

    const prefix = normalized.split(/[-/_.]/)[0];
    return prefix || "其他";
  }

  async function loadModels(channelId: string) {
    loadingModels = true;
    try {
      models = await listModels(channelId);
    } catch (error) {
      models = [];
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      loadingModels = false;
    }
  }

  async function loadChannels(nextSelectedChannelId?: string | null) {
    loading = true;
    try {
      channels = await listChannels(true);

      const preferredId =
        nextSelectedChannelId ??
        (selectedChannelId && channels.some((channel) => channel.id === selectedChannelId)
          ? selectedChannelId
          : channels[0]?.id ?? null);

      selectedChannelId = preferredId;
      if (!preferredId) {
        form = { ...EMPTY_FORM };
        models = [];
        clearModelDraft();
        return;
      }

      const channel = channels.find((item) => item.id === preferredId);
      if (!channel) {
        form = { ...EMPTY_FORM };
        models = [];
        clearModelDraft();
        return;
      }

      form = buildFormFromChannel(channel);
      clearModelDraft();
      await loadModels(channel.id);
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      loading = false;
    }
  }

  async function selectChannel(channel: Channel) {
    selectedChannelId = channel.id;
    form = buildFormFromChannel(channel);
    clearModelDraft();
    notice = null;
    await loadModels(channel.id);
  }

  function startCreateChannel() {
    selectedChannelId = null;
    form = { ...EMPTY_FORM };
    models = [];
    clearModelDraft();
    notice = null;
  }

  async function resetCurrentDraft() {
    if (selectedChannel) {
      await selectChannel(selectedChannel);
      return;
    }

    startCreateChannel();
  }

  async function handleSaveChannel() {
    const payload: ChannelInput = {
      name: form.name.trim(),
      baseUrl: form.baseUrl.trim(),
      apiKey: form.apiKey.trim() || null,
      authType: normalizeAuthType(form.authType),
      modelsEndpoint: form.modelsEndpoint.trim() || null,
      chatEndpoint: form.chatEndpoint.trim() || null,
      streamEndpoint: form.streamEndpoint.trim() || null,
      thinkingTags: serializeThinkingTagsInput(form.thinkingTagsInput),
      channelType: normalizeChannelType(form.channelType),
      enabled: form.enabled
    };

    saving = true;
    notice = null;
    try {
      const channel = selectedChannelId
        ? await updateChannel(selectedChannelId, payload)
        : await createChannel(payload);
      await loadChannels(channel.id);
      setNotice("success", selectedChannelId ? "渠道已更新" : "渠道已创建");
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      saving = false;
    }
  }

  async function handleDeleteChannel() {
    if (!selectedChannelId) {
      return;
    }

    notice = null;
    try {
      await deleteChannel(selectedChannelId);
      startCreateChannel();
      await loadChannels();
      setNotice("success", "渠道已删除");
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleTestChannel() {
    if (!selectedChannelId) {
      return;
    }

    testingId = selectedChannelId;
    notice = null;
    try {
      const result = await testChannel(selectedChannelId);
      setNotice("success", result.message ?? "渠道连通性验证成功");
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      testingId = null;
    }
  }

  async function handleCreateModel() {
    if (!selectedChannelId || !newModelId.trim()) {
      return;
    }

    const payload: ModelInput = {
      modelId: newModelId.trim(),
      displayName: newModelDisplayName.trim() || null
    };

    notice = null;
    try {
      await createModel(selectedChannelId, payload);
      newModelId = "";
      newModelDisplayName = "";
      addingModel = false;
      await loadModels(selectedChannelId);
      setNotice("success", "模型已创建");
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleDeleteModel(id: string) {
    if (!selectedChannelId) {
      return;
    }

    notice = null;
    try {
      await deleteModel(selectedChannelId, id);
      await loadModels(selectedChannelId);
      setNotice("success", "模型已删除");
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleFetchRemoteModels() {
    if (!selectedChannelId) {
      return;
    }

    loadingRemoteModels = true;
    notice = null;
    try {
      remoteModels = await fetchRemoteModels(selectedChannelId);
      setNotice("info", `已刷新 ${remoteModels.length} 个远程模型候选`);
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      loadingRemoteModels = false;
    }
  }

  async function handleImportRemoteModel(model: RemoteModelInfo) {
    if (!selectedChannelId) {
      return;
    }

    notice = null;
    try {
      await createModel(selectedChannelId, {
        modelId: model.modelId,
        displayName: model.displayName,
        contextWindow: model.contextWindow
      });
      await loadModels(selectedChannelId);
      setNotice("success", `已导入模型 ${model.displayName ?? model.modelId}`);
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  $effect(() => {
    if (initialized) {
      return;
    }

    initialized = true;
    void loadChannels();
  });
</script>

<div class="settings-page flex h-full min-h-0 flex-col bg-background text-foreground" data-ui="settings-page">
  <SettingsPageHeader onHeaderMouseDown={handleHeaderMouseDown} />

  <div class="settings-page__layout flex min-h-0 flex-1">
    <SettingsChannelSidebar
      channels={filteredChannels}
      loading={loading}
      notice={notice}
      onCreate={startCreateChannel}
      onSelect={selectChannel}
      search={search}
      selectedChannelEnabled={selectedChannelId ? form.enabled : false}
      {selectedChannelId}
    />

    <section class="settings-page__content min-h-0 flex-1 overflow-y-auto" data-ui="settings-page-content">
      <div class="settings-page__content-inner mx-auto flex flex-col gap-6 p-6">
        <SettingsChannelEditor
          authTypeOptions={AUTH_TYPE_OPTIONS}
          channelTypeOptions={CHANNEL_TYPE_OPTIONS}
          form={form}
          onDelete={handleDeleteChannel}
          onReset={resetCurrentDraft}
          onSave={(event) => {
            event.preventDefault();
            return handleSaveChannel();
          }}
          onTest={handleTestChannel}
          saving={saving}
          selectedChannel={selectedChannel}
          {selectedChannelId}
          {testingId}
        />

        <SettingsModelManager
          addingModel={addingModel}
          groupedModels={groupedModels}
          loadingModels={loadingModels}
          loadingRemoteModels={loadingRemoteModels}
          managingModels={managingModels}
          models={models}
          newModelDisplayName={newModelDisplayName}
          newModelId={newModelId}
          onCreateModel={handleCreateModel}
          onDeleteModel={handleDeleteModel}
          onFetchRemoteModels={handleFetchRemoteModels}
          onImportRemoteModel={handleImportRemoteModel}
          onNewModelDisplayNameChange={(value) => (newModelDisplayName = value)}
          onNewModelIdChange={(value) => (newModelId = value)}
          onToggleAdding={() => (addingModel = !addingModel)}
          onToggleManaging={() => (managingModels = !managingModels)}
          remoteModels={remoteModels}
          {selectedChannelId}
        />
      </div>
    </section>
  </div>
</div>
