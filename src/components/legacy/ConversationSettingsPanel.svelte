<script lang="ts">
  /**
   * 当前会话设置面板 — shadcn 组件，简洁表单。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import type { Agent } from "../../lib/transport/agents";
  import type { Channel } from "../../lib/transport/channels";
  import type { Conversation } from "../../lib/transport/conversations";
  import type { ChannelModel } from "../../lib/transport/models";
  import type { ConversationDraft } from "../app-shell/workspace-shell.svelte.js";

  type Props = {
    conversation: Conversation | null;
    draft: ConversationDraft;
    agents: Agent[];
    channels: Channel[];
    models: ChannelModel[];
    saving: boolean;
    onTitleChange: (value: string) => void;
    onAgentChange: (value: string) => void;
    onChannelChange: (value: string) => void;
    onModelChange: (value: string) => void;
    onSubmit: (event: SubmitEvent) => void | Promise<void>;
  };

  const {
    conversation, draft, agents, channels, models, saving,
    onTitleChange, onAgentChange, onChannelChange, onModelChange, onSubmit
  }: Props = $props();

  /** 统一 select 样式。 */
  const selectClass = "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring";
</script>

<div class="p-6">
  <h2 class="mb-6 text-base font-semibold">当前会话设置</h2>

  {#if !conversation}
    <div class="py-12 text-center text-sm text-muted-foreground">
      请先选择一个会话
    </div>
  {:else}
    <form class="max-w-lg space-y-5" onsubmit={onSubmit}>
      <div class="space-y-2">
        <Label class="text-sm">会话标题</Label>
        <Input
          oninput={(e) => onTitleChange((e.currentTarget as HTMLInputElement).value)}
          value={draft.title}
        />
      </div>

      <div class="space-y-2">
        <Label class="text-sm">绑定 Agent</Label>
        <select class={selectClass} onchange={(e) => onAgentChange((e.currentTarget as HTMLSelectElement).value)} value={draft.agentId}>
          <option value="">未绑定</option>
          {#each agents as agent}
            <option value={agent.id}>{agent.name}</option>
          {/each}
        </select>
      </div>

      <div class="space-y-2">
        <Label class="text-sm">绑定渠道</Label>
        <select class={selectClass} onchange={(e) => onChannelChange((e.currentTarget as HTMLSelectElement).value)} value={draft.channelId}>
          <option value="">未绑定</option>
          {#each channels as channel}
            <option value={channel.id}>{channel.name}</option>
          {/each}
        </select>
      </div>

      <div class="space-y-2">
        <Label class="text-sm">绑定模型</Label>
        <select class={selectClass} onchange={(e) => onModelChange((e.currentTarget as HTMLSelectElement).value)} value={draft.modelId}>
          <option value="">未绑定</option>
          {#each models as model}
            <option value={model.id}>{model.displayName ?? model.modelId}</option>
          {/each}
        </select>
      </div>

      <div class="pt-2">
        <Button disabled={saving} type="submit">
          {saving ? "保存中..." : "保存"}
        </Button>
      </div>
    </form>
  {/if}
</div>
