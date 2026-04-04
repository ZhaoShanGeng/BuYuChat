<script lang="ts">
  import "highlight.js/styles/github.min.css";
  import "highlight.js/styles/github-dark.min.css";
  import "katex/dist/katex.min.css";
  import { onDestroy, onMount } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { cn } from "$lib/utils.js";
  import { renderRichText } from "$lib/rich-text";

  type Props = {
    content: string | null | undefined;
    class?: string;
    throttleMs?: number;
  };

  type PreviewKind = "html" | "svg" | "markdown";
  type DevicePreset = "auto" | "desktop" | "tablet" | "phone";
  type PreviewMetrics = {
    width: number;
    height: number;
  };
  type FullscreenPreview = {
    id: string;
    kind: PreviewKind;
    source: string;
    label: string;
  };

  const DEVICE_LABELS: Record<DevicePreset, string> = {
    auto: "自动",
    desktop: "桌面",
    tablet: "平板",
    phone: "手机"
  };

  let { content, class: className = "", throttleMs = 0 }: Props = $props();
  let renderedContent = $state("");
  let queuedContent = $state("");
  let flushTimer: ReturnType<typeof setTimeout> | null = null;
  let previewSequence = 0;
  let fullscreenOpen = $state(false);
  let fullscreenPreset = $state<DevicePreset>("auto");
  let fullscreenPreview = $state<FullscreenPreview | null>(null);
  let viewportWidth = $state(1280);
  let viewportHeight = $state(800);
  let previewMetrics = $state<Record<string, PreviewMetrics>>({});

  const proseClass =
    "rich-text-content prose prose-zinc dark:prose-invert max-w-none min-w-0 break-words select-text text-[14px] leading-6 prose-headings:mb-2 prose-headings:mt-4 prose-headings:font-semibold prose-headings:tracking-tight prose-p:my-2 prose-ul:my-2 prose-ol:my-2 prose-li:my-1 prose-blockquote:text-muted-foreground prose-a:text-inherit prose-a:underline prose-a:underline-offset-4 prose-strong:text-inherit prose-code:rounded-md prose-code:bg-foreground/6 prose-code:px-1.5 prose-code:py-0.5 prose-code:font-mono prose-code:text-[0.875em] prose-code:font-normal prose-code:before:content-none prose-code:after:content-none prose-pre:my-3 prose-pre:bg-transparent prose-pre:p-0 prose-img:rounded-xl";

  function nextPreviewId() {
    previewSequence += 1;
    return `preview-${previewSequence}`;
  }

  function decodePreviewSource(source: string | undefined): string {
    if (!source) {
      return "";
    }

    try {
      return decodeURIComponent(source);
    } catch {
      return "";
    }
  }

  function updateViewport() {
    viewportWidth = window.innerWidth;
    viewportHeight = window.innerHeight;
  }

  function buildResizeBridge(previewId: string): string {
    return [
      "<script>",
      "(() => {",
      `  const previewId = ${JSON.stringify(previewId)};`,
      "  const postSize = () => {",
      "    const doc = document.documentElement;",
      "    const body = document.body;",
      "    const width = Math.max(",
      "      doc?.scrollWidth || 0,",
      "      body?.scrollWidth || 0,",
      "      doc?.offsetWidth || 0,",
      "      body?.offsetWidth || 0,",
      "      window.innerWidth || 0",
      "    );",
      "    const height = Math.max(",
      "      doc?.scrollHeight || 0,",
      "      body?.scrollHeight || 0,",
      "      doc?.offsetHeight || 0,",
      "      body?.offsetHeight || 0,",
      "      window.innerHeight || 0",
      "    );",
      "    parent.postMessage({ type: 'buyu-preview-size', previewId, width, height }, '*');",
      "  };",
      "  const queuePost = () => requestAnimationFrame(postSize);",
      "  window.addEventListener('load', () => {",
      "    postSize();",
      "    setTimeout(postSize, 32);",
      "    setTimeout(postSize, 180);",
      "  });",
      "  window.addEventListener('resize', queuePost);",
      "  if ('ResizeObserver' in window) {",
      "    new ResizeObserver(queuePost).observe(document.documentElement);",
      "  }",
      "  if ('MutationObserver' in window) {",
      "    new MutationObserver(queuePost).observe(document.documentElement, {",
      "      childList: true,",
      "      subtree: true,",
      "      attributes: true,",
      "      characterData: true",
      "    });",
      "  }",
      "  setInterval(postSize, 1200);",
      "  postSize();",
      "})();",
      "<\/script>"
    ].join("\n");
  }

  function injectResizeBridge(documentHtml: string, previewId: string): string {
    const bridge = buildResizeBridge(previewId);

    if (/<\/body>/i.test(documentHtml)) {
      return documentHtml.replace(/<\/body>/i, `${bridge}\n</body>`);
    }

    if (/<\/head>/i.test(documentHtml)) {
      return documentHtml.replace(/<\/head>/i, `${bridge}\n</head>`);
    }

    return `${documentHtml}\n${bridge}`;
  }

  function buildPreviewDocument(kind: PreviewKind, source: string, previewId: string): string {
    if (kind === "markdown") {
      return "";
    }

    if (kind === "svg") {
      return injectResizeBridge(
        [
          "<!doctype html>",
          '<html lang="en">',
          "<head>",
          '  <meta charset="utf-8" />',
          '  <meta name="viewport" content="width=device-width, initial-scale=1" />',
          "  <style>",
          "    :root { color-scheme: light; }",
          "    html, body {",
          "      margin: 0;",
          "      min-height: 100%;",
          "      background: transparent;",
          "    }",
          "    body {",
          "      display: grid;",
          "      min-height: 100vh;",
          "      place-items: center;",
          "      padding: 16px;",
          "      box-sizing: border-box;",
          "    }",
          "    svg {",
          "      display: block;",
          "      height: auto;",
          "      max-width: 100%;",
          "    }",
          "  </style>",
          "</head>",
          `<body>${source}</body>`,
          "</html>"
        ].join("\n"),
        previewId
      );
    }

    if (/<!doctype html/i.test(source) || /<html[\s>]/i.test(source)) {
      return injectResizeBridge(source, previewId);
    }

    return injectResizeBridge(
      [
        "<!doctype html>",
        '<html lang="en">',
        "<head>",
        '  <meta charset="utf-8" />',
        '  <meta name="viewport" content="width=device-width, initial-scale=1" />',
        "  <style>",
        "    html, body {",
        "      margin: 0;",
        "      min-height: 100%;",
        "    }",
        "    body {",
        "      box-sizing: border-box;",
        "      min-height: 100vh;",
        "    }",
        "    img, svg, canvas, video {",
        "      max-width: 100%;",
        "      height: auto;",
        "    }",
        "  </style>",
        "</head>",
        `<body>${source}</body>`,
        "</html>"
      ].join("\n"),
      previewId
    );
  }

  function getPreviewMetrics(previewId: string | null | undefined) {
    return previewId ? previewMetrics[previewId] : undefined;
  }

  function resolveInlineHeight(previewId: string) {
    const measuredHeight = getPreviewMetrics(previewId)?.height ?? 352;
    const maxHeight = Math.max(320, Math.floor(viewportHeight * 0.68));
    return Math.min(maxHeight, Math.max(288, Math.ceil(measuredHeight)));
  }

  function applyInlineFrameMetrics(previewId: string) {
    const inlineHeight = resolveInlineHeight(previewId);
    for (const frame of window.document.querySelectorAll<HTMLIFrameElement>(
      `.code-preview-frame[data-preview-id="${previewId}"]`
    )) {
      frame.style.height = `${inlineHeight}px`;
    }
  }

  function createPreviewFrame(
    previewId: string,
    srcdoc: string,
    className: "code-preview-frame" | "code-preview-frame code-preview-frame--fullscreen"
  ) {
    const frame = window.document.createElement("iframe");
    frame.className = className;
    frame.dataset.previewId = previewId;
    frame.setAttribute("loading", "lazy");
    frame.setAttribute("referrerpolicy", "no-referrer");
    frame.setAttribute("sandbox", "allow-scripts");
    frame.srcdoc = srcdoc;
    return frame;
  }

  function ensurePreviewPanel(panel: HTMLDivElement) {
    if (panel.dataset.previewReady === "true") {
      return;
    }

    const kind = panel.dataset.previewKind as PreviewKind | undefined;
    const source = decodePreviewSource(panel.dataset.previewSource).trim();

    if (!kind || !source) {
      panel.dataset.previewReady = "true";
      return;
    }

    if (kind === "markdown") {
      const surface = window.document.createElement("div");
      surface.className = "code-preview-surface code-preview-surface--markdown";
      surface.innerHTML = renderRichText(source);
      panel.replaceChildren(surface);
      panel.dataset.previewReady = "true";
      return;
    }

    const previewId = panel.dataset.previewId ?? nextPreviewId();
    panel.dataset.previewId = previewId;
    const frame = createPreviewFrame(
      previewId,
      buildPreviewDocument(kind, source, previewId),
      "code-preview-frame"
    );
    frame.style.height = `${resolveInlineHeight(previewId)}px`;
    panel.replaceChildren(frame);
    panel.dataset.previewReady = "true";
  }

  function activateCodeView(button: HTMLButtonElement) {
    const wrapper = button.closest<HTMLDivElement>(".code-block-wrapper--previewable");
    if (!wrapper) {
      return;
    }

    const nextView = button.dataset.view;
    if (!nextView) {
      return;
    }

    wrapper.dataset.activeView = nextView;

    for (const viewButton of wrapper.querySelectorAll<HTMLButtonElement>(".code-view-btn")) {
      const isActive = viewButton === button;
      viewButton.classList.toggle("is-active", isActive);
      viewButton.setAttribute("aria-pressed", String(isActive));
    }

    for (const panel of wrapper.querySelectorAll<HTMLDivElement>("[data-panel]")) {
      const isActive = panel.dataset.panel === nextView;
      panel.hidden = !isActive;
      panel.classList.toggle("is-active", isActive);

      if (isActive && panel.dataset.panel === "preview") {
        ensurePreviewPanel(panel);
      }
    }
  }

  function openFullscreenPreview(button: HTMLButtonElement) {
    const wrapper = button.closest<HTMLDivElement>(".code-block-wrapper--previewable");
    if (!wrapper) {
      return;
    }

    const previewPanel = wrapper.querySelector<HTMLDivElement>('[data-panel="preview"]');
    const kind = previewPanel?.dataset.previewKind as PreviewKind | undefined;
    const source = decodePreviewSource(previewPanel?.dataset.previewSource).trim();
    const label = wrapper.querySelector<HTMLElement>(".code-lang")?.textContent?.trim() ?? "preview";

    if (!kind || !source) {
      return;
    }

    fullscreenPreview = {
      id: nextPreviewId(),
      kind,
      source,
      label
    };
    fullscreenPreset = "auto";
    fullscreenOpen = true;
  }

  function handlePreviewMessage(event: MessageEvent) {
    const payload = event.data;
    if (!payload || typeof payload !== "object" || payload.type !== "buyu-preview-size") {
      return;
    }

    const previewId =
      "previewId" in payload && typeof payload.previewId === "string" ? payload.previewId : null;
    const width = "width" in payload ? Number(payload.width) : NaN;
    const height = "height" in payload ? Number(payload.height) : NaN;

    if (!previewId || !Number.isFinite(width) || !Number.isFinite(height)) {
      return;
    }

    previewMetrics[previewId] = {
      width: Math.max(0, Math.round(width)),
      height: Math.max(0, Math.round(height))
    };
    applyInlineFrameMetrics(previewId);
  }

  function closeFullscreenPreview() {
    fullscreenOpen = false;
  }

  async function handleCopyButton(button: HTMLButtonElement) {
    const encodedCode = button.dataset.code;
    if (!encodedCode) {
      return;
    }

    const originalLabel = button.textContent ?? "复制";
    await navigator.clipboard.writeText(decodeURIComponent(encodedCode));
    button.textContent = "已复制";
    button.disabled = true;

    window.setTimeout(() => {
      button.textContent = originalLabel;
      button.disabled = false;
    }, 1500);
  }

  async function handleContentClick(event: MouseEvent) {
    const target = event.target;
    if (!(target instanceof Element)) {
      return;
    }

    const viewButton = target.closest<HTMLButtonElement>(".code-view-btn");
    if (viewButton) {
      activateCodeView(viewButton);
      return;
    }

    const fullscreenButton = target.closest<HTMLButtonElement>(".code-fullscreen-btn");
    if (fullscreenButton) {
      openFullscreenPreview(fullscreenButton);
      return;
    }

    const copyButton = target.closest<HTMLButtonElement>(".code-copy-btn");
    if (copyButton) {
      await handleCopyButton(copyButton);
    }
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

  function flushQueuedContent() {
    renderedContent = queuedContent;
    flushTimer = null;
  }

  function resolveFullscreenWidth(preset: DevicePreset, availableWidth: number, measuredWidth?: number) {
    if (availableWidth <= 0) {
      return 0;
    }

    if (viewportWidth < 640) {
      return availableWidth;
    }

    if (preset === "phone") {
      return Math.min(430, availableWidth);
    }

    if (preset === "tablet") {
      return Math.min(900, availableWidth);
    }

    if (preset === "desktop") {
      return Math.min(availableWidth, 1480);
    }

    return Math.min(availableWidth, Math.max(390, Math.ceil(measuredWidth ?? availableWidth)));
  }

  function resolveFullscreenHeight(
    preset: DevicePreset,
    availableHeight: number,
    measuredHeight?: number
  ) {
    if (availableHeight <= 0) {
      return 0;
    }

    if (preset === "auto") {
      return Math.min(availableHeight, Math.max(360, Math.ceil(measuredHeight ?? availableHeight)));
    }

    return availableHeight;
  }

  let html = $derived.by(() => renderRichText(renderedContent));
  let classes = $derived(cn(proseClass, className));
  let fullscreenPreviewMetrics = $derived.by(() =>
    fullscreenPreview ? getPreviewMetrics(fullscreenPreview.id) : undefined
  );
  let fullscreenPreviewDocument = $derived.by(() =>
    fullscreenPreview && fullscreenPreview.kind !== "markdown"
      ? buildPreviewDocument(fullscreenPreview.kind, fullscreenPreview.source, fullscreenPreview.id)
      : ""
  );
  let fullscreenPreviewMarkup = $derived.by(() =>
    fullscreenPreview?.kind === "markdown" ? renderRichText(fullscreenPreview.source) : ""
  );
  let fullscreenStageStyle = $derived.by(() => {
    if (!fullscreenPreview) {
      return "";
    }

    const availableWidth = Math.max(320, viewportWidth - (viewportWidth < 768 ? 24 : 56));
    const availableHeight = Math.max(360, viewportHeight - (viewportWidth < 768 ? 116 : 152));
    const width = resolveFullscreenWidth(
      fullscreenPreset,
      availableWidth,
      fullscreenPreviewMetrics?.width
    );
    const height = resolveFullscreenHeight(
      fullscreenPreset,
      availableHeight,
      fullscreenPreviewMetrics?.height
    );

    return `width:${Math.round(width)}px;height:${Math.round(height)}px;`;
  });
  let fullscreenMeta = $derived.by(() => {
    if (!fullscreenPreviewMetrics) {
      return "等待页面尺寸…";
    }

    return `${fullscreenPreviewMetrics.width}px × ${fullscreenPreviewMetrics.height}px`;
  });

  onMount(() => {
    updateViewport();
    window.addEventListener("resize", updateViewport);
    window.addEventListener("message", handlePreviewMessage);

    return () => {
      window.removeEventListener("resize", updateViewport);
      window.removeEventListener("message", handlePreviewMessage);
    };
  });

  onDestroy(() => {
    if (flushTimer) {
      clearTimeout(flushTimer);
    }
  });

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

  $effect(() => {
    if (!fullscreenOpen) {
      fullscreenPreview = null;
    }
  });
</script>

<div class={classes} use:bindContentClick>
  {@html html}
</div>

<Dialog.Root bind:open={fullscreenOpen}>
  <Dialog.Content
    class="code-preview-dialog flex h-dvh max-w-none flex-col gap-0 rounded-none border-0 bg-slate-950/96 p-0 text-slate-50 shadow-none ring-0 sm:h-[100dvh] sm:w-screen"
    portalProps={{}}
    showCloseButton={false}
  >
    {#if fullscreenPreview}
      <div class="code-preview-dialog__toolbar flex flex-wrap items-center justify-between gap-3 border-b border-white/10 px-3 py-3 sm:px-4">
        <div class="min-w-0">
          <div class="truncate text-sm font-medium text-slate-100">{fullscreenPreview.label} 预览</div>
          <div class="text-xs text-slate-400">{fullscreenMeta}</div>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <div class="inline-flex rounded-full bg-white/6 p-1">
            {#each Object.keys(DEVICE_LABELS) as presetKey (presetKey)}
              {@const preset = presetKey as DevicePreset}
              <Button
                class={cn(
                  "rounded-full px-3 text-[11px]",
                  fullscreenPreset === preset
                    ? "bg-white text-slate-950 hover:bg-white/90"
                    : "border-transparent text-slate-300 hover:bg-white/10"
                )}
                onclick={() => (fullscreenPreset = preset)}
                size="xs"
                variant="ghost"
              >
                {DEVICE_LABELS[preset]}
              </Button>
            {/each}
          </div>

          <Button onclick={closeFullscreenPreview} size="sm" variant="outline">
            关闭
          </Button>
        </div>
      </div>

      <div class="code-preview-dialog__stage flex min-h-0 flex-1 items-center justify-center overflow-auto p-3 sm:p-5">
        {#if fullscreenPreview.kind === "markdown"}
          <div class="code-preview-dialog__surface max-w-full overflow-auto rounded-[1.25rem] border border-white/10 bg-white px-4 py-4 text-slate-900 shadow-2xl sm:px-6" style={fullscreenStageStyle}>
            <div class="code-preview-surface code-preview-surface--markdown max-w-none">
              {@html fullscreenPreviewMarkup}
            </div>
          </div>
        {:else}
          <div class="code-preview-dialog__frame-shell max-w-full overflow-hidden rounded-[1.5rem] border border-white/10 bg-white shadow-2xl" style={fullscreenStageStyle}>
            <iframe
              class="code-preview-frame code-preview-frame--fullscreen"
              data-preview-id={fullscreenPreview.id}
              loading="lazy"
              referrerpolicy="no-referrer"
              sandbox="allow-scripts"
              srcdoc={fullscreenPreviewDocument}
              title={`${fullscreenPreview.label} fullscreen preview`}
            ></iframe>
          </div>
        {/if}
      </div>
    {/if}
  </Dialog.Content>
</Dialog.Root>

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

  .rich-text-content :global(.code-block-actions) {
    align-items: center;
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .rich-text-content :global(.code-view-switch) {
    align-items: center;
    background: color-mix(in srgb, currentColor 6%, transparent);
    border-radius: 9999px;
    display: inline-flex;
    padding: 0.15rem;
  }

  .rich-text-content :global(.code-view-btn),
  .rich-text-content :global(.code-fullscreen-btn) {
    background: transparent;
    border: 0;
    border-radius: 9999px;
    color: color-mix(in srgb, currentColor 68%, transparent);
    font-size: 0.72rem;
    line-height: 1;
    padding: 0.4rem 0.7rem;
    transition:
      background-color 140ms ease,
      color 140ms ease;
  }

  .rich-text-content :global(.code-view-btn.is-active),
  .rich-text-content :global(.code-fullscreen-btn:hover) {
    background: color-mix(in srgb, currentColor 12%, transparent);
    color: inherit;
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

  .rich-text-content :global(.code-block-panels) {
    display: grid;
  }

  .rich-text-content :global(.code-block-panel),
  .rich-text-content :global(.code-preview-panel) {
    min-width: 0;
  }

  .rich-text-content :global(.code-preview-panel) {
    background:
      linear-gradient(180deg, color-mix(in srgb, currentColor 2%, transparent), transparent),
      color-mix(in srgb, currentColor 2%, transparent);
    border-top: 1px solid color-mix(in srgb, currentColor 10%, transparent);
    display: flex;
    justify-content: center;
    min-height: 18rem;
    overflow: auto;
    padding: 0.85rem;
  }

  .rich-text-content :global(.code-preview-frame) {
    background: white;
    border: 0;
    border-radius: 0.9rem;
    display: block;
    max-width: 100%;
    min-height: 18rem;
    width: 100%;
  }

  .rich-text-content :global(.code-preview-frame--fullscreen) {
    border-radius: 0;
    height: 100%;
    min-height: 100%;
  }

  .rich-text-content :global(.code-preview-surface) {
    max-width: none;
    min-height: 18rem;
    overflow: auto;
    padding: 1rem;
  }

  .rich-text-content :global(.code-preview-surface--markdown) {
    color: inherit;
  }

  .code-preview-dialog__surface :global(.code-preview-surface) {
    min-height: 100%;
    padding: 0;
  }

  .code-preview-dialog__frame-shell,
  .code-preview-dialog__surface {
    transition:
      width 180ms ease,
      height 180ms ease;
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

  @media (max-width: 640px) {
    .rich-text-content :global(.code-block-header) {
      align-items: flex-start;
      flex-direction: column;
    }

    .rich-text-content :global(.code-block-actions) {
      width: 100%;
    }

    .rich-text-content :global(.code-view-switch) {
      flex: 1 1 auto;
      width: 100%;
    }

    .rich-text-content :global(.code-view-btn) {
      flex: 1 1 0;
    }
  }
</style>
