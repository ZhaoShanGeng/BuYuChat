import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock
}));

describe("channels transport", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("maps create payload to snake_case", async () => {
    invokeMock.mockResolvedValue({
      id: "1",
      name: "My OpenAI",
      channel_type: "openai_compatible",
      base_url: "https://api.openai.com",
      api_key: "sk-xxx",
      auth_type: "bearer",
      models_endpoint: "/v1/models",
      chat_endpoint: "/v1/chat/completions",
      stream_endpoint: "/v1/chat/completions",
      enabled: true,
      created_at: 1,
      updated_at: 1
    });

    const { createChannel } = await import("./channels");
    const channel = await createChannel({
      name: "My OpenAI",
      baseUrl: "https://api.openai.com",
      apiKey: "sk-xxx"
    });

    expect(invokeMock).toHaveBeenCalledWith("create_channel", {
      input: {
        name: "My OpenAI",
        base_url: "https://api.openai.com",
        channel_type: undefined,
        api_key: "sk-xxx",
        auth_type: undefined,
        models_endpoint: undefined,
        chat_endpoint: undefined,
        stream_endpoint: undefined,
        enabled: undefined
      }
    });
    expect(channel.channelType).toBe("openai_compatible");
    expect(channel.baseUrl).toBe("https://api.openai.com");
  });
});
