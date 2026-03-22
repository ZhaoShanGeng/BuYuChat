<script lang="ts">
  import { onMount } from "svelte";
  import { Plus, Trash2 } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import { listPlugins, getPluginDetail, createPlugin, updatePlugin, deletePlugin, type PluginSummary, type PluginDetail } from "$lib/api/plugins";
  import Button from "$components/ui/button.svelte";
  import SelectDropdown from "$components/shared/select-dropdown.svelte";
  import TagInput from "$components/shared/tag-input.svelte";
  import { cn } from "$lib/utils";

  let plugins = $state<PluginSummary[]>([]);
  let activePlugin = $state<PluginDetail | null>(null);
  let selectedId = $state<string | null>(null);
  let loadingList = $state(true);
  let saving = $state(false);

  type Draft = { name: string; pluginKey: string; version: string; runtime: string; entryPoint: string; capabilities: string[]; permissionsJson: string; enabled: boolean; configJson: string; };
  let draft = $state<Draft>({ name: "", pluginKey: "", version: "1.0.0", runtime: "script", entryPoint: "", capabilities: [], permissionsJson: "{}", enabled: true, configJson: "{}" });

  const runtimeOpts = [{value:"script",label:"Script"},{value:"wasm",label:"WASM"},{value:"native",label:"Native"}];

  onMount(loadData);

  async function loadData(sel?: string | null) {
    loadingList = true;
    try {
      plugins = await listPlugins();
      if (sel) await selPlugin(sel);
      else if (!selectedId && plugins.length > 0) await selPlugin(plugins[0].id);
    } catch {
      plugins = [{ id: "mock-plugin-1", name: "Web Search", plugin_key: "web_search", version: "1.0.0", runtime: "script", entry_point: "main.py", enabled: true, capabilities: ["tool"], config_json: {}, created_at: 0, updated_at: 0 }];
      if (sel) await selPlugin(sel); else await selPlugin(plugins[0].id);
    } finally { loadingList = false; }
  }

  async function selPlugin(id: string) {
    selectedId = id;
    try {
      activePlugin = await getPluginDetail(id);
      draft = { name: activePlugin.name, pluginKey: activePlugin.plugin_key, version: activePlugin.version || "", runtime: activePlugin.runtime, entryPoint: activePlugin.entry_point || "", capabilities: activePlugin.capabilities || [], permissionsJson: JSON.stringify(activePlugin.permissions || {}, null, 2), enabled: activePlugin.enabled, configJson: JSON.stringify(activePlugin.config_json || {}, null, 2) };
    } catch {
      const sum = plugins.find(x=>x.id===id);
      if(sum) {
        activePlugin = { ...sum, permissions: {"fs":["read"]} };
        draft = { name: sum.name, pluginKey: sum.plugin_key, version: sum.version || "", runtime: sum.runtime, entryPoint: sum.entry_point || "", capabilities: sum.capabilities || [], permissionsJson: JSON.stringify(activePlugin.permissions, null, 2), enabled: sum.enabled, configJson: JSON.stringify(sum.config_json || {}, null, 2) };
      }
    }
  }

  function createNew() {
    selectedId = null; activePlugin = null;
    draft = { name: "", pluginKey: "", version: "1.0.0", runtime: "script", entryPoint: "", capabilities: ["tool"], permissionsJson: "{}", enabled: true, configJson: "{}" };
  }

  async function save() {
    saving = true;
    try {
      let perms = {}; let cfg = {};
      try { perms = JSON.parse(draft.permissionsJson); } catch {}
      try { cfg = JSON.parse(draft.configJson); } catch {}
      const inp = { name: draft.name, plugin_key: draft.pluginKey, version: draft.version||null, runtime: draft.runtime, entry_point: draft.entryPoint||null, capabilities: draft.capabilities, permissions: perms, enabled: draft.enabled, config_json: cfg };
      if (selectedId) { await updatePlugin(selectedId, inp); await loadData(selectedId); }
      else { const res = await createPlugin(inp); await loadData(res.id); }
      toast.success("保存成功");
    } catch { toast.error("保存失败"); } finally { saving = false; }
  }

  async function remove() {
    if (!selectedId || !confirm("确定删除插件？")) return;
    try { await deletePlugin(selectedId); toast.success("已删除"); selectedId = null; await loadData(); } catch { toast.error("删除失败"); }
  }

  const labelCls = "mb-1 block text-xs font-medium text-[var(--ink-muted)]";
  const inputCls = "w-full rounded-[var(--radius-sm)] border border-[var(--border-medium)] bg-[var(--bg-surface)] px-3 py-1.5 text-sm outline-none focus:border-[var(--brand)]";
</script>

<div class="flex h-full max-h-[calc(100vh-140px)] divide-x divide-[var(--border-soft)] rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-app)]">
  <div class="flex w-[240px] flex-col overflow-hidden bg-[var(--bg-sidebar)] sm:w-[280px]">
    <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-4 py-3">
      <h3 class="font-semibold text-[var(--ink-strong)]">扩展插件</h3>
      <Button variant="secondary" size="sm" className="h-7 px-2" onclick={createNew}><Plus size={14} class="mr-1" /> 新建</Button>
    </div>
    <div class="app-scrollbar flex-1 overflow-y-auto p-2">
      {#if plugins.length === 0}
        <div class="py-4 text-center text-xs text-[var(--ink-faint)]">暂无插件</div>
      {:else}
        {#each plugins as p}
          <button class={cn("mb-1 flex w-full items-center gap-2 rounded-md px-3 py-2 text-left", selectedId===p.id?"bg-[var(--bg-active)]":"hover:bg-[var(--bg-hover)]")} onclick={() => selPlugin(p.id)}>
            <div class={cn("h-full w-[3px] rounded-full", selectedId===p.id?"bg-[var(--brand)] opacity-100":"opacity-0")}></div>
            <div class="min-w-0 flex-1">
              <div class={cn("truncate text-sm font-medium", selectedId===p.id?"text-[var(--brand)]":"text-[var(--ink-body)]")}>{p.name}</div>
              <div class="truncate text-[11px] text-[var(--ink-faint)]">{p.plugin_key} v{p.version}</div>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <div class="app-scrollbar flex flex-1 flex-col overflow-y-auto bg-[var(--bg-surface)]">
    <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-3">
      <h3 class="font-semibold text-[var(--ink-strong)]">{selectedId ? '编辑插件' : '新建插件'}</h3>
      <div class="flex gap-2">
        {#if selectedId}<Button variant="ghost" size="sm" className="text-[var(--danger)]" onclick={remove}><Trash2 size={14} class="mr-1"/> 删除</Button>{/if}
        <Button size="sm" onclick={save} disabled={!draft.name.trim()||!draft.pluginKey.trim()||saving}>{saving?'保存中':'保存'}</Button>
      </div>
    </div>
    <div class="p-6">
      <div class="mx-auto max-w-2xl space-y-5">
        <div class="grid gap-4 sm:grid-cols-2">
          <div><label class={labelCls}>显示名称 *</label><input class={inputCls} bind:value={draft.name} /></div>
          <div><label class={labelCls}>Key 标识 *</label><input class={inputCls} bind:value={draft.pluginKey} /></div>
          <div><label class={labelCls}>版本号</label><input class={inputCls} bind:value={draft.version} /></div>
          <div><label class={labelCls}>运行时环境</label><SelectDropdown options={runtimeOpts} value={draft.runtime} onChange={(v) => draft.runtime = v} /></div>
          <div class="sm:col-span-2"><label class={labelCls}>启动入口 (Entry Point)</label><input class={inputCls} bind:value={draft.entryPoint} /></div>
          <div class="sm:col-span-2"><label class={labelCls}>声明能力 (Capabilities)</label><TagInput values={draft.capabilities} placeholder="输入后按 Enter 添加" onChange={(v) => draft.capabilities = v} /></div>
          <div class="sm:col-span-2"><label class={labelCls}>所需权限 (Permissions JSON)</label><textarea class={cn(inputCls,"font-mono text-xs")} rows="4" bind:value={draft.permissionsJson}></textarea></div>
          <div class="sm:col-span-2"><label class={labelCls}>配置信息 (Config JSON)</label><textarea class={cn(inputCls,"font-mono text-xs")} rows="4" bind:value={draft.configJson}></textarea></div>
          <div class="flex items-center gap-2 sm:col-span-2 mt-2"><input type="checkbox" bind:checked={draft.enabled} /><span class="text-sm">启用该插件</span></div>
        </div>
      </div>
    </div>
  </div>
</div>
