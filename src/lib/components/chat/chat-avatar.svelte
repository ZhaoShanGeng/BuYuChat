<script lang="ts">
  import { cn } from "$lib/utils";

  let {
    name = "",
    avatarUri = null,
    size = 36,
    kind = "agent",
    className = ""
  }: {
    name?: string;
    avatarUri?: string | null;
    size?: number;
    kind?: "agent" | "human" | "system";
    className?: string;
  } = $props();

  const initial = $derived(name.trim().charAt(0).toUpperCase() || "?");
  const toneClass = $derived.by(() => {
    switch (kind) {
      case "human":
        return "from-slate-700 to-slate-900 text-white";
      case "system":
        return "from-amber-100 to-orange-200 text-amber-800";
      default:
        return "from-[var(--brand-soft)] to-blue-100 text-[var(--brand)]";
    }
  });
</script>

<div
  class={cn(
    "flex shrink-0 items-center justify-center overflow-hidden rounded-full bg-gradient-to-br text-xs font-semibold shadow-sm",
    toneClass,
    className
  )}
  style={`width:${size}px;height:${size}px;`}
>
  {#if avatarUri}
    <img
      src={avatarUri}
      alt={name}
      class="h-full w-full object-cover"
    />
  {:else}
    {initial}
  {/if}
</div>
