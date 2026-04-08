<script lang="ts">
  /**
   * 渠道详情表单 — CherryStudio 风格：右侧详情面板。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import * as Switch from "$lib/components/ui/switch/index.js";
  import type { ChannelInput } from "../../lib/transport/channels";

  type Props = {
    editingId: string | null;
    form: ChannelInput;
    saving: boolean;
    onReset: () => void;
    onSubmit: (event: SubmitEvent) => void | Promise<void>;
  };

  const { editingId, form, saving, onReset, onSubmit }: Props = $props();
  let enabledDraft = $state(true);

  $effect(() => {
    enabledDraft = form.enabled ?? true;
  });

  $effect(() => {
    form.enabled = enabledDraft;
  });
</script>

<div class="p-6">
  <!-- 标题 + 启用开关 -->
  <div class="mb-6 flex items-center justify-between">
    <h2 class="text-base font-semibold">
      {editingId ? form.name || "编辑渠道" : "新建渠道"}
    </h2>
    <div class="flex items-center">
      <Switch.Root bind:checked={enabledDraft} />
    </div>
  </div>

  <form class="space-y-5" onsubmit={onSubmit}>
    <!-- API 密钥 -->
    <div class="space-y-2">
      <Label class="text-sm">API 密钥</Label>
      <div class="flex gap-2">
        <Input bind:value={form.apiKey} class="flex-1" placeholder="sk-..." type="password" />
        <Button disabled={!editingId} size="sm" type="button" variant="outline">检测</Button>
      </div>
    </div>

    <!-- API 地址 -->
    <div class="space-y-2">
      <Label class="text-sm">API 地址</Label>
      <Input bind:value={form.baseUrl} placeholder="https://api.openai.com" />
      {#if form.baseUrl}
        <p class="text-xs text-muted-foreground">预览：{form.baseUrl}{form.chatEndpoint || "/v1/chat/completions"}</p>
      {/if}
    </div>

    <!-- 名称 -->
    <div class="space-y-2">
      <Label class="text-sm">名称</Label>
      <Input bind:value={form.name} placeholder="例如：OpenAI" />
    </div>

    <!-- 鉴权方式 -->
    <div class="space-y-2">
      <Label class="text-sm">鉴权方式</Label>
      <select bind:value={form.authType} class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
        <option value="bearer">Bearer Token</option>
        <option value="x_api_key">X-API-Key</option>
        <option value="none">无鉴权</option>
      </select>
    </div>

    <!-- 端点（折叠区） -->
    <details class="group">
      <summary class="cursor-pointer text-sm text-muted-foreground hover:text-foreground">高级端点设置</summary>
      <div class="mt-3 space-y-4">
        <div class="space-y-2">
          <Label class="text-xs text-muted-foreground">Models Endpoint</Label>
          <Input bind:value={form.modelsEndpoint} placeholder="/v1/models" />
        </div>
        <div class="space-y-2">
          <Label class="text-xs text-muted-foreground">Chat Endpoint</Label>
          <Input bind:value={form.chatEndpoint} placeholder="/v1/chat/completions" />
        </div>
        <div class="space-y-2">
          <Label class="text-xs text-muted-foreground">Stream Endpoint</Label>
          <Input bind:value={form.streamEndpoint} placeholder="/v1/chat/completions" />
        </div>
      </div>
    </details>

    <!-- 操作按钮 -->
    <div class="flex items-center gap-3 pt-2">
      <Button disabled={saving} type="submit">
        {saving ? "保存中..." : editingId ? "保存修改" : "创建渠道"}
      </Button>
      <Button onclick={onReset} type="button" variant="outline">重置</Button>
    </div>
  </form>
</div>
