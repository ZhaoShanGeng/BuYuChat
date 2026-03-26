/**
 * 消息 transport 的编解码与事件通道辅助函数。
 *
 * 这里集中处理：
 * 1. snake_case 原始载荷到 camelCase 前端模型的转换。
 * 2. send_message / reroll 输入的序列化。
 * 3. Tauri Channel 生成事件的封装。
 */

import { Channel } from "@tauri-apps/api/core";
import { toOptionalValue } from "./common";
import type {
  DeleteVersionResult,
  EditMessageInput,
  EditMessageResult,
  GenerationEvent,
  MessageNode,
  MessageVersion,
  RawDeleteVersionResult,
  RawDryRunResult,
  RawEditMessageResult,
  RawGenerationEvent,
  RawMessageNode,
  RawMessageVersion,
  RawRerollResult,
  RawStartedResult,
  RawVersionContent,
  RerollInput,
  RerollResult,
  SendMessageInput,
  SendMessageResponse,
  VersionContent
} from "./message-types";

/**
 * 将原始消息版本转换为前端结构。
 */
export function fromRawMessageVersion(raw: RawMessageVersion): MessageVersion {
  return {
    id: raw.id,
    nodeId: raw.node_id,
    content: raw.content,
    status: raw.status,
    modelName: raw.model_name,
    promptTokens: raw.prompt_tokens,
    completionTokens: raw.completion_tokens,
    finishReason: raw.finish_reason,
    createdAt: raw.created_at
  };
}

/**
 * 将原始消息楼层转换为前端结构。
 */
export function fromRawMessageNode(raw: RawMessageNode): MessageNode {
  return {
    id: raw.id,
    conversationId: raw.conversation_id,
    authorAgentId: raw.author_agent_id,
    role: raw.role,
    orderKey: raw.order_key,
    activeVersionId: raw.active_version_id,
    versions: raw.versions.map(fromRawMessageVersion),
    createdAt: raw.created_at
  };
}

/**
 * 将原始版本正文转换为前端结构。
 */
export function fromRawVersionContent(raw: RawVersionContent): VersionContent {
  return {
    versionId: raw.version_id,
    content: raw.content,
    contentType: raw.content_type
  };
}

/**
 * 将原始生成事件转换为前端事件结构。
 */
export function fromRawGenerationEvent(raw: RawGenerationEvent): GenerationEvent {
  switch (raw.type) {
    case "chunk":
      return {
        type: raw.type,
        conversationId: raw.conversation_id,
        nodeId: raw.node_id,
        versionId: raw.version_id,
        delta: raw.delta
      };
    case "completed":
      return {
        type: raw.type,
        conversationId: raw.conversation_id,
        nodeId: raw.node_id,
        versionId: raw.version_id,
        promptTokens: raw.prompt_tokens,
        completionTokens: raw.completion_tokens,
        finishReason: raw.finish_reason,
        model: raw.model
      };
    case "failed":
      return {
        type: raw.type,
        conversationId: raw.conversation_id,
        nodeId: raw.node_id,
        versionId: raw.version_id,
        error: raw.error
      };
    case "cancelled":
      return {
        type: raw.type,
        conversationId: raw.conversation_id,
        nodeId: raw.node_id,
        versionId: raw.version_id
      };
    case "empty_rollback":
      return {
        type: raw.type,
        conversationId: raw.conversation_id,
        nodeId: raw.node_id,
        nodeDeleted: raw.node_deleted,
        fallbackVersionId: raw.fallback_version_id
      };
  }
}

/**
 * 把前端发送消息输入转换为后端载荷。
 */
export function toRawSendMessageInput(input: SendMessageInput) {
  return {
    content: input.content,
    stream: toOptionalValue(input.stream),
    dry_run: toOptionalValue(input.dryRun)
  };
}

/**
 * 把前端 reroll 输入转换为后端载荷。
 */
export function toRawRerollInput(input?: RerollInput) {
  return input
    ? {
        stream: toOptionalValue(input.stream)
      }
    : undefined;
}

/**
 * 把前端编辑消息输入转换为后端载荷。
 */
export function toRawEditMessageInput(input: EditMessageInput) {
  return {
    content: input.content,
    resend: toOptionalValue(input.resend),
    stream: toOptionalValue(input.stream)
  };
}

/**
 * 创建一个用于接收后端生成事件的 Tauri Channel。
 */
export function createGenerationChannel(
  onEvent?: (event: GenerationEvent) => void
): Channel<RawGenerationEvent> {
  return new Channel<RawGenerationEvent>((event) => {
    onEvent?.(fromRawGenerationEvent(event));
  });
}

/**
 * 将原始 `send_message` 返回值转换为前端联合结果。
 */
export function fromRawSendMessageResponse(
  raw: RawDryRunResult | RawStartedResult
): SendMessageResponse {
  if ("messages" in raw) {
    return {
      kind: "dryRun",
      messages: raw.messages,
      totalTokensEstimate: raw.total_tokens_estimate,
      model: raw.model
    };
  }

  return {
    kind: "started",
    userNodeId: raw.user_node_id,
    userVersionId: raw.user_version_id,
    assistantNodeId: raw.assistant_node_id,
    assistantVersionId: raw.assistant_version_id
  };
}

/**
 * 将原始删除版本结果转换为前端结构。
 */
export function fromRawDeleteVersionResult(raw: RawDeleteVersionResult): DeleteVersionResult {
  return {
    nodeDeleted: raw.node_deleted,
    newActiveVersionId: raw.new_active_version_id
  };
}

/**
 * 将原始 reroll 结果转换为前端结构。
 */
export function fromRawRerollResult(raw: RawRerollResult): RerollResult {
  return {
    newUserVersionId: raw.new_user_version_id,
    assistantNodeId: raw.assistant_node_id,
    assistantVersionId: raw.assistant_version_id
  };
}

/**
 * 将原始编辑消息结果转换为前端结构。
 */
export function fromRawEditMessageResult(raw: RawEditMessageResult): EditMessageResult {
  return {
    editedVersionId: raw.edited_version_id,
    assistantNodeId: raw.assistant_node_id,
    assistantVersionId: raw.assistant_version_id
  };
}
