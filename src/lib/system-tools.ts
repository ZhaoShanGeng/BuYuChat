import { appLocalDataDir, appLogDir, downloadDir, join } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { openPath } from "@tauri-apps/plugin-opener";
import type { Channel, ChannelInput } from "./transport/channels";
import type { ChannelModel, ModelInput } from "./transport/models";

export type SettingsBackupModel = {
  modelId: string;
  displayName?: string | null;
  contextWindow?: number | null;
  maxOutputTokens?: number | null;
  temperature?: string | null;
  topP?: string | null;
};

export type SettingsBackupChannel = {
  name: string;
  baseUrl: string;
  channelType?: string | null;
  apiKey?: string | null;
  apiKeys?: string | null;
  authType?: string | null;
  modelsEndpoint?: string | null;
  chatEndpoint?: string | null;
  streamEndpoint?: string | null;
  thinkingTags?: string | null;
  enabled?: boolean | null;
  models: SettingsBackupModel[];
};

export type SettingsBackup = {
  schemaVersion: 1;
  exportedAt: string;
  channels: SettingsBackupChannel[];
};

export function buildSettingsBackup(
  channels: Channel[],
  modelsByChannel: Map<string, ChannelModel[]>
): SettingsBackup {
  return {
    schemaVersion: 1,
    exportedAt: new Date().toISOString(),
    channels: channels.map((channel) => ({
      name: channel.name,
      baseUrl: channel.baseUrl,
      channelType: channel.channelType,
      apiKey: channel.apiKey,
      apiKeys: channel.apiKeys,
      authType: channel.authType,
      modelsEndpoint: channel.modelsEndpoint,
      chatEndpoint: channel.chatEndpoint,
      streamEndpoint: channel.streamEndpoint,
      thinkingTags: channel.thinkingTags,
      enabled: channel.enabled,
      models: (modelsByChannel.get(channel.id) ?? []).map((model) => ({
        modelId: model.modelId,
        displayName: model.displayName,
        contextWindow: model.contextWindow,
        maxOutputTokens: model.maxOutputTokens,
        temperature: model.temperature,
        topP: model.topP
      }))
    }))
  };
}

export function parseSettingsBackup(raw: string): SettingsBackup {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    throw new Error("配置文件不是有效的 JSON");
  }

  if (!parsed || typeof parsed !== "object") {
    throw new Error("配置文件格式无效");
  }

  const backup = parsed as Partial<SettingsBackup>;
  if (backup.schemaVersion !== 1) {
    throw new Error("暂不支持该配置文件版本");
  }

  if (!Array.isArray(backup.channels)) {
    throw new Error("配置文件缺少 channels 列表");
  }

  const channels = backup.channels.map((item, index) => normalizeBackupChannel(item, index));
  return {
    schemaVersion: 1,
    exportedAt: typeof backup.exportedAt === "string" ? backup.exportedAt : new Date().toISOString(),
    channels
  };
}

function normalizeBackupChannel(raw: unknown, index: number): SettingsBackupChannel {
  if (!raw || typeof raw !== "object") {
    throw new Error(`第 ${index + 1} 个渠道配置格式无效`);
  }

  const channel = raw as Partial<SettingsBackupChannel>;
  if (!channel.name?.trim()) {
    throw new Error(`第 ${index + 1} 个渠道缺少名称`);
  }
  if (!channel.baseUrl?.trim()) {
    throw new Error(`第 ${index + 1} 个渠道缺少 baseUrl`);
  }

  return {
    name: channel.name.trim(),
    baseUrl: channel.baseUrl.trim(),
    channelType: normalizeOptionalString(channel.channelType),
    apiKey: normalizeOptionalString(channel.apiKey),
    apiKeys: normalizeOptionalString(channel.apiKeys),
    authType: normalizeOptionalString(channel.authType),
    modelsEndpoint: normalizeOptionalString(channel.modelsEndpoint),
    chatEndpoint: normalizeOptionalString(channel.chatEndpoint),
    streamEndpoint: normalizeOptionalString(channel.streamEndpoint),
    thinkingTags: normalizeOptionalString(channel.thinkingTags),
    enabled: typeof channel.enabled === "boolean" ? channel.enabled : true,
    models: Array.isArray(channel.models)
      ? channel.models.map((model, modelIndex) => normalizeBackupModel(model, index, modelIndex))
      : []
  };
}

function normalizeBackupModel(
  raw: unknown,
  channelIndex: number,
  modelIndex: number
): SettingsBackupModel {
  if (!raw || typeof raw !== "object") {
    throw new Error(`第 ${channelIndex + 1} 个渠道中的第 ${modelIndex + 1} 个模型格式无效`);
  }

  const model = raw as Partial<SettingsBackupModel>;
  if (!model.modelId?.trim()) {
    throw new Error(`第 ${channelIndex + 1} 个渠道中的第 ${modelIndex + 1} 个模型缺少 modelId`);
  }

  return {
    modelId: model.modelId.trim(),
    displayName: normalizeOptionalString(model.displayName),
    contextWindow: typeof model.contextWindow === "number" ? model.contextWindow : null,
    maxOutputTokens: typeof model.maxOutputTokens === "number" ? model.maxOutputTokens : null,
    temperature: normalizeOptionalString(model.temperature),
    topP: normalizeOptionalString(model.topP)
  };
}

function normalizeOptionalString(value: unknown): string | null {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed || null;
}

export function toChannelInput(channel: SettingsBackupChannel): ChannelInput {
  return {
    name: channel.name,
    baseUrl: channel.baseUrl,
    channelType: channel.channelType ?? null,
    apiKey: channel.apiKey ?? null,
    apiKeys: channel.apiKeys ?? null,
    authType: channel.authType ?? null,
    modelsEndpoint: channel.modelsEndpoint ?? null,
    chatEndpoint: channel.chatEndpoint ?? null,
    streamEndpoint: channel.streamEndpoint ?? null,
    thinkingTags: channel.thinkingTags ?? null,
    enabled: channel.enabled ?? true
  };
}

export function toModelInput(model: SettingsBackupModel): ModelInput {
  return {
    modelId: model.modelId,
    displayName: model.displayName ?? null,
    contextWindow: model.contextWindow ?? null,
    maxOutputTokens: model.maxOutputTokens ?? null,
    temperature: model.temperature ?? null,
    topP: model.topP ?? null
  };
}

export async function pickImportFile(): Promise<string | null> {
  const path = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "BuYu Settings Backup", extensions: ["json"] }]
  });
  return typeof path === "string" ? path : null;
}

export async function readSettingsBackupFromFile(path: string): Promise<SettingsBackup> {
  const content = await readTextFile(path);
  return parseSettingsBackup(content);
}

export async function writeSettingsBackupToFile(backup: SettingsBackup): Promise<string | null> {
  const baseDir = await downloadDir();
  const filename = `buyu-settings-${backup.exportedAt.slice(0, 10)}.json`;
  const defaultPath = await join(baseDir, filename);
  const selectedPath = await save({
    defaultPath,
    filters: [{ name: "BuYu Settings Backup", extensions: ["json"] }]
  });

  if (!selectedPath) {
    return null;
  }

  await writeTextFile(selectedPath, JSON.stringify(backup, null, 2));
  return selectedPath;
}

export async function openDataDirectory(): Promise<void> {
  await openPath(await appLocalDataDir());
}

export async function openLogDirectory(): Promise<void> {
  await openPath(await appLogDir());
}
