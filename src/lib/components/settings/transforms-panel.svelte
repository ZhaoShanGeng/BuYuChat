<script lang="ts">
  import { onMount } from "svelte";
  import { Plus, Trash2, ArrowRight } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import { listTransformPipelines, getTransformPipelineDetail, createTransformPipeline, updateTransformPipeline, deleteTransformPipeline, testTransformPipeline, type TransformPipelineSummary, type TransformPipelineDetail, type TransformStep } from "$lib/api/transforms";
  import Button from "$components/ui/button.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import SelectDropdown from "$components/shared/select-dropdown.svelte";
  import { cn } from "$lib/utils";

  let pipelines = $state<TransformPipelineSummary[]>([]);
  let activePipeline = $state<TransformPipelineDetail | null>(null);
  let selectedId = $state<string | null>(null);
  let loadingList = $state(true);
  let saving = $state(false);

  type Draft = { name: string; pipelineKey: string; pipelineType: string; enabled: boolean; sortOrder: number; configJson: string; steps: TransformStep[]; };
  let draft = $state<Draft>({ name: "", pipelineKey: "", pipelineType: "regex", enabled: true, sortOrder: 0, configJson: "{}", steps: [] });

  let testInputText = $state("");
  let testOutputText = $state("");
  let testing = $state(false);

  const typeOpts = [{value:"regex",label:"正则替换 (Regex)"},{value:"script",label:"脚本处理 (Script)"}];

  onMount(loadData);

  async function loadData(sel?: string | null) {
    loadingList = true;
    try {
      pipelines = await listTransformPipelines();
      if (sel) await selPipeline(sel);
      else if (!selectedId && pipelines.length > 0) await selPipeline(pipelines[0].id);
    } catch {
      pipelines = [{ id: "mock-pipe-1", name: "清理 markdown", pipeline_key: "clean_md", pipeline_type: "regex", enabled: true, sort_order: 0, config_json: {}, created_at: 0, updated_at: 0 }];
      if (sel) await selPipeline(sel); else await selPipeline(pipelines[0].id);
    } finally { loadingList = false; }
  }

  async function selPipeline(id: string) {
    selectedId = id;
    try {
      activePipeline = await getTransformPipelineDetail(id);
      draft = { name: activePipeline.summary.name, pipelineKey: activePipeline.summary.pipeline_key, pipelineType: activePipeline.summary.pipeline_type, enabled: activePipeline.summary.enabled, sortOrder: activePipeline.summary.sort_order, configJson: JSON.stringify(activePipeline.summary.config_json || {}, null, 2), steps: activePipeline.steps.map(s=>({...s})) };
    } catch {
      const sum = pipelines.find(x=>x.id===id);
      if(sum) {
        activePipeline = { summary: sum, steps: [{step_type: "regex", pattern: "<think>.*?</think>", replacement: "", sort_order: 1}] };
        draft = { name: sum.name, pipelineKey: sum.pipeline_key, pipelineType: sum.pipeline_type, enabled: sum.enabled, sortOrder: sum.sort_order, configJson: JSON.stringify(sum.config_json || {}, null, 2), steps: activePipeline.steps.map(s=>({...s})) };
      }
    }
  }

  function createNew() {
    selectedId = null; activePipeline = null;
    draft = { name: "", pipelineKey: "", pipelineType: "regex", enabled: true, sortOrder: pipelines.length, configJson: "{}", steps: [] };
  }

  function addStep() { draft.steps = [...draft.steps, { step_type: draft.pipelineType, pattern: "", replacement: "", sort_order: draft.steps.length + 1 }]; }
  function removeStep(idx: number) { draft.steps = draft.steps.filter((_, i) => i !== idx); }

  async function save() {
    saving = true;
    try {
      let cfg = {}; try { cfg = JSON.parse(draft.configJson); } catch {}
      const inp = { name: draft.name, pipeline_key: draft.pipelineKey, pipeline_type: draft.pipelineType, enabled: draft.enabled, sort_order: Number(draft.sortOrder)||0, config_json: cfg, steps: draft.steps };
      if (selectedId) { await updateTransformPipeline(selectedId, inp); await loadData(selectedId); }
      else { const res = await createTransformPipeline(inp); await loadData(res.summary.id); }
      toast.success("保存成功");
    } catch { toast.error("保存失败"); } finally { saving = false; }
  }

  async function remove() {
    if (!selectedId || !confirm("确定删除转换管线？")) return;
    try { await deleteTransformPipeline(selectedId); toast.success("已删除"); selectedId = null; await loadData(); } catch { toast.error("删除失败"); }
  }

  async function runTest() {
    if (!selectedId) return;
    testing = true;
    try {
      const resp = await testTransformPipeline(selectedId, testInputText);
      testOutputText = resp.output_text;
      toast.success("测试完成");
    } catch (e) {
      // Mock result if backend not ready
      if (draft.steps.length > 0 && draft.pipelineType === "regex") {
        let res = testInputText;
        for (const s of draft.steps) {
          try { res = res.replace(new RegExp(s.pattern, "g"), s.replacement); } catch {}
        }
        testOutputText = res; toast.success("本地模拟测试");
      } else {
        toast.error("转换失败"); testOutputText = "";
      }
    } finally { testing = false; }
  }

  const labelCls = "mb-1 block text-xs font-medium text-[var(--ink-muted)]";
  const inputCls = "w-full rounded-[var(--radius-sm)] border border-[var(--border-medium)] bg-[var(--bg-surface)] px-3 py-1.5 text-sm outline-none focus:border-[var(--brand)]";
</script>

<div class="flex h-full max-h-[calc(100vh-140px)] divide-x divide-[var(--border-soft)] rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-app)]">
  <div class="flex w-[240px] flex-col overflow-hidden bg-[var(--bg-sidebar)] sm:w-[280px]">
    <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-4 py-3">
      <h3 class="font-semibold text-[var(--ink-strong)]">转换管线</h3>
      <Button variant="secondary" size="sm" className="h-7 px-2" onclick={createNew}><Plus size={14} class="mr-1" /> 新建</Button>
    </div>
    <div class="app-scrollbar flex-1 overflow-y-auto p-2">
      {#if pipelines.length === 0}
        <div class="py-4 text-center text-xs text-[var(--ink-faint)]">暂无管线</div>
      {:else}
        {#each pipelines as p}
          <button class={cn("mb-1 flex w-full items-center gap-2 rounded-md px-3 py-2 text-left", selectedId===p.id?"bg-[var(--bg-active)]":"hover:bg-[var(--bg-hover)]")} onclick={() => selPipeline(p.id)}>
            <div class={cn("h-full w-[3px] rounded-full", selectedId===p.id?"bg-[var(--brand)] opacity-100":"opacity-0")}></div>
            <div class="min-w-0 flex-1">
              <div class={cn("truncate text-sm font-medium", selectedId===p.id?"text-[var(--brand)]":"text-[var(--ink-body)]")}>{p.name}</div>
              <div class="truncate text-[11px] text-[var(--ink-faint)]">{p.pipeline_key}</div>
            </div>
            {#if !p.enabled}<span class="text-[10px] bg-[var(--bg-sunken)] px-1 rounded-sm text-[var(--ink-faint)]">停用</span>{/if}
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <div class="app-scrollbar flex flex-1 flex-col overflow-y-auto bg-[var(--bg-surface)]">
    <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-3">
      <h3 class="font-semibold text-[var(--ink-strong)]">{selectedId ? '编辑管线' : '新建管线'}</h3>
      <div class="flex gap-2">
        {#if selectedId}<Button variant="ghost" size="sm" className="text-[var(--danger)]" onclick={remove}><Trash2 size={14} class="mr-1"/> 删除</Button>{/if}
        <Button size="sm" onclick={save} disabled={!draft.name.trim()||!draft.pipelineKey.trim()||saving}>{saving?'保存中':'保存'}</Button>
      </div>
    </div>
    <div class="p-6">
      <div class="mx-auto max-w-3xl space-y-6">
        <div class="grid gap-4 sm:grid-cols-3">
          <div><label class={labelCls}>名称 *</label><input class={inputCls} bind:value={draft.name} /></div>
          <div><label class={labelCls}>Key 标识 *</label><input class={inputCls} bind:value={draft.pipelineKey} /></div>
          <div><label class={labelCls}>管线类型</label><SelectDropdown options={typeOpts} value={draft.pipelineType} onChange={(v) => draft.pipelineType = v} /></div>
          <div><label class={labelCls}>排序值</label><input type="number" class={inputCls} bind:value={draft.sortOrder} /></div>
          <div class="sm:col-span-2 flex items-center gap-2 pt-6"><input type="checkbox" bind:checked={draft.enabled} /><span class="text-sm">启用管线</span></div>
        </div>

        <div>
          <div class="mb-3 flex items-center justify-between"><h3 class="text-sm font-semibold text-[var(--ink-strong)]">转换步骤</h3><Button size="sm" variant="secondary" className="h-7 px-2" onclick={addStep}><Plus size={14} /> 添加步骤</Button></div>
          <div class="space-y-3">
            {#each draft.steps as step, idx}
              <div class="flex gap-3 items-end rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--bg-sunken)] p-3">
                <div class="flex-1"><label class="mb-1 block text-[11px] font-medium text-[var(--ink-muted)]">匹配 Pattern</label><input class={cn(inputCls,"font-mono text-xs")} bind:value={step.pattern} /></div>
                <ArrowRight size={14} class="mb-2 text-[var(--ink-faint)]" />
                <div class="flex-1"><label class="mb-1 block text-[11px] font-medium text-[var(--ink-muted)]">替换 Replacement</label><input class={cn(inputCls,"font-mono text-xs")} bind:value={step.replacement} /></div>
                <ActionIconButton tone="danger" className="mb-1 bg-white" onClick={()=>removeStep(idx)}><Trash2 size={14} /></ActionIconButton>
              </div>
            {/each}
            {#if draft.steps.length === 0}<div class="text-center text-xs text-[var(--ink-faint)] border border-dashed border-[var(--border-medium)] rounded-md py-4">无转换步骤</div>{/if}
          </div>
        </div>

        {#if selectedId}
          <div class="border-t border-[var(--border-soft)] pt-6">
            <h3 class="mb-3 text-sm font-semibold text-[var(--ink-strong)]">测试匹配</h3>
            <div class="grid gap-4 sm:grid-cols-2">
              <div><label class={labelCls}>输入文本</label><textarea class={cn(inputCls,"resize-y text-xs font-mono h-[120px]")} bind:value={testInputText} placeholder="在此处输入待转换文本"></textarea></div>
              <div><label class={labelCls}>输出结果</label><textarea readonly class={cn(inputCls,"resize-y text-xs font-mono h-[120px] bg-[var(--bg-sunken)]")} value={testOutputText} placeholder="输出结果"></textarea></div>
              <div class="sm:col-span-2 flex justify-end"><Button size="sm" onclick={runTest} disabled={testing||!testInputText.trim()}>{testing?'运行中...':'运行转换'}</Button></div>
            </div>
          </div>
        {/if}

      </div>
    </div>
  </div>
</div>
