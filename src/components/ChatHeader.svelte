<script lang="ts">
  /**
   * 聊天顶栏 — [Model ▾] [Agent badge] · 标题  [⋯ 菜单]
   * 直接修改会话绑定，无需跳转设置页。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import EllipsisVerticalIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import BotIcon from "@lucide/svelte/icons/bot";
  import WaypointsIcon from "@lucide/svelte/icons/waypoints";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import type { Agent } from "../lib/transport/agents";
  import type { Channel } from "../lib/transport/channels";
  import type { Conversation } from "../lib/transport/conversations";
  import type { ChannelModel } from "../lib/transport/models";

  type Props = {
    conversation: Conversation | null;
    agents: Agent[];
    channels: Channel[];
    models: ChannelModel[];
    agentName: string;
    modelName: string;
    onQuickModelChange: (modelId: string) => void | Promise<void>;
    onQuickAgentChange: (agentId: string) => void | Promise<void>;
    onQuickChannelChange: (channelId: string) => void | Promise<void>;
    onQuickTitleChange: (title: string) => void | Promise<void>;
    onOpenSettings: () => void;
  };

  const props: Props = $props();
  let titleEditing = $state(false);
  let titleInput = $state("");

  /** 开始编辑标题。 */
  function startTitleEdit() {
    titleInput = props.conversation?.title ?? "";
    titleEditing = true;
  }

  /** 保存标题。 */
  function saveTitleEdit() {
    if (titleInput.trim()) void props.onQuickTitleChange(titleInput);
    titleEditing = false;
  }
</script>

{#if props.conversation}
  <div class="flex h-14 shrink-0 items-center gap-3 border-b bg-background/80 px-4 backdrop-blur-xl">
    <!-- 模型选择器 -->
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props: tProps })}
          <Button {...tProps} class="h-7 gap-1 px-2.5 text-xs" variant="outline" size="sm">
            {props.modelName}
            <ChevronDownIcon class="size-3 opacity-50" />
          </Button>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="start" class="max-h-64 w-56 overflow-y-auto">
        <DropdownMenu.Label>选择模型</DropdownMenu.Label>
        <DropdownMenu.Separator />
        {#if props.models.length === 0}
          <DropdownMenu.Item disabled>请先在渠道中添加模型</DropdownMenu.Item>
        {:else}
          {#each props.models as model}
            <DropdownMenu.Item onclick={() => props.onQuickModelChange(model.id)}>
              {model.displayName ?? model.modelId}
            </DropdownMenu.Item>
          {/each}
        {/if}
      </DropdownMenu.Content>
    </DropdownMenu.Root>

    <!-- Agent badge -->
    {#if props.agentName}
      <Badge class="h-6 gap-1 text-[11px]" variant="secondary">
        <BotIcon class="size-3" />
        {props.agentName}
      </Badge>
    {/if}

    <!-- 标题（点击编辑） -->
    {#if titleEditing}
      <div class="flex items-center gap-1.5">
        <Input
          bind:value={titleInput}
          class="h-7 w-40 text-xs"
          onkeydown={(e) => { if (e.key === "Enter") saveTitleEdit(); if (e.key === "Escape") titleEditing = false; }}
        />
        <Button class="h-6 px-2 text-[11px]" onclick={saveTitleEdit} size="sm">确定</Button>
      </div>
    {:else}
      <button class="truncate text-xs text-muted-foreground hover:text-foreground" onclick={startTitleEdit} type="button">
        {props.conversation.title}
      </button>
    {/if}

    <!-- 右侧操作 -->
    <div class="ml-auto flex items-center gap-0.5">
      <DropdownMenu.Root>
        <DropdownMenu.Trigger>
          {#snippet child({ props: tProps })}
            <Button {...tProps} class="size-7" size="icon" variant="ghost">
              <EllipsisVerticalIcon class="size-4" />
            </Button>
          {/snippet}
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="end" class="w-48">
          <DropdownMenu.Sub>
            <DropdownMenu.SubTrigger>
              <BotIcon class="text-muted-foreground" />
              切换 Agent
            </DropdownMenu.SubTrigger>
            <DropdownMenu.SubContent class="max-h-48 w-44 overflow-y-auto">
              <DropdownMenu.Item onclick={() => props.onQuickAgentChange("")}>
                <span class="text-muted-foreground">不绑定</span>
              </DropdownMenu.Item>
              {#each props.agents as agent}
                <DropdownMenu.Item onclick={() => props.onQuickAgentChange(agent.id)}>
                  {agent.name}
                </DropdownMenu.Item>
              {/each}
            </DropdownMenu.SubContent>
          </DropdownMenu.Sub>
          <DropdownMenu.Sub>
            <DropdownMenu.SubTrigger>
              <WaypointsIcon class="text-muted-foreground" />
              切换渠道
            </DropdownMenu.SubTrigger>
            <DropdownMenu.SubContent class="max-h-48 w-44 overflow-y-auto">
              <DropdownMenu.Item onclick={() => props.onQuickChannelChange("")}>
                <span class="text-muted-foreground">不绑定</span>
              </DropdownMenu.Item>
              {#each props.channels as channel}
                <DropdownMenu.Item onclick={() => props.onQuickChannelChange(channel.id)}>
                  {channel.name}
                </DropdownMenu.Item>
              {/each}
            </DropdownMenu.SubContent>
          </DropdownMenu.Sub>
          <DropdownMenu.Separator />
          <DropdownMenu.Item onclick={props.onOpenSettings}>
            <Settings2Icon class="text-muted-foreground" />
            更多设置
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    </div>
  </div>
{:else}
  <div class="flex h-12 shrink-0 items-center border-b px-4">
    <span class="text-sm text-muted-foreground">选择或创建一个会话</span>
  </div>
{/if}
