/**
 * Agent transport 层测试。
 */

import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock
}));

describe("agents transport", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("omits enabled when creating an agent", async () => {
    invokeMock.mockResolvedValue({
      id: "agent-1",
      name: "助手",
      system_prompt: "你是助手",
      avatar_uri: null,
      enabled: true,
      created_at: 1,
      updated_at: 1
    });

    const { createAgent } = await import("./agents");
    await createAgent({
      name: "助手",
      systemPrompt: "你是助手"
    });

    expect(invokeMock).toHaveBeenCalledWith("create_agent", {
      input: {
        name: "助手",
        system_prompt: "你是助手",
        enabled: undefined
      }
    });
  });

  it("preserves null when updating system_prompt", async () => {
    invokeMock.mockResolvedValue({
      id: "agent-1",
      name: "助手",
      system_prompt: null,
      avatar_uri: null,
      enabled: true,
      created_at: 1,
      updated_at: 2
    });

    const { updateAgent } = await import("./agents");
    await updateAgent("agent-1", {
      systemPrompt: null
    });

    expect(invokeMock).toHaveBeenCalledWith("update_agent", {
      id: "agent-1",
      input: {
        name: undefined,
        system_prompt: null,
        enabled: undefined
      }
    });
  });
});
