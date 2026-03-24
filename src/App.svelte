<script lang="ts">
  /**
   * 应用入口页面，只承载渠道管理页面壳与视觉骨架。
   */
  import ChannelSettings from "./components/ChannelSettings.svelte";
</script>

<svelte:head>
  <title>BuYu 渠道管理</title>
</svelte:head>

<main class="min-h-screen bg-[radial-gradient(circle_at_top_left,_rgba(251,191,36,0.20),_transparent_32%),radial-gradient(circle_at_top_right,_rgba(249,115,22,0.14),_transparent_28%),linear-gradient(180deg,_#fffbeb_0%,_#fafaf9_48%,_#f5f5f4_100%)] px-6 py-8 text-stone-950">
  <section class="mx-auto flex min-h-[calc(100vh-4rem)] max-w-7xl flex-col gap-8 rounded-[2rem] border border-white/70 bg-white/80 p-6 shadow-[0_30px_90px_rgba(41,37,36,0.12)] backdrop-blur xl:p-8">
    <header class="grid gap-5 border-b border-stone-200 pb-6 lg:grid-cols-[1.1fr_0.9fr] lg:items-end">
      <div class="space-y-4">
        <div class="inline-flex items-center gap-2 rounded-full border border-orange-200 bg-orange-50 px-3 py-1 text-xs font-semibold uppercase tracking-[0.28em] text-orange-700">
          BuYu Settings
        </div>
        <div class="space-y-2">
          <h1 class="text-4xl font-semibold tracking-[-0.05em] text-stone-950 sm:text-5xl">
            渠道管理
          </h1>
          <p class="max-w-2xl text-sm leading-7 text-stone-600 sm:text-base">
            当前页面专注于管理 AI 服务渠道：创建、编辑、启用禁用、删除，以及对模型列表入口做连通性验证。
          </p>
        </div>
      </div>

      <div class="grid gap-3 sm:grid-cols-2">
        <div class="rounded-[1.5rem] border border-stone-200 bg-stone-50/90 p-4">
          <div class="text-xs font-semibold uppercase tracking-[0.24em] text-stone-500">Scope</div>
          <div class="mt-2 text-sm leading-6 text-stone-700">
            仅实现功能1：渠道管理，不扩展到模型管理与消息发送。
          </div>
        </div>
        <div class="rounded-[1.5rem] border border-orange-200 bg-orange-50/90 p-4">
          <div class="text-xs font-semibold uppercase tracking-[0.24em] text-orange-700">Contract</div>
          <div class="mt-2 text-sm leading-6 text-stone-700">
            与文档保持一致：UUID v7、snake_case Tauri command、OpenAI-compatible 默认值。
          </div>
        </div>
      </div>
    </header>

    <ChannelSettings />
  </section>
</main>
