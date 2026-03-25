/**
 * 模型管理相关的 Tauri transport 封装。
 */

import { invoke } from "@tauri-apps/api/core";
import { toOptionalValue, type AppError } from "./common";
import { toAppError } from "./common";

/**
 * 前端使用的模型资源。
 */
export type ChannelModel = {
  id: string;
  channelId: string;
  modelId: string;
  displayName: string | null;
  contextWindow: number | null;
  maxOutputTokens: number | null;
};

/**
 * 模型创建输入。
 */
export type ModelInput = {
  modelId: string;
  displayName?: string | null;
  contextWindow?: number | null;
  maxOutputTokens?: number | null;
};

/**
 * 模型更新补丁。
 */
export type ModelPatch = {
  displayName?: string | null;
  contextWindow?: number | null;
  maxOutputTokens?: number | null;
};

/**
 * 渠道远程返回的模型元信息。
 */
export type RemoteModelInfo = {
  modelId: string;
  displayName: string | null;
  contextWindow: number | null;
};

/**
 * 后端返回的原始模型载荷。
 */
type RawChannelModel = {
  id: string;
  channel_id: string;
  model_id: string;
  display_name: string | null;
  context_window: number | null;
  max_output_tokens: number | null;
};

/**
 * 后端返回的原始远程模型信息。
 */
type RawRemoteModelInfo = {
  model_id: string;
  display_name: string | null;
  context_window: number | null;
};

/**
 * 将原始模型对象转为前端结构。
 */
function fromRawModel(raw: RawChannelModel): ChannelModel {
  return {
    id: raw.id,
    channelId: raw.channel_id,
    modelId: raw.model_id,
    displayName: raw.display_name,
    contextWindow: raw.context_window,
    maxOutputTokens: raw.max_output_tokens
  };
}

/**
 * 将远程模型对象转为前端结构。
 */
function fromRemoteModel(raw: RawRemoteModelInfo): RemoteModelInfo {
  return {
    modelId: raw.model_id,
    displayName: raw.display_name,
    contextWindow: raw.context_window
  };
}

/**
 * 将前端模型输入转换为后端载荷。
 */
function toRawInput(input: ModelInput | ModelPatch) {
  return {
    model_id: "modelId" in input ? input.modelId : undefined,
    display_name: toOptionalValue(input.displayName),
    context_window: toOptionalValue(input.contextWindow),
    max_output_tokens: toOptionalValue(input.maxOutputTokens)
  };
}

/**
 * 获取某个渠道下的模型列表。
 */
export async function listModels(channelId: string): Promise<ChannelModel[]> {
  const models = await invoke<RawChannelModel[]>("list_models", { channelId });
  return models.map(fromRawModel);
}

/**
 * 创建模型。
 */
export async function createModel(channelId: string, input: ModelInput): Promise<ChannelModel> {
  const model = await invoke<RawChannelModel>("create_model", {
    channelId,
    input: toRawInput(input)
  });
  return fromRawModel(model);
}

/**
 * 更新模型。
 */
export async function updateModel(
  channelId: string,
  id: string,
  input: ModelPatch
): Promise<ChannelModel> {
  const model = await invoke<RawChannelModel>("update_model", {
    channelId,
    id,
    input: toRawInput(input)
  });
  return fromRawModel(model);
}

/**
 * 删除模型。
 */
export async function deleteModel(channelId: string, id: string): Promise<void> {
  await invoke("delete_model", { channelId, id });
}

/**
 * 从远程渠道拉取模型列表。
 */
export async function fetchRemoteModels(channelId: string): Promise<RemoteModelInfo[]> {
  const models = await invoke<RawRemoteModelInfo[]>("fetch_remote_models", { channelId });
  return models.map(fromRemoteModel);
}

export { toAppError, type AppError };
