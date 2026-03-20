<script lang="ts">
  import Badge from "$ui/badge.svelte";
  import Button from "$ui/button.svelte";
  import Card from "$ui/card.svelte";
  import type { MessageVersionView } from "$lib/api/messages";

  export let conversationTitle = "Conversation";
  export let loading = false;
  export let messages: MessageVersionView[] = [];
</script>

<section class="flex min-h-[calc(100vh-9.5rem)] flex-col gap-4 px-4 py-4 md:px-6 md:py-5 lg:px-8">
  <Card className="rounded-[2rem] border border-[var(--border-soft)] bg-white/88 p-5">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="space-y-2">
        <p class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--brand)]">Chat workspace</p>
        <h2 class="text-2xl font-black tracking-[-0.04em] text-[var(--fg-primary)] md:text-3xl">
          Incremental-first conversation shell
        </h2>
        <p class="max-w-3xl text-sm leading-7 text-[var(--fg-secondary)] md:text-[15px]">
          This first pass establishes the conversation timeline, inspector relationship and mobile-safe composer dock. The chat lane now reads real conversations and visible messages from the new backend.
        </p>
      </div>

      <div class="flex flex-wrap gap-2">
        <Badge>Visible chain</Badge>
        <Badge className="bg-emerald-50 text-emerald-700">Streaming</Badge>
        <Badge className="bg-orange-50 text-orange-700">Mobile ready</Badge>
      </div>
    </div>
  </Card>

  <div class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[minmax(0,1fr)_320px]">
    <Card className="rounded-[2rem] border border-[var(--border-soft)] bg-white/92 p-4 md:p-5">
      <div class="flex h-full flex-col gap-4">
        <div class="flex items-center justify-between gap-3">
          <h3 class="text-sm font-semibold uppercase tracking-[0.08em] text-[var(--fg-muted)]">Timeline</h3>
          <Button variant="secondary" size="sm">Version map</Button>
        </div>

        <div class="flex flex-1 flex-col gap-4 overflow-auto">
          {#if loading}
            <article class="rounded-[1.6rem] border border-[var(--border-soft)] bg-[var(--bg-panel-strong)] p-4 text-sm text-[var(--fg-secondary)]">
              Loading visible messages…
            </article>
          {:else if messages.length === 0}
            <article class="rounded-[1.6rem] border border-dashed border-[var(--border-strong)] bg-[var(--bg-panel-strong)] p-6">
              <p class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--brand)]">Empty conversation</p>
              <h4 class="mt-2 text-lg font-bold text-[var(--fg-primary)]">{conversationTitle}</h4>
              <p class="mt-2 max-w-2xl text-sm leading-7 text-[var(--fg-secondary)]">
                No visible messages yet. Once the composer is connected, user and assistant messages will stream into this timeline without a full reload.
              </p>
            </article>
          {:else}
            {#each messages as message}
              <article class="rounded-[1.6rem] border border-[var(--border-soft)] bg-[var(--bg-panel-strong)] p-4">
                <div class="flex items-center justify-between gap-3">
                  <div class="flex items-center gap-2">
                    <span class="inline-flex h-8 w-8 items-center justify-center rounded-full bg-[var(--bg-soft)] text-xs font-bold text-[var(--fg-primary)]">
                      {message.role === "user" ? "你" : message.role === "assistant" ? "步" : "系"}
                    </span>
                    <div>
                      <h4 class="text-sm font-semibold capitalize text-[var(--fg-primary)]">{message.role}</h4>
                      <p class="text-xs text-[var(--fg-muted)]">
                        {message.created_at} · v{message.version_index}
                      </p>
                    </div>
                  </div>

                  <Badge className="bg-white text-[var(--fg-secondary)]">
                    {message.is_active ? "active" : "inactive"}
                  </Badge>
                </div>

                <p class="mt-4 max-w-3xl whitespace-pre-wrap text-sm leading-7 text-[var(--fg-primary)] md:text-[15px]">
                  {message.primary_content.text_content ?? message.primary_content.preview_text ?? "No text body"}
                </p>
              </article>
            {/each}
          {/if}
        </div>
      </div>
    </Card>

    <Card className="rounded-[2rem] border border-[var(--border-soft)] bg-white/92 p-5">
      <div class="space-y-4">
        <div class="flex items-center justify-between gap-3">
          <h3 class="text-sm font-semibold uppercase tracking-[0.08em] text-[var(--fg-muted)]">Composer</h3>
          <Badge className="bg-blue-50 text-blue-700">Workflow aware</Badge>
        </div>

        <div class="rounded-[1.6rem] border border-[var(--border-soft)] bg-[var(--bg-soft)] p-4">
          <p class="text-sm leading-7 text-[var(--fg-secondary)]">
            The final composer will support attachments, target agent selection, workflow selection and streaming controls without forcing a route change.
          </p>
        </div>

        <div class="grid gap-2">
          <Button>Send message</Button>
          <Button variant="secondary">Attach content</Button>
          <Button variant="ghost">Insert system message</Button>
        </div>
      </div>
    </Card>
  </div>
</section>
