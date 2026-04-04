<script lang="ts">
  import { onDestroy } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import * as Avatar from "$lib/components/ui/avatar/index.js";
  import * as Textarea from "$lib/components/ui/textarea/index.js";
  import BotIcon from "@lucide/svelte/icons/bot";
  import BrainIcon from "@lucide/svelte/icons/brain";
  import CheckIcon from "@lucide/svelte/icons/check";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CircleUserIcon from "@lucide/svelte/icons/circle-user";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import SendHorizontalIcon from "@lucide/svelte/icons/send-horizontal";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import Clock3Icon from "@lucide/svelte/icons/clock-3";
  import SquareIcon from "@lucide/svelte/icons/square";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import WrenchIcon from "@lucide/svelte/icons/wrench";
  import { extractThinkingTags } from "../lib/thinking-tags";
  import RichTextContent from "./RichTextContent.svelte";
  import type { MessageNode } from "../lib/transport/messages";
  import { getActiveVersion, isNodeGenerating } from "./workspace-state";

  type Props = {
    node: MessageNode;
    isLast?: boolean;
    thinkingTags: string[];
    onCancel: (versionId: string) => void | Promise<void>;
    onReroll: (nodeId: string) => void | Promise<void>;
    onSwitchVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onDeleteVersion: (nodeId: string, versionId: string) => void | Promise<void>;
    onEditMessage: (
      nodeId: string,
      content: string,
      options?: { resend?: boolean }
    ) => void | Promise<void>;
    onLoadVersionContent: (nodeId: string, versionId: string) => Promise<string>;
  };

  const {
    node,
    isLast = false,
    thinkingTags,
    onCancel,
    onReroll,
    onSwitchVersion,
    onDeleteVersion,
    onEditMessage,
    onLoadVersionContent
  }: Props =
    $props();

  let activeVersion = $derived.by(() => getActiveVersion(node));
  let activeIndex = $derived(node.versions.findIndex((version) => version.id === node.activeVersionId) + 1);
  let extractedThinking = $derived.by(() =>
    node.role === "assistant"
      ? extractThinkingTags(activeVersion?.content ?? "", thinkingTags)
      : { thinking: null, body: activeVersion?.content ?? "" }
  );
  let displayBodyContent = $derived(
    node.role === "assistant" ? extractedThinking.body : (activeVersion?.content ?? "")
  );
  let displayThinking = $derived.by(() => {
    const parts = [activeVersion?.thinkingContent, extractedThinking.thinking]
      .map((part) => part?.trim() ?? "")
      .filter((part) => part.length > 0);
    return parts.length > 0 ? parts.join("\n\n") : null;
  });
  let copied = $state(false);
  let generating = $derived(isNodeGenerating(node));
  let editing = $state(false);
  let loadingEditContent = $state(false);
  let saving = $state(false);
  let draft = $state("");
  let toolbarVisible = $state(false);
  let thinkingOpen = $state(false);
  let copyResetTimer: ReturnType<typeof setTimeout> | null = null;
  let isTouchDevice = $derived(typeof window !== "undefined" && "ontouchstart" in window);
  let showToolbar = $derived(toolbarVisible || editing || isLast || isTouchDevice);

  $effect(() => {
    if (generating && displayThinking) {
      thinkingOpen = true;
    }
  });

  onDestroy(() => {
    if (copyResetTimer) {
      clearTimeout(copyResetTimer);
    }
  });

  function handlePointerEnter() {
    toolbarVisible = true;
  }

  function handlePointerLeave() {
    if (!editing) {
      toolbarVisible = false;
    }
  }

  async function handleCopy() {
    const text = displayBodyContent;
    if (!text) return;
    await navigator.clipboard.writeText(text);
    copied = true;
    if (copyResetTimer) {
      clearTimeout(copyResetTimer);
    }
    copyResetTimer = setTimeout(() => {
      copied = false;
      copyResetTimer = null;
    }, 1500);
  }

  async function startEdit() {
    if (!activeVersion) return;
    loadingEditContent = true;
    try {
      draft =
        activeVersion.content ??
        (await onLoadVersionContent(node.id, activeVersion.id));
      editing = true;
    } finally {
      loadingEditContent = false;
    }
  }

  function cancelEdit() {
    editing = false;
    draft = "";
  }

  async function saveEdit(resend = false) {
    if (!draft.trim()) {
      return;
    }

    saving = true;
    try {
      await onEditMessage(node.id, draft, { resend });
      editing = false;
      draft = "";
    } finally {
      saving = false;
    }
  }

  function handleDelete() {
    if (!activeVersion) return;
    void onDeleteVersion(node.id, activeVersion.id);
  }

  function prevVersion() {
    const index = activeIndex - 2;
    if (index >= 0) {
      void onSwitchVersion(node.id, node.versions[index].id);
    }
  }

  function nextVersion() {
    const index = activeIndex;
    if (index < node.versions.length) {
      void onSwitchVersion(node.id, node.versions[index].id);
    }
  }

  function formatCompactNumber(value: number | null | undefined) {
    if (value === null || value === undefined) {
      return "--";
    }

    const absoluteValue = Math.abs(value);

    if (absoluteValue >= 1_000_000) {
      return `${(value / 1_000_000).toFixed(1)}m`;
    }

    if (absoluteValue >= 1_000) {
      return `${(value / 1_000).toFixed(1)}k`;
    }

    return new Intl.NumberFormat("zh-CN").format(value);
  }

  function resolveMetricStartAt() {
    if (!activeVersion) {
      return null;
    }

    if (
      activeVersion.receivedAt &&
      activeVersion.completedAt &&
      activeVersion.completedAt > activeVersion.receivedAt
    ) {
      return activeVersion.receivedAt;
    }

    return activeVersion.createdAt;
  }

  function formatDurationSeconds(
    startedAt: number | null | undefined,
    completedAt: number | null | undefined
  ) {
    if (!startedAt || !completedAt || completedAt < startedAt) {
      return "--";
    }

    return `${((completedAt - startedAt) / 1000).toFixed(1)}s`;
  }

  function formatReceiveSpeed(
    completionTokens: number | null | undefined,
    startedAt: number | null | undefined,
    completedAt: number | null | undefined
  ) {
    if (
      completionTokens === null ||
      completionTokens === undefined ||
      !startedAt ||
      !completedAt ||
      completedAt <= startedAt
    ) {
      return "--";
    }

    const seconds = (completedAt - startedAt) / 1000;
    const speed = completionTokens / seconds;
    return `${formatCompactNumber(Number(speed.toFixed(1)))} tok/s`;
  }

  function formatDebugPayload(value: string | null | undefined) {
    if (!value) {
      return "";
    }

    try {
      return JSON.stringify(JSON.parse(value), null, 2);
    } catch {
      return value;
    }
  }

  function resolveImageSrc(image: { base64: string; mimeType: string; url?: string | null }) {
    if (image.url?.trim()) {
      return image.url;
    }

    if (image.base64.trim()) {
      return `data:${image.mimeType};base64,${image.base64}`;
    }

    return "";
  }

  let assistantImageFiles = $derived.by(() =>
    (activeVersion?.files ?? []).filter((file) => file.mimeType.startsWith("image/"))
  );

  let nonImageFiles = $derived.by(() =>
    (activeVersion?.files ?? []).filter((file) => !file.mimeType.startsWith("image/"))
  );
</script>

{#if node.role === "user"}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="message-card message-card--user flex items-start justify-end gap-2 pl-4 sm:gap-2.5 sm:pl-8 lg:pl-14"
    data-role="user"
    onmouseenter={handlePointerEnter}
    onmouseleave={handlePointerLeave}
  >
    <div class="relative w-fit max-w-[85%] sm:max-w-[72%] lg:max-w-[60%]">
      <Card.Root class="message-card__bubble message-card__bubble--user overflow-hidden rounded-[1.5rem] rounded-tr-md border border-primary/20 bg-primary/10 text-foreground shadow-sm">
        <Card.Content class="message-card__body message-card__body--user space-y-0 px-4 py-3 text-[14px] leading-[1.5] sm:px-5">
          {#if activeVersion?.images.length}
            <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
              {#each activeVersion.images as image (image.base64)}
                <img
                  alt="用户上传图片"
                  class="max-h-48 rounded-2xl object-cover"
                  src={resolveImageSrc(image)}
                />
              {/each}
            </div>
          {/if}
          {#if activeVersion?.files?.length}
            <div class="space-y-2 pt-3">
              {#each activeVersion.files ?? [] as file (`${file.name}-${file.base64.slice(0, 16)}`)}
                <div class="flex items-center gap-2 rounded-2xl border border-primary/20 bg-background/50 px-3 py-2 text-sm text-foreground">
                  <FileTextIcon class="size-4 shrink-0 text-muted-foreground" />
                  <div class="min-w-0 flex-1">
                    <div class="truncate">{file.name}</div>
                    <div class="truncate text-[11px] text-muted-foreground">{file.mimeType}</div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}

        {#if editing}
            <div class="space-y-3">
            <Textarea.Root
              bind:value={draft}
              class="min-h-[120px] resize-y border-primary-foreground/20 bg-primary/20 text-primary-foreground placeholder:text-primary-foreground/45"
              placeholder="编辑当前消息"
            />
            <div class="flex items-center justify-end gap-2">
              <Button disabled={saving} onclick={cancelEdit} size="sm" variant="secondary">取消</Button>
              <Button disabled={saving || !draft.trim()} onclick={() => void saveEdit(false)} size="sm" variant="secondary">
                保存
              </Button>
              <Button disabled={saving || !draft.trim()} onclick={() => void saveEdit(true)} size="sm">
                保存并重新发送
              </Button>
            </div>
            </div>
        {:else}
            <RichTextContent
              class="message-card__richtext message-card__richtext--user prose-neutral prose-p:text-foreground prose-headings:text-foreground prose-strong:text-foreground prose-blockquote:text-foreground/80 prose-code:bg-background/60 prose-a:text-primary prose-a:decoration-primary/60"
              content={displayBodyContent}
            />
        {/if}
        </Card.Content>
      </Card.Root>

      <div class="relative mt-1 h-6">
        {#if showToolbar}
          <div class="absolute right-0 top-0 flex items-center gap-0.5">
            <Button class="size-6" onclick={handleCopy} size="icon" variant="ghost" title="复制">
              {#if copied}
                <CheckIcon class="size-3 text-emerald-500" />
              {:else}
                <CopyIcon class="size-3 text-muted-foreground" />
              {/if}
            </Button>
            <Button class="size-6" disabled={loadingEditContent || saving} onclick={() => void startEdit()} size="icon" variant="ghost" title="编辑">
              {#if loadingEditContent}
                <LoaderCircleIcon class="size-3 animate-spin text-muted-foreground" />
              {:else}
                <PencilIcon class="size-3 text-muted-foreground" />
              {/if}
            </Button>
            <Button class="size-6" onclick={() => onReroll(node.id)} size="icon" variant="ghost" title="重发">
              <RotateCcwIcon class="size-3 text-muted-foreground" />
            </Button>
            <Button class="size-6" onclick={handleDelete} size="icon" variant="ghost" title="删除">
              <Trash2Icon class="size-3 text-muted-foreground" />
            </Button>
            {#if node.versions.length > 1}
              <div class="ml-0.5 flex items-center gap-0.5">
                <Button class="size-5" disabled={activeIndex <= 1} onclick={prevVersion} size="icon" variant="ghost">
                  <ChevronLeftIcon class="size-3" />
                </Button>
                <span class="min-w-6 text-center text-[11px] text-muted-foreground">{activeIndex}/{node.versions.length}</span>
                <Button class="size-5" disabled={activeIndex >= node.versions.length} onclick={nextVersion} size="icon" variant="ghost">
                  <ChevronRightIcon class="size-3" />
                </Button>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <Avatar.Root class="size-7 shrink-0 rounded-full">
      <Avatar.Fallback class="bg-primary/10">
        <CircleUserIcon class="size-4 text-primary" />
      </Avatar.Fallback>
    </Avatar.Root>
  </div>
{:else}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="message-card message-card--assistant flex items-start gap-2 pr-4 sm:gap-2.5 sm:pr-8 lg:pr-14"
    data-role="assistant"
    onmouseenter={handlePointerEnter}
    onmouseleave={handlePointerLeave}
  >
    <Avatar.Root class="size-7 shrink-0 rounded-full border">
      <Avatar.Fallback class="bg-muted">
        <BotIcon class="size-3.5 text-muted-foreground" />
      </Avatar.Fallback>
    </Avatar.Root>
    <div class="message-card__assistant-column min-w-0 max-w-[92%] sm:max-w-[85%] lg:max-w-[76%] flex-1">
      {#if activeVersion?.status === "failed"}
        <div class="mb-2 space-y-1.5">
          <div class="text-xs text-destructive">生成失败</div>
          {#if activeVersion.errorCode || activeVersion.errorMessage}
            <div class="rounded-lg border border-destructive/20 bg-destructive/5 px-2.5 py-2 text-[12px] leading-relaxed text-destructive/80">
              {#if activeVersion.errorCode}
                <div class="mb-1 font-mono text-[11px] text-destructive">{activeVersion.errorCode}</div>
              {/if}
              {#if activeVersion.errorMessage}
                <div>{activeVersion.errorMessage}</div>
              {/if}
            </div>
          {/if}
          {#if activeVersion.errorDetails}
            <details class="group rounded-2xl border border-destructive/15 bg-destructive/5">
              <summary class="flex items-center gap-1.5 px-3 py-2 text-xs text-destructive/80">
                <span class="flex-1">查看原始详情</span>
                <ChevronRightIcon class="size-3 text-destructive/50 transition-transform duration-200 group-open:rotate-90" />
              </summary>
              <div class="space-y-3 px-3 pb-3 text-[12px] leading-relaxed text-foreground/80">
                <div class="grid gap-2 sm:grid-cols-2">
                  {#if activeVersion.errorDetails.requestMethod}
                    <div>
                      <div class="mb-1 text-[11px] uppercase tracking-wide text-muted-foreground">Request Method</div>
                      <div class="break-all font-mono">{activeVersion.errorDetails.requestMethod}</div>
                    </div>
                  {/if}
                  {#if activeVersion.errorDetails.responseStatus !== null && activeVersion.errorDetails.responseStatus !== undefined}
                    <div>
                      <div class="mb-1 text-[11px] uppercase tracking-wide text-muted-foreground">Status Code</div>
                      <div class="break-all font-mono">{activeVersion.errorDetails.responseStatus}</div>
                    </div>
                  {/if}
                </div>
                {#if activeVersion.errorDetails.requestUrl}
                  <div>
                    <div class="mb-1 text-[11px] uppercase tracking-wide text-muted-foreground">Request URL</div>
                    <div class="break-all font-mono">{activeVersion.errorDetails.requestUrl}</div>
                  </div>
                {/if}
                {#if activeVersion.errorDetails.rawMessage}
                  <div>
                    <div class="mb-1 text-[11px] uppercase tracking-wide text-muted-foreground">Raw Message</div>
                    <pre class="max-h-40 overflow-auto rounded-xl border bg-background/80 p-2 font-mono text-[11px] whitespace-pre-wrap break-all">{activeVersion.errorDetails.rawMessage}</pre>
                  </div>
                {/if}
                {#if activeVersion.errorDetails.requestBody}
                  <div>
                    <div class="mb-1 text-[11px] uppercase tracking-wide text-muted-foreground">Request Body</div>
                    <pre class="max-h-52 overflow-auto rounded-xl border bg-background/80 p-2 font-mono text-[11px] whitespace-pre-wrap break-all">{formatDebugPayload(activeVersion.errorDetails.requestBody)}</pre>
                  </div>
                {/if}
                {#if activeVersion.errorDetails.responseBody}
                  <div>
                    <div class="mb-1 text-[11px] uppercase tracking-wide text-muted-foreground">Response Body</div>
                    <pre class="max-h-52 overflow-auto rounded-xl border bg-background/80 p-2 font-mono text-[11px] whitespace-pre-wrap break-all">{formatDebugPayload(activeVersion.errorDetails.responseBody)}</pre>
                  </div>
                {/if}
              </div>
            </details>
          {/if}
        </div>
      {:else if activeVersion?.status === "cancelled"}
        <div class="mb-1 text-xs text-muted-foreground">已取消</div>
      {/if}

      <div class="message-card__assistant-body px-0.5 py-0 text-[14px] leading-6 text-foreground/92">
        {#if editing}
          <Card.Root class="rounded-2xl shadow-sm">
            <Card.Content class="space-y-3 px-4 py-3">
              <Textarea.Root
                bind:value={draft}
                class="min-h-[120px] resize-y"
                placeholder="编辑当前消息"
              />
              <div class="flex items-center justify-end gap-2">
                <Button disabled={saving} onclick={cancelEdit} size="sm" variant="outline">取消</Button>
                <Button disabled={saving || !draft.trim()} onclick={() => void saveEdit(false)} size="sm" variant="outline">
                  保存
                </Button>
                <Button disabled={saving || !draft.trim()} onclick={() => void saveEdit(true)} size="sm">
                  保存并重新发送
                </Button>
              </div>
            </Card.Content>
          </Card.Root>
        {:else}
          <div class="message-card__assistant-content space-y-2.5">
            {#if displayThinking}
              <details
                bind:open={thinkingOpen}
                class="message-card__thinking-panel group rounded-2xl border-l-2 border-l-amber-400/60 bg-amber-50/40 dark:bg-amber-950/15"
              >
                <summary class="flex items-center gap-1.5 px-3 py-2 text-xs text-muted-foreground">
                  <BrainIcon class="size-3.5 {generating ? 'thinking-indicator text-amber-500' : 'text-amber-500/70'}" />
                  <span class="flex-1">{generating ? '正在思考...' : '思考过程'}</span>
                  <ChevronRightIcon class="size-3 text-muted-foreground/60 transition-transform duration-200 group-open:rotate-90" />
                </summary>
                <div class="details-content px-3 pb-2.5">
                  <RichTextContent
                    class="message-card__richtext message-card__richtext--thinking text-sm text-muted-foreground"
                    content={displayThinking}
                    throttleMs={generating ? 160 : 0}
                  />
                </div>
              </details>
            {/if}
            {#if activeVersion?.toolCalls?.length}
              <details open class="group rounded-2xl border-l-2 border-l-blue-400/60 bg-blue-50/30 dark:bg-blue-950/15">
                <summary class="flex items-center gap-1.5 px-3 py-2 text-xs text-muted-foreground">
                  <WrenchIcon class="size-3.5 text-blue-500/70" />
                  <span class="flex-1">工具调用 ({activeVersion.toolCalls.length})</span>
                  {#if generating && (!activeVersion.toolResults || activeVersion.toolResults.length < activeVersion.toolCalls.length)}
                    <LoaderCircleIcon class="size-3 animate-spin text-blue-500/70" />
                  {/if}
                  <ChevronRightIcon class="size-3 text-muted-foreground/60 transition-transform duration-200 group-open:rotate-90" />
                </summary>
                <div class="details-content space-y-2 px-3 pb-2.5">
                  {#each activeVersion.toolCalls ?? [] as toolCall (`${toolCall.id}-${toolCall.name}`)}
                    <div class="rounded-xl border bg-background/80 p-2.5">
                      <div class="mb-1 flex items-center gap-1.5 text-xs font-medium text-foreground">
                        {toolCall.name}
                        {#if generating && !(activeVersion.toolResults ?? []).some((r) => r.toolCallId === toolCall.id)}
                          <span class="inline-flex items-center gap-1 text-[10px] text-blue-500">
                            <LoaderCircleIcon class="size-3 animate-spin" />
                            执行中
                          </span>
                        {/if}
                      </div>
                      <RichTextContent
                        class="message-card__richtext message-card__richtext--tool text-sm text-muted-foreground"
                        content={`\`\`\`json\n${toolCall.argumentsJson}\n\`\`\``}
                      />
                    </div>
                  {/each}
                </div>
              </details>
            {/if}
            {#if activeVersion?.toolResults?.length}
              <details open class="group rounded-2xl border-l-2 border-l-emerald-400/60 bg-emerald-50/30 dark:bg-emerald-950/15">
                <summary class="flex items-center gap-1.5 px-3 py-2 text-xs text-muted-foreground">
                  <WrenchIcon class="size-3.5 text-emerald-500/70" />
                  <span class="flex-1">工具结果 ({activeVersion.toolResults.length})</span>
                  <ChevronRightIcon class="size-3 text-muted-foreground/60 transition-transform duration-200 group-open:rotate-90" />
                </summary>
                <div class="details-content space-y-2 px-3 pb-2.5">
                  {#each activeVersion.toolResults ?? [] as toolResult (`${toolResult.toolCallId}-${toolResult.name}`)}
                    <div class="rounded-xl border p-2.5 {toolResult.isError ? 'border-destructive/30 bg-destructive/5' : 'bg-background/80'}">
                      <div class="mb-1 flex items-center gap-1.5 text-xs font-medium {toolResult.isError ? 'text-destructive' : 'text-foreground'}">
                        {toolResult.name}
                        {#if toolResult.isError}
                          <span class="rounded bg-destructive/10 px-1 py-0.5 text-[10px] text-destructive">错误</span>
                        {/if}
                        {#if toolResult.content.length > 500}
                          <span class="text-[10px] font-normal text-muted-foreground/60">{toolResult.content.length} 字符</span>
                        {/if}
                      </div>
                      <div class="max-h-40 overflow-y-auto">
                        <RichTextContent
                          class="message-card__richtext message-card__richtext--tool text-sm text-muted-foreground"
                          content={toolResult.content.length > 2000 ? toolResult.content.slice(0, 2000) + "\n\n...[已截断]" : toolResult.content}
                        />
                      </div>
                    </div>
                  {/each}
                </div>
              </details>
            {/if}
            {#if activeVersion?.images.length}
              <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
                {#each activeVersion.images as image (image.base64)}
                  <img
                    alt="AI 返回图片"
                    class="max-h-56 rounded-2xl border bg-muted/20 object-cover"
                    src={resolveImageSrc(image)}
                  />
                {/each}
              </div>
            {/if}
            {#if assistantImageFiles.length}
              <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
                {#each assistantImageFiles as file (`${file.name}-${file.base64.slice(0, 16)}`)}
                  <img
                    alt={file.name}
                    class="max-h-56 rounded-2xl border bg-muted/20 object-cover"
                    src={`data:${file.mimeType};base64,${file.base64}`}
                  />
                {/each}
              </div>
            {/if}
            <RichTextContent
              class="message-card__richtext message-card__richtext--assistant prose-p:text-foreground/92 prose-headings:text-foreground prose-strong:text-foreground prose-blockquote:text-muted-foreground prose-code:bg-foreground/8 prose-a:text-foreground prose-a:decoration-foreground/35"
              content={displayBodyContent}
              throttleMs={generating ? 160 : 0}
            />
            {#if nonImageFiles.length}
              <div class="space-y-2">
                {#each nonImageFiles as file (`${file.name}-${file.base64.slice(0, 16)}`)}
                  <div class="flex items-center gap-2 rounded-2xl border bg-muted/35 px-3 py-2 text-sm text-muted-foreground">
                    <FileTextIcon class="size-4 shrink-0" />
                    <div class="min-w-0 flex-1">
                      <div class="truncate text-foreground">{file.name}</div>
                      <div class="truncate text-[11px]">{file.mimeType}</div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
          {#if generating}
            <span class="ml-0.5 inline-block size-1.5 animate-pulse rounded-full bg-foreground/40"></span>
          {/if}
        {/if}
      </div>

      <div class="relative mt-1.5 h-6">
        {#if showToolbar}
          <div class="absolute left-0 top-0 flex min-w-0 items-center gap-0.5">
            <Button class="size-6" onclick={handleCopy} size="icon" variant="ghost" title="复制">
              {#if copied}
                <CheckIcon class="size-3 text-emerald-500" />
              {:else}
                <CopyIcon class="size-3 text-muted-foreground" />
              {/if}
            </Button>
            <Button class="size-6" disabled={loadingEditContent || saving} onclick={() => void startEdit()} size="icon" variant="ghost" title="编辑">
              {#if loadingEditContent}
                <LoaderCircleIcon class="size-3 animate-spin text-muted-foreground" />
              {:else}
                <PencilIcon class="size-3 text-muted-foreground" />
              {/if}
            </Button>
            <Button class="size-6" onclick={() => onReroll(node.id)} size="icon" variant="ghost" title="重新生成">
              <RotateCcwIcon class="size-3 text-muted-foreground" />
            </Button>
            <Button class="size-6" onclick={handleDelete} size="icon" variant="ghost" title="删除">
              <Trash2Icon class="size-3 text-muted-foreground" />
            </Button>
            {#if generating && activeVersion}
              <Button class="size-6" onclick={() => onCancel(activeVersion.id)} size="icon" variant="ghost" title="停止">
                <SquareIcon class="size-3 text-muted-foreground" />
              </Button>
            {/if}
            {#if node.versions.length > 1}
              <div class="ml-1 flex items-center gap-0.5 border-l pl-1">
                <Button class="size-5" disabled={activeIndex <= 1} onclick={prevVersion} size="icon" variant="ghost">
                  <ChevronLeftIcon class="size-3" />
                </Button>
                <span class="min-w-6 text-center text-[11px] text-muted-foreground">{activeIndex}/{node.versions.length}</span>
                <Button class="size-5" disabled={activeIndex >= node.versions.length} onclick={nextVersion} size="icon" variant="ghost">
                  <ChevronRightIcon class="size-3" />
                </Button>
              </div>
            {/if}
            {#if activeVersion?.status === "committed" && (activeVersion.promptTokens !== null || activeVersion.completionTokens !== null || activeVersion.createdAt)}
              <div class="ml-2 hidden items-center gap-2 text-[10px] text-muted-foreground/70 sm:flex">
                <span class="inline-flex items-center gap-1" title="发送 Tokens">
                  <SendHorizontalIcon class="size-3" />
                  <span>{activeVersion.promptTokens === null ? "--" : `${formatCompactNumber(activeVersion.promptTokens)} tokens`}</span>
                </span>
                <span class="inline-flex items-center gap-1" title="接收 Tokens">
                  <DownloadIcon class="size-3" />
                  <span>{activeVersion.completionTokens === null ? "--" : `${formatCompactNumber(activeVersion.completionTokens)} tokens`}</span>
                </span>
                <span class="inline-flex items-center gap-1" title="生成速度">
                  <GaugeIcon class="size-3" />
                  <span>{formatReceiveSpeed(activeVersion.completionTokens, resolveMetricStartAt(), activeVersion.completedAt)}</span>
                </span>
                <span class="inline-flex items-center gap-1" title="接收时间">
                  <Clock3Icon class="size-3" />
                  <span>{formatDurationSeconds(resolveMetricStartAt(), activeVersion.completedAt)}</span>
                </span>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
