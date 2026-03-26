<script lang="ts">
  import { onDestroy } from "svelte";
  import { cn } from "$lib/utils.js";
  import { renderRichText } from "$lib/rich-text";

  type Props = {
    content: string | null | undefined;
    class?: string;
    throttleMs?: number;
  };

  let { content, class: className = "", throttleMs = 0 }: Props = $props();
  let renderedContent = $state("");
  let queuedContent = $state("");
  let flushTimer: ReturnType<typeof setTimeout> | null = null;

  onDestroy(() => {
    if (flushTimer) {
      clearTimeout(flushTimer);
    }
  });

  function flushQueuedContent() {
    renderedContent = queuedContent;
    flushTimer = null;
  }

  $effect(() => {
    const nextContent = content ?? "";

    if (throttleMs <= 0 || nextContent.length < renderedContent.length) {
      queuedContent = nextContent;
      renderedContent = nextContent;
      if (flushTimer) {
        clearTimeout(flushTimer);
        flushTimer = null;
      }
      return;
    }

    if (renderedContent === "") {
      queuedContent = nextContent;
      renderedContent = nextContent;
      return;
    }

    queuedContent = nextContent;
    if (flushTimer || queuedContent === renderedContent) {
      return;
    }

    flushTimer = setTimeout(flushQueuedContent, throttleMs);
  });

  let html = $derived(renderRichText(renderedContent));
</script>

<div class={cn("rich-text-content", className)}>
  {@html html}
</div>

<style>
  .rich-text-content {
    max-width: 100%;
    min-width: 0;
    overflow-wrap: anywhere;
    user-select: text;
  }

  .rich-text-content :global(*:first-child) {
    margin-top: 0;
  }

  .rich-text-content :global(*:last-child) {
    margin-bottom: 0;
  }

  .rich-text-content :global(h1),
  .rich-text-content :global(h2),
  .rich-text-content :global(h3),
  .rich-text-content :global(h4) {
    margin: 1.25rem 0 0.75rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1.35;
  }

  .rich-text-content :global(h1) {
    font-size: 1.5rem;
  }

  .rich-text-content :global(h2) {
    font-size: 1.25rem;
  }

  .rich-text-content :global(h3) {
    font-size: 1.125rem;
  }

  .rich-text-content :global(p),
  .rich-text-content :global(ul),
  .rich-text-content :global(ol),
  .rich-text-content :global(blockquote),
  .rich-text-content :global(pre),
  .rich-text-content :global(table),
  .rich-text-content :global(hr) {
    margin: 0.9rem 0;
  }

  .rich-text-content :global(p),
  .rich-text-content :global(li),
  .rich-text-content :global(blockquote),
  .rich-text-content :global(td),
  .rich-text-content :global(th) {
    overflow-wrap: anywhere;
    word-break: break-word;
  }

  .rich-text-content :global(ul),
  .rich-text-content :global(ol) {
    padding-left: 1.4rem;
  }

  .rich-text-content :global(li) {
    margin: 0.35rem 0;
  }

  .rich-text-content :global(blockquote) {
    border-left: 3px solid color-mix(in srgb, currentColor 18%, transparent);
    color: color-mix(in srgb, currentColor 72%, transparent);
    padding-left: 1rem;
  }

  .rich-text-content :global(a) {
    color: inherit;
    text-decoration: underline;
    text-underline-offset: 0.18em;
  }

  .rich-text-content :global(code) {
    background: color-mix(in srgb, currentColor 8%, transparent);
    border-radius: 0.375rem;
    font-family: var(--font-mono);
    font-size: 0.875em;
    padding: 0.125rem 0.375rem;
  }

  .rich-text-content :global(pre) {
    background: hsl(240 6% 10%);
    border: 1px solid hsl(240 4% 16%);
    border-radius: 0.75rem;
    color: hsl(0 0% 90%);
    font-size: 0.8125rem;
    line-height: 1.6;
    max-width: 100%;
    overflow-x: auto;
    padding: 0.9rem 1rem;
  }

  .rich-text-content :global(pre code) {
    background: transparent;
    padding: 0;
  }

  .rich-text-content :global(table) {
    border-collapse: collapse;
    display: block;
    font-size: 0.95em;
    max-width: 100%;
    overflow-x: auto;
    white-space: nowrap;
  }

  .rich-text-content :global(th),
  .rich-text-content :global(td) {
    border: 1px solid color-mix(in srgb, currentColor 12%, transparent);
    padding: 0.55rem 0.75rem;
    text-align: left;
    vertical-align: top;
  }

  .rich-text-content :global(th) {
    background: color-mix(in srgb, currentColor 5%, transparent);
    font-weight: 600;
  }

  .rich-text-content :global(img) {
    border-radius: 0.75rem;
    display: block;
    margin: 1rem 0;
    max-width: 100%;
  }

  .rich-text-content :global(hr) {
    border: 0;
    border-top: 1px solid color-mix(in srgb, currentColor 12%, transparent);
  }
</style>
