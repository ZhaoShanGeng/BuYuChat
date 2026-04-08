<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import MenuIcon from "@lucide/svelte/icons/menu";
  import type { Agent } from "../../lib/transport/agents";
  import type { Channel } from "../../lib/transport/channels";
  import type { Conversation } from "../../lib/transport/conversations";
  import type { ChannelModel } from "../../lib/transport/models";
  import { getOptionalCurrentWindow } from "../../lib/tauri-window";
  import WindowControls from "../app-shell/WindowControls.svelte";

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
    isMobile?: boolean;
    onMenuToggle?: () => void;
  };

  const props: Props = $props();
  const currentWindow = getOptionalCurrentWindow();
  let titleEditing = $state(false);
  let titleInput = $state("");
  let agentPopoverOpen = $state(false);
  let channelPopoverOpen = $state(false);
  let modelPopoverOpen = $state(false);

  async function handleHeaderMouseDown(event: MouseEvent) {
    const target = event.target as HTMLElement | null;
    if (
      event.button !== 0 ||
      target?.closest("button, input, textarea, select, a, [role='button'], [data-no-drag]")
    ) {
      return;
    }

    if (!currentWindow) {
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
    if (!channelPopoverOpen) {
      return;
    }

    void props.onQuickChannelMenuOpen();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex min-h-14 min-w-0 shrink-0 flex-col justify-center border-b bg-background/80 px-4 py-2"
  onmousedown={handleHeaderMouseDown}
>
  <div class="flex items-center gap-2">
    {#if props.isMobile}
      <button
        class="flex size-9 shrink-0 items-center justify-center rounded-xl text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        onclick={props.onMenuToggle}
        title="菜单"
        type="button"
      >
        <MenuIcon class="size-5" />
      </button>
    {/if}

    {#if props.conversation}
      <div class="min-w-0 flex-1">
        <!-- 第一行：会话标题（大号、可编辑） -->
        {#if titleEditing}
          <Input
            bind:value={titleInput}
            class="h-8 w-full max-w-sm min-w-0 rounded-lg border-transparent bg-muted/70 text-sm font-semibold shadow-none focus-visible:border-border"
            onblur={saveTitleEdit}
            onkeydown={(event) => {
              if (event.key === "Enter") saveTitleEdit();
              if (event.key === "Escape") cancelTitleEdit();
            }}
          />
        {:else}
          <button
            class="max-w-md truncate text-left text-[15px] font-semibold text-foreground transition-colors hover:text-primary"
            onclick={startTitleEdit}
            title="点击编辑标题"
            type="button"
          >
            {props.conversation.title}
          </button>
        {/if}

        <!-- 第二行：面包屑上下文 (Agent · Channel · Model) -->
        <div class="mt-0.5 flex items-center gap-0 text-[12px] text-muted-foreground" data-no-drag>
          <!-- Agent 切换 -->
          <Popover.Root bind:open={agentPopoverOpen}>
            <Popover.Trigger>
              <button
                class="rounded-md px-1.5 py-0.5 transition-colors hover:bg-accent hover:text-foreground"
                type="button"
              >
                {props.agentName || "未绑定 Agent"}
              </button>
            </Popover.Trigger>
            <Popover.Content align="start" class="w-48 p-1.5">
              <div class="mb-1 px-2 text-[11px] font-medium text-muted-foreground">切换 Agent</div>
              <button
                class="flex w-full items-center rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-accent"
                onclick={() => { void props.onQuickAgentChange(""); agentPopoverOpen = false; }}
                type="button"
              >
                不绑定
              </button>
              {#each props.agents as agent}
                <button
                  class="flex w-full items-center rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-accent"
                  onclick={() => { void props.onQuickAgentChange(agent.id); agentPopoverOpen = false; }}
                  type="button"
                >
                  {agent.name}
                </button>
              {/each}
            </Popover.Content>
          </Popover.Root>

          <span class="text-muted-foreground/40 select-none">·</span>

          <!-- Channel 切换 -->
          <Popover.Root bind:open={channelPopoverOpen}>
            <Popover.Trigger>
              <button
                class="rounded-md px-1.5 py-0.5 transition-colors hover:bg-accent hover:text-foreground"
                type="button"
              >
                {props.channelName || "未选择渠道"}
              </button>
            </Popover.Trigger>
            <Popover.Content align="start" class="w-48 p-1.5">
              <div class="mb-1 px-2 text-[11px] font-medium text-muted-foreground">切换渠道</div>
              <button
                class="flex w-full items-center rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-accent"
                onclick={() => { void props.onQuickChannelChange(""); channelPopoverOpen = false; }}
                type="button"
              >
                不绑定
              </button>
              {#each props.channels as channel}
                <button
                  class="flex w-full items-center rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-accent"
                  onclick={() => { void props.onQuickChannelChange(channel.id); channelPopoverOpen = false; }}
                  type="button"
                >
                  {channel.name}
                </button>
              {/each}
            </Popover.Content>
          </Popover.Root>

          <span class="text-muted-foreground/40 select-none">·</span>

          <!-- Model 切换 -->
          <Popover.Root bind:open={modelPopoverOpen}>
            <Popover.Trigger>
              <button
                class="rounded-md px-1.5 py-0.5 transition-colors hover:bg-accent hover:text-foreground"
                type="button"
              >
                {props.modelName}
              </button>
            </Popover.Trigger>
            <Popover.Content align="start" class="max-h-64 w-56 overflow-y-auto p-1.5">
              <div class="mb-1 px-2 text-[11px] font-medium text-muted-foreground">切换模型</div>
              {#if props.models.length === 0}
                <div class="px-2 py-1.5 text-sm text-muted-foreground">请先在设置中配置模型</div>
              {:else}
                {#each props.models as model}
                  <button
                    class="flex w-full items-center rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-accent"
                    onclick={() => { void props.onQuickModelChange(model.id); modelPopoverOpen = false; }}
                    type="button"
                  >
                    {model.displayName ?? model.modelId}
                  </button>
                {/each}
              {/if}
            </Popover.Content>
          </Popover.Root>
        </div>
      </div>
    {:else}
      <div class="min-w-0 shrink text-sm text-muted-foreground">选择或创建一个会话</div>
    {/if}

    <div aria-hidden="true" class="min-w-4 flex-1 self-stretch md:min-w-10"></div>

    <div class="hidden shrink-0 md:block">
      <WindowControls compact />
    </div>
  </div>
</div>
