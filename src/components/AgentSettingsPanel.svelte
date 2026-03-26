<script lang="ts">
  /**
   * Agent 编辑器 — 主内容区，表单编辑 Agent 属性。
   * 列表已在 WorkspaceShell 的 ContextPanel 中渲染。
   */
  import { Button } from "$lib/components/ui/button/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import * as Switch from "$lib/components/ui/switch/index.js";
  import * as Textarea from "$lib/components/ui/textarea/index.js";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { Agent } from "../lib/transport/agents";
  import type { AgentFormState } from "./workspace-shell.svelte.js";
  import WindowControls from "./WindowControls.svelte";

  type Props = {
    agents: Agent[];
    editingId: string | null;
    isCreating?: boolean;
    form: AgentFormState;
    saving: boolean;
    onNameChange: (value: string) => void;
    onSystemPromptChange: (value: string) => void;
    onReset: () => void;
    onEdit: (agent: Agent) => void;
    onDelete: (id: string) => void | Promise<void>;
    onSubmit: (event: SubmitEvent) => void | Promise<void>;
    onToggleEnabled: (agent: Agent) => void | Promise<void>;
  };

  const {
    agents, editingId, isCreating = false, form, saving,
    onNameChange, onSystemPromptChange, onReset, onEdit, onDelete, onSubmit, onToggleEnabled
  }: Props = $props();

  const currentWindow = getCurrentWindow();
  let editingAgent = $derived(editingId ? agents.find((a) => a.id === editingId) : null);
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

  async function handleToggleAgent(nextChecked: boolean) {
    if (!editingAgent) {
      return;
    }

    if (nextChecked !== editingAgent.enabled) {
      await onToggleEnabled(editingAgent);
    }
  }
</script>

<div class="flex h-full flex-col">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="flex h-14 items-center justify-between border-b px-6" onmousedown={handleHeaderMouseDown}>
    <h2 class="text-sm font-semibold">{editingId ? "编辑 Agent" : isCreating ? "新建 Agent" : "Agent"}</h2>
    <div class="flex items-center gap-3">
      {#if editingAgent}
        <div class="flex items-center rounded-full bg-muted px-3 py-1.5">
          <Switch.Root checked={editingAgent.enabled} onclick={() => void handleToggleAgent(!editingAgent.enabled)} />
        </div>
        <Button onclick={() => onDelete(editingAgent!.id)} size="sm" variant="destructive">删除</Button>
      {:else if isCreating}
        <Badge class="rounded-full border border-blue-200 bg-blue-50 text-blue-700" variant="outline">新建</Badge>
      {/if}
      <WindowControls compact />
    </div>
  </div>

  {#if !editingId && !isCreating}
    <!-- 空状态 -->
    <div class="flex flex-1 items-center justify-center">
      <div class="text-center">
        <p class="text-sm text-muted-foreground">选择或创建一个 Agent</p>
      </div>
    </div>
  {:else}
    <!-- 编辑器 -->
    <div class="flex-1 overflow-y-auto p-6">
      <form class="max-w-lg space-y-5" onsubmit={onSubmit}>
        <div class="space-y-2">
          <Label class="text-sm">名称</Label>
          <Input
            oninput={(e) => onNameChange((e.currentTarget as HTMLInputElement).value)}
            placeholder="例如：研究助手"
            value={form.name}
          />
        </div>

        <div class="space-y-2">
          <Label class="text-sm">系统提示词</Label>
          <Textarea.Root
            class="min-h-[240px] resize-y"
            oninput={(e) => onSystemPromptChange((e.currentTarget as HTMLTextAreaElement).value)}
            placeholder="描述这个 Agent 的职责、语气和约束。"
            value={form.systemPrompt}
          />
        </div>

        <div class="flex items-center gap-3 pt-2">
          <Button disabled={saving} type="submit">
            {saving ? "保存中..." : editingId ? "保存" : "创建"}
          </Button>
          <Button onclick={onReset} type="button" variant="outline">重置</Button>
        </div>
      </form>
    </div>
  {/if}
</div>
