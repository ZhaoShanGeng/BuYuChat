import { tauriInvoke } from "$lib/api/client";
import type { ChannelBindingDetail, ResourceBindingDetail } from "$lib/api/conversations";
import type { StoredContent } from "$lib/api/messages";

export type ContentWriteInput = {
  content_type: string;
  mime_type: string | null;
  text_content: string | null;
  source_file_path: string | null;
  primary_storage_uri: string | null;
  size_bytes_hint: number | null;
  preview_text: string | null;
  config_json: Record<string, unknown>;
};

export type AgentSummary = {
  id: string;
  name: string;
  title: string | null;
  avatar_uri: string | null;
  enabled: boolean;
  sort_order: number;
  created_at: number;
  updated_at: number;
};

export type AgentGreetingDetail = {
  id: string;
  agent_id: string;
  greeting_type: string;
  name: string | null;
  primary_content: StoredContent;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type AgentMediaDetail = {
  id: string;
  agent_id: string;
  media_type: string;
  name: string | null;
  content: StoredContent;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type AgentDetail = {
  summary: AgentSummary;
  description_content: StoredContent | null;
  personality_content: StoredContent | null;
  scenario_content: StoredContent | null;
  example_messages_content: StoredContent | null;
  main_prompt_override_content: StoredContent | null;
  post_history_instructions_content: StoredContent | null;
  character_note_content: StoredContent | null;
  creator_notes_content: StoredContent | null;
  character_note_depth: number | null;
  character_note_role: string | null;
  talkativeness: number;
  creator_name: string | null;
  character_version: string | null;
  greetings: AgentGreetingDetail[];
  media: AgentMediaDetail[];
  preset_bindings: ResourceBindingDetail[];
  lorebook_bindings: ResourceBindingDetail[];
  user_profile_bindings: ResourceBindingDetail[];
  channel_bindings: ChannelBindingDetail[];
  config_json: Record<string, unknown>;
};

export type CreateAgentInput = {
  name: string;
  title: string | null;
  description_content: ContentWriteInput | null;
  personality_content: ContentWriteInput | null;
  scenario_content: ContentWriteInput | null;
  example_messages_content: ContentWriteInput | null;
  main_prompt_override_content: ContentWriteInput | null;
  post_history_instructions_content: ContentWriteInput | null;
  character_note_content: ContentWriteInput | null;
  creator_notes_content: ContentWriteInput | null;
  character_note_depth: number | null;
  character_note_role: string | null;
  talkativeness: number;
  avatar_uri: string | null;
  creator_name: string | null;
  character_version: string | null;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type UpdateAgentInput = CreateAgentInput;

export type CreateAgentGreetingInput = {
  greeting_type: string;
  name: string | null;
  primary_content: ContentWriteInput;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type UpdateAgentGreetingInput = CreateAgentGreetingInput;

export type AddAgentMediaInput = {
  media_type: string;
  name: string | null;
  content: ContentWriteInput;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type AgentResourceBindingInput = {
  resource_id: string;
  binding_type: string;
  enabled: boolean;
  sort_order: number;
  config_json?: Record<string, unknown>;
};

export type AgentChannelBindingInput = {
  channel_id: string;
  channel_model_id?: string | null;
  binding_type: string;
  enabled: boolean;
  sort_order: number;
  config_json?: Record<string, unknown>;
};

export function listAgents() {
  return tauriInvoke<AgentSummary[]>("list_agents");
}

export function getAgentDetail(id: string) {
  return tauriInvoke<AgentDetail>("get_agent_detail", { id });
}

export function createAgent(input: CreateAgentInput) {
  return tauriInvoke<AgentDetail>("create_agent", { input });
}

export function updateAgent(id: string, input: UpdateAgentInput) {
  return tauriInvoke<AgentDetail>("update_agent", { id, input });
}

export function deleteAgent(id: string) {
  return tauriInvoke<void>("delete_agent", { id });
}

export function createAgentGreeting(agentId: string, input: CreateAgentGreetingInput) {
  return tauriInvoke<AgentGreetingDetail>("create_agent_greeting", { agentId, input });
}

export function updateAgentGreeting(greetingId: string, input: UpdateAgentGreetingInput) {
  return tauriInvoke<AgentGreetingDetail>("update_agent_greeting", { greetingId, input });
}

export function deleteAgentGreeting(greetingId: string) {
  return tauriInvoke<void>("delete_agent_greeting", { greetingId });
}

export function addAgentMedia(agentId: string, input: AddAgentMediaInput) {
  return tauriInvoke<AgentMediaDetail>("add_agent_media", {
    agentId,
    input: {
      media_type: input.media_type,
      name: input.name ?? null,
      content: input.content,
      enabled: input.enabled,
      sort_order: input.sort_order,
      config_json: input.config_json ?? {}
    }
  });
}

export function removeAgentMedia(mediaId: string) {
  return tauriInvoke<void>("remove_agent_media", { mediaId });
}

export function replaceAgentPresets(agentId: string, items: AgentResourceBindingInput[]) {
  return tauriInvoke<ResourceBindingDetail[]>("replace_agent_presets", {
    agentId,
    items: items.map((item) => ({
      resource_id: item.resource_id,
      binding_type: item.binding_type,
      enabled: item.enabled,
      sort_order: item.sort_order,
      config_json: item.config_json ?? {}
    }))
  });
}

export function replaceAgentLorebooks(agentId: string, items: AgentResourceBindingInput[]) {
  return tauriInvoke<ResourceBindingDetail[]>("replace_agent_lorebooks", {
    agentId,
    items: items.map((item) => ({
      resource_id: item.resource_id,
      binding_type: item.binding_type,
      enabled: item.enabled,
      sort_order: item.sort_order,
      config_json: item.config_json ?? {}
    }))
  });
}

export function replaceAgentUserProfiles(agentId: string, items: AgentResourceBindingInput[]) {
  return tauriInvoke<ResourceBindingDetail[]>("replace_agent_user_profiles", {
    agentId,
    items: items.map((item) => ({
      resource_id: item.resource_id,
      binding_type: item.binding_type,
      enabled: item.enabled,
      sort_order: item.sort_order,
      config_json: item.config_json ?? {}
    }))
  });
}

export function replaceAgentChannels(agentId: string, items: AgentChannelBindingInput[]) {
  return tauriInvoke<ChannelBindingDetail[]>("replace_agent_channels", {
    agentId,
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
