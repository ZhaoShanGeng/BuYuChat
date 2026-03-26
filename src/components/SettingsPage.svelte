<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import * as Switch from "$lib/components/ui/switch/index.js";
  import SearchIcon from "@lucide/svelte/icons/search";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import GlobeIcon from "@lucide/svelte/icons/globe";
  import KeyRoundIcon from "@lucide/svelte/icons/key-round";
  import SparklesIcon from "@lucide/svelte/icons/sparkles";
  import ServerCogIcon from "@lucide/svelte/icons/server-cog";
  import TestTubeDiagonalIcon from "@lucide/svelte/icons/test-tube-diagonal";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    parseThinkingTagsConfig,
    serializeThinkingTagsInput
  } from "../lib/thinking-tags";
  import type { AppError } from "../lib/transport/common";
  import { toAppError } from "../lib/transport/common";
  import type { Channel, ChannelInput } from "../lib/transport/channels";
  import {
    createChannel,
    deleteChannel,
    listChannels,
    testChannel,
    updateChannel
  } from "../lib/transport/channels";
  import type { ChannelModel, ModelInput, RemoteModelInfo } from "../lib/transport/models";
  import {
    createModel,
    deleteModel,
    fetchRemoteModels,
    listModels
  } from "../lib/transport/models";
  import WindowControls from "./WindowControls.svelte";

  type Props = {
    onChanged?: () => void | Promise<void>;
  };

  type Notice = {
    kind: "success" | "error" | "info";
    text: string;
  };

  type ChannelFormState = {
    name: string;
    baseUrl: string;
    apiKey: string;
    authType: string;
    modelsEndpoint: string;
    chatEndpoint: string;
    streamEndpoint: string;
    thinkingTagsInput: string;
    channelType: string;
    enabled: boolean;
  };

  const { onChanged = async () => undefined }: Props = $props();
  const currentWindow = getCurrentWindow();

  const EMPTY_FORM: ChannelFormState = {
    name: "",
    baseUrl: "https://api.openai.com",
    apiKey: "",
    authType: "bearer",
    modelsEndpoint: "/v1/models",
    chatEndpoint: "/v1/chat/completions",
    streamEndpoint: "/v1/chat/completions",
    thinkingTagsInput: "",
    channelType: "openai_compatible",
    enabled: true
  };

  const CHANNEL_TYPE_OPTIONS = [
    { value: "openai_compatible", label: "OpenAI Compatible" }
  ];

  const AUTH_TYPE_OPTIONS = [
    { value: "bearer", label: "Bearer Token" },
    { value: "x_api_key", label: "X-API-Key" },
    { value: "none", label: "无鉴权" }
  ];

  let initialized = false;
  let search = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let testingId = $state<string | null>(null);
  let notice = $state<Notice | null>(null);
  let channels = $state<Channel[]>([]);
  let selectedChannelId = $state<string | null>(null);
  let form = $state<ChannelFormState>({ ...EMPTY_FORM });
  let models = $state<ChannelModel[]>([]);
  let remoteModels = $state<RemoteModelInfo[]>([]);
  let loadingModels = $state(false);
  let loadingRemoteModels = $state(false);
  let managingModels = $state(false);
  let addingModel = $state(false);
  let newModelId = $state("");
  let newModelDisplayName = $state("");

  async function handleHeaderMouseDown(event: MouseEvent) {
    const target = event.target as HTMLElement | null;
    if (
      event.button !== 0 ||
      target?.closest("button, input, textarea, select, a, [role='button'], [data-no-drag]")
    ) {
      return;
    }

    await currentWindow.startDragging();
  }

  function setNotice(kind: Notice["kind"], text: string) {
    notice = { kind, text };
  }

  function humanizeError(error: AppError): string {
    switch (error.errorCode) {
      case "INVALID_URL":
        return "请输入有效的 API 地址";
      case "NAME_EMPTY":
        return "名称不能为空";
      case "VALIDATION_ERROR":
        return "输入不合法，请检查后重试";
      case "CHANNEL_UNREACHABLE":
        return "无法连接到该渠道，请检查地址和密钥";
      case "MODEL_ID_CONFLICT":
        return "该渠道下已存在相同模型 ID";
      case "NOT_FOUND":
        return "目标资源不存在，列表已为你刷新";
      default:
        return error.message || "操作失败，请稍后重试";
    }
  }

  function buildFormFromChannel(channel: Channel): ChannelFormState {
    return {
      name: channel.name,
      baseUrl: channel.baseUrl,
      apiKey: channel.apiKey ?? "",
      authType: channel.authType ?? "bearer",
      modelsEndpoint: channel.modelsEndpoint ?? "/v1/models",
      chatEndpoint: channel.chatEndpoint ?? "/v1/chat/completions",
      streamEndpoint: channel.streamEndpoint ?? "/v1/chat/completions",
      thinkingTagsInput: parseThinkingTagsConfig(channel.thinkingTags).join(", "),
      channelType: channel.channelType || "openai_compatible",
      enabled: channel.enabled
    };
  }

  function clearModelDraft() {
    addingModel = false;
    managingModels = false;
    newModelId = "";
    newModelDisplayName = "";
    remoteModels = [];
  }

  function normalizeChannelType(value: string): string | null {
    const normalized = value.trim().toLowerCase().replace(/[\s-]+/g, "_");
    return normalized || null;
  }

  function normalizeAuthType(value: string): string | null {
    const normalized = value.trim().toLowerCase().replace(/[\s-]+/g, "_");
    if (!normalized) {
      return null;
    }
    if (normalized === "bearer_token") {
      return "bearer";
    }
    return normalized;
  }

  function getChannelEnabled(channel: Channel) {
    if (channel.id === selectedChannelId) {
      return form.enabled;
    }

    return channel.enabled;
  }

  const filteredChannels = $derived.by(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return channels;
    }

    return channels.filter((channel) =>
      [channel.name, channel.baseUrl, channel.channelType].some((value) =>
        value.toLowerCase().includes(keyword)
      )
    );
  });

  const selectedChannel = $derived.by(() =>
    selectedChannelId ? channels.find((channel) => channel.id === selectedChannelId) ?? null : null
  );

  const groupedModels = $derived.by(() => {
    const groups = new Map<string, ChannelModel[]>();
    for (const model of models) {
      const key = resolveModelGroup(model.modelId);
      const bucket = groups.get(key) ?? [];
      bucket.push(model);
      groups.set(key, bucket);
    }

    return [...groups.entries()]
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([group, items]) => [
        group,
        [...items].sort((a, b) =>
          (a.displayName ?? a.modelId).localeCompare(b.displayName ?? b.modelId)
        )
      ] as const);
  });

  function resolveModelGroup(modelId: string) {
    const normalized = modelId.trim().toLowerCase();
    if (!normalized) {
      return "其他";
    }

    const prefix = normalized.split(/[-/_.]/)[0];
    return prefix || "其他";
  }

  async function loadModels(channelId: string) {
    loadingModels = true;
    try {
      models = await listModels(channelId);
    } catch (error) {
      models = [];
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      loadingModels = false;
    }
  }

  async function loadChannels(nextSelectedChannelId?: string | null) {
    loading = true;
    try {
      channels = await listChannels(true);

      const preferredId =
        nextSelectedChannelId ??
        (selectedChannelId && channels.some((channel) => channel.id === selectedChannelId)
          ? selectedChannelId
          : channels[0]?.id ?? null);

      selectedChannelId = preferredId;
      if (!preferredId) {
        form = { ...EMPTY_FORM };
        models = [];
        clearModelDraft();
        return;
      }

      const channel = channels.find((item) => item.id === preferredId);
      if (!channel) {
        form = { ...EMPTY_FORM };
        models = [];
        clearModelDraft();
        return;
      }

      form = buildFormFromChannel(channel);
      clearModelDraft();
      await loadModels(channel.id);
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      loading = false;
    }
  }

  async function selectChannel(channel: Channel) {
    selectedChannelId = channel.id;
    form = buildFormFromChannel(channel);
    clearModelDraft();
    notice = null;
    await loadModels(channel.id);
  }

  function startCreateChannel() {
    selectedChannelId = null;
    form = { ...EMPTY_FORM };
    models = [];
    clearModelDraft();
    notice = null;
  }

  async function resetCurrentDraft() {
    if (selectedChannel) {
      await selectChannel(selectedChannel);
      return;
    }

    startCreateChannel();
  }

  async function handleSaveChannel() {
    const payload: ChannelInput = {
      name: form.name.trim(),
      baseUrl: form.baseUrl.trim(),
      apiKey: form.apiKey.trim() || null,
      authType: normalizeAuthType(form.authType),
      modelsEndpoint: form.modelsEndpoint.trim() || null,
      chatEndpoint: form.chatEndpoint.trim() || null,
      streamEndpoint: form.streamEndpoint.trim() || null,
      thinkingTags: serializeThinkingTagsInput(form.thinkingTagsInput),
      channelType: normalizeChannelType(form.channelType),
      enabled: form.enabled
    };

    saving = true;
    notice = null;
    try {
      const channel = selectedChannelId
        ? await updateChannel(selectedChannelId, payload)
        : await createChannel(payload);
      await loadChannels(channel.id);
      setNotice("success", selectedChannelId ? "渠道已更新" : "渠道已创建");
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      saving = false;
    }
  }

  async function handleDeleteChannel() {
    if (!selectedChannelId) {
      return;
    }

    notice = null;
    try {
      await deleteChannel(selectedChannelId);
      startCreateChannel();
      await loadChannels();
      setNotice("success", "渠道已删除");
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleTestChannel() {
    if (!selectedChannelId) {
      return;
    }

    testingId = selectedChannelId;
    notice = null;
    try {
      const result = await testChannel(selectedChannelId);
      setNotice("success", result.message ?? "渠道连通性验证成功");
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      testingId = null;
    }
  }

  async function handleCreateModel() {
    if (!selectedChannelId || !newModelId.trim()) {
      return;
    }

    const payload: ModelInput = {
      modelId: newModelId.trim(),
      displayName: newModelDisplayName.trim() || null
    };

    notice = null;
    try {
      await createModel(selectedChannelId, payload);
      newModelId = "";
      newModelDisplayName = "";
      addingModel = false;
      await loadModels(selectedChannelId);
      setNotice("success", "模型已创建");
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleDeleteModel(id: string) {
    if (!selectedChannelId) {
      return;
    }

    notice = null;
    try {
      await deleteModel(selectedChannelId, id);
      await loadModels(selectedChannelId);
      setNotice("success", "模型已删除");
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  async function handleFetchRemoteModels() {
    if (!selectedChannelId) {
      return;
    }

    loadingRemoteModels = true;
    notice = null;
    try {
      remoteModels = await fetchRemoteModels(selectedChannelId);
      setNotice("info", `已刷新 ${remoteModels.length} 个远程模型候选`);
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    } finally {
      loadingRemoteModels = false;
    }
  }

  async function handleImportRemoteModel(model: RemoteModelInfo) {
    if (!selectedChannelId) {
      return;
    }

    notice = null;
    try {
      await createModel(selectedChannelId, {
        modelId: model.modelId,
        displayName: model.displayName,
        contextWindow: model.contextWindow
      });
      await loadModels(selectedChannelId);
      setNotice("success", `已导入模型 ${model.displayName ?? model.modelId}`);
      await onChanged();
    } catch (error) {
      setNotice("error", humanizeError(toAppError(error)));
    }
  }

  $effect(() => {
    if (initialized) {
      return;
    }

    initialized = true;
    void loadChannels();
  });
</script>

<div class="flex h-full min-h-0 flex-col bg-background text-foreground">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="flex h-14 items-center justify-between border-b px-6" onmousedown={handleHeaderMouseDown}>
    <div>
      <h2 class="text-sm font-semibold">模型服务</h2>
      <p class="text-xs text-muted-foreground">统一管理渠道、模型和远程模型候选。</p>
    </div>
    <WindowControls compact />
  </div>

  <div class="flex min-h-0 flex-1">
    <aside class="flex w-[22rem] shrink-0 flex-col border-r bg-muted/20">
      <div class="border-b p-4">
        <div class="relative">
          <SearchIcon class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            bind:value={search}
            class="h-10 rounded-xl bg-background pl-10"
            placeholder="搜索渠道名称或地址"
          />
        </div>

        <Button class="mt-3 w-full rounded-xl" onclick={startCreateChannel} variant="outline">
          <PlusIcon class="mr-1 size-4" />
          添加渠道
        </Button>
      </div>

      {#if notice}
        <div class="px-4 pt-4">
          <div
            class={`rounded-xl px-3 py-2 text-sm ${
              notice.kind === "success"
                ? "bg-emerald-500/10 text-emerald-700"
                : notice.kind === "info"
                  ? "bg-blue-500/10 text-blue-700"
                  : "bg-destructive/10 text-destructive"
            }`}
          >
            {notice.text}
          </div>
        </div>
      {/if}

      <div class="min-h-0 flex-1 overflow-y-auto p-3">
        {#if loading}
          <div class="rounded-2xl border border-dashed p-6 text-center text-sm text-muted-foreground">
            渠道加载中...
          </div>
        {:else if filteredChannels.length === 0}
          <div class="rounded-2xl border border-dashed p-6 text-center text-sm text-muted-foreground">
            {search.trim() ? "没有匹配的渠道" : "还没有渠道，先创建一个。"}
          </div>
        {:else}
          <div class="space-y-1.5">
            {#each filteredChannels as channel (channel.id)}
              <button
                class={`flex w-full items-center gap-3 rounded-2xl border px-3 py-3 text-left transition-colors ${
                  selectedChannelId === channel.id
                    ? "border-border bg-background shadow-sm"
                    : "border-transparent hover:bg-background"
                }`}
                onclick={() => void selectChannel(channel)}
                type="button"
              >
                <div class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-sm font-semibold text-primary">
                  {channel.name.slice(0, 1).toUpperCase()}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="truncate text-sm font-medium">{channel.name}</div>
                  <div class="truncate text-xs text-muted-foreground">{channel.baseUrl}</div>
                </div>
                <Badge variant="outline">
                  {getChannelEnabled(channel) ? "启用" : "禁用"}
                </Badge>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </aside>

    <section class="min-h-0 flex-1 overflow-y-auto">
      <div class="mx-auto flex max-w-5xl flex-col gap-6 p-6">
        <div class="flex flex-wrap items-start justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <h1 class="text-2xl font-semibold tracking-tight">
                {selectedChannel ? selectedChannel.name : "新建渠道"}
              </h1>
              {#if selectedChannel}
                <Badge variant="outline">{selectedChannel.channelType}</Badge>
              {/if}
            </div>
            <p class="mt-1 text-sm text-muted-foreground">
              渠道配置保存成功后，下面的模型列表会自动同步刷新。
            </p>
          </div>
          <div class="flex items-center gap-3 rounded-full border bg-background px-4 py-2">
            <span class="text-sm text-muted-foreground">启用渠道</span>
            <Switch.Root bind:checked={form.enabled} />
          </div>
        </div>

        <form
          class="rounded-3xl border bg-card p-6 shadow-sm"
          onsubmit={(event) => {
            event.preventDefault();
            void handleSaveChannel();
          }}
        >
          <div class="grid gap-5 lg:grid-cols-2">
            <div class="space-y-2">
              <Label>名称</Label>
              <Input bind:value={form.name} class="h-11 rounded-xl" placeholder="例如：OpenAI" />
            </div>
            <div class="space-y-2">
              <Label>API 地址</Label>
              <Input
                bind:value={form.baseUrl}
                class="h-11 rounded-xl"
                placeholder="https://api.openai.com"
              />
            </div>

            <div class="space-y-2">
              <Label>渠道类型</Label>
              <select
                bind:value={form.channelType}
                class="flex h-11 w-full rounded-xl border border-input bg-background px-3 text-sm shadow-sm outline-none transition-colors focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring"
              >
                {#each CHANNEL_TYPE_OPTIONS as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>
            <div class="space-y-2">
              <Label>鉴权方式</Label>
              <select
                bind:value={form.authType}
                class="flex h-11 w-full rounded-xl border border-input bg-background px-3 text-sm shadow-sm outline-none transition-colors focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring"
              >
                {#each AUTH_TYPE_OPTIONS as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>
          </div>

          <div class="mt-5 space-y-2">
            <Label>API 密钥</Label>
            <div class="flex flex-col gap-3 sm:flex-row">
              <div class="relative flex-1">
                <KeyRoundIcon class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  bind:value={form.apiKey}
                  class="h-11 rounded-xl pl-10"
                  placeholder="sk-..."
                  type="password"
                />
              </div>
              <Button
                class="h-11 rounded-xl px-5"
                disabled={!selectedChannelId || testingId === selectedChannelId}
                onclick={handleTestChannel}
                type="button"
                variant="outline"
              >
                <TestTubeDiagonalIcon class="mr-1 size-4" />
                {testingId === selectedChannelId ? "检测中..." : "检测"}
              </Button>
            </div>
          </div>

          <div class="mt-5 rounded-2xl border bg-muted/30 p-4">
            <div class="text-sm font-medium">高级设置</div>
            <p class="mt-1 text-xs text-muted-foreground">
              默认值已经适配 OpenAI-compatible 渠道；只有在服务端接口不一致时才需要调整。
            </p>

            <div class="mt-4 grid gap-4 lg:grid-cols-2">
              <div class="space-y-2">
                <Label>模型端点</Label>
                <Input bind:value={form.modelsEndpoint} class="h-10 rounded-xl" />
              </div>
              <div class="space-y-2">
                <Label>聊天端点</Label>
                <Input bind:value={form.chatEndpoint} class="h-10 rounded-xl" />
              </div>
              <div class="space-y-2">
                <Label>流式端点</Label>
                <Input bind:value={form.streamEndpoint} class="h-10 rounded-xl" />
              </div>
              <div class="space-y-2">
                <Label>思维链标签</Label>
                <Input
                  bind:value={form.thinkingTagsInput}
                  class="h-10 rounded-xl"
                  placeholder="think, reasoning, thought"
                />
              </div>
            </div>
          </div>

          <div class="mt-5 flex flex-wrap items-center gap-3">
            <Button class="rounded-xl px-5" disabled={saving} type="submit">
              {saving ? "保存中..." : selectedChannelId ? "保存修改" : "创建渠道"}
            </Button>
            <Button class="rounded-xl px-5" onclick={() => void resetCurrentDraft()} type="button" variant="outline">
              重置
            </Button>
            {#if selectedChannelId}
              <Button class="rounded-xl px-5" onclick={handleDeleteChannel} type="button" variant="destructive">
                删除
              </Button>
            {/if}
          </div>
        </form>

        <div class="rounded-3xl border bg-card p-6 shadow-sm">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <div class="flex items-center gap-2">
              <div class="flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
                <ServerCogIcon class="size-4" />
              </div>
              <div>
                <div class="text-sm font-medium">模型列表</div>
                <div class="text-xs text-muted-foreground">
                  {selectedChannelId ? "管理当前渠道下可用的模型" : "请先创建并保存渠道"}
                </div>
              </div>
              <Badge variant="outline">{models.length}</Badge>
            </div>

            <div class="flex flex-wrap items-center gap-2">
              <Button
                class="rounded-xl px-4"
                disabled={!selectedChannelId || loadingRemoteModels}
                onclick={handleFetchRemoteModels}
                type="button"
                variant="outline"
              >
                <GlobeIcon class="mr-1 size-4" />
                {loadingRemoteModels ? "拉取中..." : "拉取远程模型"}
              </Button>
              <Button
                class="rounded-xl px-4"
                disabled={!selectedChannelId}
                onclick={() => (managingModels = !managingModels)}
                type="button"
                variant={managingModels ? "default" : "outline"}
              >
                管理
              </Button>
              <Button
                class="rounded-xl px-4"
                disabled={!selectedChannelId}
                onclick={() => (addingModel = !addingModel)}
                type="button"
                variant="outline"
              >
                <PlusIcon class="mr-1 size-4" />
                添加模型
              </Button>
            </div>
          </div>

          {#if addingModel}
            <div class="mt-5 grid gap-3 rounded-2xl border border-dashed p-4 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto]">
              <Input
                bind:value={newModelId}
                class="h-10 rounded-xl"
                placeholder="模型 ID，例如 gpt-4o-mini"
              />
              <Input
                bind:value={newModelDisplayName}
                class="h-10 rounded-xl"
                placeholder="显示名称（可选）"
              />
              <Button class="h-10 rounded-xl px-4" onclick={handleCreateModel} type="button">
                保存模型
              </Button>
            </div>
          {/if}

          <div class="mt-5">
            {#if !selectedChannelId}
              <div class="rounded-2xl border border-dashed p-8 text-center text-sm text-muted-foreground">
                先保存一个渠道，再继续配置模型。
              </div>
            {:else if loadingModels}
              <div class="rounded-2xl border border-dashed p-8 text-center text-sm text-muted-foreground">
                模型加载中...
              </div>
            {:else if groupedModels.length === 0}
              <div class="rounded-2xl border border-dashed p-8 text-center text-sm text-muted-foreground">
                当前渠道还没有模型，可以手动添加或从远程拉取。
              </div>
            {:else}
              <div class="space-y-3">
                {#each groupedModels as [group, items]}
                  <div class="overflow-hidden rounded-2xl border bg-muted/20">
                    <div class="flex items-center gap-2 border-b px-4 py-3 text-sm font-medium">
                      <SparklesIcon class="size-4 text-muted-foreground" />
                      <span>{group}</span>
                      <Badge variant="outline">{items.length}</Badge>
                    </div>
                    <div class="space-y-2 p-3">
                      {#each items as model (model.id)}
                        <div class="flex items-center gap-3 rounded-xl border bg-background px-3 py-3">
                          <div class="flex size-9 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
                            <SparklesIcon class="size-4" />
                          </div>
                          <div class="min-w-0 flex-1">
                            <div class="truncate text-sm font-medium">
                              {model.displayName ?? model.modelId}
                            </div>
                            <div class="truncate text-xs text-muted-foreground">{model.modelId}</div>
                          </div>
                          {#if model.contextWindow}
                            <Badge variant="outline">{model.contextWindow}</Badge>
                          {/if}
                          {#if managingModels}
                            <Button
                              class="size-8 rounded-xl"
                              onclick={() => void handleDeleteModel(model.id)}
                              size="icon"
                              type="button"
                              variant="ghost"
                            >
                              <Trash2Icon class="size-4 text-muted-foreground" />
                            </Button>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          {#if remoteModels.length > 0}
            <div class="mt-5 rounded-2xl border border-dashed p-4">
              <div class="flex items-center gap-2 text-sm font-medium">
                <GlobeIcon class="size-4 text-muted-foreground" />
                <span>远程模型候选</span>
                <Badge variant="outline">{remoteModels.length}</Badge>
              </div>

              <div class="mt-3 space-y-2">
                {#each remoteModels as model (model.modelId)}
                  <div class="flex flex-wrap items-center gap-3 rounded-xl border bg-background px-3 py-3">
                    <div class="min-w-0 flex-1">
                      <div class="truncate text-sm font-medium">
                        {model.displayName ?? model.modelId}
                      </div>
                      <div class="truncate text-xs text-muted-foreground">{model.modelId}</div>
                    </div>
                    <Button
                      class="rounded-xl px-4"
                      onclick={() => void handleImportRemoteModel(model)}
                      size="sm"
                      type="button"
                      variant="outline"
                    >
                      导入
                    </Button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
    </section>
  </div>
</div>
