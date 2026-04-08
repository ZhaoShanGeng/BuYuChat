<script lang="ts">
  /**
   * 渠道列表面板 — CherryStudio 风格：带首字母头像和 ON/OFF 状态。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Avatar from "$lib/components/ui/avatar/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import type { Channel } from "../../lib/transport/channels";
  import type { Notice } from "./channel-settings-state";

  type Props = {
    channels: Channel[];
    loading: boolean;
    notice: Notice | null;
    testingId: string | null;
    editingId: string | null;
    onCreate: () => void;
    onEdit: (channel: Channel) => void;
    onDelete: (id: string) => void | Promise<void>;
    onTest: (id: string) => void | Promise<void>;
  };

  const { channels, loading, notice, testingId, editingId, onCreate, onEdit, onDelete, onTest }: Props = $props();
</script>

<div class="flex h-full flex-col">
  <!-- 标题 + 添加按钮 -->
  <div class="flex h-12 items-center justify-between border-b px-4">
    <span class="text-sm font-medium">渠道</span>
  </div>

  <!-- 通知 -->
  {#if notice}
    <div class={`mx-3 mt-3 rounded-lg px-3 py-2 text-xs ${notice.kind === "success" ? "bg-emerald-500/10 text-emerald-600" : "bg-rose-500/10 text-rose-600"}`}>
      {notice.text}
    </div>
  {/if}

  <!-- 渠道列表 -->
  <div class="min-h-0 flex-1 overflow-y-auto p-2">
    {#if loading}
      <div class="px-3 py-8 text-center text-xs text-muted-foreground">正在加载...</div>
    {:else if channels.length === 0}
      <div class="px-3 py-8 text-center text-xs text-muted-foreground">还没有渠道</div>
    {:else}
      <div class="flex flex-col gap-0.5">
        {#each channels as channel}
          {@const isActive = editingId === channel.id}
          <button
            class={`flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition-colors ${
              isActive ? "bg-accent" : "hover:bg-accent/50"
            }`}
            onclick={() => onEdit(channel)}
            type="button"
          >
            <Avatar.Root class="size-8 shrink-0 rounded-lg text-xs font-bold">
              <Avatar.Fallback class={`rounded-lg ${channel.enabled ? "bg-emerald-600 text-white" : "bg-muted text-muted-foreground"}`}>
                {channel.name.charAt(0).toLowerCase()}
              </Avatar.Fallback>
            </Avatar.Root>
            <span class="min-w-0 flex-1 truncate text-[13px]">{channel.name}</span>
            {#if channel.enabled}
              <Badge class="shrink-0 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] font-semibold text-emerald-600" variant="secondary">ON</Badge>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- 底部添加按钮 -->
  <div class="border-t p-3">
    <Button class="w-full" onclick={onCreate} variant="outline" size="sm">
      <PlusIcon class="size-4" />
      添加
    </Button>
  </div>
</div>
