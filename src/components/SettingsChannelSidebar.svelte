<script lang="ts">
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import SearchIcon from "@lucide/svelte/icons/search";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import type { Channel } from "../lib/transport/channels";
  import SettingsNoticeBanner from "./SettingsNoticeBanner.svelte";
  import type { Notice } from "./settings-page.types";

  type Props = {
    channels: Channel[];
    loading: boolean;
    selectedChannelId: string | null;
    selectedChannelEnabled: boolean;
    notice: Notice | null;
    search: string;
    onCreate: () => void | Promise<void>;
    onSelect: (channel: Channel) => void | Promise<void>;
  };

  let {
    channels,
    loading,
    selectedChannelId,
    selectedChannelEnabled,
    notice,
    search = $bindable(),
    onCreate,
    onSelect
  }: Props = $props();
</script>

<aside class="flex h-full shrink-0 flex-col bg-muted/20" data-ui="settings-channel-sidebar">
  <div class="settings-page__sidebar-header border-b p-4">
    <div class="relative">
      <SearchIcon class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
      <Input
        bind:value={search}
        class="settings-page__search-input h-10 rounded-xl bg-background pl-10"
        placeholder="搜索渠道名称或地址"
      />
    </div>

    <Button class="settings-page__create-channel mt-3 w-full rounded-xl" onclick={() => void onCreate()} variant="outline">
      <PlusIcon class="mr-1 size-4" />
      添加渠道
    </Button>
  </div>

  {#if notice}
    <div class="px-4 pt-4">
      <SettingsNoticeBanner {notice} />
    </div>
  {/if}

  <div class="settings-page__sidebar-body min-h-0 flex-1 overflow-y-auto p-3">
    {#if loading}
      <div class="rounded-2xl border border-dashed p-6 text-center text-sm text-muted-foreground">
        渠道加载中...
      </div>
    {:else if channels.length === 0}
      <div class="rounded-2xl border border-dashed p-6 text-center text-sm text-muted-foreground">
        {search.trim() ? "没有匹配的渠道" : "还没有渠道，先创建一个。"}
      </div>
    {:else}
      <div class="space-y-1.5">
        {#each channels as channel (channel.id)}
          {@const isSelected = selectedChannelId === channel.id}
          <button
            class={`settings-page__channel-card flex w-full items-center gap-3 rounded-2xl border px-3 py-3 text-left transition-colors ${
              isSelected
                ? "border-border bg-background shadow-sm"
                : "border-transparent hover:bg-background"
            }`}
            data-active={isSelected}
            data-ui="settings-channel-card"
            onclick={() => void onSelect(channel)}
            type="button"
          >
            <div class="settings-page__channel-avatar flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-sm font-semibold text-primary">
              {channel.name.slice(0, 1).toUpperCase()}
            </div>
            <div class="min-w-0 flex-1">
              <div class="truncate text-sm font-medium">{channel.name}</div>
              <div class="truncate text-xs text-muted-foreground">{channel.baseUrl}</div>
            </div>
            <Badge variant="outline">
              {isSelected ? (selectedChannelEnabled ? "启用" : "禁用") : (channel.enabled ? "启用" : "禁用")}
            </Badge>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</aside>
