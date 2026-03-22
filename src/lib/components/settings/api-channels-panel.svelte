<script lang="ts">
  import { flip } from "svelte/animate";
  import { cubicOut } from "svelte/easing";
  import { onDestroy, onMount } from "svelte";
  import { Dialog } from "bits-ui";
  import {
    Cable,
    ChevronDown,
    ChevronRight,
    Eye,
    EyeOff,
    GripVertical,
    Layers3,
    Minus,
    Pencil,
    Plus,
    RefreshCw,
    Save,
    Trash2,
    X
  } from "lucide-svelte";
  import {
    createApiChannel,
    deleteApiChannel,
    deleteApiChannelModel,
    fetchApiChannelRemoteModels,
    listApiChannelModels,
    listApiChannels,
    testApiChannelMessage,
    updateApiChannel,
    upsertApiChannelModel,
    type ApiChannel,
    type ApiChannelModel,
    type ApiChannelTestResponse,
    type CreateApiChannelInput,
    type UpsertApiChannelModelInput
  } from "$lib/api/api-channels";
  import Badge from "$components/ui/badge.svelte";
  import Button from "$components/ui/button.svelte";
  import Card from "$components/ui/card.svelte";
  import SearchField from "$components/shared/search-field.svelte";
  import { toast } from "svelte-sonner";
  import { cn } from "$lib/utils";

  type ChannelDraft = {
    name: string;
    channelType: string;
    baseUrl: string;
    authType: string;
    apiKey: string;
    modelsEndpoint: string;
    chatEndpoint: string;
    streamEndpoint: string;
    modelsMode: string;
    enabled: boolean;
    sortOrder: number;
  };

  type ChannelModelDraft = {
    modelId: string;
    displayName: string;
    modelType: string;
    contextWindow: string;
    maxOutputTokens: string;
    sortOrder: number;
  };

  type RemoteModelGroup = {
    id: string;
    label: string;
    count: number;
    savedCount: number;
    models: ApiChannelModel[];
  };

  const channelTypeOptions = [
    { value: "openai_compatible", label: "OpenAI Compatible" },
    { value: "anthropic", label: "Anthropic" },
    { value: "gemini", label: "Gemini" },
    { value: "custom", label: "Custom" }
  ];

  const authTypeOptions = [
    { value: "bearer", label: "Bearer" },
    { value: "x_api_key", label: "X-API-Key" },
    { value: "none", label: "None" }
  ];

  const modelsModeOptions = [
    { value: "remote", label: "仅远端" },
    { value: "local", label: "仅本地" },
    { value: "hybrid", label: "混合" }
  ];

  const inputClass =
    "w-full rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-2 text-sm text-[var(--ink-body)] outline-none transition-colors focus:border-[var(--brand)]";
  const monoInputClass = `${inputClass} font-mono`;
  const labelClass = "space-y-1";
  const labelTextClass = "text-xs font-medium text-[var(--ink-muted)]";

  let channels = $state<ApiChannel[]>([]);
  let channelModels = $state<Record<string, ApiChannelModel[]>>({});
  let channelsLoading = $state(true);
  let channelSearch = $state("");
  let selectedChannelId = $state<string | null>(null);

  let channelDialogOpen = $state(false);
  let channelDraft = $state<ChannelDraft>(emptyChannelDraft());
  let inlineChannelDraft = $state<ChannelDraft>(emptyChannelDraft());
  let savingChannel = $state(false);
  let savingInlineChannel = $state(false);
  let deletingChannel = $state(false);

  let modelDialogOpen = $state(false);
  let modelDraft = $state<ChannelModelDraft>(emptyModelDraft());
  let editingModelId = $state<string | null>(null);
  let savingModel = $state(false);
  let deletingModel = $state(false);
  let reorderingModelId = $state<string | null>(null);
  let draggingModelId = $state<string | null>(null);
  let dragOverModelId = $state<string | null>(null);
  let dragPointerId = $state<number | null>(null);
  let localModelOrder = $state<ApiChannelModel[]>([]);
  let dragPreview = $state<{
    x: number;
    y: number;
    width: number;
    height: number;
    offsetX: number;
    offsetY: number;
  } | null>(null);
  let dragHandleElement: HTMLElement | null = null;

  let remoteDialogOpen = $state(false);
  let remoteModelsLoading = $state(false);
  let remoteModels = $state<ApiChannelModel[]>([]);
  let remoteModelSearch = $state("");
  let remoteModelFilter = $state<"all" | "saved" | "available">("all");
  let remoteGroupOpen = $state<Record<string, boolean>>({});
  let syncingRemoteModelId = $state<string | null>(null);
  let localModelSearch = $state("");

  let testDialogOpen = $state(false);
  let selectedTestModelId = $state("");
  let testingChannel = $state(false);
  let apiKeyVisible = $state(false);

  const visibleChannels = $derived(
    channelSearch
      ? channels.filter((channel) =>
          `${channel.name} ${channel.base_url} ${channel.channel_type}`.toLowerCase().includes(channelSearch.toLowerCase())
        )
      : channels
  );

  const selectedChannel = $derived(
    selectedChannelId ? channels.find((channel) => channel.id === selectedChannelId) ?? null : null
  );

  const selectedModels = $derived(
    selectedChannelId ? channelModels[selectedChannelId] ?? [] : []
  );

  const existingModelIds = $derived(new Set(selectedModels.map((model) => model.model_id)));

  const visibleRemoteModels = $derived(
    remoteModels.filter((model) => {
      const matchesSearch = remoteModelSearch
        ? `${model.model_id} ${model.display_name ?? ""} ${model.model_type ?? ""}`
            .toLowerCase()
            .includes(remoteModelSearch.toLowerCase())
        : true;
      const imported = existingModelIds.has(model.model_id);
      const matchesFilter =
        remoteModelFilter === "all" ||
        (remoteModelFilter === "saved" && imported) ||
        (remoteModelFilter === "available" && !imported);
      return matchesSearch && matchesFilter;
    })
  );

  const visibleRemoteModelGroups = $derived.by<RemoteModelGroup[]>(() => {
    const groups = new Map<string, ApiChannelModel[]>();

    for (const model of visibleRemoteModels) {
      const groupId = getModelFamily(model.model_id);
      const current = groups.get(groupId) ?? [];
      current.push(model);
      groups.set(groupId, current);
    }

    return Array.from(groups.entries())
      .map(([id, models]) => ({
        id,
        label: id,
        count: models.length,
        savedCount: models.filter((model) => existingModelIds.has(model.model_id)).length,
        models
      }))
      .sort((a, b) => a.label.localeCompare(b.label));
  });

  const visibleLocalModels = $derived(
    localModelSearch
      ? localModelOrder.filter((model) =>
          `${model.model_id} ${model.display_name ?? ""} ${model.model_type ?? ""}`
            .toLowerCase()
            .includes(localModelSearch.toLowerCase())
        )
      : localModelOrder
  );

  const draggingModel = $derived(
    draggingModelId ? localModelOrder.find((model) => model.model_id === draggingModelId) ?? null : null
  );

  const inlineDraftDirty = $derived(
    selectedChannel
      ? JSON.stringify(mapChannelToDraft(selectedChannel)) !== JSON.stringify(inlineChannelDraft)
      : false
  );

  onMount(() => {
    void loadChannels();
  });

  onDestroy(() => {
    document.body.style.userSelect = "";
    document.body.style.cursor = "";
  });

  $effect(() => {
    if (selectedChannel) {
      inlineChannelDraft = mapChannelToDraft(selectedChannel);
    } else {
      inlineChannelDraft = emptyChannelDraft();
    }
  });

  $effect(() => {
    if (!draggingModelId && !reorderingModelId) {
      localModelOrder = [...selectedModels];
    }
  });

  $effect(() => {
    if (visibleRemoteModelGroups.length === 0) return;

    const next = { ...remoteGroupOpen };
    let changed = false;

    for (const group of visibleRemoteModelGroups) {
      if (!(group.id in next)) {
        next[group.id] = true;
        changed = true;
      }
    }

    if (changed) {
      remoteGroupOpen = next;
    }
  });

  function emptyChannelDraft(sortOrder = 0): ChannelDraft {
    return {
      name: "",
      channelType: "openai_compatible",
      baseUrl: "",
      authType: "bearer",
      apiKey: "",
      modelsEndpoint: "/models",
      chatEndpoint: "/chat/completions",
      streamEndpoint: "/chat/completions",
      modelsMode: "hybrid",
      enabled: true,
      sortOrder
    };
  }

  function emptyModelDraft(sortOrder = 0): ChannelModelDraft {
    return {
      modelId: "",
      displayName: "",
      modelType: "",
      contextWindow: "",
      maxOutputTokens: "",
      sortOrder
    };
  }

  function getModelFamily(modelId: string) {
    const normalized = modelId.replace(/[_:/]+/g, "-");
    const parts = normalized.split("-").filter(Boolean);
    if (parts.length >= 2) {
      return `${parts[0]}-${parts[1]}`;
    }
    return normalized;
  }

  function normalizeNullable(value: string) {
    const trimmed = value.trim();
    return trimmed ? trimmed : null;
  }

  function formatError(error: unknown) {
    if (error instanceof Error && error.message) return error.message;
    if (typeof error === "string") return error;
    if (error && typeof error === "object" && "message" in error) {
      const message = (error as { message?: unknown }).message;
      if (typeof message === "string" && message) return message;
    }
    return "请求失败，请稍后重试";
  }

  function parseOptionalNumber(value: string) {
    const trimmed = value.trim();
    if (!trimmed) return null;
    const parsed = Number(trimmed);
    return Number.isFinite(parsed) ? parsed : null;
  }

  function mapChannelToDraft(channel: ApiChannel): ChannelDraft {
    return {
      name: channel.name,
      channelType: channel.channel_type,
      baseUrl: channel.base_url,
      authType: channel.auth_type,
      apiKey: channel.api_key ?? "",
      modelsEndpoint: channel.models_endpoint ?? "",
      chatEndpoint: channel.chat_endpoint ?? "",
      streamEndpoint: channel.stream_endpoint ?? "",
      modelsMode: channel.models_mode,
      enabled: channel.enabled,
      sortOrder: channel.sort_order
    };
  }

  function mapModelToDraft(model: ApiChannelModel): ChannelModelDraft {
    return {
      modelId: model.model_id,
      displayName: model.display_name ?? "",
      modelType: model.model_type ?? "",
      contextWindow: model.context_window?.toString() ?? "",
      maxOutputTokens: model.max_output_tokens?.toString() ?? "",
      sortOrder: model.sort_order
    };
  }

  async function loadChannels(preferredSelection?: string | null) {
    channelsLoading = true;
    try {
      const items = await listApiChannels();
      channels = [...items].sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));

      const modelEntries = await Promise.all(
        channels.map(async (channel) => {
          const models = await listApiChannelModels(channel.id);
          return [
            channel.id,
            [...models].sort((a, b) => a.sort_order - b.sort_order || a.model_id.localeCompare(b.model_id))
          ] as const;
        })
      );
      channelModels = Object.fromEntries(modelEntries);

      const nextSelection =
        preferredSelection !== undefined
          ? preferredSelection
          : selectedChannelId && channels.some((channel) => channel.id === selectedChannelId)
            ? selectedChannelId
            : channels[0]?.id ?? null;

      selectedChannelId = nextSelection;
    } catch (error) {
      console.error("Failed to load API channels:", error);
      channels = [];
      channelModels = {};
      selectedChannelId = null;
    } finally {
      channelsLoading = false;
    }
  }

  function openCreateChannelDialog() {
    channelDraft = emptyChannelDraft(channels.length);
    channelDialogOpen = true;
  }

  async function saveChannel() {
    const input: CreateApiChannelInput = {
      name: channelDraft.name.trim() || "未命名渠道",
      channel_type: channelDraft.channelType,
      base_url: channelDraft.baseUrl.trim(),
      auth_type: channelDraft.authType,
      api_key: normalizeNullable(channelDraft.apiKey),
      models_endpoint: normalizeNullable(channelDraft.modelsEndpoint),
      chat_endpoint: normalizeNullable(channelDraft.chatEndpoint),
      stream_endpoint: normalizeNullable(channelDraft.streamEndpoint),
      models_mode: channelDraft.modelsMode,
      enabled: channelDraft.enabled,
      sort_order: Number(channelDraft.sortOrder) || 0,
      config_json: {}
    };

    savingChannel = true;
    try {
      const saved = await createApiChannel(input);
      channelDialogOpen = false;
      await loadChannels(saved.id);
    } catch (error) {
      console.error("Failed to save channel:", error);
    } finally {
      savingChannel = false;
    }
  }

  async function saveInlineChannel() {
    if (!selectedChannelId || !selectedChannel) return;

    const input: CreateApiChannelInput = {
      name: inlineChannelDraft.name.trim() || "未命名渠道",
      channel_type: inlineChannelDraft.channelType,
      base_url: inlineChannelDraft.baseUrl.trim(),
      auth_type: inlineChannelDraft.authType,
      api_key: normalizeNullable(inlineChannelDraft.apiKey),
      models_endpoint: normalizeNullable(inlineChannelDraft.modelsEndpoint),
      chat_endpoint: normalizeNullable(inlineChannelDraft.chatEndpoint),
      stream_endpoint: normalizeNullable(inlineChannelDraft.streamEndpoint),
      models_mode: inlineChannelDraft.modelsMode,
      enabled: inlineChannelDraft.enabled,
      sort_order: Number(inlineChannelDraft.sortOrder) || 0,
      config_json: selectedChannel.config_json ?? {}
    };

    savingInlineChannel = true;
    try {
      await updateApiChannel(selectedChannelId, input);
      await loadChannels(selectedChannelId);
    } catch (error) {
      console.error("Failed to save inline channel:", error);
    } finally {
      savingInlineChannel = false;
    }
  }

  async function removeChannel() {
    if (!selectedChannelId) return;
    if (!confirm("确定删除这个 API 渠道吗？")) return;

    deletingChannel = true;
    try {
      await deleteApiChannel(selectedChannelId);
      channelDialogOpen = false;
      await loadChannels(null);
    } catch (error) {
      console.error("Failed to delete channel:", error);
    } finally {
      deletingChannel = false;
    }
  }

  function openCreateModelDialog() {
    if (!selectedChannelId) return;
    editingModelId = null;
    modelDraft = emptyModelDraft(selectedModels.length);
    modelDialogOpen = true;
  }

  function openEditModelDialog(model: ApiChannelModel) {
    editingModelId = model.model_id;
    modelDraft = mapModelToDraft(model);
    modelDialogOpen = true;
  }

  async function saveModel() {
    if (!selectedChannelId) return;

    const input: UpsertApiChannelModelInput = {
      channel_id: selectedChannelId,
      model_id: modelDraft.modelId.trim(),
      display_name: normalizeNullable(modelDraft.displayName),
      model_type: normalizeNullable(modelDraft.modelType),
      context_window: parseOptionalNumber(modelDraft.contextWindow),
      max_output_tokens: parseOptionalNumber(modelDraft.maxOutputTokens),
      capabilities_json: {},
      pricing_json: {},
      default_parameters_json: {},
      sort_order: Number(modelDraft.sortOrder) || 0,
      config_json: {}
    };

    if (!input.model_id) return;

    savingModel = true;
    try {
      await upsertApiChannelModel(input);
      modelDialogOpen = false;
      editingModelId = null;
      await loadChannels(selectedChannelId);
    } catch (error) {
      console.error("Failed to save model:", error);
    } finally {
      savingModel = false;
    }
  }

  function toModelInput(channelId: string, model: ApiChannelModel, sortOrder = model.sort_order): UpsertApiChannelModelInput {
    return {
      channel_id: channelId,
      model_id: model.model_id,
      display_name: model.display_name,
      model_type: model.model_type,
      context_window: model.context_window,
      max_output_tokens: model.max_output_tokens,
      capabilities_json: model.capabilities_json,
      pricing_json: model.pricing_json,
      default_parameters_json: model.default_parameters_json,
      sort_order: sortOrder,
      config_json: model.config_json
    };
  }

  async function removeModel(modelId: string) {
    if (!selectedChannelId) return;
    deletingModel = true;
    try {
      await deleteApiChannelModel(selectedChannelId, modelId);
      modelDialogOpen = false;
      editingModelId = null;
      await loadChannels(selectedChannelId);
    } catch (error) {
      console.error("Failed to delete model:", error);
    } finally {
      deletingModel = false;
    }
  }

  async function openRemoteModelsDialog() {
    if (!selectedChannelId) return;
    remoteDialogOpen = true;
    remoteModelsLoading = true;
    remoteModelSearch = "";
    remoteModelFilter = "all";
    try {
      remoteModels = await fetchApiChannelRemoteModels(selectedChannelId);
    } catch (error) {
      console.error("Failed to fetch remote models:", error);
      remoteModels = [];
    } finally {
      remoteModelsLoading = false;
    }
  }

  function toggleRemoteGroup(groupId: string) {
    remoteGroupOpen = {
      ...remoteGroupOpen,
      [groupId]: !remoteGroupOpen[groupId]
    };
  }

  function openTestDialog() {
    if (!selectedChannelId) return;
    if (selectedModels.length === 0) {
      toast.error("请先添加至少一个本地模型");
      return;
    }
    selectedTestModelId = selectedModels[0]?.model_id ?? "";
    testDialogOpen = true;
  }

  async function runChannelTest() {
    if (!selectedChannelId || !selectedTestModelId) return;

    testingChannel = true;
    testDialogOpen = false;
    try {
      const response: ApiChannelTestResponse = await testApiChannelMessage(
        selectedChannelId,
        selectedTestModelId
      );
      toast.success(`${response.model_id} 检测成功`, {
        description: response.response_text
      });
    } catch (error) {
      console.error("Failed to test channel:", error);
      toast.error("检测失败", {
        description: formatError(error)
      });
    } finally {
      testingChannel = false;
    }
  }

  async function toggleRemoteModel(model: ApiChannelModel) {
    if (!selectedChannelId) return;
    syncingRemoteModelId = model.model_id;
    try {
      if (existingModelIds.has(model.model_id)) {
        await deleteApiChannelModel(selectedChannelId, model.model_id);
      } else {
        await upsertApiChannelModel({
          channel_id: selectedChannelId,
          model_id: model.model_id,
          display_name: model.display_name,
          model_type: model.model_type,
          context_window: model.context_window,
          max_output_tokens: model.max_output_tokens,
          capabilities_json: model.capabilities_json,
          pricing_json: model.pricing_json,
          default_parameters_json: model.default_parameters_json,
          sort_order: selectedModels.length,
          config_json: model.config_json
        });
      }
      await loadChannels(selectedChannelId);
    } catch (error) {
      console.error("Failed to sync remote model:", error);
    } finally {
      syncingRemoteModelId = null;
    }
  }

  async function persistModelOrder(channelId: string, models: ApiChannelModel[], activeModelId: string) {
    reorderingModelId = activeModelId;
    try {
      for (const [order, model] of models.entries()) {
        await upsertApiChannelModel(toModelInput(channelId, model, order));
      }
      await loadChannels(channelId);
    } catch (error) {
      console.error("Failed to reorder models:", error);
    } finally {
      reorderingModelId = null;
    }
  }

  function releaseDragPointerCapture() {
    if (dragHandleElement && dragPointerId !== null && dragHandleElement.hasPointerCapture?.(dragPointerId)) {
      dragHandleElement.releasePointerCapture(dragPointerId);
    }
    dragHandleElement = null;
  }

  function startModelDrag(event: PointerEvent, modelId: string) {
    if (reorderingModelId) return;
    event.preventDefault();
    event.stopPropagation();
    const handle = event.currentTarget as HTMLElement | null;
    const row = handle?.closest<HTMLElement>("[data-model-row-id]");
    if (!handle || !row) return;

    const rect = row.getBoundingClientRect();
    dragHandleElement = handle;
    handle.setPointerCapture?.(event.pointerId);

    draggingModelId = modelId;
    dragOverModelId = modelId;
    dragPointerId = event.pointerId;
    localModelOrder = [...selectedModels];
    dragPreview = {
      x: rect.left,
      y: rect.top,
      width: rect.width,
      height: rect.height,
      offsetX: event.clientX - rect.left,
      offsetY: event.clientY - rect.top
    };
    document.body.style.userSelect = "none";
    document.body.style.cursor = "grabbing";
  }

  function moveModelInPreview(targetId: string) {
    if (!draggingModelId || draggingModelId === targetId) return;

    const sourceIndex = localModelOrder.findIndex((model) => model.model_id === draggingModelId);
    const targetIndex = localModelOrder.findIndex((model) => model.model_id === targetId);

    if (sourceIndex < 0 || targetIndex < 0 || sourceIndex === targetIndex) return;

    const reordered = [...localModelOrder];
    const [moved] = reordered.splice(sourceIndex, 1);
    reordered.splice(targetIndex, 0, moved);
    localModelOrder = reordered;
  }

  function updatePointerDragTarget(event: PointerEvent) {
    if (!draggingModelId || dragPointerId !== event.pointerId) return;

    if (dragPreview) {
      dragPreview = {
        ...dragPreview,
        x: event.clientX - dragPreview.offsetX,
        y: event.clientY - dragPreview.offsetY
      };
    }

    const hovered = document
      .elementFromPoint(event.clientX, event.clientY)
      ?.closest<HTMLElement>("[data-model-row-id]");
    const hoveredId = hovered?.dataset.modelRowId;
    if (hoveredId) {
      moveModelInPreview(hoveredId);
      dragOverModelId = hoveredId;
    } else {
      dragOverModelId = null;
    }
  }

  async function finishModelDrag() {
    if (!selectedChannelId || !draggingModelId) {
      draggingModelId = null;
      dragOverModelId = null;
      dragPointerId = null;
      dragPreview = null;
      localModelOrder = [...selectedModels];
      return;
    }

    const currentIndex = selectedModels.findIndex((model) => model.model_id === draggingModelId);
    const finalIndex = localModelOrder.findIndex((model) => model.model_id === draggingModelId);
    if (currentIndex < 0 || finalIndex < 0) {
      draggingModelId = null;
      dragOverModelId = null;
      dragPointerId = null;
      dragPreview = null;
      localModelOrder = [...selectedModels];
      return;
    }

    if (currentIndex === finalIndex) {
      draggingModelId = null;
      dragOverModelId = null;
      dragPointerId = null;
      dragPreview = null;
      localModelOrder = [...selectedModels];
      return;
    }

    const movedModelId = draggingModelId;
    draggingModelId = null;
    dragOverModelId = null;
    dragPointerId = null;
    dragPreview = null;
    await persistModelOrder(selectedChannelId, localModelOrder, movedModelId);
  }

  async function endModelDrag(event?: PointerEvent) {
    if (event && dragPointerId !== event.pointerId) return;
    releaseDragPointerCapture();
    document.body.style.userSelect = "";
    document.body.style.cursor = "";
    await finishModelDrag();
  }

  function cancelModelDrag(event?: PointerEvent) {
    if (event && dragPointerId !== event.pointerId) return;
    releaseDragPointerCapture();
    document.body.style.userSelect = "";
    document.body.style.cursor = "";
    draggingModelId = null;
    dragOverModelId = null;
    dragPointerId = null;
    dragPreview = null;
    localModelOrder = [...selectedModels];
  }

  function closeModelDialog() {
    modelDialogOpen = false;
    editingModelId = null;
  }
</script>

<svelte:window
  onpointermove={updatePointerDragTarget}
  onpointerup={(event) => void endModelDrag(event)}
  onpointercancel={cancelModelDrag}
/>

<div class="space-y-5">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="max-w-2xl">
      <p class="text-sm font-medium text-[var(--ink-strong)]">连接模型服务、维护本地模型清单，并按需从远端拉取可选模型。</p>
      <p class="mt-1 text-xs text-[var(--ink-muted)]">选择左侧渠道后查看详情。新增、编辑和模型导入都通过弹窗完成，避免整页表单平铺。</p>
    </div>
    <Button type="button" size="sm" onclick={openCreateChannelDialog}>
      <Plus size={14} />
      新建渠道
    </Button>
  </div>

  <div class="grid gap-5 xl:grid-cols-[minmax(18rem,22rem)_minmax(0,1fr)]">
    <Card className="flex min-h-[38rem] flex-col overflow-hidden">
      <div class="border-b border-[var(--border-soft)] px-4 py-3">
        <SearchField bind:value={channelSearch} placeholder="搜索渠道..." />
      </div>

      <div class="app-scrollbar flex-1 overflow-y-auto p-3">
        {#if channelsLoading}
          <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-4 py-8 text-center text-sm text-[var(--ink-muted)]">
            正在读取 API 渠道...
          </div>
        {:else if visibleChannels.length === 0}
          <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-4 py-8 text-center text-sm text-[var(--ink-faint)]">
            还没有渠道，先创建一个连接。
          </div>
        {:else}
          <div class="space-y-2">
            {#each visibleChannels as channel (channel.id)}
              <Button
                type="button"
                variant="ghost"
                size="md"
                className={cn(
                  "h-auto w-full justify-start rounded-[var(--radius-lg)] border p-3 text-left transition-all hover:border-[var(--border-medium)] hover:shadow-[var(--shadow-sm)]",
                  selectedChannelId === channel.id
                    ? "border-[var(--brand)] bg-[var(--brand-soft)] text-[var(--ink-strong)] hover:bg-[var(--brand-soft)]"
                    : "border-[var(--border-soft)] bg-[var(--bg-surface)] text-[var(--ink-strong)]"
                )}
                onclick={() => {
                  selectedChannelId = channel.id;
                }}
              >
                <div class="flex items-start gap-3">
                  <div class="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-gradient-to-br from-cyan-400 to-cyan-600 text-sm font-bold text-white shadow-sm">
                    {channel.channel_type.charAt(0).toUpperCase()}
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <p class="truncate text-sm font-semibold text-[var(--ink-strong)]">{channel.name}</p>
                      {#if !channel.enabled}
                        <Badge className="bg-[var(--danger)]/10 text-[var(--danger)]">禁用</Badge>
                      {/if}
                    </div>
                    <p class="mt-1 truncate text-xs text-[var(--ink-faint)]">{channel.base_url}</p>
                    <div class="mt-2 flex items-center gap-2 text-[10px] text-[var(--ink-faint)]">
                      <Layers3 size={10} />
                      {(channelModels[channel.id] ?? []).length} 个本地模型
                    </div>
                  </div>
                </div>
              </Button>
            {/each}
          </div>
        {/if}
      </div>
    </Card>

    <div class="space-y-4">
      {#if selectedChannel}
        <Card className="p-5">
          <div class="flex flex-wrap items-start justify-between gap-4">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <h2 class="text-lg font-semibold text-[var(--ink-strong)]">{inlineChannelDraft.name || selectedChannel.name}</h2>
                <Badge>{inlineChannelDraft.channelType}</Badge>
                <Badge>{inlineChannelDraft.authType}</Badge>
                <Badge>{inlineChannelDraft.modelsMode}</Badge>
              </div>
              <p class="mt-2 text-xs text-[var(--ink-muted)]">常用字段直接在右侧编辑，不必再单独点“编辑渠道”。</p>
            </div>
            <div class="flex flex-wrap items-center gap-2">
              <Button type="button" variant="secondary" size="sm" onclick={saveInlineChannel} disabled={!inlineDraftDirty || savingInlineChannel || !inlineChannelDraft.baseUrl.trim()}>
                <Save size={14} />
                {savingInlineChannel ? "保存中..." : "保存更改"}
              </Button>
              <Button type="button" variant="destructive" size="sm" onclick={removeChannel} disabled={deletingChannel}>
                <Trash2 size={14} />
                删除
              </Button>
            </div>
          </div>

          <div class="mt-5 grid gap-4 md:grid-cols-2">
            <label class={labelClass}>
              <span class={labelTextClass}>名称</span>
              <input class={inputClass} bind:value={inlineChannelDraft.name} />
            </label>
            <label class={labelClass}>
              <span class={labelTextClass}>基础地址</span>
              <input class={inputClass} bind:value={inlineChannelDraft.baseUrl} />
            </label>
            <label class={labelClass}>
              <span class={labelTextClass}>模型列表路径</span>
              <input class={monoInputClass} bind:value={inlineChannelDraft.modelsEndpoint} />
            </label>
            <label class={labelClass}>
              <span class={labelTextClass}>聊天路径</span>
              <input class={monoInputClass} bind:value={inlineChannelDraft.chatEndpoint} />
            </label>
            <label class={labelClass}>
              <span class={labelTextClass}>流式路径</span>
              <input class={monoInputClass} bind:value={inlineChannelDraft.streamEndpoint} />
            </label>
            <label class={labelClass}>
              <span class={labelTextClass}>模型模式</span>
              <select class={inputClass} bind:value={inlineChannelDraft.modelsMode}>
                {#each modelsModeOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </label>
            <label class={`${labelClass} md:col-span-2`}>
              <span class={labelTextClass}>API Key</span>
              <div class="flex items-center gap-2">
                <input
                  class={monoInputClass}
                  bind:value={inlineChannelDraft.apiKey}
                  type={apiKeyVisible ? "text" : "password"}
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-10 w-10 px-0"
                  onclick={() => {
                    apiKeyVisible = !apiKeyVisible;
                  }}
                  title={apiKeyVisible ? "隐藏密钥" : "显示密钥"}
                >
                  {#if apiKeyVisible}
                    <EyeOff size={16} />
                  {:else}
                    <Eye size={16} />
                  {/if}
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  size="sm"
                  className="h-10 shrink-0"
                  onclick={openTestDialog}
                  disabled={selectedModels.length === 0 || testingChannel}
                >
                  {#if testingChannel}
                    <RefreshCw size={14} class="animate-spin" />
                  {/if}
                  检测
                </Button>
              </div>
              <p class="text-[11px] text-[var(--ink-faint)]">
                选择一个本地已保存模型后发送测试消息，结果会以 toast 返回。
              </p>
            </label>
            <label class={`${labelClass} flex items-center gap-2 md:col-span-2`}>
              <input type="checkbox" bind:checked={inlineChannelDraft.enabled} />
              <span class="text-sm text-[var(--ink-body)]">启用该渠道</span>
            </label>
          </div>
        </Card>

        <Card className="p-5">
          <div class="flex flex-wrap items-start justify-between gap-4">
            <div>
              <h3 class="text-sm font-semibold text-[var(--ink-strong)]">模型清单</h3>
              <p class="mt-1 text-xs text-[var(--ink-muted)]">先查看远端返回，再按需选择导入；本地自定义模型也通过弹窗维护。</p>
            </div>
            <div class="flex flex-wrap items-center gap-2">
              <Button type="button" variant="secondary" size="sm" onclick={() => void openRemoteModelsDialog()}>
                <RefreshCw size={14} />
                拉取模型
              </Button>
              <Button type="button" size="sm" onclick={openCreateModelDialog}>
                <Plus size={14} />
                手动添加
              </Button>
            </div>
          </div>

          <div class="mt-4">
            <SearchField bind:value={localModelSearch} placeholder="搜索本地模型..." />
          </div>

          <div class="mt-4 space-y-2" role="list">
            {#if selectedModels.length === 0}
              <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-4 py-8 text-center text-sm text-[var(--ink-faint)]">
                还没有本地模型，点击“拉取模型”从远端选择导入，或手动新增。
              </div>
            {:else if visibleLocalModels.length === 0}
              <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-4 py-8 text-center text-sm text-[var(--ink-faint)]">
                没有匹配的本地模型。
              </div>
            {:else}
              {#each visibleLocalModels as model (model.id)}
                <div
                  data-model-row-id={model.model_id}
                  role="listitem"
                  aria-grabbed={draggingModelId === model.model_id}
                  animate:flip={{ duration: 180, easing: cubicOut }}
                >
                  {#if draggingModelId === model.model_id}
                    <div
                      style={`height: ${dragPreview?.height ?? 84}px;`}
                      class={cn(
                        "rounded-[var(--radius-md)] border border-dashed bg-[var(--brand-soft)]/45",
                        dragOverModelId === model.model_id ? "border-[var(--brand)]" : "border-[var(--border-medium)]"
                      )}
                    ></div>
                  {:else}
                    <div
                      class={cn(
                        "flex items-center justify-between gap-3 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-4 py-3 transition-[transform,box-shadow,border-color,background-color] duration-150",
                        dragOverModelId === model.model_id && "border-[var(--brand)] bg-[var(--brand-soft)]"
                      )}
                    >
                      <div class="flex items-center gap-3">
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-8 w-8 cursor-grab touch-none border border-[var(--border-soft)] bg-[var(--bg-surface)] px-0 text-[var(--ink-faint)] active:cursor-grabbing"
                          onpointerdown={(event) => startModelDrag(event, model.model_id)}
                          onpointercancel={cancelModelDrag}
                          title="拖拽排序"
                          disabled={reorderingModelId !== null || deletingModel}
                        >
                          <GripVertical size={14} />
                        </Button>
                      </div>
                      <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                          <p class="truncate text-sm font-semibold text-[var(--ink-strong)]">{model.display_name?.trim() || model.model_id}</p>
                          {#if model.model_type}
                            <Badge>{model.model_type}</Badge>
                          {/if}
                        </div>
                        <p class="mt-1 truncate font-mono text-xs text-[var(--ink-faint)]">{model.model_id}</p>
                      </div>
                      <div class="flex items-center gap-1">
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-8 w-8 px-0 text-[var(--ink-faint)]"
                          onclick={() => openEditModelDialog(model)}
                          disabled={deletingModel || reorderingModelId === model.model_id}
                          title="编辑模型"
                        >
                          <Pencil size={15} />
                        </Button>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-8 w-8 px-0 text-[var(--danger)]"
                          onclick={() => void removeModel(model.model_id)}
                          disabled={deletingModel || reorderingModelId === model.model_id}
                        >
                          <Minus size={16} />
                        </Button>
                      </div>
                    </div>
                  {/if}
                </div>
              {/each}
            {/if}
          </div>
        </Card>
      {:else}
        <Card className="flex min-h-[24rem] items-center justify-center p-8">
          <div class="max-w-md text-center">
            <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-[var(--radius-xl)] bg-[var(--brand-soft)] text-[var(--brand)]">
              <Cable size={24} />
            </div>
            <h2 class="mt-4 text-base font-semibold text-[var(--ink-strong)]">选择一个渠道查看详情</h2>
            <p class="mt-2 text-sm text-[var(--ink-muted)]">左侧选择现有渠道，或直接新建一个模型服务连接。编辑和模型导入都在弹窗中完成。</p>
          </div>
        </Card>
      {/if}
    </div>
  </div>
</div>

{#if draggingModel && dragPreview}
  <div
    class="pointer-events-none fixed left-0 top-0 z-[160]"
    style={`width: ${dragPreview.width}px; height: ${dragPreview.height}px; transform: translate3d(${dragPreview.x}px, ${dragPreview.y}px, 0);`}
  >
    <div class="flex h-full items-center justify-between gap-3 rounded-[var(--radius-md)] border border-[var(--brand)] bg-[var(--bg-surface)] px-4 py-3 shadow-[var(--shadow-lg)] ring-1 ring-[var(--brand)]/15">
      <div class="flex items-center gap-3">
        <div class="flex h-8 w-8 items-center justify-center rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] text-[var(--ink-faint)]">
          <GripVertical size={14} />
        </div>
      </div>
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <p class="truncate text-sm font-semibold text-[var(--ink-strong)]">{draggingModel.display_name?.trim() || draggingModel.model_id}</p>
          {#if draggingModel.model_type}
            <Badge>{draggingModel.model_type}</Badge>
          {/if}
        </div>
        <p class="mt-1 truncate font-mono text-xs text-[var(--ink-faint)]">{draggingModel.model_id}</p>
      </div>
    </div>
  </div>
{/if}

<Dialog.Root bind:open={channelDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/20 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(720px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-lg)] outline-none">
      <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-4">
        <div>
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">新建 API 渠道</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">只在弹窗中编辑，避免主界面出现重复标题和冗长表单。</p>
        </div>
        <Button type="button" variant="ghost" size="sm" className="h-9 w-9 px-0" onclick={() => (channelDialogOpen = false)}>
          <X size={16} />
        </Button>
      </div>

      <div class="app-scrollbar max-h-[72dvh] overflow-y-auto px-6 py-5">
        <div class="grid gap-4 md:grid-cols-2">
          <label class={labelClass}>
            <span class={labelTextClass}>名称</span>
            <input class={inputClass} bind:value={channelDraft.name} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>基础地址</span>
            <input class={inputClass} bind:value={channelDraft.baseUrl} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>渠道类型</span>
            <select class={inputClass} bind:value={channelDraft.channelType}>
              {#each channelTypeOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>认证方式</span>
            <select class={inputClass} bind:value={channelDraft.authType}>
              {#each authTypeOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>API Key</span>
            <input class={monoInputClass} bind:value={channelDraft.apiKey} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>模型列表路径</span>
            <input class={inputClass} bind:value={channelDraft.modelsEndpoint} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>聊天路径</span>
            <input class={inputClass} bind:value={channelDraft.chatEndpoint} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>流式路径</span>
            <input class={inputClass} bind:value={channelDraft.streamEndpoint} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>模型模式</span>
            <select class={inputClass} bind:value={channelDraft.modelsMode}>
              {#each modelsModeOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>排序值</span>
            <input class={inputClass} type="number" bind:value={channelDraft.sortOrder} />
          </label>
          <label class={`${labelClass} flex items-center gap-2 pt-6 md:col-span-2`}>
            <input type="checkbox" bind:checked={channelDraft.enabled} />
            <span class="text-sm text-[var(--ink-body)]">启用该渠道</span>
          </label>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-6 py-4">
        <Button type="button" variant="secondary" onclick={() => (channelDialogOpen = false)}>取消</Button>
        <Button type="button" onclick={saveChannel} disabled={savingChannel || !channelDraft.baseUrl.trim()}>
          {savingChannel ? "保存中..." : "创建渠道"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<Dialog.Root bind:open={modelDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/20 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(640px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-lg)] outline-none">
      <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-4">
        <div>
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">{editingModelId ? "编辑模型" : "手动添加模型"}</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">
            {editingModelId
              ? "修改本地模型的显示信息、类型与排序。"
              : "用于补充接口没有返回、但你仍然想保留的本地模型。"}
          </p>
        </div>
        <Button type="button" variant="ghost" size="sm" className="h-9 w-9 px-0" onclick={closeModelDialog}>
          <X size={16} />
        </Button>
      </div>

      <div class="app-scrollbar max-h-[72dvh] overflow-y-auto px-6 py-5">
        <div class="grid gap-4 md:grid-cols-2">
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>模型 ID</span>
            <input class={monoInputClass} bind:value={modelDraft.modelId} readonly={!!editingModelId} />
          </label>
          <label class={`${labelClass} md:col-span-2`}>
            <span class={labelTextClass}>展示名称</span>
            <input class={inputClass} bind:value={modelDraft.displayName} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>模型类型</span>
            <input class={inputClass} bind:value={modelDraft.modelType} placeholder="chat / embed / image" />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>排序值</span>
            <input class={inputClass} type="number" bind:value={modelDraft.sortOrder} />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>上下文窗口</span>
            <input class={inputClass} bind:value={modelDraft.contextWindow} placeholder="128000" />
          </label>
          <label class={labelClass}>
            <span class={labelTextClass}>最大输出</span>
            <input class={inputClass} bind:value={modelDraft.maxOutputTokens} placeholder="8192" />
          </label>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-6 py-4">
        <Button type="button" variant="secondary" onclick={closeModelDialog}>取消</Button>
        <Button type="button" onclick={saveModel} disabled={savingModel || !modelDraft.modelId.trim()}>
          {savingModel ? "保存中..." : editingModelId ? "保存模型" : "添加模型"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<Dialog.Root bind:open={testDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/20 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(540px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-lg)] outline-none">
      <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-4">
        <div>
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">请选择要检测的模型</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">
            将使用所选模型发送一条测试消息，检查当前渠道是否可用。
          </p>
        </div>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-9 w-9 px-0"
          onclick={() => (testDialogOpen = false)}
          disabled={testingChannel}
        >
          <X size={16} />
        </Button>
      </div>

      <div class="space-y-4 px-6 py-5">
        <label class={labelClass}>
          <span class={labelTextClass}>检测模型</span>
          <select class={inputClass} bind:value={selectedTestModelId}>
            {#each selectedModels as model (model.id)}
              <option value={model.model_id}>
                {model.display_name?.trim() || model.model_id}
                {model.display_name?.trim() && model.display_name?.trim() !== model.model_id ? ` | ${model.model_id}` : ""}
              </option>
            {/each}
          </select>
        </label>
      </div>

      <div class="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-6 py-4">
        <Button
          type="button"
          variant="secondary"
          onclick={() => (testDialogOpen = false)}
          disabled={testingChannel}
        >
          取消
        </Button>
        <Button
          type="button"
          onclick={() => void runChannelTest()}
          disabled={testingChannel || !selectedTestModelId}
        >
          {testingChannel ? "检测中..." : "确定"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<Dialog.Root bind:open={remoteDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-[120] bg-black/20 backdrop-blur-sm" />
    <Dialog.Content class="fixed left-1/2 top-1/2 z-[130] w-[min(960px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[var(--bg-surface)] shadow-[var(--shadow-lg)] outline-none">
      <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-4">
        <div>
          <h2 class="text-lg font-semibold text-[var(--ink-strong)]">{selectedChannel?.name ?? "渠道"}模型</h2>
          <p class="mt-1 text-xs text-[var(--ink-muted)]">这里只展示接口返回的模型，不会自动全部写入本地。直接点击右侧 + / - 即可快速添加或移除。</p>
        </div>
        <div class="flex items-center gap-2">
          <Button type="button" variant="secondary" size="sm" onclick={() => void openRemoteModelsDialog()} disabled={remoteModelsLoading}>
            <RefreshCw size={14} class={cn(remoteModelsLoading && "animate-spin")} />
            重新拉取
          </Button>
          <Button type="button" variant="ghost" size="sm" className="h-9 w-9 px-0" onclick={() => (remoteDialogOpen = false)}>
            <X size={16} />
          </Button>
        </div>
      </div>

      <div class="px-6 py-4">
        <SearchField bind:value={remoteModelSearch} placeholder="搜索模型 ID 或名称..." />
        <div class="mt-3 flex flex-wrap items-center gap-2">
          <Button
            type="button"
            size="sm"
            variant={remoteModelFilter === "all" ? "default" : "secondary"}
            onclick={() => {
              remoteModelFilter = "all";
            }}
          >
            全部
          </Button>
          <Button
            type="button"
            size="sm"
            variant={remoteModelFilter === "saved" ? "default" : "secondary"}
            onclick={() => {
              remoteModelFilter = "saved";
            }}
          >
            已添加
          </Button>
          <Button
            type="button"
            size="sm"
            variant={remoteModelFilter === "available" ? "default" : "secondary"}
            onclick={() => {
              remoteModelFilter = "available";
            }}
          >
            未添加
          </Button>
        </div>
      </div>

      <div class="app-scrollbar max-h-[56dvh] overflow-y-auto px-6 pb-4">
        {#if remoteModelsLoading}
          <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-4 py-10 text-center text-sm text-[var(--ink-muted)]">
            正在从远端接口读取模型列表...
          </div>
        {:else if visibleRemoteModelGroups.length === 0}
          <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-4 py-10 text-center text-sm text-[var(--ink-faint)]">
            没有匹配的远端模型
          </div>
        {:else}
          <div class="space-y-2">
            {#each visibleRemoteModelGroups as group (group.id)}
              <div class="overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-app)]">
                <button
                  type="button"
                  class="flex w-full items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4 py-3 text-left transition-colors hover:bg-[var(--bg-surface)]"
                  onclick={() => toggleRemoteGroup(group.id)}
                >
                  <div class="flex min-w-0 items-center gap-2">
                    {#if remoteGroupOpen[group.id]}
                      <ChevronDown size={15} class="text-[var(--ink-faint)]" />
                    {:else}
                      <ChevronRight size={15} class="text-[var(--ink-faint)]" />
                    {/if}
                    <span class="truncate text-sm font-semibold text-[var(--ink-strong)]">{group.label}</span>
                    <Badge>{group.count}</Badge>
                    {#if group.savedCount > 0}
                      <Badge className="bg-emerald-500/10 text-emerald-600">{group.savedCount} 已添加</Badge>
                    {/if}
                  </div>
                  <span class="text-[11px] text-[var(--ink-faint)]">点击展开</span>
                </button>

                {#if remoteGroupOpen[group.id]}
                  <div class="space-y-2 p-3">
                    {#each group.models as model (model.model_id)}
                      {@const alreadyImported = existingModelIds.has(model.model_id)}
                      <div
                        class={cn(
                          "flex items-start justify-between gap-3 rounded-[var(--radius-md)] border px-4 py-3 transition-colors",
                          alreadyImported
                            ? "border-emerald-500/20 bg-emerald-500/5"
                            : "border-[var(--border-soft)] bg-[var(--bg-surface)] hover:border-[var(--border-medium)]"
                        )}
                      >
                        <div class="min-w-0 flex-1">
                          <div class="flex flex-wrap items-center gap-2">
                            <p class="text-sm font-semibold text-[var(--ink-strong)]">{model.display_name?.trim() || model.model_id}</p>
                            {#if model.model_type}
                              <Badge>{model.model_type}</Badge>
                            {/if}
                            {#if alreadyImported}
                              <Badge className="bg-emerald-500/10 text-emerald-600">已添加</Badge>
                            {/if}
                          </div>
                          <p class="mt-1 break-all font-mono text-xs text-[var(--ink-faint)]">{model.model_id}</p>
                        </div>
                        <Button
                          type="button"
                          variant={alreadyImported ? "secondary" : "ghost"}
                          size="sm"
                          className={cn(
                            "mt-0.5 h-9 w-9 flex-shrink-0 px-0",
                            alreadyImported
                              ? "border border-emerald-500/20 bg-emerald-500/10 text-emerald-700 hover:bg-emerald-500/15"
                              : "text-[var(--ink-muted)]"
                          )}
                          onclick={() => void toggleRemoteModel(model)}
                          disabled={syncingRemoteModelId === model.model_id}
                          title={alreadyImported ? "移除已保存模型" : "添加到本地模型"}
                        >
                          {#if syncingRemoteModelId === model.model_id}
                            <RefreshCw size={14} class="animate-spin" />
                          {:else if alreadyImported}
                            <Minus size={16} />
                          {:else}
                            <Plus size={16} />
                          {/if}
                        </Button>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-between border-t border-[var(--border-soft)] px-6 py-4">
        <p class="text-xs text-[var(--ink-muted)]">点击每一行右侧的 + / -，即可快速添加或移除已保存模型。</p>
        <Button type="button" variant="secondary" onclick={() => (remoteDialogOpen = false)}>关闭</Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
