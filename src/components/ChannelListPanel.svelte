<script lang="ts">
  /**
   * 渠道列表面板，展示列表、通知与列表侧操作按钮。
   */

  import type { Channel } from "../lib/transport/channels";
  import type { Notice } from "./channel-settings-state";

  type Props = {
    channels: Channel[];
    loading: boolean;
    notice: Notice | null;
    testingId: string | null;
    onCreate: () => void;
    onEdit: (channel: Channel) => void;
    onDelete: (id: string) => void | Promise<void>;
    onTest: (id: string) => void | Promise<void>;
  };

  const { channels, loading, notice, testingId, onCreate, onEdit, onDelete, onTest }: Props =
    $props();
</script>

<div class="rounded-[1.75rem] border border-stone-300 bg-white p-6 shadow-[0_24px_60px_rgba(28,25,23,0.08)]">
  <div class="mb-5 flex items-end justify-between gap-4 border-b border-stone-200 pb-4">
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.3em] text-orange-700">Channels</p>
      <h2 class="mt-2 text-2xl font-semibold tracking-[-0.04em] text-stone-950">渠道列表</h2>
    </div>
    <button
      class="rounded-full border border-stone-300 px-4 py-2 text-sm font-medium text-stone-700 transition hover:border-stone-900 hover:text-stone-950"
      onclick={onCreate}
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
              <button class="rounded-full bg-stone-950 px-3 py-2 text-sm font-medium text-white" onclick={() => onEdit(channel)} type="button">
                编辑
              </button>
              <button
                class="rounded-full border border-stone-300 px-3 py-2 text-sm font-medium text-stone-700"
                disabled={testingId === channel.id}
                onclick={() => onTest(channel.id)}
                type="button"
              >
                {testingId === channel.id ? "测试中..." : "测试连通性"}
              </button>
              <button class="rounded-full border border-rose-200 px-3 py-2 text-sm font-medium text-rose-700" onclick={() => onDelete(channel.id)} type="button">
                删除
              </button>
            </div>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</div>
