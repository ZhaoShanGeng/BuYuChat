<script lang="ts">
  import * as Avatar from "$lib/components/ui/avatar/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import type { Agent } from "../../lib/transport/agents";

  type Props = {
    agent: Agent;
    isActive: boolean;
    onSelect: (agent: Agent) => void | Promise<void>;
  };

  const props: Props = $props();
</script>

<button
  class={`agent-sidebar__item flex w-full items-center gap-2.5 rounded-2xl px-3 py-2.5 text-left transition-colors ${
    props.isActive ? "bg-accent" : "hover:bg-accent/50"
  }`}
  data-active={props.isActive}
  data-ui="agent-sidebar-item"
  onclick={() => void props.onSelect(props.agent)}
  type="button"
>
  <Avatar.Root class="size-8 shrink-0 rounded-xl text-[11px] font-bold">
    <Avatar.Fallback class={`rounded-xl ${props.agent.enabled ? "bg-violet-600 text-white" : "bg-muted text-muted-foreground"}`}>
      {props.agent.name.charAt(0)}
    </Avatar.Fallback>
  </Avatar.Root>

  <div class="agent-sidebar__item-main min-w-0 flex-1">
    <span class="agent-sidebar__item-title truncate text-[13px] font-medium">{props.agent.name}</span>
  </div>

  <Badge
    class={`agent-sidebar__item-badge rounded-full border px-2 py-0.5 text-[10px] ${
      props.agent.enabled
        ? "border-emerald-200 bg-emerald-50 text-emerald-700"
        : "border-slate-200 bg-slate-100 text-slate-500"
    }`}
    variant="outline"
  >
    {props.agent.enabled ? "启用" : "禁用"}
  </Badge>
</button>
