<script lang="ts">
  /**
   * 左侧图标导航栏 — 顶层区域切换：聊天 / Agent / 设置。
   */
  import MessageSquareIcon from "@lucide/svelte/icons/message-square";
  import BotIcon from "@lucide/svelte/icons/bot";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import CircleUserIcon from "@lucide/svelte/icons/circle-user";
  import SunIcon from "@lucide/svelte/icons/sun";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import MonitorIcon from "@lucide/svelte/icons/monitor";
  import type { ActiveSection } from "./workspace-shell.svelte.js";
  import { getThemeMode, setThemeMode, type ThemeMode } from "../lib/theme.svelte";
  import { cn } from "$lib/utils";

  type Props = {
    active: ActiveSection;
    onSwitch: (section: ActiveSection) => void;
  };

  const { active, onSwitch }: Props = $props();

  const THEME_CYCLE: ThemeMode[] = ["light", "dark", "system"];
  const THEME_LABELS: Record<ThemeMode, string> = {
    light: "浅色",
    dark: "深色",
    system: "跟随系统"
  };

  function cycleTheme() {
    const current = getThemeMode();
    const next = THEME_CYCLE[(THEME_CYCLE.indexOf(current) + 1) % THEME_CYCLE.length];
    setThemeMode(next);
  }

  let themeMode = $derived(getThemeMode());

  const items: Array<{ section: ActiveSection; icon: typeof MessageSquareIcon; label: string }> = [
    { section: "chat", icon: MessageSquareIcon, label: "对话" },
    { section: "agents", icon: BotIcon, label: "Agent" },
    { section: "settings", icon: Settings2Icon, label: "设置" }
  ];
</script>

<nav class="workspace-shell__rail flex shrink-0 flex-col items-center border-r border-transparent bg-[var(--buyu-rail-bg)] py-3 relative z-10" data-ui="workspace-rail">
  <!-- 顶部导航项 -->
  <div class="flex flex-1 flex-col items-center gap-1">
    {#each items as item}
      {@const isActive = active === item.section}
      <button
        class={cn(
          "workspace-shell__rail-button group relative flex size-10 items-center justify-center rounded-xl transition-all duration-200",
          isActive
            ? "bg-primary text-primary-foreground shadow-md before:absolute before:left-0 before:top-1.5 before:bottom-1.5 before:w-[3px] before:bg-primary before:rounded-r-md"
            : "text-muted-foreground hover:bg-primary/10 hover:text-foreground"
        )}
        data-active={isActive}
        onclick={() => onSwitch(item.section)}
        title={item.label}
        type="button"
      >
        <item.icon class="size-[18px]" />
      </button>
    {/each}
  </div>

  <!-- 底部主题切换 + 用户头像 -->
  <div class="mt-auto flex flex-col items-center gap-1">
    <button
      class="flex size-10 items-center justify-center rounded-xl text-muted-foreground transition-colors hover:bg-primary/10 hover:text-foreground"
      onclick={cycleTheme}
      title={THEME_LABELS[themeMode]}
      type="button"
    >
      {#if themeMode === "light"}
        <SunIcon class="size-[18px]" />
      {:else if themeMode === "dark"}
        <MoonIcon class="size-[18px]" />
      {:else}
        <MonitorIcon class="size-[18px]" />
      {/if}
    </button>
    <button
      class="workspace-shell__rail-user flex size-10 items-center justify-center rounded-xl text-muted-foreground transition-colors hover:bg-primary/10 hover:text-foreground"
      title="用户"
      type="button"
    >
      <CircleUserIcon class="size-[18px]" />
    </button>
  </div>
</nav>
