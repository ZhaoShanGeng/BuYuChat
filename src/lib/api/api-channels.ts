import { tauriInvoke } from "$lib/api/client";

export type ApiChannel = {
  id: string;
  name: string;
  channel_type: string;
  base_url: string;
  auth_type: string;
  api_key: string | null;
  models_endpoint: string | null;
  chat_endpoint: string | null;
  stream_endpoint: string | null;
  models_mode: string;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type ApiChannelModel = {
  id: string;
  channel_id: string;
  model_id: string;
  display_name: string | null;
  model_type: string | null;
  context_window: number | null;
  max_output_tokens: number | null;
  capabilities_json: Record<string, unknown>;
  pricing_json: Record<string, unknown>;
  default_parameters_json: Record<string, unknown>;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type CreateApiChannelInput = {
  name: string;
  channel_type: string;
  base_url: string;
  auth_type: string;
  api_key: string | null;
  models_endpoint: string | null;
  chat_endpoint: string | null;
  stream_endpoint: string | null;
  models_mode: string;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type UpdateApiChannelInput = CreateApiChannelInput;

export type UpsertApiChannelModelInput = {
  channel_id: string;
  model_id: string;
  display_name: string | null;
  model_type: string | null;
  context_window: number | null;
  max_output_tokens: number | null;
  capabilities_json: Record<string, unknown>;
  pricing_json: Record<string, unknown>;
  default_parameters_json: Record<string, unknown>;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type ApiChannelTestResponse = {
  model_id: string;
  response_text: string;
};

export function listApiChannels() {
  return tauriInvoke<ApiChannel[]>("list_api_channels");
}

export function getApiChannel(id: string) {
  return tauriInvoke<ApiChannel>("get_api_channel", { id });
}

export function listApiChannelModels(channelId: string) {
  return tauriInvoke<ApiChannelModel[]>("list_api_channel_models", { channelId });
}

export function fetchApiChannelRemoteModels(channelId: string) {
  return tauriInvoke<ApiChannelModel[]>("fetch_api_channel_remote_models", { channelId });
}

export function createApiChannel(input: CreateApiChannelInput) {
  return tauriInvoke<ApiChannel>("create_api_channel", { input });
}

export function updateApiChannel(id: string, input: UpdateApiChannelInput) {
  return tauriInvoke<ApiChannel>("update_api_channel", { id, input });
}

export function deleteApiChannel(id: string) {
  return tauriInvoke<void>("delete_api_channel", { id });
}

export function upsertApiChannelModel(input: UpsertApiChannelModelInput) {
  return tauriInvoke<ApiChannelModel>("upsert_api_channel_model", { input });
}

export function deleteApiChannelModel(channelId: string, modelId: string) {
  return tauriInvoke<void>("delete_api_channel_model", { channelId, modelId });
}

export function refreshApiChannelModels(channelId: string) {
  return tauriInvoke<ApiChannelModel[]>("refresh_api_channel_models", { channelId });
}

export function testApiChannelMessage(channelId: string, modelId: string) {
  return tauriInvoke<ApiChannelTestResponse>("test_api_channel_message", { channelId, modelId });
}
