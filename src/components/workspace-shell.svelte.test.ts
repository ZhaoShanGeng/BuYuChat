/**
 * 聊天工作台 runes 状态工厂测试。
 */

import { flushSync } from "svelte";
import { describe, expect, it, vi } from "vitest";

import { createWorkspaceShellState } from "./workspace-shell.svelte.js";
import type { MessageNode } from "../lib/transport/messages";

/**
 * 构造一个最小可用的工作台依赖集合。
 */
function createWorkspaceDeps() {
  return {
    listChannels: vi.fn().mockResolvedValue([
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
    ]),
    listAgents: vi.fn().mockResolvedValue([
      {
        id: "agent-1",
        name: "助手",
        systemPrompt: "你是助手",
        avatarUri: null,
        enabled: true,
        createdAt: 1,
        updatedAt: 1
      }
    ]),
    createAgent: vi.fn(),
    updateAgent: vi.fn(),
    deleteAgent: vi.fn(),
    listConversations: vi.fn().mockResolvedValue([
      {
        id: "conv-1",
        title: "新会话",
        agentId: "agent-1",
        channelId: "channel-1",
        channelModelId: "model-1",
        archived: false,
        pinned: false,
        updatedAt: 2
      }
    ]),
    getConversation: vi.fn().mockResolvedValue({
      id: "conv-1",
      title: "新会话",
      agentId: "agent-1",
      channelId: "channel-1",
      channelModelId: "model-1",
      archived: false,
      pinned: false,
      createdAt: 1,
      updatedAt: 2
    }),
    createConversation: vi.fn(),
    updateConversation: vi.fn(),
    deleteConversation: vi.fn(),
    listModels: vi.fn().mockResolvedValue([
      {
        id: "model-1",
        channelId: "channel-1",
        modelId: "gpt-4o-mini",
        displayName: "GPT-4o Mini",
        contextWindow: 128000,
        maxOutputTokens: 4096
      }
    ]),
    createModel: vi.fn(),
    updateModel: vi.fn(),
    deleteModel: vi.fn(),
    fetchRemoteModels: vi.fn().mockResolvedValue([]),
    listMessages: vi.fn().mockResolvedValue([
      {
        id: "node-1",
        conversationId: "conv-1",
        authorAgentId: null,
        role: "assistant",
        orderKey: "0001",
        activeVersionId: "ver-1",
        versions: [
          {
            id: "ver-1",
            nodeId: "node-1",
            content: "第一版",
            thinkingContent: null,
            images: [],
            status: "committed",
            modelName: "gpt-4o-mini",
            promptTokens: 1,
            completionTokens: 2,
            finishReason: "stop",
            createdAt: 1
          },
          {
            id: "ver-2",
            nodeId: "node-1",
            content: null,
            thinkingContent: null,
            images: [],
            status: "committed",
            modelName: "gpt-4o-mini",
            promptTokens: 1,
            completionTokens: 3,
            finishReason: "stop",
            createdAt: 2
          }
        ],
        createdAt: 1
      }
    ]),
    getVersionContent: vi.fn().mockResolvedValue({
      versionId: "ver-2",
      content: "第二版正文",
      contentType: "text/plain"
    }),
    setActiveVersion: vi.fn().mockResolvedValue(undefined),
    sendMessage: vi.fn(),
    reroll: vi.fn(),
    editMessage: vi.fn().mockResolvedValue({
      editedVersionId: "ver-3",
      assistantNodeId: null,
      assistantVersionId: null
    }),
    cancelGeneration: vi.fn()
  };
}

/**
 * 创建一个可手动控制 resolve 时机的 Promise。
 */
function createDeferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((nextResolve) => {
    resolve = nextResolve;
  });

  return { promise, resolve };
}

/**
 * 等待 runes effect 和相关异步任务收敛。
 */
async function settleState() {
  for (let index = 0; index < 5; index += 1) {
    flushSync();
    await Promise.resolve();
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  flushSync();
}

describe("workspace shell runes state", () => {
  it("bootstraps workspace data and selects the first conversation", async () => {
    const deps = createWorkspaceDeps();
    const workspace = createWorkspaceShellState(deps);

    await settleState();

    expect(deps.listChannels).toHaveBeenCalledWith(true);
    expect(deps.listAgents).toHaveBeenCalledWith(true);
    expect(deps.listConversations).toHaveBeenCalledWith(false);
    expect(workspace.state.activeConversationId).toBe("conv-1");
    expect(workspace.state.selectedModelChannelId).toBe("channel-1");
    expect(workspace.activeMessages[0]?.id).toBe("node-1");
    workspace.destroy();
  });

  it("loads missing version content when switching active version", async () => {
    const deps = createWorkspaceDeps();
    const workspace = createWorkspaceShellState(deps);

    await settleState();
    await workspace.handleSwitchVersion("node-1", "ver-2");

    expect(deps.setActiveVersion).toHaveBeenCalledWith("conv-1", "node-1", "ver-2");
    expect(deps.getVersionContent).toHaveBeenCalledWith("ver-2");
    expect(workspace.activeMessages[0]?.activeVersionId).toBe("ver-2");
    expect(workspace.activeMessages[0]?.versions[1]?.content).toBe("第二版正文");
    workspace.destroy();
  });

  it("stores dry run summary without mutating messages", async () => {
    const deps = createWorkspaceDeps();
    deps.sendMessage.mockResolvedValue({
      kind: "dryRun",
      messages: [{ role: "system", content: "你是助手" }],
      totalTokensEstimate: 12,
      model: "gpt-4o-mini"
    });

    const workspace = createWorkspaceShellState(deps);
    await settleState();
    workspace.setComposer("你好");

    await workspace.handleDryRun();

    expect(deps.sendMessage).toHaveBeenCalledWith("conv-1", {
      content: "你好",
      images: [],
      stream: false,
      dryRun: true
    });
    expect(workspace.state.dryRunSummary).toContain("目标模型：gpt-4o-mini");
    expect(workspace.activeMessages[0]?.versions[0]?.content).toBe("第一版");
    workspace.destroy();
  });

  it("applies streaming chunks immediately to the current version", async () => {
    const deps = createWorkspaceDeps();
    const workspace = createWorkspaceShellState(deps);

    await settleState();

    workspace.activeMessages[0]!.versions[0]!.status = "generating";
    workspace.handleGenerationEvent({
      type: "chunk",
      conversationId: "conv-1",
      nodeId: "node-1",
      versionId: "ver-1",
      delta: "你"
    });
    workspace.handleGenerationEvent({
      type: "chunk",
      conversationId: "conv-1",
      nodeId: "node-1",
      versionId: "ver-1",
      delta: "好"
    });

    flushSync();
    await Promise.resolve();

    expect(workspace.activeMessages[0]?.versions[0]?.content).toBe("第一版你好");
    expect(workspace.activeMessages[0]?.versions[0]?.status).toBe("generating");
    workspace.destroy();
  });

  it("keeps the latest reloadMessages result when older requests resolve later", async () => {
    const deps = createWorkspaceDeps();
    const firstReload = createDeferred<MessageNode[]>();
    const secondReload = createDeferred<MessageNode[]>();
    deps.listMessages = vi
      .fn()
      .mockResolvedValueOnce([
        {
          id: "node-1",
          conversationId: "conv-1",
          authorAgentId: null,
          role: "assistant",
          orderKey: "0001",
          activeVersionId: "ver-1",
          versions: [
            {
              id: "ver-1",
              nodeId: "node-1",
              content: "第一版",
              thinkingContent: null,
              images: [],
              status: "committed",
              modelName: "gpt-4o-mini",
              promptTokens: 1,
              completionTokens: 2,
              finishReason: "stop",
              createdAt: 1
            }
          ],
          createdAt: 1
        }
      ])
      .mockReturnValueOnce(firstReload.promise)
      .mockReturnValueOnce(secondReload.promise);
    const workspace = createWorkspaceShellState(deps);

    await settleState();

    const staleNodes = [
      {
        id: "node-1",
        conversationId: "conv-1",
        authorAgentId: null,
        role: "assistant" as const,
        orderKey: "0001",
        activeVersionId: "ver-1",
        versions: [
          {
            id: "ver-1",
            nodeId: "node-1",
            content: null,
            thinkingContent: null,
            images: [],
            status: "generating" as const,
            modelName: "gpt-4o-mini",
            promptTokens: null,
            completionTokens: null,
            finishReason: null,
            createdAt: 1
          }
        ],
        createdAt: 1
      }
    ];
    const freshNodes = [
      {
        id: "node-1",
        conversationId: "conv-1",
        authorAgentId: null,
        role: "assistant" as const,
        orderKey: "0001",
        activeVersionId: "ver-1",
        versions: [
          {
            id: "ver-1",
            nodeId: "node-1",
            content: "最终结果",
            thinkingContent: null,
            images: [],
            status: "committed" as const,
            modelName: "gpt-4o-mini",
            promptTokens: 1,
            completionTokens: 2,
            finishReason: "stop",
            createdAt: 1
          }
        ],
        createdAt: 1
      }
    ];

    const older = workspace.reloadMessages("conv-1");
    const newer = workspace.reloadMessages("conv-1");

    secondReload.resolve(freshNodes);
    await newer;
    expect(workspace.activeMessages[0]?.versions[0]?.content).toBe("最终结果");

    firstReload.resolve(staleNodes);
    await older;
    expect(workspace.activeMessages[0]?.versions[0]?.content).toBe("最终结果");

    workspace.destroy();
  });

  it("inserts optimistic nodes after send_message starts", async () => {
    const deps = createWorkspaceDeps();
    deps.sendMessage.mockResolvedValue({
      kind: "started",
      userNodeId: "node-user-2",
      userVersionId: "ver-user-2",
      assistantNodeId: "node-assistant-2",
      assistantVersionId: "ver-assistant-2"
    });
    deps.listMessages = vi
      .fn()
      .mockResolvedValueOnce([
        {
          id: "node-1",
          conversationId: "conv-1",
          authorAgentId: null,
          role: "assistant",
          orderKey: "0001",
          activeVersionId: "ver-1",
          versions: [
            {
              id: "ver-1",
              nodeId: "node-1",
              content: "第一版",
              thinkingContent: null,
              images: [],
              status: "committed",
              modelName: "gpt-4o-mini",
              promptTokens: 1,
              completionTokens: 2,
              finishReason: "stop",
              createdAt: 1
            }
          ],
          createdAt: 1
        }
      ])
      .mockImplementationOnce(async () => {
        await new Promise((resolve) => setTimeout(resolve, 30));
        return [
          {
            id: "node-1",
            conversationId: "conv-1",
            authorAgentId: null,
            role: "assistant",
            orderKey: "0001",
            activeVersionId: "ver-1",
            versions: [
              {
                id: "ver-1",
                nodeId: "node-1",
                content: "第一版",
                thinkingContent: null,
                images: [],
                status: "committed",
                modelName: "gpt-4o-mini",
                promptTokens: 1,
                completionTokens: 2,
                finishReason: "stop",
                createdAt: 1
              }
            ],
            createdAt: 1
          }
        ];
      });

    const workspace = createWorkspaceShellState(deps);
    await settleState();
    workspace.setComposer("新的问题");

    await workspace.handleSendMessage();

    expect(workspace.activeMessages.at(-2)?.id).toBe("node-user-2");
    expect(workspace.activeMessages.at(-2)?.versions[0]?.content).toBe("新的问题");
    expect(workspace.activeMessages.at(-1)?.id).toBe("node-assistant-2");
    expect(workspace.activeMessages.at(-1)?.versions[0]?.status).toBe("generating");

    workspace.destroy();
  });

  it("refreshes latest channels before sending when the bound channel is missing from cache", async () => {
    const deps = createWorkspaceDeps();
    deps.listChannels = vi
      .fn()
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([
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
    deps.sendMessage.mockResolvedValue({
      kind: "started",
      userNodeId: "node-user-2",
      userVersionId: "ver-user-2",
      assistantNodeId: "node-assistant-2",
      assistantVersionId: "ver-assistant-2"
    });

    const workspace = createWorkspaceShellState(deps);
    await settleState();
    workspace.setComposer("同步后再发");

    await workspace.handleSendMessage();

    expect(deps.listChannels).toHaveBeenCalledTimes(2);
    expect(workspace.state.channels[0]?.id).toBe("channel-1");
    expect(deps.sendMessage).toHaveBeenCalledTimes(1);
    workspace.destroy();
  });

  it("retries once after syncing channels when send_message hits channel not found", async () => {
    const deps = createWorkspaceDeps();
    deps.sendMessage
      .mockRejectedValueOnce({
        error_code: "NOT_FOUND",
        message: "channel 'channel-1' not found"
      })
      .mockResolvedValueOnce({
        kind: "started",
        userNodeId: "node-user-2",
        userVersionId: "ver-user-2",
        assistantNodeId: "node-assistant-2",
        assistantVersionId: "ver-assistant-2"
      });

    const workspace = createWorkspaceShellState(deps);
    await settleState();
    workspace.setComposer("重试一次");

    await workspace.handleSendMessage();

    expect(deps.sendMessage).toHaveBeenCalledTimes(2);
    expect(deps.listChannels).toHaveBeenCalledTimes(2);
    expect(workspace.activeMessages.at(-1)?.id).toBe("node-assistant-2");
    workspace.destroy();
  });

  it("retries quick channel binding after refreshing channels when update hits channel not found", async () => {
    const deps = createWorkspaceDeps();
    deps.updateConversation
      .mockRejectedValueOnce({
        error_code: "NOT_FOUND",
        message: "channel 'channel-1' not found"
      })
      .mockResolvedValueOnce({
        id: "conv-1",
        title: "新会话",
        agentId: "agent-1",
        channelId: "channel-1",
        channelModelId: null,
        archived: false,
        pinned: false,
        createdAt: 1,
        updatedAt: 3
      });

    const workspace = createWorkspaceShellState(deps);
    await settleState();

    await workspace.handleQuickChannelChange("channel-1");

    expect(deps.updateConversation).toHaveBeenCalledTimes(2);
    expect(deps.listChannels).toHaveBeenCalledTimes(2);
    expect(workspace.state.activeConversation?.channelId).toBe("channel-1");
    workspace.destroy();
  });

  it("loads version content before inline edit save", async () => {
    const deps = createWorkspaceDeps();
    const workspace = createWorkspaceShellState(deps);

    await settleState();
    const content = await workspace.ensureMessageVersionContent("node-1", "ver-2");

    expect(deps.getVersionContent).toHaveBeenCalledWith("ver-2");
    expect(content).toBe("第二版正文");
    expect(workspace.activeMessages[0]?.versions[1]?.content).toBe("第二版正文");
    workspace.destroy();
  });

  it("enters create mode when starting a new agent", async () => {
    const deps = createWorkspaceDeps();
    const workspace = createWorkspaceShellState(deps);

    await settleState();
    workspace.startCreateAgent();

    expect(workspace.state.agentEditorMode).toBe("create");
    expect(workspace.state.agentEditingId).toBeNull();
    expect(workspace.state.agentForm.name).toBe("");
    workspace.destroy();
  });
});
