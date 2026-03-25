<script lang="ts">
  /**
   * 模型设置面板 — CherryStudio 风格：渠道选择 + 模型列表 + 模型详情。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import type { Channel } from "../lib/transport/channels";
  import type { ChannelModel, RemoteModelInfo } from "../lib/transport/models";
  import type { ModelFormState } from "./workspace-shell.svelte.js";

  type Props = {
    channels: Channel[];
    selectedChannelId: string;
    models: ChannelModel[];
    remoteModels: RemoteModelInfo[];
    modelsLoadingChannelId: string | null;
    remoteModelsLoadingChannelId: string | null;
    editingId: string | null;
    form: ModelFormState;
    saving: boolean;
    onSelectChannel: (channelId: string) => void | Promise<void>;
    onFetchRemoteModels: () => void | Promise<void>;
    onReset: () => void;
    onEdit: (model: ChannelModel) => void;
    onDelete: (id: string) => void | Promise<void>;
    onImportRemoteModel: (model: RemoteModelInfo) => void | Promise<void>;
    onFieldChange: (field: keyof ModelFormState, value: string) => void;
    onSubmit: (event: SubmitEvent) => void | Promise<void>;
  };

  const {
    channels, selectedChannelId, models, remoteModels,
    modelsLoadingChannelId, remoteModelsLoadingChannelId,
    editingId, form, saving,
    onSelectChannel, onFetchRemoteModels, onReset, onEdit, onDelete,
    onImportRemoteModel, onFieldChange, onSubmit
  }: Props = $props();
</script>

<div class="flex h-full min-h-0">
  <!-- 左侧：渠道选择 + 模型列表 -->
  <div class="flex w-72 shrink-0 flex-col border-r">
    <!-- 渠道选择器 -->
    <div class="border-b p-3">
      <select
        class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        onchange={(e) => void onSelectChannel((e.currentTarget as HTMLSelectElement).value)}
        value={selectedChannelId}
      >
        {#if channels.length === 0}
          <option value="">无渠道</option>
        {:else}
          {#each channels as channel}
            <option value={channel.id}>{channel.name}</option>
          {/each}
        {/if}
      </select>
    </div>

    <!-- 模型列表 -->
    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      {#if !selectedChannelId}
        <div class="px-3 py-8 text-center text-xs text-muted-foreground">请先创建渠道</div>
      {:else if modelsLoadingChannelId === selectedChannelId}
        <div class="px-3 py-8 text-center text-xs text-muted-foreground">加载中...</div>
      {:else if models.length === 0}
        <div class="px-3 py-8 text-center text-xs text-muted-foreground">暂无模型</div>
      {:else}
        <div class="flex flex-col gap-0.5">
          {#each models as model}
            {@const isActive = editingId === model.id}
            <button
              class={`flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition-colors ${
                isActive ? "bg-accent" : "hover:bg-accent/50"
              }`}
              onclick={() => onEdit(model)}
              type="button"
            >
              <div class="min-w-0 flex-1">
                <div class="truncate text-[13px]">{model.displayName ?? model.modelId}</div>
                {#if model.displayName}
                  <div class="truncate text-[11px] text-muted-foreground">{model.modelId}</div>
                {/if}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- 底部操作 -->
    <div class="flex gap-2 border-t p-3">
      <Button class="flex-1" onclick={onFetchRemoteModels} size="sm" variant="outline" disabled={!selectedChannelId || remoteModelsLoadingChannelId === selectedChannelId}>
        <DownloadIcon class="size-4" />
        {remoteModelsLoadingChannelId ? "拉取中" : "拉取"}
      </Button>
      <Button class="flex-1" onclick={onReset} size="sm" variant="outline">
        <PlusIcon class="size-4" />
        新建
      </Button>
    </div>
  </div>

  <!-- 右侧详情表单 -->
  <div class="min-h-0 flex-1 overflow-y-auto p-6">
    <div class="mb-6 flex items-center justify-between">
      <h2 class="text-base font-semibold">{editingId ? "编辑模型" : "新建模型"}</h2>
      {#if editingId}
        <Button onclick={() => onDelete(editingId)} size="sm" variant="destructive">删除</Button>
      {/if}
    </div>

    <form class="space-y-5" onsubmit={onSubmit}>
      <div class="space-y-2">
        <Label class="text-sm">模型 ID</Label>
        <Input
          oninput={(e) => onFieldChange("modelId", (e.currentTarget as HTMLInputElement).value)}
          placeholder="例如：gpt-4o-mini"
          value={form.modelId}
        />
      </div>

      <div class="space-y-2">
        <Label class="text-sm">显示名称</Label>
        <Input
          oninput={(e) => onFieldChange("displayName", (e.currentTarget as HTMLInputElement).value)}
          placeholder="例如：GPT-4o Mini"
          value={form.displayName}
        />
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <div class="space-y-2">
          <Label class="text-sm">上下文窗口</Label>
          <Input
            oninput={(e) => onFieldChange("contextWindow", (e.currentTarget as HTMLInputElement).value)}
            placeholder="128000"
            type="number"
            value={form.contextWindow}
          />
        </div>
        <div class="space-y-2">
          <Label class="text-sm">最大输出 Tokens</Label>
          <Input
            oninput={(e) => onFieldChange("maxOutputTokens", (e.currentTarget as HTMLInputElement).value)}
            placeholder="4096"
            type="number"
            value={form.maxOutputTokens}
          />
        </div>
      </div>

      <div class="flex items-center gap-3 pt-2">
        <Button disabled={saving || !selectedChannelId} type="submit">
          {saving ? "保存中..." : editingId ? "保存修改" : "创建模型"}
        </Button>
        <Button onclick={onReset} type="button" variant="outline">重置</Button>
      </div>
    </form>

    <!-- 远程模型候选 -->
    {#if remoteModels.length > 0}
      <div class="mt-8 border-t pt-6">
        <h3 class="mb-3 text-sm font-medium">远程模型候选 <Badge variant="secondary">{remoteModels.length}</Badge></h3>
        <div class="space-y-2">
          {#each remoteModels as model}
            <div class="flex items-center justify-between rounded-lg border px-4 py-2.5">
              <div>
                <div class="text-sm">{model.displayName ?? model.modelId}</div>
                <div class="text-xs text-muted-foreground">{model.modelId}</div>
              </div>
              <Button onclick={() => onImportRemoteModel(model)} size="sm" variant="outline">导入</Button>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
