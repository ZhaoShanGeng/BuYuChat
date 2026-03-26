/**
 * 会话 transport 层测试。
 */

import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock
}));

describe("conversations transport", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("does not include archived or pinned on create payload", async () => {
    invokeMock.mockResolvedValue({
      id: "conv-1",
      title: "新会话",
      agent_id: null,
      channel_id: null,
      channel_model_id: null,
      archived: false,
      pinned: false,
      created_at: 1,
      updated_at: 2
    });

    const { createConversation } = await import("./conversations");
    await createConversation({
      title: "新会话",
      agentId: null
    });

    expect(invokeMock).toHaveBeenCalledWith("create_conversation", {
      input: {
        title: "新会话",
        agent_id: null,
        channel_id: undefined,
        channel_model_id: undefined,
        archived: undefined,
        pinned: undefined
      }
    });
  });

  it("preserves explicit null when clearing bindings", async () => {
    invokeMock.mockResolvedValue({
      id: "conv-1",
      title: "新会话",
      agent_id: null,
      channel_id: null,
      channel_model_id: null,
      archived: false,
      pinned: false,
      created_at: 1,
      updated_at: 2
    });

    const { updateConversation } = await import("./conversations");
    await updateConversation("conv-1", {
      agentId: null,
      channelId: null,
      channelModelId: null
    });

    expect(invokeMock).toHaveBeenCalledWith("update_conversation", {
      id: "conv-1",
      input: {
        title: undefined,
        agent_id_set: true,
        agent_id: null,
        channel_id_set: true,
        channel_id: null,
        channel_model_id_set: true,
        channel_model_id: null,
        archived: undefined,
        pinned: undefined
      }
    });
  });

  it("preserves boolean false in patch payload", async () => {
    invokeMock.mockResolvedValue({
      id: "conv-1",
      title: "新会话",
      agent_id: null,
      channel_id: null,
      channel_model_id: null,
      archived: false,
      pinned: false,
      created_at: 1,
      updated_at: 3
    });

    const { updateConversation } = await import("./conversations");
    await updateConversation("conv-1", {
      archived: false,
      pinned: false
    });

    expect(invokeMock).toHaveBeenCalledWith("update_conversation", {
      id: "conv-1",
      input: {
        title: undefined,
        agent_id_set: false,
        agent_id: undefined,
        channel_id_set: false,
        channel_id: undefined,
        channel_model_id_set: false,
        channel_model_id: undefined,
        archived: false,
        pinned: false
      }
    });
  });
});
