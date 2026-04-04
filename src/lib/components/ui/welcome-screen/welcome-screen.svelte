<script lang="ts">
  import { cn } from "$lib/utils";
  import PaperTexture from "$lib/components/ui/paper-texture/paper-texture.svelte";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { getThemeMode } from "$lib/theme.svelte";

  type Props = {
    class?: string;
    agentName?: string;
    onSuggestionClick?: (suggestion: string) => void;
  };

  const { class: cls, agentName = "助手", onSuggestionClick }: Props = $props();

  const suggestions = [
    "写一封感谢信",
    "解释量子计算",
    "帮我写总结报告",
    "推荐几本书"
  ];

  let timeGreeting = $derived.by(() => {
    const hour = new Date().getHours();
    if (hour < 6) return "凌晨好";
    if (hour < 12) return "上午好";
    if (hour < 14) return "中午好";
    if (hour < 18) return "下午好";
    return "晚上好";
  });
</script>

<div class={cn("relative flex h-full flex-col items-center justify-center p-8 text-center", cls)}>
  <PaperTexture opacity={0.02} />

  <div class="relative z-10 max-w-md space-y-6">
    <div class="space-y-2">
      <h1 class="text-4xl font-serif font-semibold tracking-tight text-foreground">
        {timeGreeting}
      </h1>
      <p class="text-muted-foreground">
        我是你的 <span class="font-medium text-foreground">{agentName}</span>，今天想探索些什么？
      </p>
    </div>

    <div class="flex flex-wrap justify-center gap-2 pt-4">
      {#each suggestions as suggestion}
        <Badge
          variant="secondary"
          class="cursor-pointer hover:bg-primary hover:text-primary-foreground transition-colors py-1.5 px-3 rounded-xl font-normal"
          onclick={() => onSuggestionClick?.(suggestion)}
        >
          {suggestion}
        </Badge>
      {/each}
    </div>
  </div>
</div>
