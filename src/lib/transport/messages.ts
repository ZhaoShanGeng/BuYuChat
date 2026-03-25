/**
 * 消息查询、版本控制与生成事件相关的 Tauri transport 封装。
 *
 * 这个入口文件只负责：
 * 1. 发起 Tauri `invoke()` 调用。
 * 2. 复用 codec 将 snake_case 载荷转为前端结构。
 * 3. 统一 re-export 消息相关公共类型，避免组件依赖内部拆分文件。
 */

import { invoke } from "@tauri-apps/api/core";
import { toAppError, toOptionalValue, type AppError } from "./common";
import {
  createGenerationChannel,
  fromRawDeleteVersionResult,
  fromRawMessageNode,
  fromRawRerollResult,
  fromRawSendMessageResponse,
  fromRawVersionContent,
  toRawRerollInput,
  toRawSendMessageInput
} from "./message-codecs";
import type {
  DeleteVersionResult,
  GenerationEvent,
  MessageNode,
  RawDeleteVersionResult,
  RawDryRunResult,
  RawMessageNode,
  RawRerollResult,
  RawStartedResult,
  RawVersionContent
} from "./message-types";
import type {
  RerollInput,
  RerollResult,
  SendMessageInput,
  SendMessageResponse,
  VersionContent
} from "./message-types";

/**
 * 获取会话消息列表。
 */
export async function listMessages(
  id: string,
  beforeOrderKey?: string | null,
  limit?: number
): Promise<MessageNode[]> {
  const messages = await invoke<RawMessageNode[]>("list_messages", {
    id,
    beforeOrderKey: toOptionalValue(beforeOrderKey),
    limit: toOptionalValue(limit)
  });
  return messages.map(fromRawMessageNode);
}

/**
 * 按需获取单个版本的完整正文。
 */
export async function getVersionContent(versionId: string): Promise<VersionContent> {
  const content = await invoke<RawVersionContent>("get_version_content", { versionId });
  return fromRawVersionContent(content);
}

/**
 * 切换某个楼层的 active version。
 */
export async function setActiveVersion(
  conversationId: string,
  nodeId: string,
  versionId: string
): Promise<void> {
  await invoke("set_active_version", {
    id: conversationId,
    nodeId,
    versionId
  });
}

/**
 * 删除指定版本。
 */
export async function deleteVersion(
  conversationId: string,
  nodeId: string,
  versionId: string
): Promise<DeleteVersionResult> {
  const result = await invoke<RawDeleteVersionResult>("delete_version", {
    id: conversationId,
    nodeId,
    versionId
  });
  return fromRawDeleteVersionResult(result);
}

/**
 * 发送消息并建立本次生成的事件通道。
 */
export async function sendMessage(
  id: string,
  input: SendMessageInput,
  onEvent?: (event: GenerationEvent) => void
): Promise<SendMessageResponse> {
  const eventChannel = createGenerationChannel(onEvent);
  const response = await invoke<RawDryRunResult | RawStartedResult>("send_message", {
    id,
    input: toRawSendMessageInput(input),
    eventChannel
  });
  return fromRawSendMessageResponse(response);
}

/**
 * 对指定楼层执行 reroll，并建立本次生成的事件通道。
 */
export async function reroll(
  id: string,
  nodeId: string,
  input?: RerollInput,
  onEvent?: (event: GenerationEvent) => void
): Promise<RerollResult> {
  const eventChannel = createGenerationChannel(onEvent);
  const result = await invoke<RawRerollResult>("reroll", {
    id,
    nodeId,
    input: toRawRerollInput(input),
    eventChannel
  });
  return fromRawRerollResult(result);
}

/**
 * 取消某个生成中的版本。
 */
export async function cancelGeneration(versionId: string): Promise<void> {
  await invoke("cancel_generation", { versionId });
}

export type {
  DeleteVersionResult,
  DryRunResult,
  GenerationEvent,
  MessageNode,
  MessageVersion,
  PromptMessage,
  RerollInput,
  RerollResult,
  SendMessageInput,
  SendMessageResponse,
  SendMessageResult,
  VersionContent
} from "./message-types";
export { toAppError, type AppError };
