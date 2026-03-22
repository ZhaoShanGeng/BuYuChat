<script lang="ts">
  import { marked } from "marked";
  import { cn } from "$lib/utils";

  let {
    text = "",
    className = ""
  }: {
    text?: string;
    className?: string;
  } = $props();

  marked.setOptions({ breaks: true, gfm: true });

  function renderMarkdown(content: string): string {
    try {
      return marked.parse(content) as string;
    } catch {
      return content;
    }
  }
</script>

<div class={cn("rich-content text-sm leading-relaxed text-[var(--ink-body)]", className)}>
  {@html renderMarkdown(text)}
</div>

<style>
  :global(.rich-content) {
    word-wrap: break-word;
    overflow-wrap: break-word;
  }

  :global(.rich-content p) {
    margin: 0.25em 0;
  }

  :global(.rich-content p:first-child) {
    margin-top: 0;
  }

  :global(.rich-content p:last-child) {
    margin-bottom: 0;
  }

  :global(.rich-content h1),
  :global(.rich-content h2),
  :global(.rich-content h3),
  :global(.rich-content h4) {
    margin: 0.75em 0 0.25em;
    color: var(--ink-strong);
    font-weight: 600;
  }

  :global(.rich-content h1) { font-size: 1.25em; }
  :global(.rich-content h2) { font-size: 1.125em; }
  :global(.rich-content h3) { font-size: 1em; }

  :global(.rich-content ul),
  :global(.rich-content ol) {
    margin: 0.5em 0;
    padding-left: 1.5em;
  }

  :global(.rich-content li) {
    margin: 0.15em 0;
  }

  :global(.rich-content code) {
    padding: 0.15em 0.35em;
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
    color: var(--ink-strong);
    font-size: 0.9em;
    font-family: ui-monospace, "SFMono-Regular", "Cascadia Code", "Consolas", monospace;
  }

  :global(.rich-content pre) {
    position: relative;
    margin: 0.5em 0;
    overflow-x: auto;
    border-radius: var(--radius-md);
    background: #1e1e2e;
    padding: 0.75em 1em;
    color: #cdd6f4;
  }

  :global(.rich-content pre code) {
    padding: 0;
    background: transparent;
    color: inherit;
    font-size: 0.85em;
  }

  :global(.rich-content blockquote) {
    margin: 0.5em 0;
    border-left: 3px solid var(--brand);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    background: var(--brand-soft);
    padding: 0.25em 0.75em;
    color: var(--ink-muted);
  }

  :global(.rich-content a) {
    color: var(--brand);
    text-decoration: underline;
  }

  :global(.rich-content table) {
    margin: 0.5em 0;
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9em;
  }

  :global(.rich-content th),
  :global(.rich-content td) {
    border: 1px solid var(--border-soft);
    padding: 0.35em 0.5em;
    text-align: left;
  }

  :global(.rich-content th) {
    background: var(--bg-hover);
    font-weight: 600;
  }

  :global(.rich-content hr) {
    margin: 0.75em 0;
    border: none;
    border-top: 1px solid var(--border-soft);
  }

  :global(.rich-content img) {
    margin: 0.5em 0;
    max-width: 100%;
    border-radius: var(--radius-md);
  }
</style>
