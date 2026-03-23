<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Tabs } from "bits-ui";
  import { Layers3, Rocket, SquareTerminal } from "lucide-svelte";

  let activeTab = $state("overview");
  let name = $state("BuYu");
  let greeting = $state("点击按钮后，这里会显示来自 Rust 命令的返回值。");
  let loading = $state(false);

  async function handleGreet() {
    loading = true;

    try {
      greeting = await invoke<string>("greet", { name });
    } catch (error) {
      greeting =
        error instanceof Error
          ? error.message
          : "当前不在 Tauri 容器内，前端骨架已加载，但 Rust 调用不可用。";
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>BuYu Skeleton</title>
</svelte:head>

<main class="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(78,128,255,0.18),_transparent_38%),linear-gradient(180deg,_#f8fafc_0%,_#eef2ff_46%,_#f8fafc_100%)] px-6 py-8 text-slate-950">
  <section class="mx-auto flex min-h-[calc(100vh-4rem)] w-full max-w-6xl flex-col gap-6 rounded-[2rem] border border-white/70 bg-white/80 p-6 shadow-[0_30px_80px_rgba(15,23,42,0.12)] backdrop-blur xl:p-8">
    <header class="flex flex-col gap-4 border-b border-slate-200/80 pb-6 lg:flex-row lg:items-end lg:justify-between">
      <div class="space-y-4">
        <div class="inline-flex items-center gap-2 rounded-full border border-sky-200 bg-sky-50 px-3 py-1 text-xs font-semibold uppercase tracking-[0.24em] text-sky-700">
          <Rocket class="size-4" />
          Clean Framework
        </div>
        <div class="space-y-2">
          <h1 class="text-4xl font-semibold tracking-[-0.04em] text-slate-950 sm:text-5xl">
            Rust + Tauri + Svelte 5
          </h1>
          <p class="max-w-2xl text-sm leading-7 text-slate-600 sm:text-base">
            当前仓库已压回最小桌面应用骨架，只保留 TypeScript、Tailwind CSS 4、bits-ui 和 lucide-svelte。
          </p>
        </div>
      </div>

      <div class="grid gap-2 text-sm text-slate-600 sm:grid-cols-2">
        <div class="rounded-2xl border border-slate-200 bg-slate-50/80 px-4 py-3">
          <div class="font-medium text-slate-900">前端</div>
          <div>Svelte 5 + TypeScript + Tailwind CSS 4</div>
        </div>
        <div class="rounded-2xl border border-slate-200 bg-slate-50/80 px-4 py-3">
          <div class="font-medium text-slate-900">桌面壳</div>
          <div>Tauri v2 + Rust command skeleton</div>
        </div>
      </div>
    </header>

    <div class="grid flex-1 gap-6 lg:grid-cols-[1.25fr_0.95fr]">
      <section class="rounded-[1.75rem] border border-slate-200 bg-white p-6 shadow-sm">
        <div class="mb-6 flex items-center gap-3">
          <div class="rounded-2xl bg-slate-950 p-3 text-white">
            <SquareTerminal class="size-5" />
          </div>
          <div>
            <h2 class="text-lg font-semibold text-slate-950">Rust Bridge</h2>
            <p class="text-sm text-slate-500">用一个最小命令验证前后端骨架已经接通。</p>
          </div>
        </div>

        <div class="space-y-4">
          <label class="block space-y-2">
            <span class="text-sm font-medium text-slate-700">Name</span>
            <input
              bind:value={name}
              class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400 focus:bg-white focus:ring-4 focus:ring-sky-100"
              placeholder="输入一个名字"
            />
          </label>

          <button
            class="inline-flex items-center justify-center rounded-2xl bg-slate-950 px-5 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400"
            disabled={loading}
            onclick={handleGreet}
            type="button"
          >
            {loading ? "Calling Rust..." : "Invoke greet"}
          </button>

          <div class="rounded-3xl border border-sky-100 bg-sky-50/70 p-4 text-sm leading-7 text-slate-700">
            {greeting}
          </div>
        </div>
      </section>

      <section class="rounded-[1.75rem] border border-slate-200 bg-[linear-gradient(180deg,_rgba(15,23,42,0.03),_rgba(15,23,42,0))] p-6 shadow-sm">
        <div class="mb-6 flex items-center gap-3">
          <div class="rounded-2xl bg-sky-100 p-3 text-sky-700">
            <Layers3 class="size-5" />
          </div>
          <div>
            <h2 class="text-lg font-semibold text-slate-950">bits-ui Skeleton</h2>
            <p class="text-sm text-slate-500">保留一个可扩展的 headless 组件入口。</p>
          </div>
        </div>

        <Tabs.Root bind:value={activeTab} class="space-y-4">
          <Tabs.List class="grid grid-cols-3 gap-2 rounded-2xl border border-slate-200 bg-white p-2">
            <Tabs.Trigger
              class="rounded-xl px-3 py-2 text-sm font-medium text-slate-500 transition data-[state=active]:bg-slate-950 data-[state=active]:text-white"
              value="overview"
            >
              Overview
            </Tabs.Trigger>
            <Tabs.Trigger
              class="rounded-xl px-3 py-2 text-sm font-medium text-slate-500 transition data-[state=active]:bg-slate-950 data-[state=active]:text-white"
              value="stack"
            >
              Stack
            </Tabs.Trigger>
            <Tabs.Trigger
              class="rounded-xl px-3 py-2 text-sm font-medium text-slate-500 transition data-[state=active]:bg-slate-950 data-[state=active]:text-white"
              value="next"
            >
              Next
            </Tabs.Trigger>
          </Tabs.List>

          <Tabs.Content class="rounded-3xl border border-slate-200 bg-white p-5 text-sm leading-7 text-slate-600" value="overview">
            当前前端只有一个页面、一个 bits-ui 组件组和一个 Rust 调用示例，适合作为后续功能开发起点。
          </Tabs.Content>
          <Tabs.Content class="rounded-3xl border border-slate-200 bg-white p-5 text-sm leading-7 text-slate-600" value="stack">
            依赖已经收敛到 `Svelte 5`、`TypeScript`、`Tailwind CSS 4`、`bits-ui`、`lucide-svelte` 和 Tauri 必需项。
          </Tabs.Content>
          <Tabs.Content class="rounded-3xl border border-slate-200 bg-white p-5 text-sm leading-7 text-slate-600" value="next">
            你后面可以在这个骨架上继续补路由、状态管理、Tauri commands、数据库层和具体业务界面。
          </Tabs.Content>
        </Tabs.Root>
      </section>
    </div>
  </section>
</main>
