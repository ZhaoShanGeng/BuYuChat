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

export type ChannelBindingInput = {
  channel_id: string;
  channel_model_id?: string | null;
  binding_type: string;
  enabled: boolean;
  sort_order: number;
  config_json?: Record<string, unknown>;
};

export type ResourceBindingInput = {
  resource_id: string;
  binding_type: string;
  enabled: boolean;
  sort_order: number;
  config_json?: Record<string, unknown>;
};

export type ConversationDetail = {
  summary: ConversationSummary;
  participants: ConversationParticipantDetail[];
  preset_bindings: ResourceBindingDetail[];
  lorebook_bindings: ResourceBindingDetail[];
  user_profile_bindings: ResourceBindingDetail[];
  channel_bindings: ChannelBindingDetail[];
};

export type ConversationParticipantInput = {
  agent_id?: string | null;
  display_name?: string | null;
  participant_type: string;
  enabled: boolean;
  sort_order: number;
  config_json?: Record<string, unknown>;
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
  archived?: boolean;
  pinned?: boolean;
  participants?: ConversationParticipantInput[];
  config_json?: Record<string, unknown>;
};

export function createConversation(input: CreateConversationInput) {
  return tauriInvoke<ConversationDetail>("create_conversation", {
    input: {
      title: input.title,
      description: input.description ?? null,
      conversation_mode: input.conversation_mode ?? "chat",
      archived: input.archived ?? false,
      pinned: input.pinned ?? false,
      participants:
        input.participants?.map((item) => ({
          agent_id: item.agent_id ?? null,
          display_name: item.display_name ?? null,
          participant_type: item.participant_type,
          enabled: item.enabled,
          sort_order: item.sort_order,
          config_json: item.config_json ?? {}
        })) ?? [],
      config_json: input.config_json ?? {}
    }
  });
}

export function renameConversation(id: string, title: string) {
  return tauriInvoke<ConversationDetail>("rename_conversation", { id, title });
}

export type UpdateConversationMetaInput = {
  title: string;
  description: string | null;
  archived: boolean;
  pinned: boolean;
  config_json: Record<string, unknown>;
};

export function updateConversationMeta(id: string, input: UpdateConversationMetaInput) {
  return tauriInvoke<ConversationDetail>("update_conversation_meta", {
    id,
    input: {
      title: input.title,
      description: input.description ?? null,
      archived: input.archived,
      pinned: input.pinned,
      config_json: input.config_json ?? {}
    }
  });
}

export function deleteConversation(id: string) {
  return tauriInvoke<void>("delete_conversation", { id });
}

export function replaceConversationParticipants(
  conversationId: string,
  items: ConversationParticipantInput[]
) {
  return tauriInvoke<ConversationParticipantDetail[]>("replace_conversation_participants", {
    conversationId,
    items: items.map((item) => ({
      agent_id: item.agent_id ?? null,
      display_name: item.display_name ?? null,
      participant_type: item.participant_type,
      enabled: item.enabled,
      sort_order: item.sort_order,
      config_json: item.config_json ?? {}
    }))
  });
}

export function replaceConversationPresets(
  conversationId: string,
  items: ResourceBindingInput[]
) {
  return tauriInvoke<ResourceBindingDetail[]>("replace_conversation_presets", {
    conversationId,
    items: items.map((item) => ({
      resource_id: item.resource_id,
      binding_type: item.binding_type,
      enabled: item.enabled,
      sort_order: item.sort_order,
      config_json: item.config_json ?? {}
    }))
  });
}

export function replaceConversationLorebooks(
  conversationId: string,
  items: ResourceBindingInput[]
) {
  return tauriInvoke<ResourceBindingDetail[]>("replace_conversation_lorebooks", {
    conversationId,
    items: items.map((item) => ({
      resource_id: item.resource_id,
      binding_type: item.binding_type,
      enabled: item.enabled,
      sort_order: item.sort_order,
      config_json: item.config_json ?? {}
    }))
  });
}

export function replaceConversationUserProfiles(
  conversationId: string,
  items: ResourceBindingInput[]
) {
  return tauriInvoke<ResourceBindingDetail[]>("replace_conversation_user_profiles", {
    conversationId,
    items: items.map((item) => ({
      resource_id: item.resource_id,
      binding_type: item.binding_type,
      enabled: item.enabled,
      sort_order: item.sort_order,
      config_json: item.config_json ?? {}
    }))
  });
}

export function replaceConversationChannels(
  conversationId: string,
  items: ChannelBindingInput[]
) {
  return tauriInvoke<ChannelBindingDetail[]>("replace_conversation_channels", {
    conversationId,
    items: items.map((item) => ({
      channel_id: item.channel_id,
      channel_model_id: item.channel_model_id ?? null,
      binding_type: item.binding_type,
      enabled: item.enabled,
      sort_order: item.sort_order,
      config_json: item.config_json ?? {}
    }))
  });
}
