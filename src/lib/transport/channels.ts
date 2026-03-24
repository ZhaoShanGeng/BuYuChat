/**
 * 渠道管理相关的 Tauri transport 封装。
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * 前端使用的渠道模型。
 */
export type Channel = {
  id: string;
  name: string;
  channelType: string;
  baseUrl: string;
  apiKey: string | null;
  authType: string | null;
  modelsEndpoint: string | null;
  chatEndpoint: string | null;
  streamEndpoint: string | null;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
};

/**
 * 创建渠道时使用的输入模型。
 */
export type ChannelInput = {
  name: string;
  baseUrl: string;
  channelType?: string | null;
  apiKey?: string | null;
  authType?: string | null;
  modelsEndpoint?: string | null;
  chatEndpoint?: string | null;
  streamEndpoint?: string | null;
  enabled?: boolean | null;
};

/**
 * 渠道更新时使用的补丁模型。
 */
export type ChannelPatch = Partial<ChannelInput>;

/**
 * 渠道连通性测试结果。
 */
export type ChannelTestResult = {
  success: boolean;
  message: string | null;
};

/**
 * 前端统一使用的错误模型。
 */
export type AppError = {
  errorCode: string;
  message: string;
};

/**
 * Tauri IPC 返回的原始渠道载荷。
 */
type RawChannel = {
  id: string;
  name: string;
  channel_type: string;
  base_url: string;
  api_key: string | null;
  auth_type: string | null;
  models_endpoint: string | null;
  chat_endpoint: string | null;
  stream_endpoint: string | null;
  enabled: boolean;
  created_at: number;
  updated_at: number;
};

/**
 * Tauri IPC 返回的原始错误载荷。
 */
type RawError = {
  error_code?: string;
  message?: string;
};

/**
 * 将后端 snake_case 渠道对象转换为前端 camelCase 模型。
 */
function fromRawChannel(raw: RawChannel): Channel {
  return {
    id: raw.id,
    name: raw.name,
    channelType: raw.channel_type,
    baseUrl: raw.base_url,
    apiKey: raw.api_key,
    authType: raw.auth_type,
    modelsEndpoint: raw.models_endpoint,
    chatEndpoint: raw.chat_endpoint,
    streamEndpoint: raw.stream_endpoint,
    enabled: raw.enabled,
    createdAt: raw.created_at,
    updatedAt: raw.updated_at
  };
}

/**
 * 将前端输入转换为 Tauri 命令使用的 snake_case 载荷。
 */
function toRawInput(input: ChannelInput | ChannelPatch) {
  return {
    name: input.name,
    base_url: input.baseUrl,
    channel_type: input.channelType ?? undefined,
    api_key: input.apiKey ?? undefined,
    auth_type: input.authType ?? undefined,
    models_endpoint: input.modelsEndpoint ?? undefined,
    chat_endpoint: input.chatEndpoint ?? undefined,
    stream_endpoint: input.streamEndpoint ?? undefined,
    enabled: input.enabled ?? undefined
  };
}

/**
 * 将未知错误归一化为前端统一错误结构。
 */
export function toAppError(error: unknown): AppError {
  const fallback: AppError = {
    errorCode: "INTERNAL_ERROR",
    message: "unexpected client error"
  };

  if (!error || typeof error !== "object") {
    return fallback;
  }

  const raw = error as RawError;
  return {
    errorCode: raw.error_code ?? fallback.errorCode,
    message: raw.message ?? fallback.message
  };
}

/**
 * 获取渠道列表。
 */
export async function listChannels(includeDisabled = true): Promise<Channel[]> {
  const channels = await invoke<RawChannel[]>("list_channels", {
    includeDisabled
  });
  return channels.map(fromRawChannel);
}

/**
 * 创建渠道。
 */
export async function createChannel(input: ChannelInput): Promise<Channel> {
  const channel = await invoke<RawChannel>("create_channel", {
    input: toRawInput(input)
  });
  return fromRawChannel(channel);
}

/**
 * 更新渠道。
 */
export async function updateChannel(id: string, input: ChannelPatch): Promise<Channel> {
  const channel = await invoke<RawChannel>("update_channel", {
    id,
    input: toRawInput(input)
  });
  return fromRawChannel(channel);
}

/**
 * 删除渠道。
 */
export async function deleteChannel(id: string): Promise<void> {
  await invoke("delete_channel", { id });
}

/**
 * 测试渠道连通性。
 */
export async function testChannel(id: string): Promise<ChannelTestResult> {
  return invoke<ChannelTestResult>("test_channel", { id });
}
