/**
 * 渠道管理相关的 Tauri transport 封装。
 */

import { invoke } from "@tauri-apps/api/core";
import { toAppError, toOptionalValue, type AppError } from "./common";

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
    channel_type: toOptionalValue(input.channelType),
    api_key: toOptionalValue(input.apiKey),
    auth_type: toOptionalValue(input.authType),
    models_endpoint: toOptionalValue(input.modelsEndpoint),
    chat_endpoint: toOptionalValue(input.chatEndpoint),
    stream_endpoint: toOptionalValue(input.streamEndpoint),
    enabled: toOptionalValue(input.enabled)
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

export { toAppError, type AppError };
