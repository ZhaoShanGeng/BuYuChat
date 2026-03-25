/**
 * 聊天工作台页面的纯函数辅助工具。
 *
 * 这个文件只处理前端状态层中不依赖 DOM 的逻辑：
 * 1. 错误码到中文提示的映射。
 * 2. 会话时间格式化。
 * 3. 消息楼层 / 版本的本地投影更新。
 */

import type { AppError } from "../lib/transport/common";
import type { GenerationEvent, MessageNode, MessageVersion } from "../lib/transport/messages";

/**
 * 定位某个版本在消息楼层数组中的位置。
 *
 * 这个辅助函数让后续更新只克隆命中的楼层和版本，避免在流式场景下对整棵消息树做全量映射。
 */
function findVersionLocation(
  nodes: MessageNode[],
  versionId: string,
  nodeId?: string
): { nodeIndex: number; versionIndex: number } | null {
  if (nodeId) {
    const nodeIndex = nodes.findIndex((node) => node.id === nodeId);
    if (nodeIndex === -1) {
      return null;
    }

    const versionIndex = nodes[nodeIndex].versions.findIndex((version) => version.id === versionId);
    return versionIndex === -1 ? null : { nodeIndex, versionIndex };
  }

  for (let nodeIndex = 0; nodeIndex < nodes.length; nodeIndex += 1) {
    const versionIndex = nodes[nodeIndex].versions.findIndex((version) => version.id === versionId);
    if (versionIndex !== -1) {
      return { nodeIndex, versionIndex };
    }
  }

  return null;
}

/**
 * 只替换单个版本，尽量减少数组和对象的重建范围。
 */
function replaceVersion(
  nodes: MessageNode[],
  versionId: string,
  updater: (version: MessageVersion) => MessageVersion,
  nodeId?: string
): MessageNode[] {
  const location = findVersionLocation(nodes, versionId, nodeId);
  if (!location) {
    return nodes;
  }

  const nextNodes = nodes.slice();
  const targetNode = nodes[location.nodeIndex];
  const nextVersions = targetNode.versions.slice();
  nextVersions[location.versionIndex] = updater(targetNode.versions[location.versionIndex]);
  nextNodes[location.nodeIndex] = {
    ...targetNode,
    versions: nextVersions
  };

  return nextNodes;
}

/**
 * 工作台顶部通知结构。
 */
export type Notice = {
  kind: "success" | "error" | "info";
  text: string;
};

/**
 * 后端错误码到中文文案的统一映射表。
 */
const ERROR_MESSAGES: Record<string, string> = {
  NO_AGENT: "请先为会话绑定 Agent",
  AGENT_DISABLED: "当前 Agent 已禁用，请启用或更换",
  NO_CHANNEL: "请先为会话绑定渠道",
  CHANNEL_DISABLED: "当前渠道已禁用，请启用或更换",
  NO_MODEL: "请先为会话选择模型",
  VALIDATION_ERROR: "输入不合法，请检查",
  INVALID_URL: "请输入有效的 URL",
  NAME_EMPTY: "名称不能为空",
  CONTENT_EMPTY: "消息内容不能为空",
  MODEL_ID_CONFLICT: "该渠道下已存在相同模型 ID",
  NOT_LAST_USER_NODE: "只能对最后一条用户消息执行重生成",
  VERSION_NOT_IN_NODE: "版本不属于该消息楼层",
  CHANNEL_UNREACHABLE: "无法连接到当前渠道，请检查配置",
  AI_REQUEST_FAILED: "AI 服务返回错误，请稍后重试",
  NOT_FOUND: "资源不存在",
  INTERNAL_ERROR: "系统内部错误，请重试"
};

/**
 * 将结构化错误转换为中文提示文案。
 */
export function humanizeWorkspaceError(error: AppError): string {
  return ERROR_MESSAGES[error.errorCode] ?? ERROR_MESSAGES.INTERNAL_ERROR;
}

/**
 * 将时间戳格式化为相对时间。
 */
export function formatRelativeTime(timestamp: number): string {
  const diffMs = Date.now() - timestamp;
  const diffMinutes = Math.floor(diffMs / 60_000);

  if (diffMinutes <= 0) {
    return "刚刚";
  }

  if (diffMinutes < 60) {
    return `${diffMinutes} 分钟前`;
  }

  const diffHours = Math.floor(diffMinutes / 60);
  if (diffHours < 24) {
    return `${diffHours} 小时前`;
  }

  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays} 天前`;
}

/**
 * 获取楼层当前 active version。
 */
export function getActiveVersion(node: MessageNode): MessageVersion | null {
  if (!node.activeVersionId) {
    return null;
  }

  return node.versions.find((version) => version.id === node.activeVersionId) ?? null;
}

/**
 * 判断楼层当前是否处于生成中。
 */
export function isNodeGenerating(node: MessageNode): boolean {
  return getActiveVersion(node)?.status === "generating";
}

/**
 * 在本地消息数组中替换某个版本的完整内容。
 */
export function withVersionContent(
  nodes: MessageNode[],
  versionId: string,
  content: string
): MessageNode[] {
  return replaceVersion(nodes, versionId, (version) => ({
    ...version,
    content
  }));
}

/**
 * 在本地消息数组中切换某个楼层的 active version。
 */
export function withActiveVersion(
  nodes: MessageNode[],
  nodeId: string,
  versionId: string
): MessageNode[] {
  const nodeIndex = nodes.findIndex((node) => node.id === nodeId);
  if (nodeIndex === -1 || nodes[nodeIndex].activeVersionId === versionId) {
    return nodes;
  }

  const nextNodes = nodes.slice();
  nextNodes[nodeIndex] = {
    ...nodes[nodeIndex],
    activeVersionId: versionId
  };

  return nextNodes;
}

/**
 * 在本地消息数组中应用生成事件的可预测部分。
 *
 * 终态事件后仍建议主动向后端刷新一次，以收敛 token、finish_reason 等最终状态。
 */
export function applyGenerationEvent(
  nodes: MessageNode[],
  event: GenerationEvent
): MessageNode[] {
  switch (event.type) {
    case "chunk":
      return replaceVersion(
        nodes,
        event.versionId,
        (version) => ({
          ...version,
          status: "generating",
          content: `${version.content ?? ""}${event.delta}`
        }),
        event.nodeId
      );
    case "completed":
      return replaceVersion(
        nodes,
        event.versionId,
        (version) => ({
          ...version,
          status: "committed",
          promptTokens: event.promptTokens,
          completionTokens: event.completionTokens,
          finishReason: event.finishReason,
          modelName: event.model
        }),
        event.nodeId
      );
    case "failed":
      return replaceVersion(
        nodes,
        event.versionId,
        (version) => ({
          ...version,
          status: "failed"
        }),
        event.nodeId
      );
    case "cancelled":
      return replaceVersion(
        nodes,
        event.versionId,
        (version) => ({
          ...version,
          status: "cancelled"
        }),
        event.nodeId
      );
    case "empty_rollback":
      if (event.nodeDeleted) {
        return nodes.filter((node) => node.id !== event.nodeId);
      }

      return (() => {
        const nodeIndex = nodes.findIndex((node) => node.id === event.nodeId);
        if (nodeIndex === -1) {
          return nodes;
        }

        const nextNodes = nodes.slice();
        const targetNode = nodes[nodeIndex];
        nextNodes[nodeIndex] = {
          ...targetNode,
          activeVersionId: event.fallbackVersionId,
          versions: targetNode.versions.filter((version) => version.status !== "generating")
        };

        return nextNodes;
      })();
  }
}
