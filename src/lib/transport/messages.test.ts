/**
 * 消息 transport 层测试。
 */

import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();

class MockChannel<T> {
  onmessage?: (payload: T) => void;

  constructor(onmessage?: (payload: T) => void) {
    this.onmessage = onmessage;
  }
}

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
  Channel: MockChannel
}));

describe("messages transport", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("maps dry_run payload and passes a Tauri Channel", async () => {
    invokeMock.mockResolvedValue({
      messages: [{ role: "system", content: "你是助手" }],
      total_tokens_estimate: 12,
      model: "gpt-4o"
    });

    const { sendMessage } = await import("./messages");
    const response = await sendMessage("conv-1", {
      content: "你好",
      dryRun: true
    });

    expect(invokeMock).toHaveBeenCalledWith(
      "send_message",
      expect.objectContaining({
        id: "conv-1",
        input: {
          content: "你好",
          stream: undefined,
          dry_run: true
        },
        eventChannel: expect.any(MockChannel)
      })
    );
    expect(response).toEqual({
      kind: "dryRun",
      messages: [{ role: "system", content: "你是助手", images: [] }],
      totalTokensEstimate: 12,
      model: "gpt-4o"
    });
  });

  it("maps generation channel events to camelCase payloads", async () => {
    invokeMock.mockResolvedValue({
      user_node_id: "node-user",
      user_version_id: "ver-user",
      assistant_node_id: "node-assistant",
      assistant_version_id: "ver-assistant"
    });

    const onEvent = vi.fn();
    const { sendMessage } = await import("./messages");
    await sendMessage(
      "conv-1",
      {
        content: "你好",
        stream: true
      },
      onEvent
    );

    const call = invokeMock.mock.calls[0]?.[1] as {
      eventChannel: MockChannel<{
        type: "completed";
        conversation_id: string;
        node_id: string;
        version_id: string;
        prompt_tokens: number;
        completion_tokens: number;
        finish_reason: string;
        model: string;
      }>;
    };

    call.eventChannel.onmessage?.({
      type: "completed",
      conversation_id: "conv-1",
      node_id: "node-assistant",
      version_id: "ver-assistant",
      prompt_tokens: 11,
      completion_tokens: 22,
      finish_reason: "stop",
      model: "gpt-4o-mini"
    });

    expect(onEvent).toHaveBeenCalledWith({
      type: "completed",
      conversationId: "conv-1",
      nodeId: "node-assistant",
      versionId: "ver-assistant",
      promptTokens: 11,
      completionTokens: 22,
      finishReason: "stop",
      model: "gpt-4o-mini"
    });
  });

  it("maps edit_message payload and result", async () => {
    invokeMock.mockResolvedValue({
      edited_version_id: "ver-3",
      assistant_node_id: "node-2",
      assistant_version_id: "ver-4"
    });

    const { editMessage } = await import("./messages");
    const result = await editMessage(
      "conv-1",
      "node-1",
      {
        content: "编辑后的内容",
        resend: true,
        stream: true
      },
      vi.fn()
    );

    expect(invokeMock).toHaveBeenCalledWith(
      "edit_message",
      expect.objectContaining({
        id: "conv-1",
        nodeId: "node-1",
        input: {
          content: "编辑后的内容",
          resend: true,
          stream: true
        },
        eventChannel: expect.any(MockChannel)
      })
    );
    expect(result).toEqual({
      editedVersionId: "ver-3",
      assistantNodeId: "node-2",
      assistantVersionId: "ver-4"
    });
  });
});
