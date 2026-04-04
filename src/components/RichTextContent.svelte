<script lang="ts">
  import "highlight.js/styles/github.min.css";
  import "highlight.js/styles/github-dark.min.css";
  import "katex/dist/katex.min.css";
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
  const proseClass =
    "rich-text-content prose prose-zinc dark:prose-invert max-w-none min-w-0 break-words select-text text-[14px] leading-6 prose-headings:mb-2 prose-headings:mt-4 prose-headings:font-semibold prose-headings:tracking-tight prose-p:my-2 prose-ul:my-2 prose-ol:my-2 prose-li:my-1 prose-blockquote:text-muted-foreground prose-a:text-inherit prose-a:underline prose-a:underline-offset-4 prose-strong:text-inherit prose-code:rounded-md prose-code:bg-foreground/6 prose-code:px-1.5 prose-code:py-0.5 prose-code:font-mono prose-code:text-[0.875em] prose-code:font-normal prose-code:before:content-none prose-code:after:content-none prose-pre:my-3 prose-pre:bg-transparent prose-pre:p-0 prose-img:rounded-xl";

  async function handleContentClick(event: MouseEvent) {
    const target = event.target;
    if (!(target instanceof HTMLButtonElement)) {
      return;
    }

    if (!target.classList.contains("code-copy-btn")) {
      return;
    }

    const encodedCode = target.dataset.code;
    if (!encodedCode) {
      return;
    }

    const originalLabel = target.textContent ?? "复制";
    await navigator.clipboard.writeText(decodeURIComponent(encodedCode));
    target.textContent = "已复制";
    target.disabled = true;

    window.setTimeout(() => {
      target.textContent = originalLabel;
      target.disabled = false;
    }, 1500);
  }

  onDestroy(() => {
    if (flushTimer) {
      clearTimeout(flushTimer);
    }
  });

  function flushQueuedContent() {
    renderedContent = queuedContent;
    flushTimer = null;
  }

  function bindContentClick(node: HTMLDivElement) {
    const listener = (event: MouseEvent) => {
      void handleContentClick(event);
    };

    node.addEventListener("click", listener);

    return {
      destroy() {
        node.removeEventListener("click", listener);
      }
    };
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

  let html = $derived.by(() => renderRichText(renderedContent));
  let classes = $derived(cn(proseClass, className));
</script>

<div class={classes} use:bindContentClick>
  {@html html}
</div>

<style>
  .rich-text-content {
    color: inherit;
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

  .rich-text-content :global(p),
  .rich-text-content :global(li),
  .rich-text-content :global(blockquote),
  .rich-text-content :global(td),
  .rich-text-content :global(th) {
    overflow-wrap: anywhere;
    word-break: break-word;
  }

  .rich-text-content :global(a) {
    text-decoration-thickness: 1px;
  }

  .rich-text-content :global(::selection) {
    background: color-mix(in srgb, currentColor 18%, transparent);
    color: inherit;
  }

  .rich-text-content :global(pre) {
    border-radius: 0.75rem;
    font-size: 0.8125rem;
    line-height: 1.6;
    max-width: 100%;
    overflow-x: auto;
    padding: 0;
  }

  .rich-text-content :global(pre code) {
    background: transparent;
    display: block;
    min-width: max-content;
    overflow-wrap: normal;
    padding: 1rem;
  }

  .rich-text-content :global(.code-block-wrapper) {
    border-radius: 0.95rem;
    margin: 0.75rem 0;
    overflow: hidden;
  }

  .rich-text-content :global(.code-block-header) {
    align-items: center;
    display: flex;
    gap: 0.75rem;
    justify-content: space-between;
    padding: 0.65rem 0.85rem;
  }

  .rich-text-content :global(.code-lang) {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .rich-text-content :global(.code-copy-btn) {
    background: transparent;
    border-radius: 9999px;
    border-width: 1px;
    border-style: solid;
    cursor: pointer;
    font-size: 0.72rem;
    line-height: 1;
    padding: 0.35rem 0.65rem;
    transition:
      background-color 140ms ease,
      border-color 140ms ease,
      color 140ms ease;
  }

  .rich-text-content :global(.code-copy-btn:disabled) {
    cursor: default;
    opacity: 0.9;
  }

  .rich-text-content :global(.code-block-wrapper pre) {
    margin: 0;
  }

  .rich-text-content :global(.katex-display) {
    margin: 0.75rem 0;
    overflow-x: auto;
    overflow-y: hidden;
    padding: 0.25rem 0;
  }

  .rich-text-content :global(.katex-display > .katex) {
    min-width: max-content;
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

  .rich-text-content :global(hr) {
    border: 0;
    border-top: 1px solid color-mix(in srgb, currentColor 12%, transparent);
  }
</style>
