import { describe, expect, it } from "vitest";
import { buildSettingsBackup, parseSettingsBackup } from "./system-tools";

describe("system-tools", () => {
  it("builds a serializable settings backup", () => {
    const backup = buildSettingsBackup(
      [
        {
          id: "channel-1",
          name: "OpenAI",
          channelType: "openai_compatible",
          baseUrl: "https://api.openai.com",
          apiKey: "sk-test",
          apiKeys: null,
          authType: "bearer",
          modelsEndpoint: "/v1/models",
          chatEndpoint: "/v1/chat/completions",
          streamEndpoint: "/v1/chat/completions",
          thinkingTags: null,
          enabled: true,
          createdAt: 1,
          updatedAt: 2
        }
      ],
      new Map([
        [
          "channel-1",
          [
            {
              id: "model-1",
              channelId: "channel-1",
              modelId: "gpt-4.1",
              displayName: "GPT-4.1",
              contextWindow: 128000,
              maxOutputTokens: 8192,
              temperature: "0.7",
              topP: "0.95"
            }
          ]
        ]
      ])
    );

    expect(backup.schemaVersion).toBe(1);
    expect(backup.channels).toHaveLength(1);
    expect(backup.channels[0]?.models[0]?.modelId).toBe("gpt-4.1");
  });

  it("parses a valid backup", () => {
    const backup = parseSettingsBackup(`{
      "schemaVersion": 1,
      "exportedAt": "2026-04-04T12:00:00.000Z",
      "channels": [
        {
          "name": "OpenAI",
          "baseUrl": "https://api.openai.com",
          "channelType": "openai_compatible",
          "enabled": true,
          "models": [
            { "modelId": "gpt-4.1", "displayName": "GPT-4.1" }
          ]
        }
      ]
    }`);

    expect(backup.channels[0]?.name).toBe("OpenAI");
    expect(backup.channels[0]?.models[0]?.displayName).toBe("GPT-4.1");
  });

  it("rejects invalid backup json", () => {
    expect(() => parseSettingsBackup("not-json")).toThrow("配置文件不是有效的 JSON");
  });
});
