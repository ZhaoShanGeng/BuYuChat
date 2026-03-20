import { tauriInvoke } from "$lib/api/client";

export type StoredContent = {
  content_id: string;
  content_type: string;
  storage_kind: string;
  mime_type: string | null;
  size_bytes: number;
  preview_text: string | null;
  primary_storage_uri: string | null;
  text_content: string | null;
  chunk_count: number;
  sha256: string | null;
  config_json: Record<string, unknown>;
};

export type MessageContentRefView = {
  ref_id: string;
  ref_role: string;
  plugin_id: string | null;
  sort_order: number;
  content: StoredContent;
  config_json: Record<string, unknown>;
};

export type MessageVersionView = {
  node_id: string;
  version_id: string;
  conversation_id: string;
  author_participant_id: string;
  role: "system" | "user" | "assistant" | "tool";
  reply_to_node_id: string | null;
  order_key: string;
  version_index: number;
  is_active: boolean;
  primary_content: StoredContent;
  content_refs: MessageContentRefView[];
  context_policy: string;
  viewer_policy: string;
  api_channel_id: string | null;
  api_channel_model_id: string | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  total_tokens: number | null;
  finish_reason: string | null;
  generation_run_id: string | null;
  created_at: number;
};

// ─── Queries ───

export function listVisibleMessages(conversationId: string) {
  return tauriInvoke<MessageVersionView[]>("list_visible_messages", {
    conversationId
  });
}

export function listMessageVersions(nodeId: string) {
  return tauriInvoke<MessageVersionView[]>("list_message_versions", {
    nodeId
  });
}

export function getMessageBody(versionId: string) {
  return tauriInvoke<StoredContent>("get_message_body", { versionId });
}

// ─── Mutations ───

export type CreateMessageInput = {
  conversation_id: string;
  text: string;
  reply_to_node_id?: string | null;
};

export function createUserMessage(input: CreateMessageInput) {
  return tauriInvoke<MessageVersionView>("create_user_message", { input });
}

export function createSystemMessage(input: CreateMessageInput) {
  return tauriInvoke<MessageVersionView>("create_system_message", { input });
}

export type GenerateReplyInput = {
  conversation_id: string;
  reply_to_node_id?: string | null;
};

export type GenerateReplyStreamInput = {
  request: GenerateReplyInput;
  stream_id: string;
};

export function generateReplyStream(input: GenerateReplyStreamInput) {
  return tauriInvoke<MessageVersionView>("generate_reply_stream", { input });
}

export type RegenerateReplyInput = {
  node_id: string;
};

export type RegenerateReplyStreamInput = {
  request: RegenerateReplyInput;
  stream_id: string;
};

export function regenerateReplyStream(input: RegenerateReplyStreamInput) {
  return tauriInvoke<MessageVersionView>("regenerate_reply_stream", { input });
}

export type EditMessageVersionInput = {
  node_id: string;
  version_id: string;
  text: string;
};

export function editMessageVersion(input: EditMessageVersionInput) {
  return tauriInvoke<MessageVersionView>("edit_message_version", { input });
}

export function switchMessageVersion(nodeId: string, versionId: string) {
  return tauriInvoke<MessageVersionView>("switch_message_version", {
    nodeId,
    versionId
  });
}

export function deleteMessageVersion(nodeId: string, versionId: string) {
  return tauriInvoke<void>("delete_message_version", { nodeId, versionId });
}

export function deleteMessageNode(nodeId: string) {
  return tauriInvoke<void>("delete_message_node", { nodeId });
}
