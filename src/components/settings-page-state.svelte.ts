/**
 * 设置页的响应式状态工厂。
 *
 * 从 SettingsPage.svelte 提取，使 WorkspaceShell 能同时驱动
 * 侧边栏（渠道列表）和主内容区（编辑器 + 模型管理）。
 */

import {
  buildSettingsBackup,
  openDataDirectory,
  openLogDirectory,
  pickImportFile,
  readSettingsBackupFromFile,
  toChannelInput,
  toModelInput,
  writeSettingsBackupToFile
} from "../lib/system-tools";
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
  listModels,
  updateModel
} from "../lib/transport/models";
import type {
  ChannelFormState,
  Notice,
  SelectOption
} from "./settings-page.types";

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

export const CHANNEL_TYPE_OPTIONS: SelectOption[] = [
  { value: "openai_compatible", label: "OpenAI Compatible" }
];

export const AUTH_TYPE_OPTIONS: SelectOption[] = [
  { value: "bearer", label: "Bearer Token" },
  { value: "x_api_key", label: "X-API-Key" },
  { value: "none", label: "无鉴权" }
];

export type SettingsPageDeps = {
  onChanged: () => void | Promise<void>;
};

export function createSettingsPageState(deps: SettingsPageDeps) {
  let initialized = false;
  let modelsRequestId = 0;
  let remoteModelsRequestId = 0;

  const state = $state({
    search: "",
    loading: true,
    saving: false,
    testingId: null as string | null,
    notice: null as Notice | null,
    utilitiesBusy: false,
    channels: [] as Channel[],
    selectedChannelId: null as string | null,
    form: { ...EMPTY_FORM } as ChannelFormState,
    models: [] as ChannelModel[],
    remoteModels: [] as RemoteModelInfo[],
    loadingModels: false,
    loadingRemoteModels: false,
    managingModels: false,
    addingModel: false,
    newModelId: "",
    newModelDisplayName: ""
  });

  const filteredChannels = $derived.by(() => {
    const keyword = state.search.trim().toLowerCase();
    if (!keyword) return state.channels;
    return state.channels.filter((channel) =>
      [channel.name, channel.baseUrl, channel.channelType].some((value) =>
        value.toLowerCase().includes(keyword)
      )
    );
  });

  const selectedChannel = $derived.by(() =>
    state.selectedChannelId
      ? state.channels.find((ch) => ch.id === state.selectedChannelId) ?? null
      : null
  );

  const groupedModels = $derived.by(() => {
    const groups = new Map<string, ChannelModel[]>();
    for (const model of state.models) {
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

  function setNotice(kind: Notice["kind"], text: string) {
    state.notice = { kind, text };
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
    state.addingModel = false;
    state.managingModels = false;
    state.newModelId = "";
    state.newModelDisplayName = "";
    state.remoteModels = [];
  }

  function normalizeChannelType(value: string): string | null {
    const normalized = value.trim().toLowerCase().replace(/[\s-]+/g, "_");
    return normalized || null;
  }

  function normalizeAuthType(value: string): string | null {
    const normalized = value.trim().toLowerCase().replace(/[\s-]+/g, "_");
    if (!normalized) return null;
    if (normalized === "bearer_token") return "bearer";
    return normalized;
  }

  function resolveModelGroup(modelId: string) {
    const normalized = modelId.trim().toLowerCase();
    if (!normalized) return "其他";
    const prefix = normalized.split(/[-/_.]/)[0];
    return prefix || "其他";
  }

  async function loadModels(channelId: string) {
    const requestId = ++modelsRequestId;
    state.loadingModels = true;
    state.models = [];
    try {
      const models = await listModels(channelId);
      if (requestId !== modelsRequestId || state.selectedChannelId !== channelId) {
        return;
      }
      state.models = models;
    } catch (error) {
      if (requestId !== modelsRequestId || state.selectedChannelId !== channelId) {
        return;
      }
      state.models = [];
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      if (requestId === modelsRequestId && state.selectedChannelId === channelId) {
        state.loadingModels = false;
      }
    }
  }

  async function loadChannels(nextSelectedChannelId?: string | null) {
    state.loading = true;
    try {
      state.channels = await listChannels(true);

      const preferredId =
        nextSelectedChannelId ??
        (state.selectedChannelId &&
        state.channels.some((ch) => ch.id === state.selectedChannelId)
          ? state.selectedChannelId
          : state.channels[0]?.id ?? null);

      state.selectedChannelId = preferredId;
      if (!preferredId) {
        state.form = { ...EMPTY_FORM };
        state.models = [];
        clearModelDraft();
        return;
      }

      const channel = state.channels.find((item) => item.id === preferredId);
      if (!channel) {
        state.form = { ...EMPTY_FORM };
        state.models = [];
        clearModelDraft();
        return;
      }

      state.form = buildFormFromChannel(channel);
      clearModelDraft();
      await loadModels(channel.id);
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      state.loading = false;
    }
  }

  async function selectChannel(channel: Channel) {
    state.selectedChannelId = channel.id;
    state.form = buildFormFromChannel(channel);
    clearModelDraft();
    state.models = [];
    state.notice = null;
    await loadModels(channel.id);
  }

  function startCreateChannel() {
    state.selectedChannelId = null;
    state.form = { ...EMPTY_FORM };
    state.models = [];
    clearModelDraft();
    state.notice = null;
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
      name: state.form.name.trim(),
      baseUrl: state.form.baseUrl.trim(),
      apiKey: state.form.apiKey.trim() || null,
      authType: normalizeAuthType(state.form.authType),
      modelsEndpoint: state.form.modelsEndpoint.trim() || null,
      chatEndpoint: state.form.chatEndpoint.trim() || null,
      streamEndpoint: state.form.streamEndpoint.trim() || null,
      thinkingTags: serializeThinkingTagsInput(state.form.thinkingTagsInput),
      channelType: normalizeChannelType(state.form.channelType),
      enabled: state.form.enabled
    };

    state.saving = true;
    state.notice = null;
    try {
      const channel = state.selectedChannelId
        ? await updateChannel(state.selectedChannelId, payload)
        : await createChannel(payload);
      await loadChannels(channel.id);
      setNotice("success", state.selectedChannelId ? "渠道已更新" : "渠道已创建");
      await deps.onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      state.saving = false;
    }
  }

  async function handleDeleteChannel() {
    if (!state.selectedChannelId) return;
    state.notice = null;
    try {
      await deleteChannel(state.selectedChannelId);
      startCreateChannel();
      await loadChannels();
      setNotice("success", "渠道已删除");
      await deps.onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleTestChannel() {
    if (!state.selectedChannelId) return;
    state.testingId = state.selectedChannelId;
    state.notice = null;
    try {
      const result = await testChannel(state.selectedChannelId);
      setNotice("success", result.message ?? "渠道连通性验证成功");
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      state.testingId = null;
    }
  }

  async function handleCreateModel() {
    if (!state.selectedChannelId || !state.newModelId.trim()) return;
    const payload: ModelInput = {
      modelId: state.newModelId.trim(),
      displayName: state.newModelDisplayName.trim() || null
    };
    state.notice = null;
    try {
      await createModel(state.selectedChannelId, payload);
      state.newModelId = "";
      state.newModelDisplayName = "";
      state.addingModel = false;
      await loadModels(state.selectedChannelId);
      setNotice("success", "模型已创建");
      await deps.onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleDeleteModel(id: string) {
    if (!state.selectedChannelId) return;
    state.notice = null;
    try {
      await deleteModel(state.selectedChannelId, id);
      await loadModels(state.selectedChannelId);
      setNotice("success", "模型已删除");
      await deps.onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleFetchRemoteModels() {
    if (!state.selectedChannelId) return;
    const channelId = state.selectedChannelId;
    const requestId = ++remoteModelsRequestId;
    state.loadingRemoteModels = true;
    state.notice = null;
    try {
      const remoteModels = await fetchRemoteModels(channelId);
      if (requestId !== remoteModelsRequestId || state.selectedChannelId !== channelId) {
        return;
      }
      state.remoteModels = remoteModels;
      setNotice("info", `已刷新 ${state.remoteModels.length} 个远程模型候选`);
    } catch (error) {
      if (requestId !== remoteModelsRequestId || state.selectedChannelId !== channelId) {
        return;
      }
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      if (requestId === remoteModelsRequestId && state.selectedChannelId === channelId) {
        state.loadingRemoteModels = false;
      }
    }
  }

  async function handleImportRemoteModel(model: RemoteModelInfo) {
    if (!state.selectedChannelId) return;
    state.notice = null;
    try {
      await createModel(state.selectedChannelId, {
        modelId: model.modelId,
        displayName: model.displayName,
        contextWindow: model.contextWindow
      });
      await loadModels(state.selectedChannelId);
      setNotice("success", `已导入模型 ${model.displayName ?? model.modelId}`);
      await deps.onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleExportSettings() {
    state.utilitiesBusy = true;
    state.notice = null;
    try {
      const channels = await listChannels(true);
      const modelsByChannel = new Map<string, ChannelModel[]>();
      for (const channel of channels) {
        modelsByChannel.set(channel.id, await listModels(channel.id));
      }

      const backup = buildSettingsBackup(channels, modelsByChannel);
      const path = await writeSettingsBackupToFile(backup);
      if (!path) {
        setNotice("info", "已取消导出");
        return;
      }

      setNotice("success", `配置已导出到 ${path}`);
    } catch (error) {
      const message =
        error && typeof error === "object" && "error_code" in error
          ? humanizeError(toAppError(error))
          : error instanceof Error
            ? error.message
            : "导出配置失败";
      setNotice("error", message);
    } finally {
      state.utilitiesBusy = false;
    }
  }

  async function handleImportSettings() {
    state.utilitiesBusy = true;
    state.notice = null;
    try {
      const path = await pickImportFile();
      if (!path) {
        setNotice("info", "已取消导入");
        return;
      }

      const backup = await readSettingsBackupFromFile(path);
      const existingChannels = await listChannels(true);

      let importedChannels = 0;
      let importedModels = 0;

      for (const backupChannel of backup.channels) {
        const matchedChannel = existingChannels.find(
          (channel) =>
            channel.name.trim().toLowerCase() === backupChannel.name.trim().toLowerCase() &&
            channel.baseUrl.trim().toLowerCase() === backupChannel.baseUrl.trim().toLowerCase()
        );

        const channel = matchedChannel
          ? await updateChannel(matchedChannel.id, toChannelInput(backupChannel))
          : await createChannel(toChannelInput(backupChannel));

        if (!matchedChannel) {
          existingChannels.push(channel);
        }

        importedChannels += 1;

        const existingModels = await listModels(channel.id);
        for (const backupModel of backupChannel.models) {
          const matchedModel = existingModels.find(
            (model) => model.modelId.trim().toLowerCase() === backupModel.modelId.trim().toLowerCase()
          );

          if (matchedModel) {
            await updateModel(channel.id, matchedModel.id, {
              displayName: backupModel.displayName ?? null,
              contextWindow: backupModel.contextWindow ?? null,
              maxOutputTokens: backupModel.maxOutputTokens ?? null,
              temperature: backupModel.temperature ?? null,
              topP: backupModel.topP ?? null
            });
          } else {
            await createModel(channel.id, toModelInput(backupModel));
          }

          importedModels += 1;
        }
      }

      await loadChannels(state.selectedChannelId);
      await deps.onChanged();
      setNotice("success", `已导入 ${importedChannels} 个渠道和 ${importedModels} 个模型`);
    } catch (error) {
      setNotice("error", error instanceof Error ? error.message : "导入配置失败");
    } finally {
      state.utilitiesBusy = false;
    }
  }

  async function handleOpenDataDirectory() {
    state.utilitiesBusy = true;
    state.notice = null;
    try {
      await openDataDirectory();
      setNotice("info", "已打开数据目录");
    } catch (error) {
      setNotice("error", error instanceof Error ? error.message : "打开数据目录失败");
    } finally {
      state.utilitiesBusy = false;
    }
  }

  async function handleOpenLogDirectory() {
    state.utilitiesBusy = true;
    state.notice = null;
    try {
      await openLogDirectory();
      setNotice("info", "已打开日志目录");
    } catch (error) {
      setNotice("error", error instanceof Error ? error.message : "打开日志目录失败");
    } finally {
      state.utilitiesBusy = false;
    }
  }

  function init() {
    if (initialized) return;
    initialized = true;
    void loadChannels();
  }

  return {
    state,
    get filteredChannels() {
      return filteredChannels;
    },
    get selectedChannel() {
      return selectedChannel;
    },
    get groupedModels() {
      return groupedModels;
    },
    init,
    selectChannel,
    startCreateChannel,
    resetCurrentDraft,
    handleSaveChannel,
    handleDeleteChannel,
    handleTestChannel,
    handleCreateModel,
    handleDeleteModel,
    handleFetchRemoteModels,
    handleImportRemoteModel,
    handleImportSettings,
    handleExportSettings,
    handleOpenDataDirectory,
    handleOpenLogDirectory
  };
}

export type SettingsPageStateReturn = ReturnType<typeof createSettingsPageState>;
