/**
 * 消息 transport 的公开类型与原始载荷定义。
 *
 * 这个文件只负责声明：
 * 1. 前端消费的消息、版本、事件联合类型。
 * 2. `invoke()` 返回的 snake_case 原始载荷结构。
 * 3. 供 codec / invoke 层共享的最小输入输出类型。
 */

import type { ErrorDetails, RawErrorDetails } from "./common";

/**
 * 前端使用的 prompt 消息结构。
 */
export type PromptMessage = {
  role: string;
  content: string;
  images: ImageAttachment[];
  files?: FileAttachment[];
  toolCalls?: ToolCallRecord[];
  toolResults?: ToolResultRecord[];
};

export type ImageAttachment = {
  base64: string;
  mimeType: string;
  url?: string | null;
};

export type FileAttachment = {
  name: string;
  base64: string;
  mimeType: string;
};

export type ToolCallRecord = {
  id: string;
  name: string;
  argumentsJson: string;
};

export type ToolResultRecord = {
  toolCallId: string;
  name: string;
  content: string;
  isError: boolean;
};

export type ToolCallDelta = {
  id?: string;
  name?: string;
  argumentsDelta: string;
  index: number;
};

/**
 * 消息版本。
 */
export type MessageVersion = {
  id: string;
  nodeId: string;
  content: string | null;
  thinkingContent: string | null;
  images: ImageAttachment[];
  files?: FileAttachment[];
  toolCalls?: ToolCallRecord[];
  toolResults?: ToolResultRecord[];
  status: "generating" | "committed" | "failed" | "cancelled";
  errorCode?: string | null;
  errorMessage?: string | null;
  errorDetails?: ErrorDetails | null;
  modelName: string | null;
  promptTokens: number | null;
  completionTokens: number | null;
  finishReason: string | null;
  receivedAt: number | null;
  completedAt: number | null;
  createdAt: number;
};

/**
 * 消息楼层。
 */
export type MessageNode = {
  id: string;
  conversationId: string;
  authorAgentId: string | null;
  role: "user" | "assistant";
  orderKey: string;
  activeVersionId: string | null;
  versions: MessageVersion[];
  createdAt: number;
};

/**
 * 版本内容。
 */
export type VersionContent = {
  versionId: string;
  content: string;
  contentType: string;
};

/**
 * 发送消息输入。
 */
export type SendMessageInput = {
  content: string;
  images?: ImageAttachment[];
  files?: FileAttachment[];
  toolResults?: ToolResultRecord[];
  stream?: boolean;
  dryRun?: boolean;
};

/**
 * dry run 返回结果。
 */
export type DryRunResult = {
  kind: "dryRun";
  messages: PromptMessage[];
  totalTokensEstimate: number;
  model: string;
};

/**
 * 正常发送后的即时返回结果。
 */
export type SendMessageResult = {
  kind: "started";
  userNodeId: string;
  userVersionId: string;
  assistantNodeId: string;
  assistantVersionId: string;
};

/**
 * `send_message` 的前端联合返回值。
 */
export type SendMessageResponse = DryRunResult | SendMessageResult;

/**
 * Reroll 输入。
 */
export type RerollInput = {
  stream?: boolean;
};

/**
 * 编辑消息输入。
 */
export type EditMessageInput = {
  content: string;
  resend?: boolean;
  stream?: boolean;
};

/**
 * Reroll 返回值。
 */
export type RerollResult = {
  newUserVersionId: string | null;
  assistantNodeId: string;
  assistantVersionId: string;
};

/**
 * 编辑消息返回值。
 */
export type EditMessageResult = {
  editedVersionId: string;
  assistantNodeId: string | null;
  assistantVersionId: string | null;
};

/**
 * 删除版本结果。
 */
export type DeleteVersionResult = {
  nodeDeleted: boolean;
  newActiveVersionId: string | null;
};

/**
 * 前端消费的生成事件联合类型。
 */
export type GenerationEvent =
  | {
      type: "chunk";
      conversationId: string;
      nodeId: string;
      versionId: string;
      delta: string;
      reasoningDelta?: string;
      toolCallDeltas?: ToolCallDelta[];
    }
  | {
      type: "completed";
      conversationId: string;
      nodeId: string;
      versionId: string;
      promptTokens: number;
      completionTokens: number;
      finishReason: string;
      model: string;
      receivedAt: number;
      completedAt: number;
    }
  | {
      type: "failed";
      conversationId: string;
      nodeId: string;
      versionId: string;
      errorCode: string;
      errorMessage: string;
      errorDetails?: ErrorDetails | null;
    }
  | {
      type: "cancelled";
      conversationId: string;
      nodeId: string;
      versionId: string;
    }
  | {
      type: "empty_rollback";
      conversationId: string;
      nodeId: string;
      nodeDeleted: boolean;
      fallbackVersionId: string | null;
    }
  | {
      type: "tool_call_start";
      conversationId: string;
      nodeId: string;
      versionId: string;
      toolCalls: ToolCallRecord[];
    }
  | {
      type: "tool_result";
      conversationId: string;
      nodeId: string;
      versionId: string;
      results: ToolResultRecord[];
    };

/**
 * 后端返回的原始消息版本载荷。
 */
export type RawMessageVersion = {
  id: string;
  node_id: string;
  content: string | null;
  thinking_content?: string | null;
  images?: RawImageAttachment[];
  files?: RawFileAttachment[];
  tool_calls?: RawToolCallRecord[];
  tool_results?: RawToolResultRecord[];
  status: "generating" | "committed" | "failed" | "cancelled";
  error_code?: string | null;
  error_message?: string | null;
  error_details?: RawErrorDetails | null;
  model_name: string | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  finish_reason: string | null;
  received_at?: number | null;
  completed_at?: number | null;
  created_at: number;
};

/**
 * 后端返回的原始消息楼层载荷。
 */
export type RawMessageNode = {
  id: string;
  conversation_id: string;
  author_agent_id: string | null;
  role: "user" | "assistant";
  order_key: string;
  active_version_id: string | null;
  versions: RawMessageVersion[];
  created_at: number;
};

/**
 * 后端返回的原始版本内容载荷。
 */
export type RawVersionContent = {
  version_id: string;
  content: string;
  content_type: string;
};

/**
 * 后端返回的原始 prompt 消息。
 */
export type RawPromptMessage = {
  role: string;
  content: string;
  images?: RawImageAttachment[];
  files?: RawFileAttachment[];
  tool_calls?: RawToolCallRecord[];
  tool_results?: RawToolResultRecord[];
};

export type RawImageAttachment = {
  base64: string;
  mime_type: string;
  url?: string | null;
};

export type RawFileAttachment = {
  name: string;
  base64: string;
  mime_type: string;
};

export type RawToolCallRecord = {
  id: string;
  name: string;
  arguments_json: string;
};

export type RawToolResultRecord = {
  tool_call_id: string;
  name: string;
  content: string;
  is_error: boolean;
};

export type RawToolCallDelta = {
  id?: string | null;
  name?: string | null;
  arguments_delta: string;
  index: number;
};

/**
 * 后端返回的原始 dry run 结果。
 */
export type RawDryRunResult = {
  messages: RawPromptMessage[];
  total_tokens_estimate: number;
  model: string;
};

/**
 * 后端返回的原始 started 结果。
 */
export type RawStartedResult = {
  user_node_id: string;
  user_version_id: string;
  assistant_node_id: string;
  assistant_version_id: string;
};

/**
 * 后端返回的原始 Reroll 结果。
 */
export type RawRerollResult = {
  new_user_version_id: string | null;
  assistant_node_id: string;
  assistant_version_id: string;
};

/**
 * 后端返回的原始编辑消息结果。
 */
export type RawEditMessageResult = {
  edited_version_id: string;
  assistant_node_id: string | null;
  assistant_version_id: string | null;
};

/**
 * 后端返回的原始删除版本结果。
 */
export type RawDeleteVersionResult = {
  node_deleted: boolean;
  new_active_version_id: string | null;
};

/**
 * 后端返回的原始生成事件。
 */
export type RawGenerationEvent =
  | {
      type: "chunk";
      conversation_id: string;
      node_id: string;
      version_id: string;
      delta: string;
      reasoning_delta?: string | null;
      tool_call_deltas?: RawToolCallDelta[];
    }
  | {
      type: "completed";
      conversation_id: string;
      node_id: string;
      version_id: string;
      prompt_tokens: number;
      completion_tokens: number;
      finish_reason: string;
      model: string;
      received_at: number;
      completed_at: number;
    }
  | {
      type: "failed";
      conversation_id: string;
      node_id: string;
      version_id: string;
      error_code: string;
      error_message: string;
      error_details?: RawErrorDetails | null;
    }
  | {
      type: "cancelled";
      conversation_id: string;
      node_id: string;
      version_id: string;
    }
  | {
      type: "empty_rollback";
      conversation_id: string;
      node_id: string;
      node_deleted: boolean;
      fallback_version_id: string | null;
    }
  | {
      type: "tool_call_start";
      conversation_id: string;
      node_id: string;
      version_id: string;
      tool_calls: RawToolCallRecord[];
    }
  | {
      type: "tool_result";
      conversation_id: string;
      node_id: string;
      version_id: string;
      results: RawToolResultRecord[];
    };
