<script lang="ts">
  /**
   * 渠道表单面板，负责创建与编辑渠道字段。
   */

  import type { ChannelInput } from "../lib/transport/channels";

  type Props = {
    editingId: string | null;
    form: ChannelInput;
    saving: boolean;
    onReset: () => void;
    onSubmit: (event: SubmitEvent) => void | Promise<void>;
  };

  const { editingId, form, saving, onReset, onSubmit }: Props = $props();
</script>

<div class="rounded-[1.75rem] border border-stone-300 bg-[linear-gradient(180deg,_#fff7ed_0%,_#fffbeb_100%)] p-6 shadow-[0_24px_60px_rgba(120,53,15,0.12)]">
  <div class="mb-5 border-b border-orange-200 pb-4">
    <p class="text-xs font-semibold uppercase tracking-[0.3em] text-orange-700">Editor</p>
    <h2 class="mt-2 text-2xl font-semibold tracking-[-0.04em] text-stone-950">
      {editingId ? "编辑渠道" : "创建渠道"}
    </h2>
  </div>

  <form class="space-y-4" onsubmit={onSubmit}>
    <label class="block space-y-2">
      <span class="text-sm font-medium text-stone-700">名称</span>
      <input bind:value={form.name} class="w-full rounded-2xl border border-orange-200 bg-white px-4 py-3 text-sm text-stone-950 outline-none focus:border-orange-400" />
    </label>

    <label class="block space-y-2">
      <span class="text-sm font-medium text-stone-700">Base URL</span>
      <input bind:value={form.baseUrl} class="w-full rounded-2xl border border-orange-200 bg-white px-4 py-3 text-sm text-stone-950 outline-none focus:border-orange-400" />
    </label>

    <label class="block space-y-2">
      <span class="text-sm font-medium text-stone-700">API Key</span>
      <input bind:value={form.apiKey} class="w-full rounded-2xl border border-orange-200 bg-white px-4 py-3 text-sm text-stone-950 outline-none focus:border-orange-400" placeholder="sk-..." />
    </label>

    <div class="grid gap-4 sm:grid-cols-2">
      <label class="block space-y-2">
        <span class="text-sm font-medium text-stone-700">鉴权方式</span>
        <select bind:value={form.authType} class="w-full rounded-2xl border border-orange-200 bg-white px-4 py-3 text-sm text-stone-950 outline-none focus:border-orange-400">
          <option value="bearer">bearer</option>
          <option value="x_api_key">x_api_key</option>
          <option value="none">none</option>
        </select>
      </label>

      <label class="flex items-center gap-3 rounded-2xl border border-orange-200 bg-white px-4 py-3 text-sm text-stone-700">
        <input bind:checked={form.enabled} type="checkbox" />
        启用渠道
      </label>
    </div>

    <label class="block space-y-2">
      <span class="text-sm font-medium text-stone-700">Models Endpoint</span>
      <input bind:value={form.modelsEndpoint} class="w-full rounded-2xl border border-orange-200 bg-white px-4 py-3 text-sm text-stone-950 outline-none focus:border-orange-400" />
    </label>

    <label class="block space-y-2">
      <span class="text-sm font-medium text-stone-700">Chat Endpoint</span>
      <input bind:value={form.chatEndpoint} class="w-full rounded-2xl border border-orange-200 bg-white px-4 py-3 text-sm text-stone-950 outline-none focus:border-orange-400" />
    </label>

    <label class="block space-y-2">
      <span class="text-sm font-medium text-stone-700">Stream Endpoint</span>
      <input bind:value={form.streamEndpoint} class="w-full rounded-2xl border border-orange-200 bg-white px-4 py-3 text-sm text-stone-950 outline-none focus:border-orange-400" />
    </label>

    <div class="flex flex-wrap gap-3 pt-2">
      <button class="rounded-full bg-stone-950 px-5 py-3 text-sm font-semibold text-white" disabled={saving} type="submit">
        {saving ? "保存中..." : editingId ? "保存修改" : "创建渠道"}
      </button>
      <button class="rounded-full border border-stone-300 px-5 py-3 text-sm font-medium text-stone-700" onclick={onReset} type="button">
        清空
      </button>
    </div>
  </form>
</div>
