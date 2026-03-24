<script lang="ts">
  import { onMount } from "svelte";

  import {
    createChannel,
    deleteChannel,
    listChannels,
    testChannel,
    toAppError,
    updateChannel,
    type Channel,
    type ChannelInput
  } from "../lib/transport/channels";
  import {
    createEmptyForm,
    createFormFromChannel,
    humanizeError,
    removeChannel,
    submitChannelForm,
    verifyChannelConnectivity,
    type Notice
  } from "./channel-settings-state";

  let channels = $state<Channel[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let testingId = $state<string | null>(null);
  let editingId = $state<string | null>(null);
  let notice = $state<Notice | null>(null);
  let form = $state<ChannelInput>(createEmptyForm());

  function resetForm() {
    editingId = null;
    form = createEmptyForm();
  }

  async function reloadChannels() {
    loading = true;
    try {
      channels = await listChannels(true);
    } catch (error) {
      notice = { kind: "error", text: humanizeError(toAppError(error)) };
    } finally {
      loading = false;
    }
  }

  function startEdit(channel: Channel) {
    editingId = channel.id;
    form = createFormFromChannel(channel);
    notice = null;
  }

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    notice = null;

    try {
      notice = await submitChannelForm(
        {
          createChannel,
          updateChannel
        },
        editingId,
        form
      );
      resetForm();
      await reloadChannels();
    } finally {
      saving = false;
    }
  }

  async function handleDelete(id: string) {
    notice = await removeChannel({ deleteChannel }, id);
    if (notice.kind === "success") {
      if (editingId === id) {
        resetForm();
      }
      await reloadChannels();
    }
  }

  async function handleConnectivityTest(id: string) {
    testingId = id;
    notice = null;

    notice = await verifyChannelConnectivity({ testChannel }, id);
    testingId = null;
  }

  onMount(async () => {
    await reloadChannels();
  });
</script>

<section class="grid gap-6 lg:grid-cols-[1.05fr_0.95fr]">
  <div class="rounded-[1.75rem] border border-stone-300 bg-white p-6 shadow-[0_24px_60px_rgba(28,25,23,0.08)]">
    <div class="mb-5 flex items-end justify-between gap-4 border-b border-stone-200 pb-4">
      <div>
        <p class="text-xs font-semibold uppercase tracking-[0.3em] text-orange-700">Channels</p>
        <h2 class="mt-2 text-2xl font-semibold tracking-[-0.04em] text-stone-950">渠道列表</h2>
      </div>
      <button
        class="rounded-full border border-stone-300 px-4 py-2 text-sm font-medium text-stone-700 transition hover:border-stone-900 hover:text-stone-950"
        onclick={resetForm}
        type="button"
      >
        新建渠道
      </button>
    </div>

    {#if notice}
      <div class={`mb-4 rounded-2xl px-4 py-3 text-sm ${notice.kind === "success" ? "bg-emerald-50 text-emerald-800" : "bg-rose-50 text-rose-800"}`}>
        {notice.text}
      </div>
    {/if}
    {#if loading}
      <div class="rounded-3xl border border-dashed border-stone-300 bg-stone-50 px-5 py-10 text-center text-sm text-stone-500">
        正在加载渠道...
      </div>
    {:else if channels.length === 0}
      <div class="rounded-3xl border border-dashed border-stone-300 bg-stone-50 px-5 py-10 text-center text-sm text-stone-500">
        还没有渠道，先在右侧创建一个 OpenAI-compatible 渠道。
      </div>
    {:else}
      <div class="space-y-3">
        {#each channels as channel}
          <article class="rounded-[1.5rem] border border-stone-200 bg-stone-50/70 p-4">
            <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
              <div class="space-y-2">
                <div class="flex items-center gap-3">
                  <h3 class="text-lg font-semibold text-stone-950">{channel.name}</h3>
                  <span class={`rounded-full px-2.5 py-1 text-xs font-semibold ${channel.enabled ? "bg-emerald-100 text-emerald-800" : "bg-stone-200 text-stone-700"}`}>
                    {channel.enabled ? "启用中" : "已禁用"}
                  </span>
                </div>
                <div class="text-sm text-stone-600">{channel.baseUrl}</div>
                <div class="text-xs uppercase tracking-[0.24em] text-stone-400">{channel.channelType}</div>
              </div>
              <div class="flex flex-wrap gap-2">
                <button class="rounded-full bg-stone-950 px-3 py-2 text-sm font-medium text-white" onclick={() => startEdit(channel)} type="button">
                  编辑
                </button>
                <button
                  class="rounded-full border border-stone-300 px-3 py-2 text-sm font-medium text-stone-700"
                  disabled={testingId === channel.id}
                  onclick={() => handleConnectivityTest(channel.id)}
                  type="button"
                >
                  {testingId === channel.id ? "测试中..." : "测试连通性"}
                </button>
                <button class="rounded-full border border-rose-200 px-3 py-2 text-sm font-medium text-rose-700" onclick={() => handleDelete(channel.id)} type="button">
                  删除
                </button>
              </div>
            </div>
          </article>
        {/each}
      </div>
    {/if}
  </div>

  <div class="rounded-[1.75rem] border border-stone-300 bg-[linear-gradient(180deg,_#fff7ed_0%,_#fffbeb_100%)] p-6 shadow-[0_24px_60px_rgba(120,53,15,0.12)]">
    <div class="mb-5 border-b border-orange-200 pb-4">
      <p class="text-xs font-semibold uppercase tracking-[0.3em] text-orange-700">Editor</p>
      <h2 class="mt-2 text-2xl font-semibold tracking-[-0.04em] text-stone-950">
        {editingId ? "编辑渠道" : "创建渠道"}
      </h2>
    </div>
    <form class="space-y-4" onsubmit={handleSubmit}>
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
        <button class="rounded-full border border-stone-300 px-5 py-3 text-sm font-medium text-stone-700" onclick={resetForm} type="button">
          清空
        </button>
      </div>
    </form>
  </div>
</section>
