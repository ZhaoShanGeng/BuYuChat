<script lang="ts">
  import Button from "$ui/button.svelte";
  import { Bot, SlidersHorizontal, BookOpenText, Workflow, Cable, Sparkles } from "lucide-svelte";

  const workspaceIcons = {
    agents: Bot,
    presets: SlidersHorizontal,
    lorebooks: BookOpenText,
    workflows: Workflow,
    settings: Cable
  } as Record<string, typeof Bot>;

  let {
    eyebrow = "Workspace",
    title = "",
    description = "",
    bullets = [],
    cta = "Configure"
  }: {
    eyebrow?: string;
    title?: string;
    description?: string;
    bullets?: string[];
    cta?: string;
  } = $props();

  const Icon = workspaceIcons[eyebrow.toLowerCase()] ?? Sparkles;
</script>

<section class="flex h-full flex-col items-center justify-center gap-8 px-6 py-10">
  <div class="flex flex-col items-center gap-4 text-center">
    <!-- Icon -->
    <div class="flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-[var(--brand-soft)] to-blue-100 shadow-md">
      <Icon size={28} class="text-[var(--brand)]" />
    </div>

    <div>
      <p class="text-xs font-semibold uppercase tracking-widest text-[var(--ink-faint)]">{eyebrow}</p>
      <h2 class="mt-1.5 text-xl font-bold text-[var(--ink-strong)]">{title}</h2>
      <p class="mx-auto mt-2 max-w-sm text-sm leading-relaxed text-[var(--ink-muted)]">{description}</p>
    </div>
  </div>

  {#if bullets.length > 0}
    <div class="grid max-w-lg gap-3 text-sm text-[var(--ink-muted)] md:grid-cols-3">
      {#each bullets as bullet}
        <div class="suggestion-card rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-4 text-center leading-relaxed">
          {bullet}
        </div>
      {/each}
    </div>
  {/if}

  <Button>{cta}</Button>
</section>
