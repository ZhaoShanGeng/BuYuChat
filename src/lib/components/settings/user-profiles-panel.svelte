<script lang="ts">
  import { onMount } from "svelte";
  import { Plus, Trash2, Edit3, X } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import {
    listUserProfiles, getUserProfileDetail, createUserProfile, updateUserProfile, deleteUserProfile,
    type UserProfileSummary, type UserProfileDetail
  } from "$lib/api/user-profiles";
  import Button from "$components/ui/button.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import SelectDropdown from "$components/shared/select-dropdown.svelte";
  import { cn } from "$lib/utils";

  let profiles = $state<UserProfileSummary[]>([]);
  let activeProfile = $state<UserProfileDetail | null>(null);
  let selectedId = $state<string | null>(null);
  let loadingList = $state(true);
  let saving = $state(false);

  type Draft = {
    name: string; title: string; avatarUri: string; description: string;
    insertionPosition: string; insertionDepth: string; insertionRole: string;
    enabled: boolean; sortOrder: number;
  };
  let draft = $state<Draft>({
    name: "", title: "", avatarUri: "", description: "",
    insertionPosition: "prompt_manager", insertionDepth: "0", insertionRole: "system",
    enabled: true, sortOrder: 0
  });

  const positionOptions = [
    { value: "prompt_manager", label: "system prompt 末尾 / prompt_manager" },
    { value: "in_chat", label: "消息流中 / in_chat" }
  ];
  const roleOptions = [
    { value: "system", label: "System" },
    { value: "user", label: "User" },
    { value: "assistant", label: "Assistant" }
  ];

  onMount(loadData);

  async function loadData(sel?: string | null) {
    loadingList = true;
    try {
      profiles = await listUserProfiles();
      if (sel) await selectProfile(sel);
      else if (!selectedId && profiles.length > 0) await selectProfile(profiles[0].id);
    } catch {
      // Mock for early stages or when backend command misses
      profiles = [{ id: "mock-1", name: "默认画像", title: "Developer", avatar_uri: null, insertion_position: "prompt_manager", insertion_depth: null, insertion_role: "system", enabled: true, sort_order: 0, config_json: {}, created_at: Date.now(), updated_at: Date.now() }];
      if (sel) await selectProfile(sel);
      else await selectProfile(profiles[0].id);
    } finally {
      loadingList = false;
    }
  }

  function mapDetailToDraft(d: UserProfileDetail): Draft {
    return {
      name: d.summary.name, title: d.summary.title || "", avatarUri: d.summary.avatar_uri || "",
      description: d.description_content?.text_content || d.description_content?.preview_text || "",
      insertionPosition: d.summary.insertion_position || "prompt_manager",
      insertionDepth: d.summary.insertion_depth !== null ? String(d.summary.insertion_depth) : "",
      insertionRole: d.summary.insertion_role || "system",
      enabled: d.summary.enabled, sortOrder: d.summary.sort_order
    };
  }

  function getDraftInput() {
    return {
      name: draft.name.trim() || "未命名", title: draft.title.trim() || null, avatar_uri: draft.avatarUri.trim() || null,
      description_content: draft.description.trim() ? { content_type: "text", mime_type: "text/plain", text_content: draft.description.trim(), source_file_path: null, primary_storage_uri: null, size_bytes_hint: null, preview_text: draft.description.trim().slice(0, 50), config_json: {} } : null,
      insertion_position: draft.insertionPosition || null, insertion_depth: draft.insertionDepth ? Number(draft.insertionDepth) : null,
      insertion_role: draft.insertionRole || null, enabled: draft.enabled, sort_order: Number(draft.sortOrder) || 0, config_json: {}
    };
  }

  async function selectProfile(id: string) {
    selectedId = id;
    try {
      activeProfile = await getUserProfileDetail(id);
      draft = mapDetailToDraft(activeProfile);
    } catch {
      // Mock detail
      const sum = profiles.find(p => p.id === id);
      if (sum) {
        activeProfile = { summary: sum, description_content: { text_content: "这是一个模拟的用户画像描述...", ...({} as any) } };
        draft = mapDetailToDraft(activeProfile);
      }
    }
  }

  function createNew() {
    selectedId = null;
    activeProfile = null;
    draft = { name: "", title: "", avatarUri: "", description: "", insertionPosition: "prompt_manager", insertionDepth: "", insertionRole: "system", enabled: true, sortOrder: profiles.length };
  }

  async function save() {
    saving = true;
    try {
      if (selectedId) {
        await updateUserProfile(selectedId, getDraftInput());
        toast.success("保存成功");
        await loadData(selectedId);
      } else {
        const created = await createUserProfile(getDraftInput());
        toast.success("创建成功");
        await loadData(created.summary.id);
      }
    } catch (e) { toast.error("保存失败"); } finally { saving = false; }
  }

  async function remove() {
    if (!selectedId || !confirm("确定删除？")) return;
    try {
      await deleteUserProfile(selectedId);
      toast.success("已删除");
      selectedId = null; activeProfile = null;
      await loadData();
    } catch { toast.error("删除失败"); }
  }

  const labelCls = "mb-1 block text-xs font-medium text-[var(--ink-muted)]";
  const inputCls = "w-full rounded-[var(--radius-sm)] border border-[var(--border-medium)] bg-[var(--bg-surface)] px-3 py-1.5 text-sm outline-none focus:border-[var(--brand)] focus:shadow-[0_0_0_2px_var(--brand-glow)]";
</script>

<div class="flex h-full max-h-[calc(100vh-140px)] divide-x divide-[var(--border-soft)] rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-app)]">
  <!-- Left Side: List -->
  <div class="flex w-[240px] flex-col overflow-hidden bg-[var(--bg-sidebar)] sm:w-[280px]">
    <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-4 py-3">
      <h3 class="font-semibold text-[var(--ink-strong)]">用户画像</h3>
      <Button type="button" variant="secondary" size="sm" className="h-7 px-2" onclick={createNew}>
        <Plus size={14} class="mr-1" /> 新建
      </Button>
    </div>
    <div class="app-scrollbar flex-1 overflow-y-auto p-2">
      {#if loadingList}
        <div class="py-4 text-center text-xs text-[var(--ink-muted)]">加载中...</div>
      {:else if profiles.length === 0}
        <div class="py-4 text-center text-xs text-[var(--ink-faint)]">暂无画像</div>
      {:else}
        {#each profiles as p}
          <button
            class={cn("mb-1 flex w-full items-center gap-2 rounded-md px-3 py-2 text-left transition-colors", selectedId === p.id ? "bg-[var(--bg-active)]" : "hover:bg-[var(--bg-hover)]")}
            onclick={() => selectProfile(p.id)}
          >
            <div class={cn("h-full w-[3px] rounded-full", selectedId === p.id ? "bg-[var(--brand)] opacity-100" : "opacity-0")}></div>
            <div class="min-w-0 flex-1">
              <div class={cn("truncate text-sm font-medium", selectedId === p.id ? "text-[var(--brand)]" : "text-[var(--ink-body)]")}>{p.name}</div>
              {#if p.title}<div class="truncate text-[11px] text-[var(--ink-faint)]">{p.title}</div>{/if}
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Right Side: Editor -->
  <div class="app-scrollbar flex flex-1 flex-col overflow-y-auto bg-[var(--bg-surface)]">
    <div class="flex items-center justify-between border-b border-[var(--border-soft)] px-6 py-3">
      <h3 class="font-semibold text-[var(--ink-strong)]">{selectedId ? '编辑用户画像' : '新建用户画像'}</h3>
      <div class="flex gap-2">
        {#if selectedId}<Button variant="ghost" size="sm" className="text-[var(--danger)] hover:bg-red-50" onclick={remove}><Trash2 size={14} class="mr-1"/> 删除</Button>{/if}
        <Button size="sm" onclick={save} disabled={!draft.name.trim() || saving}>{saving ? '保存中' : '保存'}</Button>
      </div>
    </div>
    
    <div class="p-6">
      <div class="mx-auto max-w-2xl space-y-5">
        <div class="grid gap-4 sm:grid-cols-2">
          <div><label class={labelCls}>名称 *</label><input class={inputCls} bind:value={draft.name} placeholder="例如：默认画像" /></div>
          <div><label class={labelCls}>标题</label><input class={inputCls} bind:value={draft.title} placeholder="例如：Developer" /></div>
          <div class="sm:col-span-2"><label class={labelCls}>头像 URI</label><input class={inputCls} bind:value={draft.avatarUri} /></div>
          <div class="sm:col-span-2">
            <label class={labelCls}>详情描述</label>
            <textarea class={cn(inputCls, "resize-y")} rows="4" bind:value={draft.description} placeholder="介绍本画像的细节、技能、偏好等..."></textarea>
          </div>
          <div>
            <label class={labelCls}>注入位置</label>
            <SelectDropdown options={positionOptions} value={draft.insertionPosition} onChange={(v) => draft.insertionPosition = v} />
          </div>
          <div class="flex gap-2">
            <div class="flex-1"><label class={labelCls}>注入尝试深度</label><input type="number" class={inputCls} bind:value={draft.insertionDepth} placeholder="默认为最浅层" /></div>
            <div class="flex-1">
              <label class={labelCls}>注入角色</label>
              <SelectDropdown options={roleOptions} value={draft.insertionRole} onChange={(v) => draft.insertionRole = v} />
            </div>
          </div>
          <div class="flex gap-4 sm:col-span-2">
            <div class="w-1/2"><label class={labelCls}>排序号</label><input type="number" class={inputCls} bind:value={draft.sortOrder} /></div>
            <div class="w-1/2 flex items-center gap-2 pt-6">
              <input type="checkbox" class="h-4 w-4" bind:checked={draft.enabled} />
              <span class="text-sm">启用</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
