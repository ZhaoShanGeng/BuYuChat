<script lang="ts">
  import { onMount } from "svelte";
  import { Settings } from "lucide-svelte";
  import { listApiChannels, listApiChannelModels, type ApiChannel, type ApiChannelModel } from "$lib/api/api-channels";
  import { replaceConversationChannels } from "$lib/api/conversations";
  import { cn } from "$lib/utils";
  import { clickOutside } from "$lib/actions/click-outside";
  import { i18n } from "$lib/i18n.svelte";
  import { toast } from "svelte-sonner";

  let {
    conversationId,
    currentChannelId = null,
    currentModelId = null,
    onOpenSettings = undefined
  }: {
    conversationId: string;
    currentChannelId?: string | null;
    currentModelId?: string | null;
    onOpenSettings?: () => void;
  } = $props();

  let channels = $state<ApiChannel[]>([]);
  let modelsByChannel = $state<Record<string, ApiChannelModel[]>>({});
  let isOpen = $state(false);
  let loading = $state(true);

  const activeLabel = $derived.by(() => {
    if (!currentChannelId) return "选择渠道与模型";
    const ch = channels.find((c) => c.id === currentChannelId);
    if (!ch) return "未找到渠道";
    if (!currentModelId) return ch.name;
    const model = modelsByChannel[currentChannelId]?.find((m) => m.id === currentModelId);
    return `${ch.name} / ${model ? model.display_name || model.model_id : currentModelId}`;
  });

  async function loadData() {
    loading = true;
    try {
      const respChannels = await listApiChannels();
      channels = respChannels;
      for (const ch of channels) {
        const models = await listApiChannelModels(ch.id);
        modelsByChannel[ch.id] = models;
      }
    } catch (e) {
      console.error(e);
      toast.error("加载渠道与模型失败");
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });

  async function selectMapping(channelId: string, modelId: string) {
    if (!conversationId) {
      toast.error("请先创建对话");
      return;
    }
    try {
      await replaceConversationChannels(conversationId, [{
        channel_id: channelId,
        channel_model_id: modelId,
        binding_type: "active",
        enabled: true,
        sort_order: 0
      }]);
      toast.success("已切换聊天模型");
      isOpen = false;
    } catch (e) {
      console.error(e);
      toast.error("切换渠道/模型失败");
    }
  }
</script>

<div class="relative z-10 flex-shrink-0 hidden sm:block">
  <button
    type="button"
    class="flex items-center gap-1.5 rounded-[var(--radius-sm)] border border-[var(--border-medium)] bg-[var(--bg-surface)] px-2 py-1 text-xs font-medium text-[var(--ink-body)] hover:bg-[var(--bg-hover)]"
    onclick={() => (isOpen = !isOpen)}
  >
    <span class="truncate max-w-[140px]">{activeLabel}</span>
    <span class="text-[9px]">▼</span>
  </button>

  {#if isOpen}
    <div
      class="absolute top-full right-0 mt-1 max-h-[320px] w-[260px] overflow-auto rounded-[var(--radius-md)] border border-[var(--border-strong)] bg-[var(--bg-surface)] p-1 shadow-[var(--shadow-md)] animate-in fade-in slide-in-from-top-2 z-50 app-scrollbar"
      use:clickOutside={() => (isOpen = false)}
    >
      {#if loading}
        <div class="px-3 py-2 text-center text-xs text-[var(--ink-faint)]">加载中...</div>
      {:else}
        {#each channels as channel}
          <div class="px-2 py-1.5 text-xs font-semibold text-[var(--ink-muted)]">
            {channel.name} — {channel.channel_type}
          </div>
          {#each modelsByChannel[channel.id] || [] as model}
            <button
              type="button"
              class={cn(
                "flex w-full items-center justify-between rounded-sm px-3 py-1.5 text-left text-xs transition-colors hover:bg-[var(--bg-hover)]",
                channel.id === currentChannelId && model.id === currentModelId
                  ? "bg-[var(--bg-active)] text-[var(--brand)] font-semibold"
                  : "text-[var(--ink-body)]"
              )}
              onclick={() => selectMapping(channel.id, model.id)}
            >
              <span class="truncate">{model.display_name || model.model_id}</span>
              {#if channel.id === currentChannelId && model.id === currentModelId}
                <span>✓</span>
              {/if}
            </button>
          {/each}
          <div class="my-1 h-px bg-[var(--border-soft)]"></div>
        {/each}
        {#if onOpenSettings}
          <button
            type="button"
            class="flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-xs text-[var(--ink-muted)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)]"
            onclick={() => {
              isOpen = false;
              onOpenSettings();
            }}
          >
            <Settings size={14} /> 管理 API 渠道
          </button>
        {/if}
      {/if}
    </div>
  {/if}
</div>
