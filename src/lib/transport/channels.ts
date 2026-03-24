import { invoke } from "@tauri-apps/api/core";

export type Channel = {
  id: string;
  name: string;
  channelType: string;
  baseUrl: string;
  apiKey: string | null;
  authType: string | null;
  modelsEndpoint: string | null;
  chatEndpoint: string | null;
  streamEndpoint: string | null;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
};

export type ChannelInput = {
  name: string;
  baseUrl: string;
  channelType?: string | null;
  apiKey?: string | null;
  authType?: string | null;
  modelsEndpoint?: string | null;
  chatEndpoint?: string | null;
  streamEndpoint?: string | null;
  enabled?: boolean | null;
};

export type ChannelPatch = Partial<ChannelInput>;

export type ChannelTestResult = {
  success: boolean;
  message: string | null;
};

export type AppError = {
  errorCode: string;
  message: string;
};

type RawChannel = {
  id: string;
  name: string;
  channel_type: string;
  base_url: string;
  api_key: string | null;
  auth_type: string | null;
  models_endpoint: string | null;
  chat_endpoint: string | null;
  stream_endpoint: string | null;
  enabled: boolean;
  created_at: number;
  updated_at: number;
};

type RawError = {
  error_code?: string;
  message?: string;
};

function fromRawChannel(raw: RawChannel): Channel {
  return {
    id: raw.id,
    name: raw.name,
    channelType: raw.channel_type,
    baseUrl: raw.base_url,
    apiKey: raw.api_key,
    authType: raw.auth_type,
    modelsEndpoint: raw.models_endpoint,
    chatEndpoint: raw.chat_endpoint,
    streamEndpoint: raw.stream_endpoint,
    enabled: raw.enabled,
    createdAt: raw.created_at,
    updatedAt: raw.updated_at
  };
}

function toRawInput(input: ChannelInput | ChannelPatch) {
  return {
    name: input.name,
    base_url: input.baseUrl,
    channel_type: input.channelType ?? undefined,
    api_key: input.apiKey ?? undefined,
    auth_type: input.authType ?? undefined,
    models_endpoint: input.modelsEndpoint ?? undefined,
    chat_endpoint: input.chatEndpoint ?? undefined,
    stream_endpoint: input.streamEndpoint ?? undefined,
    enabled: input.enabled ?? undefined
  };
}

export function toAppError(error: unknown): AppError {
  const fallback: AppError = {
    errorCode: "INTERNAL_ERROR",
    message: "unexpected client error"
  };

  if (!error || typeof error !== "object") {
    return fallback;
  }

  const raw = error as RawError;
  return {
    errorCode: raw.error_code ?? fallback.errorCode,
    message: raw.message ?? fallback.message
  };
}

export async function listChannels(includeDisabled = true): Promise<Channel[]> {
  const channels = await invoke<RawChannel[]>("list_channels", {
    includeDisabled
  });
  return channels.map(fromRawChannel);
}

export async function createChannel(input: ChannelInput): Promise<Channel> {
  const channel = await invoke<RawChannel>("create_channel", {
    input: toRawInput(input)
  });
  return fromRawChannel(channel);
}

export async function updateChannel(id: string, input: ChannelPatch): Promise<Channel> {
  const channel = await invoke<RawChannel>("update_channel", {
    id,
    input: toRawInput(input)
  });
  return fromRawChannel(channel);
}

export async function deleteChannel(id: string): Promise<void> {
  await invoke("delete_channel", { id });
}

export async function testChannel(id: string): Promise<ChannelTestResult> {
  return invoke<ChannelTestResult>("test_channel", { id });
}
