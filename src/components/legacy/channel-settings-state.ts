/**
 * 渠道管理页面的状态辅助函数与文案映射。
 */

import { toAppError, type AppError, type Channel, type ChannelInput } from "../../lib/transport/channels";

/**
 * 后端错误码到中文提示文案的映射表。
 */
export const errorMessages: Record<string, string> = {
  VALIDATION_ERROR: "输入不合法，请检查",
  INVALID_URL: "请输入有效的 URL（以 http:// 或 https:// 开头）",
  NAME_EMPTY: "名称不能为空",
  NOT_FOUND: "资源不存在",
  CHANNEL_UNREACHABLE: "无法连接到 AI 服务，请检查渠道配置",
  INTERNAL_ERROR: "系统内部错误，请重试"
};

/**
 * 页面顶部通知的统一结构。
 */
export type Notice = {
  kind: "success" | "error";
  text: string;
};

/**
 * 渠道页面依赖的 transport 能力集合。
 */
export type ChannelTransport = {
  createChannel: (input: ChannelInput) => Promise<Channel>;
  updateChannel: (id: string, input: ChannelInput) => Promise<Channel>;
  deleteChannel: (id: string) => Promise<void>;
  testChannel: (id: string) => Promise<{ success: boolean; message: string | null }>;
};

/**
 * 创建一个新的空白渠道表单。
 */
export function createEmptyForm(): ChannelInput {
  return {
    name: "",
    baseUrl: "https://api.openai.com",
    channelType: "openai_compatible",
    apiKey: "",
    authType: "bearer",
    modelsEndpoint: "/v1/models",
    chatEndpoint: "/v1/chat/completions",
    streamEndpoint: "/v1/chat/completions",
    enabled: true
  };
}

/**
 * 根据已有渠道生成可编辑表单。
 */
export function createFormFromChannel(channel: Channel): ChannelInput {
  return {
    name: channel.name,
    baseUrl: channel.baseUrl,
    channelType: channel.channelType,
    apiKey: channel.apiKey ?? "",
    authType: channel.authType ?? "bearer",
    modelsEndpoint: channel.modelsEndpoint ?? "/v1/models",
    chatEndpoint: channel.chatEndpoint ?? "/v1/chat/completions",
    streamEndpoint: channel.streamEndpoint ?? "/v1/chat/completions",
    enabled: channel.enabled
  };
}

/**
 * 将结构化错误转换为中文提示文案。
 */
export function humanizeError(error: AppError): string {
  return errorMessages[error.errorCode] ?? errorMessages.INTERNAL_ERROR;
}

/**
 * 将表单中的空字符串归一化为 null。
 */
export function normalizeChannelInput(form: ChannelInput): ChannelInput {
  return {
    ...form,
    apiKey: form.apiKey || null,
    authType: form.authType || null,
    modelsEndpoint: form.modelsEndpoint || null,
    chatEndpoint: form.chatEndpoint || null,
    streamEndpoint: form.streamEndpoint || null
  };
}

/**
 * 提交渠道表单并返回页面通知。
 */
export async function submitChannelForm(
  transport: Pick<ChannelTransport, "createChannel" | "updateChannel">,
  editingId: string | null,
  form: ChannelInput
): Promise<Notice> {
  try {
    const payload = normalizeChannelInput(form);
    if (editingId) {
      await transport.updateChannel(editingId, payload);
      return { kind: "success", text: "渠道已更新" };
    }

    await transport.createChannel(payload);
    return { kind: "success", text: "渠道已创建" };
  } catch (error) {
    return { kind: "error", text: humanizeError(toAppError(error)) };
  }
}

/**
 * 删除渠道并返回页面通知。
 */
export async function removeChannel(
  transport: Pick<ChannelTransport, "deleteChannel">,
  id: string
): Promise<Notice> {
  try {
    await transport.deleteChannel(id);
    return { kind: "success", text: "渠道已删除" };
  } catch (error) {
    return { kind: "error", text: humanizeError(toAppError(error)) };
  }
}

/**
 * 触发连通性测试并返回页面通知。
 */
export async function verifyChannelConnectivity(
  transport: Pick<ChannelTransport, "testChannel">,
  id: string
): Promise<Notice> {
  try {
    const result = await transport.testChannel(id);
    return {
      kind: result.success ? "success" : "error",
      text: result.success ? "渠道连通性验证成功" : "渠道连通性验证失败"
    };
  } catch (error) {
    return { kind: "error", text: humanizeError(toAppError(error)) };
  }
}
