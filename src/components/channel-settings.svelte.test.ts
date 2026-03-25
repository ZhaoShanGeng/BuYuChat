/**
 * 渠道设置 runes 状态工厂测试。
 */

import { flushSync } from "svelte";
import { describe, expect, it, vi } from "vitest";

import { createChannelSettingsState } from "./channel-settings.svelte.js";

/**
 * 等待 runes effect 和相关异步任务收敛。
 */
async function settleState() {
  for (let index = 0; index < 3; index += 1) {
    flushSync();
    await Promise.resolve();
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  flushSync();
}

describe("channel settings runes state", () => {
  it("bootstraps channel list and notifies parent callback", async () => {
    const listChannels = vi.fn().mockResolvedValue([
      {
        id: "channel-1",
        name: "OpenAI",
        channelType: "openai_compatible",
        baseUrl: "https://api.openai.com",
        apiKey: null,
        authType: "bearer",
        modelsEndpoint: "/v1/models",
        chatEndpoint: "/v1/chat/completions",
        streamEndpoint: "/v1/chat/completions",
        enabled: true,
        createdAt: 1,
        updatedAt: 1
      }
    ]);
    const onChanged = vi.fn();

    const channelSettings = createChannelSettingsState({
      listChannels,
      onChanged
    });

    await settleState();

    expect(listChannels).toHaveBeenCalledWith(true);
    expect(onChanged).toHaveBeenCalledTimes(1);
    expect(channelSettings.state.channels[0]?.name).toBe("OpenAI");
    channelSettings.destroy();
  });

  it("submits create flow through the state factory", async () => {
    const createChannel = vi.fn().mockResolvedValue({
      id: "channel-1",
      name: "OpenAI",
      channelType: "openai_compatible",
      baseUrl: "https://api.openai.com",
      apiKey: null,
      authType: "bearer",
      modelsEndpoint: "/v1/models",
      chatEndpoint: "/v1/chat/completions",
      streamEndpoint: "/v1/chat/completions",
      enabled: true,
      createdAt: 1,
      updatedAt: 1
    });
    const listChannels = vi.fn().mockResolvedValue([]);

    const channelSettings = createChannelSettingsState({
      createChannel,
      listChannels
    });

    await settleState();
    channelSettings.state.form.name = "OpenAI";
    channelSettings.state.form.baseUrl = "https://api.openai.com";

    await channelSettings.handleSubmit(new SubmitEvent("submit"));

    expect(createChannel).toHaveBeenCalledWith(
      expect.objectContaining({
        name: "OpenAI",
        apiKey: null
      })
    );
    expect(channelSettings.state.notice).toEqual({
      kind: "success",
      text: "渠道已创建"
    });
    channelSettings.destroy();
  });
});
