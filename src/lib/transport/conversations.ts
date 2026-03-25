/**
 * 会话管理相关的 Tauri transport 封装。
 */

import { invoke } from "@tauri-apps/api/core";
import { toOptionalValue, type AppError } from "./common";
import { toAppError } from "./common";

/**
 * 会话详情资源。
 */
export type Conversation = {
  id: string;
  title: string;
  agentId: string | null;
  channelId: string | null;
  channelModelId: string | null;
  archived: boolean;
  pinned: boolean;
  createdAt: number;
  updatedAt: number;
};

/**
 * 会话列表项。
 */
export type ConversationSummary = {
  id: string;
  title: string;
  agentId: string | null;
  channelId: string | null;
  channelModelId: string | null;
  archived: boolean;
  pinned: boolean;
  updatedAt: number;
};

/**
 * 会话创建输入。
 */
export type ConversationInput = {
  title?: string | null;
  agentId?: string | null;
  channelId?: string | null;
  channelModelId?: string | null;
};

/**
 * 会话更新补丁。
 */
export type ConversationPatch = {
  title?: string;
  agentId?: string | null;
  channelId?: string | null;
  channelModelId?: string | null;
  archived?: boolean;
  pinned?: boolean;
};

/**
 * Tauri 返回的原始会话详情。
 */
type RawConversation = {
  id: string;
  title: string;
  agent_id: string | null;
  channel_id: string | null;
  channel_model_id: string | null;
  archived: boolean;
  pinned: boolean;
  created_at: number;
  updated_at: number;
};

/**
 * Tauri 返回的原始会话列表项。
 */
type RawConversationSummary = {
  id: string;
  title: string;
  agent_id: string | null;
  channel_id: string | null;
  channel_model_id: string | null;
  archived: boolean;
  pinned: boolean;
  updated_at: number;
};

/**
 * 将原始会话详情转换为前端结构。
 */
function fromRawConversation(raw: RawConversation): Conversation {
  return {
    id: raw.id,
    title: raw.title,
    agentId: raw.agent_id,
    channelId: raw.channel_id,
    channelModelId: raw.channel_model_id,
    archived: raw.archived,
    pinned: raw.pinned,
    createdAt: raw.created_at,
    updatedAt: raw.updated_at
  };
}

/**
 * 将原始会话列表项转换为前端结构。
 */
function fromRawConversationSummary(raw: RawConversationSummary): ConversationSummary {
  return {
    id: raw.id,
    title: raw.title,
    agentId: raw.agent_id,
    channelId: raw.channel_id,
    channelModelId: raw.channel_model_id,
    archived: raw.archived,
    pinned: raw.pinned,
    updatedAt: raw.updated_at
  };
}

/**
 * 将会话创建输入转换为后端载荷。
 */
function toRawCreateInput(input: ConversationInput) {
  return {
    title: toOptionalValue(input.title),
    agent_id: toOptionalValue(input.agentId),
    channel_id: toOptionalValue(input.channelId),
    channel_model_id: toOptionalValue(input.channelModelId),
    archived: undefined,
    pinned: undefined
  };
}

/**
 * 将会话更新补丁转换为后端载荷。
 */
function toRawPatch(input: ConversationPatch) {
  return {
    title: toOptionalValue(input.title),
    agent_id: toOptionalValue(input.agentId),
    channel_id: toOptionalValue(input.channelId),
    channel_model_id: toOptionalValue(input.channelModelId),
    archived: toOptionalValue(input.archived),
    pinned: toOptionalValue(input.pinned)
  };
}

/**
 * 获取会话列表。
 */
export async function listConversations(archived = false): Promise<ConversationSummary[]> {
  const conversations = await invoke<RawConversationSummary[]>("list_conversations", { archived });
  return conversations.map(fromRawConversationSummary);
}

/**
 * 获取单个会话详情。
 */
export async function getConversation(id: string): Promise<Conversation> {
  const conversation = await invoke<RawConversation>("get_conversation", { id });
  return fromRawConversation(conversation);
}

/**
 * 创建会话。
 */
export async function createConversation(input: ConversationInput = {}): Promise<Conversation> {
  const conversation = await invoke<RawConversation>("create_conversation", {
    input: Object.keys(input).length > 0 ? toRawCreateInput(input) : undefined
  });
  return fromRawConversation(conversation);
}

/**
 * 更新会话。
 */
export async function updateConversation(
  id: string,
  input: ConversationPatch
): Promise<Conversation> {
  const conversation = await invoke<RawConversation>("update_conversation", {
    id,
    input: toRawPatch(input)
  });
  return fromRawConversation(conversation);
}

/**
 * 删除会话。
 */
export async function deleteConversation(id: string): Promise<void> {
  await invoke("delete_conversation", { id });
}

export { toAppError, type AppError };
