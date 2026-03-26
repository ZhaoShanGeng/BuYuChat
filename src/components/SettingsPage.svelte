<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import * as Switch from "$lib/components/ui/switch/index.js";
  import BotIcon from "@lucide/svelte/icons/bot";
  import CableIcon from "@lucide/svelte/icons/cable";
  import DatabaseIcon from "@lucide/svelte/icons/database";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import GlobeIcon from "@lucide/svelte/icons/globe";
  import InfoIcon from "@lucide/svelte/icons/info";
  import KeyRoundIcon from "@lucide/svelte/icons/key-round";
  import MemoryStickIcon from "@lucide/svelte/icons/memory-stick";
  import MessageSquareQuoteIcon from "@lucide/svelte/icons/message-square-quote";
  import PaletteIcon from "@lucide/svelte/icons/palette";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import SearchIcon from "@lucide/svelte/icons/search";
  import ServerCogIcon from "@lucide/svelte/icons/server-cog";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
  import SparklesIcon from "@lucide/svelte/icons/sparkles";
  import TestTubeDiagonalIcon from "@lucide/svelte/icons/test-tube-diagonal";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import WandSparklesIcon from "@lucide/svelte/icons/wand-sparkles";
  import KeyboardIcon from "@lucide/svelte/icons/keyboard";
  import WorkflowIcon from "@lucide/svelte/icons/workflow";
  import MonitorCogIcon from "@lucide/svelte/icons/monitor-cog";
  import BookTextIcon from "@lucide/svelte/icons/book-text";
  import FileSearchIcon from "@lucide/svelte/icons/file-search";
  import CircleHelpIcon from "@lucide/svelte/icons/circle-help";
  import { getCurrentWindow } from "@tauri-apps/api/window";
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

  type CategoryId =
    | "model-service"
    | "default-model"
    | "general"
    | "display"
    | "data"
    | "mcp"
    | "search"
    | "memory"
    | "api"
    | "files"
    | "snippets"
    | "hotkeys"
    | "assistant"
    | "selection-assistant"
    | "about";

  type ChannelFormState = {
    name: string;
    baseUrl: string;
    apiKey: string;
    authType: string;
    modelsEndpoint: string;
    chatEndpoint: string;
    streamEndpoint: string;
    channelType: string;
    enabled: boolean;
  };

  const { onChanged = async () => undefined }: Props = $props();
  const currentWindow = getCurrentWindow();

  const categories: Array<{ id: CategoryId; label: string; icon: typeof CableIcon }> = [
    { id: "model-service", label: "模型服务", icon: CableIcon },
    { id: "default-model", label: "默认模型", icon: BotIcon },
    { id: "general", label: "常规设置", icon: SlidersHorizontalIcon },
    { id: "display", label: "显示设置", icon: MonitorCogIcon },
    { id: "data", label: "数据设置", icon: DatabaseIcon },
    { id: "mcp", label: "MCP 服务器", icon: WorkflowIcon },
    { id: "search", label: "网络搜索", icon: SearchIcon },
    { id: "memory", label: "全局记忆", icon: MemoryStickIcon },
    { id: "api", label: "API 服务", icon: ServerCogIcon },
    { id: "files", label: "文件处理", icon: FileSearchIcon },
    { id: "snippets", label: "快捷短语", icon: MessageSquareQuoteIcon },
    { id: "hotkeys", label: "快捷键", icon: KeyboardIcon },
    { id: "assistant", label: "快捷助手", icon: SparklesIcon },
    { id: "selection-assistant", label: "划词助手", icon: WandSparklesIcon },
    { id: "about", label: "关于我们", icon: InfoIcon }
  ];

  const EMPTY_FORM: ChannelFormState = {
    name: "",
    baseUrl: "",
    apiKey: "",
    authType: "Bearer Token",
    modelsEndpoint: "",
    chatEndpoint: "",
    streamEndpoint: "",
    channelType: "openai-compatible",
    enabled: true
  };

  let initialized = false;
  let activeCategory = $state<CategoryId>("model-service");
  let channels = $state<Channel[]>([]);
  let selectedChannelId = $state<string | null>(null);
  let search = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let testingId = $state<string | null>(null);
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
    return [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  });

  function resolveModelGroup(modelId: string) {
    const normalized = modelId.trim().toLowerCase();
    if (!normalized) {
      return "其他";
    }

    const prefix = normalized.split(/[-/_.]/)[0];
    return prefix ? prefix : "其他";
  }

  function buildFormFromChannel(channel: Channel): ChannelFormState {
    return {
      name: channel.name,
      baseUrl: channel.baseUrl,
      apiKey: channel.apiKey ?? "",
      authType: channel.authType ?? "Bearer Token",
      modelsEndpoint: channel.modelsEndpoint ?? "",
      chatEndpoint: channel.chatEndpoint ?? "",
      streamEndpoint: channel.streamEndpoint ?? "",
      channelType: channel.channelType,
      enabled: channel.enabled
    };
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
      if (preferredId) {
        const channel = channels.find((item) => item.id === preferredId);
        if (channel) {
          form = buildFormFromChannel(channel);
          await loadModels(channel.id);
        }
      } else {
        form = { ...EMPTY_FORM };
        models = [];
      }
    } finally {
      loading = false;
    }
  }

  async function loadModels(channelId: string) {
    loadingModels = true;
    try {
      models = await listModels(channelId);
    } finally {
      loadingModels = false;
    }
  }

  async function selectChannel(channel: Channel) {
    selectedChannelId = channel.id;
    form = buildFormFromChannel(channel);
    remoteModels = [];
    addingModel = false;
    await loadModels(channel.id);
  }

  function startCreateChannel() {
    selectedChannelId = null;
    form = { ...EMPTY_FORM };
    models = [];
    remoteModels = [];
    addingModel = false;
  }

  async function handleSaveChannel() {
    const payload: ChannelInput = {
      name: form.name,
      baseUrl: form.baseUrl,
      apiKey: form.apiKey.trim() || null,
      authType: form.authType.trim() || null,
      modelsEndpoint: form.modelsEndpoint.trim() || null,
      chatEndpoint: form.chatEndpoint.trim() || null,
      streamEndpoint: form.streamEndpoint.trim() || null,
      channelType: form.channelType.trim() || null,
      enabled: form.enabled
    };

    saving = true;
    try {
      const channel = selectedChannelId
        ? await updateChannel(selectedChannelId, payload)
        : await createChannel(payload);
      await loadChannels(channel.id);
      await onChanged();
    } finally {
      saving = false;
    }
  }

  async function handleDeleteChannel() {
    if (!selectedChannelId) {
      return;
    }

    await deleteChannel(selectedChannelId);
    startCreateChannel();
    await loadChannels();
    await onChanged();
  }

  async function handleTestChannel() {
    if (!selectedChannelId) {
      return;
    }

    testingId = selectedChannelId;
    try {
      await testChannel(selectedChannelId);
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
    await createModel(selectedChannelId, payload);
    newModelId = "";
    newModelDisplayName = "";
    addingModel = false;
    await loadModels(selectedChannelId);
    await onChanged();
  }

  async function handleDeleteModel(id: string) {
    if (!selectedChannelId) {
      return;
    }

    await deleteModel(selectedChannelId, id);
    await loadModels(selectedChannelId);
    await onChanged();
  }

  async function handleFetchRemoteModels() {
    if (!selectedChannelId) {
      return;
    }

    loadingRemoteModels = true;
    try {
      remoteModels = await fetchRemoteModels(selectedChannelId);
    } finally {
      loadingRemoteModels = false;
    }
  }

  async function handleImportRemoteModel(model: RemoteModelInfo) {
    if (!selectedChannelId) {
      return;
    }

    await createModel(selectedChannelId, {
      modelId: model.modelId,
      displayName: model.displayName,
      contextWindow: model.contextWindow
    });
    await loadModels(selectedChannelId);
    await onChanged();
  }

  $effect(() => {
    if (initialized) {
      return;
    }

    initialized = true;
    void loadChannels();
  });
</script>

<div class="flex h-full min-h-0 flex-col bg-[#f6f3eb]">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="flex h-14 items-center border-b border-black/5 bg-[#f6f3eb] px-5" onmousedown={handleHeaderMouseDown}>
    <div class="text-2xl font-semibold tracking-tight text-slate-950">设置</div>
    <div class="min-w-6 flex-1"></div>
    <div class="shrink-0">
      <WindowControls />
    </div>
  </div>

  <div class="flex min-h-0 flex-1">
    <aside class="flex w-72 shrink-0 flex-col border-r border-black/5 bg-[#f9f7f1] px-4 py-4">
      <div class="space-y-1">
        {#each categories as category}
          <button
            class={`flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left text-[15px] transition-colors ${
              activeCategory === category.id
                ? "bg-white text-slate-950 shadow-sm ring-1 ring-black/5"
                : "text-slate-600 hover:bg-white/70 hover:text-slate-950"
            }`}
            onclick={() => (activeCategory = category.id)}
            type="button"
          >
            <category.icon class="size-4 shrink-0" />
            <span>{category.label}</span>
          </button>
        {/each}
      </div>
    </aside>

    {#if activeCategory === "model-service"}
      <div class="flex min-h-0 flex-1">
        <section class="flex w-[22rem] shrink-0 flex-col border-r border-black/5 bg-[#fbfaf6] px-3 py-4">
          <div class="relative">
            <SearchIcon class="pointer-events-none absolute left-4 top-1/2 size-4 -translate-y-1/2 text-slate-400" />
            <Input
              bind:value={search}
              class="h-12 rounded-2xl border-black/10 bg-white pl-11 shadow-none"
              placeholder="搜索模型平台..."
            />
          </div>

          <div class="mt-4 min-h-0 flex-1 overflow-y-auto">
            {#if loading}
              <div class="px-3 py-10 text-sm text-slate-500">加载中...</div>
            {:else}
              <div class="space-y-2">
                {#each filteredChannels as channel}
                  <button
                    class={`flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left transition-colors ${
                      selectedChannelId === channel.id
                        ? "bg-white shadow-sm ring-1 ring-black/5"
                        : "hover:bg-white/70"
                    }`}
                    onclick={() => void selectChannel(channel)}
                    type="button"
                  >
                    <div class="flex size-10 shrink-0 items-center justify-center rounded-full bg-emerald-500 text-sm font-semibold text-white">
                      {channel.name.slice(0, 1).toLowerCase()}
                    </div>
                    <div class="min-w-0 flex-1">
                      <div class="truncate text-[15px] font-medium text-slate-950">{channel.name}</div>
                      <div class="truncate text-xs text-slate-500">{channel.baseUrl}</div>
                    </div>
                    <Badge
                      class={`rounded-full border px-2 py-0.5 text-[11px] ${
                        getChannelEnabled(channel)
                          ? "border-emerald-200 bg-emerald-50 text-emerald-700"
                          : "border-slate-200 bg-slate-100 text-slate-500"
                      }`}
                      variant="outline"
                    >
                      {getChannelEnabled(channel) ? "启用" : "禁用"}
                    </Badge>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <Button class="mt-4 h-11 rounded-2xl" onclick={startCreateChannel} variant="outline">
            <PlusIcon class="mr-1 size-4" />
            添加
          </Button>
        </section>

        <section class="min-h-0 flex-1 overflow-y-auto bg-[#fcfbf8] px-6 py-5">
          <div class="mx-auto flex max-w-4xl flex-col gap-6">
            <div class="flex items-start justify-between gap-4">
              <div>
                <div class="flex items-center gap-3">
                  <h2 class="text-3xl font-semibold tracking-tight text-slate-950">
                    {selectedChannel ? selectedChannel.name : "新增渠道"}
                  </h2>
                  {#if selectedChannel}
                    <Badge
                      class={`rounded-full border px-2 py-0.5 text-[11px] ${
                        form.enabled
                          ? "border-emerald-200 bg-emerald-50 text-emerald-700"
                          : "border-slate-200 bg-slate-100 text-slate-500"
                      }`}
                      variant="outline"
                    >
                      {form.enabled ? "启用" : "禁用"}
                    </Badge>
                  {/if}
                </div>
                <p class="mt-1 text-sm text-slate-500">在这里配置渠道地址、密钥和该渠道下的模型。</p>
              </div>
              <div class="flex items-center rounded-full bg-white px-4 py-2 shadow-sm ring-1 ring-black/5">
                <Switch.Root bind:checked={form.enabled} />
              </div>
            </div>

            <div class="grid gap-5 md:grid-cols-2">
              <div class="space-y-2">
                <Label>名称</Label>
                <Input bind:value={form.name} class="h-11 rounded-xl border-black/10 bg-white" />
              </div>
              <div class="space-y-2">
                <Label>鉴权方式</Label>
                <Input bind:value={form.authType} class="h-11 rounded-xl border-black/10 bg-white" />
              </div>
            </div>

            <div class="space-y-2">
              <Label>API 密钥</Label>
              <div class="flex gap-3">
                <div class="relative flex-1">
                  <KeyRoundIcon class="pointer-events-none absolute left-4 top-1/2 size-4 -translate-y-1/2 text-slate-400" />
                  <Input bind:value={form.apiKey} class="h-11 rounded-xl border-black/10 bg-white pl-11" />
                </div>
                <Button class="h-11 rounded-xl px-5" disabled={!selectedChannelId || testingId === selectedChannelId} onclick={handleTestChannel} variant="outline">
                  <TestTubeDiagonalIcon class="mr-1 size-4" />
                  {testingId === selectedChannelId ? "检测中" : "检测"}
                </Button>
              </div>
            </div>

            <div class="space-y-2">
              <Label>API 地址</Label>
              <Input bind:value={form.baseUrl} class="h-11 rounded-xl border-black/10 bg-white" />
              <p class="text-xs text-slate-400">预览：{form.baseUrl || "未设置"}{form.chatEndpoint || "/v1/chat/completions"}</p>
            </div>

            <details class="rounded-2xl border border-black/5 bg-white px-4 py-4 shadow-sm">
              <summary class="cursor-pointer list-none text-sm font-medium text-slate-900">高级端点设置</summary>
              <div class="mt-4 grid gap-4 md:grid-cols-2">
                <div class="space-y-2">
                  <Label>模型端点</Label>
                  <Input bind:value={form.modelsEndpoint} class="h-10 rounded-xl border-black/10 bg-[#fcfbf8]" />
                </div>
                <div class="space-y-2">
                  <Label>聊天端点</Label>
                  <Input bind:value={form.chatEndpoint} class="h-10 rounded-xl border-black/10 bg-[#fcfbf8]" />
                </div>
                <div class="space-y-2">
                  <Label>流式端点</Label>
                  <Input bind:value={form.streamEndpoint} class="h-10 rounded-xl border-black/10 bg-[#fcfbf8]" />
                </div>
                <div class="space-y-2">
                  <Label>渠道类型</Label>
                  <Input bind:value={form.channelType} class="h-10 rounded-xl border-black/10 bg-[#fcfbf8]" />
                </div>
              </div>
            </details>

            <div class="flex items-center gap-3">
              <Button class="rounded-xl px-5" disabled={saving} onclick={handleSaveChannel}>
                {saving ? "保存中..." : selectedChannelId ? "保存修改" : "创建渠道"}
              </Button>
              <Button class="rounded-xl px-5" onclick={() => selectedChannel && void selectChannel(selectedChannel)} variant="outline">
                重置
              </Button>
              {#if selectedChannelId}
                <Button class="rounded-xl px-5" onclick={handleDeleteChannel} variant="destructive">
                  删除
                </Button>
              {/if}
            </div>

            <div class="flex items-center justify-between gap-3">
              <div class="flex items-center gap-2">
                <h3 class="text-lg font-semibold text-slate-950">模型</h3>
                <Badge class="rounded-full border border-black/5 bg-[#f4f2eb] px-2 py-0.5 text-xs text-slate-600" variant="outline">
                  {models.length}
                </Badge>
              </div>
              <Button class="size-8 rounded-lg" disabled={!selectedChannelId || loadingRemoteModels} onclick={handleFetchRemoteModels} size="icon" variant="ghost" title={loadingRemoteModels ? "拉取中..." : "从远程拉取模型列表"}>
                <GlobeIcon class="size-4 text-slate-500" />
              </Button>
            </div>

            {#if addingModel}
              <div class="mt-3 grid gap-3 rounded-2xl border border-dashed border-black/10 bg-[#fcfbf8] p-4 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto]">
                <Input bind:value={newModelId} class="h-10 rounded-xl border-black/10 bg-white" placeholder="模型 ID，例如 claude-sonnet-4-5" />
                <Input bind:value={newModelDisplayName} class="h-10 rounded-xl border-black/10 bg-white" placeholder="显示名称（可选）" />
                <Button class="h-10 rounded-xl px-4" onclick={handleCreateModel}>保存模型</Button>
              </div>
            {/if}

            {#if loadingModels}
              <div class="py-10 text-sm text-slate-500">正在加载模型...</div>
            {:else if groupedModels.length === 0}
              <div class="mt-3 rounded-2xl border border-dashed border-black/10 bg-[#fcfbf8] px-4 py-8 text-center text-sm text-slate-500">
                当前渠道还没有模型。
              </div>
            {:else}
              <div class="mt-3 space-y-1">
                {#each groupedModels as [group, items]}
                  <details class="overflow-hidden rounded-xl bg-[#f4f2eb]/60" open>
                    <summary class="flex cursor-pointer list-none items-center gap-2 px-4 py-2.5 text-sm font-semibold text-slate-700">
                      {group}
                    </summary>
                    <div class="space-y-px px-2 pb-2">
                      {#each items as model}
                        <div class="flex items-center gap-3 rounded-lg bg-white px-3 py-2.5">
                          <div class="flex size-8 shrink-0 items-center justify-center rounded-full bg-orange-100 text-orange-600">
                            <SparklesIcon class="size-3.5" />
                          </div>
                          <div class="min-w-0 flex-1">
                            <div class="truncate text-sm font-medium text-slate-950">
                              {model.displayName ?? model.modelId}
                            </div>
                          </div>
                          {#if managingModels}
                            <Button class="size-7 rounded-lg" onclick={() => void handleDeleteModel(model.id)} size="icon" variant="ghost" title="删除模型">
                              <Trash2Icon class="size-3.5 text-muted-foreground" />
                            </Button>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </details>
                {/each}
              </div>
            {/if}

            {#if remoteModels.length > 0}
              <div class="mt-4 rounded-2xl border border-dashed border-black/10 bg-[#fcfbf8] p-4">
                <div class="mb-3 flex items-center gap-2 text-sm font-medium text-slate-900">
                  <BookTextIcon class="size-4" />
                  远程模型候选
                </div>
                <div class="space-y-1">
                  {#each remoteModels as model}
                    <div class="flex items-center gap-3 rounded-lg bg-white px-3 py-2.5 ring-1 ring-black/5">
                      <div class="min-w-0 flex-1">
                        <div class="truncate text-sm font-medium text-slate-950">
                          {model.displayName ?? model.modelId}
                        </div>
                        <div class="truncate text-xs text-slate-500">{model.modelId}</div>
                      </div>
                      <Button class="rounded-lg px-3" onclick={() => void handleImportRemoteModel(model)} size="sm" variant="outline">
                        导入
                      </Button>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            <div class="mt-4 flex items-center gap-2">
              <Button class="h-9 rounded-xl px-4" onclick={() => (managingModels = !managingModels)} variant={managingModels ? "default" : "outline"}>
                <GaugeIcon class="mr-1 size-4" />
                管理
              </Button>
              <Button class="h-9 rounded-xl px-4" onclick={() => (addingModel = !addingModel)} variant="outline">
                <PlusIcon class="mr-1 size-4" />
                添加
              </Button>
            </div>
          </div>
        </section>
      </div>
    {:else}
      <div class="flex min-h-0 flex-1 items-center justify-center bg-[#fcfbf8]">
        <div class="max-w-md rounded-[2rem] border border-black/5 bg-white px-8 py-10 text-center shadow-sm">
          <div class="mx-auto flex size-14 items-center justify-center rounded-2xl bg-[#f6f3eb] text-slate-600">
            {#if activeCategory === "about"}
              <CircleHelpIcon class="size-6" />
            {:else if activeCategory === "display"}
              <PaletteIcon class="size-6" />
            {:else}
              <FileTextIcon class="size-6" />
            {/if}
          </div>
          <h3 class="mt-4 text-xl font-semibold text-slate-950">
            {categories.find((item) => item.id === activeCategory)?.label}
          </h3>
          <p class="mt-2 text-sm leading-6 text-slate-500">
            这个分类会保留稳定布局，但本轮只实现“模型服务”的完整交互。其余分类先作为占位页。
          </p>
        </div>
      </div>
    {/if}
  </div>
</div>
