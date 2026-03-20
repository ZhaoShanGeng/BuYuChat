import { tauriInvoke } from "$lib/api/client";

export type ConversationSummary = {
  id: string;
  title: string;
  description: string | null;
  conversation_mode: string;
  archived: boolean;
  pinned: boolean;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type ConversationParticipantDetail = {
  id: string;
  conversation_id: string;
  agent_id: string | null;
  display_name: string | null;
  participant_type: string;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type ResourceBindingDetail = {
  id: string;
  resource_id: string;
  binding_type: string;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type ChannelBindingDetail = {
  id: string;
  channel_id: string;
  channel_model_id: string | null;
  binding_type: string;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type ConversationDetail = {
  summary: ConversationSummary;
  participants: ConversationParticipantDetail[];
  preset_bindings: ResourceBindingDetail[];
  lorebook_bindings: ResourceBindingDetail[];
  user_profile_bindings: ResourceBindingDetail[];
  channel_bindings: ChannelBindingDetail[];
};

// ─── Queries ───

export function listConversations() {
  return tauriInvoke<ConversationSummary[]>("list_conversations");
}

export function getConversationDetail(id: string) {
  return tauriInvoke<ConversationDetail>("get_conversation_detail", { id });
}

// ─── Mutations ───

export type CreateConversationInput = {
  title: string;
  description?: string | null;
  conversation_mode?: string;
};

export function createConversation(input: CreateConversationInput) {
  return tauriInvoke<ConversationDetail>("create_conversation", { input });
}

export function renameConversation(id: string, title: string) {
  return tauriInvoke<ConversationDetail>("rename_conversation", { id, title });
}

export function deleteConversation(id: string) {
  return tauriInvoke<void>("delete_conversation", { id });
}
