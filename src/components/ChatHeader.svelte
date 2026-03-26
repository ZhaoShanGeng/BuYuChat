<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import BotIcon from "@lucide/svelte/icons/bot";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import WaypointsIcon from "@lucide/svelte/icons/waypoints";
  import CpuIcon from "@lucide/svelte/icons/cpu";
  import type { Agent } from "../lib/transport/agents";
  import type { Channel } from "../lib/transport/channels";
  import type { Conversation } from "../lib/transport/conversations";
  import type { ChannelModel } from "../lib/transport/models";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import WindowControls from "./WindowControls.svelte";

  type Props = {
    conversation: Conversation | null;
    agents: Agent[];
    channels: Channel[];
    models: ChannelModel[];
    agentName: string;
    channelName: string;
    modelName: string;
    onQuickModelChange: (modelId: string) => void | Promise<void>;
    onQuickAgentChange: (agentId: string) => void | Promise<void>;
    onQuickChannelChange: (channelId: string) => void | Promise<void>;
    onQuickChannelMenuOpen: () => void | Promise<void>;
    onQuickTitleChange: (title: string) => void | Promise<void>;
  };

  const props: Props = $props();
  const currentWindow = getCurrentWindow();
  let titleEditing = $state(false);
  let titleInput = $state("");
  let channelMenuOpen = $state(false);

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

  function startTitleEdit() {
    titleInput = props.conversation?.title ?? "";
    titleEditing = true;
  }

  function cancelTitleEdit() {
    titleEditing = false;
    titleInput = props.conversation?.title ?? "";
  }

  function saveTitleEdit() {
    if (titleInput.trim()) {
      void props.onQuickTitleChange(titleInput.trim());
    }
    titleEditing = false;
  }

  $effect(() => {
    if (!channelMenuOpen) {
      return;
    }

    void props.onQuickChannelMenuOpen();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex min-h-14 min-w-0 shrink-0 items-center gap-2 border-b bg-background px-3 py-2 pr-4"
  onmousedown={handleHeaderMouseDown}
>
  {#if props.conversation}
    <div class="flex min-w-0 shrink items-center gap-2 overflow-hidden">
      <DropdownMenu.Root>
        <DropdownMenu.Trigger>
          {#snippet child({ props: triggerProps })}
            <Button
              {...triggerProps}
              class="h-8 min-w-0 max-w-[10rem] shrink gap-2 rounded-xl px-3 text-xs"
              size="sm"
              variant="outline"
            >
              <BotIcon class="size-3.5 text-muted-foreground" />
              <span class="truncate">{props.agentName || "选择 Agent"}</span>
              <ChevronDownIcon class="size-3 opacity-60" />
            </Button>
          {/snippet}
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="start" class="w-52">
          <DropdownMenu.Item onclick={() => props.onQuickAgentChange("")}>不绑定</DropdownMenu.Item>
          <DropdownMenu.Separator />
          {#each props.agents as agent}
            <DropdownMenu.Item onclick={() => props.onQuickAgentChange(agent.id)}>
              {agent.name}
            </DropdownMenu.Item>
          {/each}
        </DropdownMenu.Content>
      </DropdownMenu.Root>

      <DropdownMenu.Root bind:open={channelMenuOpen}>
        <DropdownMenu.Trigger>
          {#snippet child({ props: triggerProps })}
            <Button
              {...triggerProps}
              class="h-8 min-w-0 max-w-[10rem] shrink gap-2 rounded-xl px-3 text-xs"
              size="sm"
              variant="outline"
            >
              <WaypointsIcon class="size-3.5 text-muted-foreground" />
              <span class="truncate">{props.channelName || "选择渠道"}</span>
              <ChevronDownIcon class="size-3 opacity-60" />
            </Button>
          {/snippet}
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="start" class="w-52">
          <DropdownMenu.Item onclick={() => props.onQuickChannelChange("")}>不绑定</DropdownMenu.Item>
          <DropdownMenu.Separator />
          {#each props.channels as channel}
            <DropdownMenu.Item onclick={() => props.onQuickChannelChange(channel.id)}>
              {channel.name}
            </DropdownMenu.Item>
          {/each}
        </DropdownMenu.Content>
      </DropdownMenu.Root>

      <DropdownMenu.Root>
        <DropdownMenu.Trigger>
          {#snippet child({ props: triggerProps })}
            <Button
              {...triggerProps}
              class="h-8 min-w-0 max-w-[12rem] shrink gap-2 rounded-xl px-3 text-xs"
              size="sm"
              variant="outline"
            >
              <CpuIcon class="size-3.5 text-muted-foreground" />
              <span class="truncate">{props.modelName}</span>
              <ChevronDownIcon class="size-3 opacity-60" />
            </Button>
          {/snippet}
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="start" class="max-h-72 w-64 overflow-y-auto">
          {#if props.models.length === 0}
            <DropdownMenu.Item disabled>请先在设置中配置模型</DropdownMenu.Item>
          {:else}
            {#each props.models as model}
              <DropdownMenu.Item onclick={() => props.onQuickModelChange(model.id)}>
                {model.displayName ?? model.modelId}
              </DropdownMenu.Item>
            {/each}
          {/if}
        </DropdownMenu.Content>
      </DropdownMenu.Root>

      <div class="ml-1 min-w-0 w-[clamp(7rem,16vw,14rem)] shrink">
        {#if titleEditing}
          <Input
            bind:value={titleInput}
            class="h-8 w-full min-w-0 rounded-xl border-transparent bg-muted/70 text-sm shadow-none focus-visible:border-border"
            onblur={saveTitleEdit}
            onkeydown={(event) => {
              if (event.key === "Enter") saveTitleEdit();
              if (event.key === "Escape") cancelTitleEdit();
            }}
          />
        {:else}
          <button
            class="w-full truncate rounded-xl px-3 py-1.5 text-left text-sm font-medium text-foreground transition-colors hover:bg-muted/70"
            onclick={startTitleEdit}
            type="button"
          >
            {props.conversation.title}
          </button>
        {/if}
      </div>
    </div>
  {:else}
    <div class="min-w-0 shrink text-sm text-muted-foreground">选择或创建一个会话</div>
  {/if}

  <div aria-hidden="true" class="min-w-10 flex-1 self-stretch"></div>

  <div class="shrink-0">
    <WindowControls compact />
  </div>
</div>
