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

export function listVisibleMessages(conversationId: string) {
  return tauriInvoke<MessageVersionView[]>("list_visible_messages", {
    conversationId
  });
}
