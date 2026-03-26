<script lang="ts">
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import * as Switch from "$lib/components/ui/switch/index.js";
  import KeyRoundIcon from "@lucide/svelte/icons/key-round";
  import TestTubeDiagonalIcon from "@lucide/svelte/icons/test-tube-diagonal";
  import type { Channel } from "../lib/transport/channels";
  import type {
    ChannelFormState,
    SelectOption
  } from "./settings-page.types";

  type Props = {
    selectedChannel: Channel | null;
    selectedChannelId: string | null;
    form: ChannelFormState;
    saving: boolean;
    testingId: string | null;
    channelTypeOptions: SelectOption[];
    authTypeOptions: SelectOption[];
    onSave: (event: SubmitEvent) => void | Promise<void>;
    onReset: () => void | Promise<void>;
    onDelete: () => void | Promise<void>;
    onTest: () => void | Promise<void>;
  };

  let {
    selectedChannel,
    selectedChannelId,
    form = $bindable(),
    saving,
    testingId,
    channelTypeOptions,
    authTypeOptions,
    onSave,
    onReset,
    onDelete,
    onTest
  }: Props = $props();
</script>

<section class="settings-page__editor flex flex-col gap-6" data-ui="settings-channel-editor">
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

  <form class="settings-page__form-card rounded-3xl border bg-card p-6 shadow-sm" onsubmit={onSave}>
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
          {#each channelTypeOptions as option}
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
          {#each authTypeOptions as option}
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
          onclick={() => void onTest()}
          type="button"
          variant="outline"
        >
          <TestTubeDiagonalIcon class="mr-1 size-4" />
          {testingId === selectedChannelId ? "检测中..." : "检测"}
        </Button>
      </div>
    </div>

    <div class="settings-page__advanced-card mt-5 rounded-2xl border bg-muted/30 p-4" data-ui="settings-channel-advanced">
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
      <Button class="rounded-xl px-5" onclick={() => void onReset()} type="button" variant="outline">
        重置
      </Button>
      {#if selectedChannelId}
        <Button class="rounded-xl px-5" onclick={() => void onDelete()} type="button" variant="destructive">
          删除
        </Button>
      {/if}
    </div>
  </form>
</section>
