import { invoke } from "@tauri-apps/api/core";

export interface ConversationRow {
  id: string;
  title: string;
  model_id: string;
  provider: string;
  assistant_id: string | null;
  system_prompt: string | null;
  pinned: boolean;
  created_at: number;
  updated_at: number;
}

export interface MessageRow {
  id: string;
  conversation_id: string;
  parent_message_id: string | null;
  version_group_id: string;
  version_index: number;
  is_active: boolean;
  role: string;
  content: string | null;
  content_parts: string | null;
  tool_calls: string | null;
  tool_call_id: string | null;
  citations_json: string | null;
  tokens_used: number | null;
  provider: string | null;
  model_id: string | null;
  created_at: number;
}

export interface ProviderConfigRow {
  id: string;
  provider: string;
  api_key_id: string | null;
  base_url: string | null;
  extra_json: string | null;
  enabled: boolean;
  updated_at: number;
}

export interface CustomChannelRow {
  id: string;
  name: string;
  channel_type: string;
  base_url: string;
  auth_json: string;
  endpoints_json: string;
  stream_protocol: string;
  request_template_json: string;
  response_mapping_json: string;
  stream_mapping_json: string;
  models_json: string;
  enabled: boolean;
  created_at: number;
  updated_at: number;
}

export interface ModelInfo {
  id: string;
  name: string;
  context_length: number | null;
  supports_vision: boolean;
  supports_function_calling: boolean;
}

export interface PageResponse<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
}

export interface TestConnectionResponse {
  ok: boolean;
  message: string;
}

export interface SendMessageResponse {
  user_msg_id: string;
  assistant_msg_id: string;
}

export interface RegenerateMessageResponse {
  assistant_msg_id: string;
}

export interface SaveMessageEditResponse {
  message_id: string;
}

export interface MessageBundleResponse {
  active_messages: MessageRow[];
  versions_by_group: Record<string, MessageRow[]>;
}

export async function listProviderConfigs() {
  return invoke<ProviderConfigRow[]>("list_provider_configs");
}

export async function listCustomChannels() {
  return invoke<CustomChannelRow[]>("list_custom_channels");
}

export async function getProviderApiKey(provider: string) {
  return invoke<string | null>("get_provider_api_key", { provider });
}

export async function getCustomChannelApiKey(id: string) {
  return invoke<string | null>("get_custom_channel_api_key", { id });
}

export async function saveProviderConfig(input: {
  provider: string;
  apiKey?: string | null;
  baseUrl?: string;
}) {
  return invoke<void>("save_provider_config", {
    provider: input.provider,
    apiKey: input.apiKey ?? null,
    baseUrl: input.baseUrl ?? null,
  });
}

export async function testProviderConnection(provider: string) {
  return invoke<TestConnectionResponse>("test_provider_connection", { provider });
}

export async function createCustomChannel(input: {
  name: string;
  channelType: string;
  baseUrl: string;
  modelsPath: string;
  chatPath: string;
  streamPath: string;
  apiKey?: string | null;
}) {
  return invoke<CustomChannelRow>("create_custom_channel", {
    name: input.name,
    channelType: input.channelType,
    baseUrl: input.baseUrl,
    modelsPath: input.modelsPath,
    chatPath: input.chatPath,
    streamPath: input.streamPath,
    apiKey: input.apiKey ?? null,
  });
}

export async function updateCustomChannel(input: {
  id: string;
  name: string;
  channelType: string;
  baseUrl: string;
  modelsPath: string;
  chatPath: string;
  streamPath: string;
  apiKey?: string | null;
}) {
  return invoke<CustomChannelRow>("update_custom_channel", {
    id: input.id,
    name: input.name,
    channelType: input.channelType,
    baseUrl: input.baseUrl,
    modelsPath: input.modelsPath,
    chatPath: input.chatPath,
    streamPath: input.streamPath,
    apiKey: input.apiKey ?? null,
  });
}

export async function deleteCustomChannel(id: string) {
  return invoke<void>("delete_custom_channel", { id });
}

export async function listModels(provider: string) {
  return invoke<ModelInfo[]>("list_models", { provider });
}

export async function listConversations(page = 1, perPage = 50) {
  return invoke<PageResponse<ConversationRow>>("list_conversations", {
    page,
    perPage,
  });
}

export async function createConversation(input: {
  modelId: string;
  provider: string;
}) {
  return invoke<ConversationRow>("create_conversation", {
    modelId: input.modelId,
    provider: input.provider,
    assistantId: null,
  });
}

export async function forkConversationFromMessage(input: {
  convId: string;
  messageId: string;
}) {
  return invoke<ConversationRow>("fork_conversation_from_message", {
    convId: input.convId,
    messageId: input.messageId,
  });
}

export async function updateConversationModel(input: {
  id: string;
  modelId: string;
  provider: string;
}) {
  return invoke<void>("update_conversation_model", {
    id: input.id,
    modelId: input.modelId,
    provider: input.provider,
  });
}

export async function updateConversationTitle(input: {
  id: string;
  title: string;
}) {
  return invoke<void>("update_conversation_title", {
    id: input.id,
    title: input.title,
  });
}

export async function deleteConversation(id: string) {
  return invoke<void>("delete_conversation", { id });
}

export async function updateConversationSystemPrompt(input: {
  id: string;
  systemPrompt: string | null;
}) {
  return invoke<void>("update_conversation_system_prompt", {
    id: input.id,
    systemPrompt: input.systemPrompt,
  });
}

export async function listMessages(convId: string) {
  return invoke<MessageRow[]>("list_messages", { convId });
}

export async function listMessageBundle(convId: string) {
  return invoke<MessageBundleResponse>("list_message_bundle", { convId });
}

export async function getMessageVersions(versionGroupId: string) {
  return invoke<MessageRow[]>("get_message_versions", { versionGroupId });
}

export async function switchMessageVersion(input: {
  versionGroupId: string;
  targetIndex: number;
}) {
  return invoke<MessageRow>("switch_message_version", {
    versionGroupId: input.versionGroupId,
    targetIndex: input.targetIndex,
  });
}

export async function deleteMessage(input: { convId: string; messageId: string }) {
  return invoke<void>("delete_message", {
    convId: input.convId,
    messageId: input.messageId,
  });
}

export async function sendMessage(input: {
  convId: string;
  content: string;
  overrideModel?: string;
}) {
  return invoke<SendMessageResponse>("send_message", {
    convId: input.convId,
    content: input.content,
    overrideModel: input.overrideModel ?? null,
  });
}

export async function regenerateMessage(input: {
  convId: string;
  messageId?: string | null;
}) {
  return invoke<RegenerateMessageResponse>("regenerate_message", {
    convId: input.convId,
    messageId: input.messageId ?? null,
  });
}

export async function editUserMessage(input: {
  convId: string;
  messageId: string;
  newContent: string;
}) {
  return invoke<SendMessageResponse>("edit_user_message", {
    convId: input.convId,
    messageId: input.messageId,
    newContent: input.newContent,
  });
}

export async function saveMessageEdit(input: {
  convId: string;
  messageId: string;
  newContent: string;
}) {
  return invoke<SaveMessageEditResponse>("save_message_edit", {
    convId: input.convId,
    messageId: input.messageId,
    newContent: input.newContent,
  });
}

export async function refreshCustomChannelModels(id: string) {
  return invoke<ModelInfo[]>("refresh_custom_channel_models", { id });
}

export async function saveCustomChannelModels(input: {
  id: string;
  models: ModelInfo[];
}) {
  return invoke<void>("save_custom_channel_models", {
    id: input.id,
    models: input.models,
  });
}
