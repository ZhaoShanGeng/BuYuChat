<script lang="ts">
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import GlobeIcon from "@lucide/svelte/icons/globe";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import ServerCogIcon from "@lucide/svelte/icons/server-cog";
  import SparklesIcon from "@lucide/svelte/icons/sparkles";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import type { ChannelModel, RemoteModelInfo } from "../lib/transport/models";

  type Props = {
    selectedChannelId: string | null;
    models: ChannelModel[];
    groupedModels: ReadonlyArray<readonly [string, ChannelModel[]]>;
    remoteModels: RemoteModelInfo[];
    loadingModels: boolean;
    loadingRemoteModels: boolean;
    managingModels: boolean;
    addingModel: boolean;
    newModelId: string;
    newModelDisplayName: string;
    onToggleManaging: () => void;
    onToggleAdding: () => void;
    onNewModelIdChange: (value: string) => void;
    onNewModelDisplayNameChange: (value: string) => void;
    onCreateModel: () => void | Promise<void>;
    onDeleteModel: (id: string) => void | Promise<void>;
    onFetchRemoteModels: () => void | Promise<void>;
    onImportRemoteModel: (model: RemoteModelInfo) => void | Promise<void>;
  };

  const props: Props = $props();
</script>

<section class="settings-page__models-card rounded-3xl border bg-card p-6 shadow-sm" data-ui="settings-model-manager">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="flex items-center gap-2">
      <div class="flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
        <ServerCogIcon class="size-4" />
      </div>
      <div>
        <div class="text-sm font-medium">模型列表</div>
        <div class="text-xs text-muted-foreground">
          {props.selectedChannelId ? "管理当前渠道下可用的模型" : "请先创建并保存渠道"}
        </div>
      </div>
      <Badge variant="outline">{props.models.length}</Badge>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <Button
        class="rounded-xl px-4"
        disabled={!props.selectedChannelId || props.loadingRemoteModels}
        onclick={() => void props.onFetchRemoteModels()}
        type="button"
        variant="outline"
      >
        <GlobeIcon class="mr-1 size-4" />
        {props.loadingRemoteModels ? "拉取中..." : "拉取远程模型"}
      </Button>
      <Button
        class="rounded-xl px-4"
        disabled={!props.selectedChannelId}
        onclick={props.onToggleManaging}
        type="button"
        variant={props.managingModels ? "default" : "outline"}
      >
        管理
      </Button>
      <Button
        class="rounded-xl px-4"
        disabled={!props.selectedChannelId}
        onclick={props.onToggleAdding}
        type="button"
        variant="outline"
      >
        <PlusIcon class="mr-1 size-4" />
        添加模型
      </Button>
    </div>
  </div>

  {#if props.addingModel}
    <div class="mt-5 grid gap-3 rounded-2xl border border-dashed p-4 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto]">
      <Input
        class="h-10 rounded-xl"
        oninput={(event) => props.onNewModelIdChange((event.currentTarget as HTMLInputElement).value)}
        placeholder="模型 ID，例如 gpt-4o-mini"
        value={props.newModelId}
      />
      <Input
        class="h-10 rounded-xl"
        oninput={(event) =>
          props.onNewModelDisplayNameChange((event.currentTarget as HTMLInputElement).value)}
        placeholder="显示名称（可选）"
        value={props.newModelDisplayName}
      />
      <Button class="h-10 rounded-xl px-4" onclick={() => void props.onCreateModel()} type="button">
        保存模型
      </Button>
    </div>
  {/if}

  <div class="mt-5">
    {#if !props.selectedChannelId}
      <div class="rounded-2xl border border-dashed p-8 text-center text-sm text-muted-foreground">
        先保存一个渠道，再继续配置模型。
      </div>
    {:else if props.loadingModels}
      <div class="rounded-2xl border border-dashed p-8 text-center text-sm text-muted-foreground">
        模型加载中...
      </div>
    {:else if props.groupedModels.length === 0}
      <div class="rounded-2xl border border-dashed p-8 text-center text-sm text-muted-foreground">
        当前渠道还没有模型，可以手动添加或从远程拉取。
      </div>
    {:else}
      <div class="space-y-3">
        {#each props.groupedModels as [group, items]}
          <div class="settings-page__model-group overflow-hidden rounded-2xl border bg-muted/20" data-ui="settings-model-group">
            <div class="flex items-center gap-2 border-b px-4 py-3 text-sm font-medium">
              <SparklesIcon class="size-4 text-muted-foreground" />
              <span>{group}</span>
              <Badge variant="outline">{items.length}</Badge>
            </div>
            <div class="space-y-2 p-3">
              {#each items as model (model.id)}
                <div class="settings-page__model-card flex items-center gap-3 rounded-xl border bg-background px-3 py-3" data-ui="settings-model-card">
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
                  {#if props.managingModels}
                    <Button
                      class="size-8 rounded-xl"
                      onclick={() => void props.onDeleteModel(model.id)}
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

  {#if props.remoteModels.length > 0}
    <div class="settings-page__remote-models mt-5 rounded-2xl border border-dashed p-4" data-ui="settings-remote-models">
      <div class="flex items-center gap-2 text-sm font-medium">
        <GlobeIcon class="size-4 text-muted-foreground" />
        <span>远程模型候选</span>
        <Badge variant="outline">{props.remoteModels.length}</Badge>
      </div>

      <div class="mt-3 space-y-2">
        {#each props.remoteModels as model (model.modelId)}
          <div class="settings-page__remote-model-card flex flex-wrap items-center gap-3 rounded-xl border bg-background px-3 py-3" data-ui="settings-remote-model-card">
            <div class="min-w-0 flex-1">
              <div class="truncate text-sm font-medium">
                {model.displayName ?? model.modelId}
              </div>
              <div class="truncate text-xs text-muted-foreground">{model.modelId}</div>
            </div>
            <Button
              class="rounded-xl px-4"
              onclick={() => void props.onImportRemoteModel(model)}
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
</section>
