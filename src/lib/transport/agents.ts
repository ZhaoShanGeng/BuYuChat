/**
 * Agent 管理相关的 Tauri transport 封装。
 */

import { invoke } from "@tauri-apps/api/core";
import { toOptionalValue, type AppError } from "./common";
import { toAppError } from "./common";

/**
 * 前端使用的 Agent 模型。
 */
export type Agent = {
  id: string;
  name: string;
  systemPrompt: string | null;
  avatarUri: string | null;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
};

/**
 * 创建 Agent 时使用的输入模型。
 */
export type AgentInput = {
  name: string;
  systemPrompt?: string | null;
};

/**
 * 更新 Agent 时使用的补丁模型。
 */
export type AgentPatch = {
  name?: string;
  systemPrompt?: string | null;
  enabled?: boolean;
};

/**
 * Tauri IPC 返回的原始 Agent 载荷。
 */
type RawAgent = {
  id: string;
  name: string;
  system_prompt: string | null;
  avatar_uri: string | null;
  enabled: boolean;
  created_at: number;
  updated_at: number;
};

/**
 * 将后端 snake_case Agent 转为前端 camelCase 结构。
 */
function fromRawAgent(raw: RawAgent): Agent {
  return {
    id: raw.id,
    name: raw.name,
    systemPrompt: raw.system_prompt,
    avatarUri: raw.avatar_uri,
    enabled: raw.enabled,
    createdAt: raw.created_at,
    updatedAt: raw.updated_at
  };
}

/**
 * 将前端 Agent 创建输入转换为命令层所需的 snake_case 载荷。
 */
function toRawCreateInput(input: AgentInput) {
  return {
    name: input.name,
    system_prompt: toOptionalValue(input.systemPrompt),
    enabled: undefined
  };
}

/**
 * 将前端 Agent 更新补丁转换为命令层所需的 snake_case 载荷。
 */
function toRawPatch(input: AgentPatch) {
  return {
    name: input.name,
    system_prompt: toOptionalValue(input.systemPrompt),
    enabled: toOptionalValue(input.enabled)
  };
}

/**
 * 获取 Agent 列表。
 */
export async function listAgents(includeDisabled = true): Promise<Agent[]> {
  const agents = await invoke<RawAgent[]>("list_agents", {
    includeDisabled
  });
  return agents.map(fromRawAgent);
}

/**
 * 获取单个 Agent 详情。
 */
export async function getAgent(id: string): Promise<Agent> {
  const agent = await invoke<RawAgent>("get_agent", { id });
  return fromRawAgent(agent);
}

/**
 * 创建 Agent。
 */
export async function createAgent(input: AgentInput): Promise<Agent> {
  const agent = await invoke<RawAgent>("create_agent", {
    input: toRawCreateInput(input)
  });
  return fromRawAgent(agent);
}

/**
 * 更新 Agent。
 */
export async function updateAgent(id: string, input: AgentPatch): Promise<Agent> {
  const agent = await invoke<RawAgent>("update_agent", {
    id,
    input: toRawPatch(input)
  });
  return fromRawAgent(agent);
}

/**
 * 删除 Agent。
 */
export async function deleteAgent(id: string): Promise<void> {
  await invoke("delete_agent", { id });
}

export { toAppError, type AppError };
