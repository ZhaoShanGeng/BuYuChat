<script lang="ts">
  import Badge from "$ui/badge.svelte";
  import Button from "$ui/button.svelte";
  import Card from "$ui/card.svelte";

  export let eyebrow = "Workspace";
  export let title = "";
  export let description = "";
  export let bullets: string[] = [];
  export let cta = "Configure";
</script>

<section class="px-4 py-4 md:px-6 md:py-5 lg:px-8">
  <Card className="rounded-[2rem] border border-[var(--border-soft)] bg-white/90 p-6 md:p-8">
    <div class="max-w-4xl space-y-5">
      <div class="flex flex-wrap items-start justify-between gap-4">
        <div class="space-y-2">
          <p class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--brand)]">{eyebrow}</p>
          <h2 class="text-2xl font-black tracking-[-0.04em] text-[var(--fg-primary)] md:text-4xl">{title}</h2>
        </div>
        <Badge className="bg-[var(--brand-soft)] text-[var(--brand)]">Structured editor</Badge>
      </div>

      <p class="text-sm leading-7 text-[var(--fg-secondary)] md:text-[15px]">{description}</p>

      <div class="grid gap-3 md:grid-cols-3">
        {#each bullets as bullet}
          <div class="rounded-[1.5rem] border border-[var(--border-soft)] bg-[var(--bg-panel-strong)] p-4 text-sm leading-7 text-[var(--fg-secondary)]">
            {bullet}
          </div>
        {/each}
      </div>

      <div class="flex flex-wrap gap-3">
        <Button>{cta}</Button>
        <Button variant="secondary">Open inspector</Button>
      </div>
    </div>
  </Card>
</section>
