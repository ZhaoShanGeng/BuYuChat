<script lang="ts">
  /**
   * 左侧图标导航栏 — 顶层区域切换：聊天 / Agent / 设置。
   */
  import MessageSquareIcon from "@lucide/svelte/icons/message-square";
  import BotIcon from "@lucide/svelte/icons/bot";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import CircleUserIcon from "@lucide/svelte/icons/circle-user";
  import type { ActiveSection } from "./workspace-shell.svelte.js";

  type Props = {
    active: ActiveSection;
    onSwitch: (section: ActiveSection) => void;
  };

  const { active, onSwitch }: Props = $props();

  const items: Array<{ section: ActiveSection; icon: typeof MessageSquareIcon; label: string }> = [
    { section: "chat", icon: MessageSquareIcon, label: "对话" },
    { section: "agents", icon: BotIcon, label: "Agent" },
    { section: "settings", icon: Settings2Icon, label: "设置" }
  ];
</script>

<nav class="workspace-shell__rail flex shrink-0 flex-col items-center border-r bg-muted/30 py-3" data-ui="workspace-rail">
  <!-- 顶部导航项 -->
  <div class="flex flex-1 flex-col items-center gap-1">
    {#each items as item}
      {@const isActive = active === item.section}
      <button
        class={`workspace-shell__rail-button group relative flex size-10 items-center justify-center rounded-xl transition-all duration-200 ${
          isActive
            ? "bg-primary text-primary-foreground shadow-sm"
            : "text-muted-foreground hover:bg-accent hover:text-foreground"
        }`}
        data-active={isActive}
        onclick={() => onSwitch(item.section)}
        title={item.label}
        type="button"
      >
        <item.icon class="size-[18px]" />
      </button>
    {/each}
  </div>

  <!-- 底部用户头像 -->
  <div class="mt-auto">
    <button
      class="workspace-shell__rail-user flex size-10 items-center justify-center rounded-xl text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
      title="用户"
      type="button"
    >
      <CircleUserIcon class="size-[18px]" />
    </button>
  </div>
</nav>
