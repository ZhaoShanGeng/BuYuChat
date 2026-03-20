<script lang="ts">
  import {
    Copy, Edit3, RefreshCw, Trash2, ChevronLeft, ChevronRight,
    SendHorizontal, Paperclip, Loader2, Check, X, Square,
    Sparkles, MessageCircle, Lightbulb, PanelLeft, PanelRight,
    Pencil, Eraser, Download
  } from "lucide-svelte";
  import type { MessageVersionView } from "$lib/api/messages";
  import {
    createUserMessage,
    generateReplyStream,
    regenerateReplyStream,
    editMessageVersion,
    listMessageVersions,
    switchMessageVersion,
    deleteMessageNode
  } from "$lib/api/messages";
  import { listenGenerationStream, type GenerationStreamEvent } from "$lib/events/generation-stream";
  import { onMount, tick } from "svelte";
  import { marked } from "marked";
  import { i18n } from "$lib/i18n.svelte";

  let {
    conversationTitle = "Conversation",
    conversationId = "",
    loading = false,
    messages = [],
    editable = false,
    onRename = undefined,
    onToggleSidebar = () => {},
    onToggleInspector = () => {}
  }: {
    conversationTitle?: string;
    conversationId?: string;
    loading?: boolean;
    messages?: MessageVersionView[];
    editable?: boolean;
    onRename?: ((title: string) => void) | undefined;
    onToggleSidebar?: () => void;
    onToggleInspector?: () => void;
  } = $props();

  // ─── Inline header edit state ───
  let headerEditing = $state(false);
  let headerEditText = $state("");

  function startHeaderEdit() {
    if (!editable || !onRename) return;
    headerEditing = true;
    headerEditText = conversationTitle;
    requestAnimationFrame(() => {
      const input = document.querySelector(".inline-title-input") as HTMLInputElement | null;
      input?.focus();
      input?.select();
    });
  }

  function submitHeaderEdit() {
    if (headerEditText.trim() && onRename) {
      onRename(headerEditText.trim());
    }
    headerEditing = false;
  }

  function cancelHeaderEdit() {
    headerEditing = false;
    headerEditText = "";
  }

  function handleHeaderKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") { event.preventDefault(); submitHeaderEdit(); }
    if (event.key === "Escape") cancelHeaderEdit();
  }

  // ─── Composer state ───
  let composerText = $state("");
  let sending = $state(false);
  let streaming = $state(false);
  let streamingText = $state("");
  let streamingNodeId = $state("");
  let currentStreamId = $state("");

  // ─── Version swipe state ───
  let versionsByNode = $state<Record<string, MessageVersionView[]>>({});
  let loadingVersions = $state<Record<string, boolean>>({});

  // ─── Edit state ───
  let editingNodeId = $state("");
  let editingVersionId = $state("");
  let editText = $state("");
  let editSaving = $state(false);

  // ─── Hover state ───
  let hoveredMessageId = $state("");

  // ─── Scroll ref ───
  let scrollContainer: HTMLDivElement;

  // ─── Stream listener ───
  let unlistenStream: (() => void) | undefined;

  // ─── Copy feedback ───
  let copiedVersionId = $state("");

  // ─── Textarea ref ───
  let composerTextarea: HTMLTextAreaElement;

  // ─── Markdown config ───
  marked.setOptions({ breaks: true, gfm: true });

  function renderMarkdown(text: string): string {
    try { return marked.parse(text) as string; }
    catch { return text; }
  }

  onMount(() => {
    void (async () => {
      unlistenStream = await listenGenerationStream(handleStreamEvent);
    })();
    return () => { unlistenStream?.(); };
  });

  function handleStreamEvent(event: GenerationStreamEvent) {
    if (event.stream_id !== currentStreamId) return;
    if (event.kind === "started") {
      streamingText = "";
    } else if (event.kind === "delta" && event.delta_text) {
      streamingText += event.delta_text;
      void scrollToBottom();
    } else if (event.kind === "completed") {
      streaming = false; streamingText = ""; streamingNodeId = ""; currentStreamId = "";
    } else if (event.kind === "failed") {
      streaming = false; streamingText = ""; streamingNodeId = ""; currentStreamId = "";
      console.error("Generation failed:", event.error_text);
    }
  }

  async function scrollToBottom() {
    await tick();
    if (scrollContainer) scrollContainer.scrollTop = scrollContainer.scrollHeight;
  }

  $effect(() => { if (messages.length) void scrollToBottom(); });

  // ─── Composer auto-resize ───
  function resizeComposer() {
    if (!composerTextarea) return;
    composerTextarea.style.height = "auto";
    const maxH = 240;
    composerTextarea.style.height = Math.min(composerTextarea.scrollHeight, maxH) + "px";
    composerTextarea.style.overflowY = composerTextarea.scrollHeight > maxH ? "auto" : "hidden";
  }

  $effect(() => { void composerText; resizeComposer(); });

  // ─── Send message ───
  async function handleSend() {
    const text = composerText.trim();
    if (!text || !conversationId || sending || streaming) return;
    composerText = "";
    sending = true;
    try {
      const userMsg = await createUserMessage({ conversation_id: conversationId, text });
      streaming = true; streamingText = "";
      const streamId = `stream-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
      currentStreamId = streamId;
      streamingNodeId = userMsg.node_id;
      await generateReplyStream({
        request: { conversation_id: conversationId, reply_to_node_id: userMsg.node_id },
        stream_id: streamId
      });
    } catch (err) {
      console.error("Send failed:", err);
      streaming = false; currentStreamId = "";
    } finally { sending = false; }
  }

  // ─── Regenerate ───
  async function handleRegenerate(nodeId: string) {
    if (streaming) return;
    streaming = true; streamingText = "";
    const streamId = `regen-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    currentStreamId = streamId; streamingNodeId = nodeId;
    try {
      await regenerateReplyStream({ request: { node_id: nodeId }, stream_id: streamId });
    } catch (err) {
      console.error("Regenerate failed:", err);
      streaming = false; currentStreamId = "";
    }
  }

  function handleStopGeneration() {
    streaming = false; streamingText = ""; streamingNodeId = ""; currentStreamId = "";
  }

  // ─── Edit message ───
  function startEdit(msg: MessageVersionView) {
    editingNodeId = msg.node_id; editingVersionId = msg.version_id; editText = getMessageText(msg);
  }
  function cancelEdit() { editingNodeId = ""; editingVersionId = ""; editText = ""; }
  async function submitEdit() {
    if (!editText.trim() || editSaving) return;
    editSaving = true;
    try {
      await editMessageVersion({ node_id: editingNodeId, version_id: editingVersionId, text: editText.trim() });
      cancelEdit();
    } catch (err) { console.error("Edit failed:", err); }
    finally { editSaving = false; }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) { event.preventDefault(); void handleSend(); }
  }

  function handleEditKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) { event.preventDefault(); void submitEdit(); }
    if (event.key === "Escape") cancelEdit();
  }

  // ─── Version swipe ───
  async function loadVersions(nodeId: string) {
    if (versionsByNode[nodeId] || loadingVersions[nodeId]) return;
    loadingVersions = { ...loadingVersions, [nodeId]: true };
    try {
      const versions = await listMessageVersions(nodeId);
      versionsByNode = { ...versionsByNode, [nodeId]: versions };
    } finally { loadingVersions = { ...loadingVersions, [nodeId]: false }; }
  }

  async function handleSwitchVersion(nodeId: string, versionId: string) {
    try {
      await switchMessageVersion(nodeId, versionId);
      const versions = await listMessageVersions(nodeId);
      versionsByNode = { ...versionsByNode, [nodeId]: versions };
    } catch (err) { console.error("Version switch failed:", err); }
  }

  async function handleDelete(nodeId: string) {
    try { await deleteMessageNode(nodeId); } catch (err) { console.error("Delete failed:", err); }
  }

  function copyText(text: string, versionId: string) {
    void navigator.clipboard.writeText(text);
    copiedVersionId = versionId;
    setTimeout(() => { copiedVersionId = ""; }, 1500);
  }

  function getMessageText(msg: MessageVersionView): string {
    return msg.primary_content.text_content ?? msg.primary_content.preview_text ?? "";
  }

  function getVersionInfo(msg: MessageVersionView): { current: number; total: number } | null {
    const versions = versionsByNode[msg.node_id];
    if (!versions || versions.length <= 1) return null;
    const idx = versions.findIndex(v => v.version_id === msg.version_id);
    return { current: idx + 1, total: versions.length };
  }

  function formatTimestamp(ts: number): string {
    const date = new Date(ts);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (seconds < 60) return i18n.t("time.just_now");
    if (minutes < 60) return i18n.t("time.minutes_ago", { n: minutes });
    if (hours < 24) return i18n.t("time.hours_ago", { n: hours });
    if (days < 7) return i18n.t("time.days_ago", { n: days });

    const isThisYear = date.getFullYear() === now.getFullYear();
    if (isThisYear) return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
  }

  function editAutoResize(node: HTMLTextAreaElement) {
    function resize() { node.style.height = "auto"; node.style.height = Math.min(node.scrollHeight, 300) + "px"; }
    node.addEventListener("input", resize);
    requestAnimationFrame(resize);
    return { destroy() { node.removeEventListener("input", resize); } };
  }

  const suggestions = $derived([
    { icon: MessageCircle, text: i18n.t("suggest.chat"), desc: i18n.t("suggest.chat_desc") },
    { icon: Lightbulb, text: i18n.t("suggest.brainstorm"), desc: i18n.t("suggest.brainstorm_desc") },
    { icon: Sparkles, text: i18n.t("suggest.write"), desc: i18n.t("suggest.write_desc") }
  ]);
</script>

<div class="relative flex h-full flex-1 flex-col overflow-hidden">
  <!-- ─── Inline header (replaces topbar) ─── -->
  <header class="flex h-12 flex-shrink-0 items-center gap-3 border-b border-[var(--border-soft)] px-4 pr-[140px]" data-tauri-drag-region>
    <!-- Mobile sidebar toggle -->
    <button
      type="button"
      class="icon-hover inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)] lg:hidden"
      onclick={onToggleSidebar}
    >
      <PanelLeft size={18} />
    </button>

    <!-- Title -->
    <div class="flex min-w-0 flex-1 items-center gap-2">
      {#if headerEditing}
        <div class="flex items-center gap-1.5">
          <input
            class="inline-title-input rounded-[var(--radius-sm)] border border-[var(--brand)] bg-[var(--bg-app)] px-2 py-0.5 text-sm font-semibold text-[var(--ink-strong)] outline-none shadow-[0_0_0_2px_var(--brand-glow)]"
            style="width: {Math.max(headerEditText.length * 8 + 24, 100)}px"
            bind:value={headerEditText}
            onkeydown={handleHeaderKeydown}
            onblur={submitHeaderEdit}
          />
          <button type="button" class="inline-flex h-6 w-6 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] hover:text-[var(--ink-muted)]" onclick={(e) => { e.preventDefault(); cancelHeaderEdit(); }}>
            <X size={14} />
          </button>
        </div>
      {:else}
        <button
          type="button"
          class="group/title flex items-center gap-1.5 truncate"
          ondblclick={startHeaderEdit}
          title={editable ? i18n.t("chat.edit_title") : ""}
        >
          <h1 class="truncate text-sm font-semibold text-[var(--ink-strong)]">{conversationTitle}</h1>
          {#if editable}
            <Pencil size={12} class="flex-shrink-0 text-[var(--ink-faint)] opacity-0 transition-opacity group-hover/title:opacity-100" />
          {/if}
        </button>
      {/if}
    </div>

    <!-- Right tools -->
    <div class="flex items-center gap-1">
      <button type="button" title={i18n.t("chat.clear")} class="icon-hover hidden h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-muted)] sm:inline-flex">
        <Eraser size={16} />
      </button>
      <button type="button" title={i18n.t("chat.export")} class="icon-hover hidden h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-faint)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-muted)] sm:inline-flex">
        <Download size={16} />
      </button>
      <button
        type="button"
        class="icon-hover inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-[var(--radius-sm)] text-[var(--ink-muted)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-strong)] xl:hidden"
        onclick={onToggleInspector}
      >
        <PanelRight size={18} />
      </button>
    </div>
  </header>

  <!-- ─── Messages area ─── -->
  <div bind:this={scrollContainer} class="app-scrollbar flex-1 overflow-y-auto">
    {#if loading}
      <div class="flex h-full items-center justify-center">
        <div class="flex flex-col items-center gap-3">
          <Loader2 size={28} class="animate-spin text-[var(--brand)]" />
          <span class="text-sm text-[var(--ink-faint)]">{i18n.t("chat.loading")}</span>
        </div>
      </div>
    {:else if messages.length === 0 && !streaming}
      <!-- Empty state -->
      <div class="flex h-full flex-col items-center justify-center gap-8 px-6">
        <div class="flex flex-col items-center gap-4">
          <div class="relative">
            <div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-[var(--brand)] to-[#3b82f6] shadow-lg">
              <span class="text-3xl font-bold text-white">步</span>
            </div>
            <div class="absolute -bottom-1 -right-1 flex h-7 w-7 items-center justify-center rounded-full border-2 border-[var(--bg-surface)] bg-[var(--success)] shadow-sm">
              <Sparkles size={14} class="text-white" />
            </div>
          </div>
          <div class="text-center">
            <h2 class="text-xl font-bold text-[var(--ink-strong)]">{conversationTitle}</h2>
            <p class="mt-1 text-sm text-[var(--ink-muted)]">{i18n.t("chat.start_hint")}</p>
          </div>
        </div>
        <div class="grid w-full max-w-lg gap-3 sm:grid-cols-3">
          {#each suggestions as s}
            <button type="button" class="suggestion-card flex flex-col items-center gap-2 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-5 text-center" onclick={() => { composerText = s.text; composerTextarea?.focus(); }}>
              <div class="flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-[var(--brand-soft)]">
                <s.icon size={20} class="text-[var(--brand)]" />
              </div>
              <span class="text-sm font-medium text-[var(--ink-strong)]">{s.text}</span>
              <span class="text-xs text-[var(--ink-faint)]">{s.desc}</span>
            </button>
          {/each}
        </div>
      </div>
    {:else}
      <!-- Message list -->
      <div class="mx-auto max-w-[var(--message-max-width)] px-4 py-6">
        {#each messages as message, i (message.version_id)}
          {@const text = getMessageText(message)}
          {@const versionInfo = getVersionInfo(message)}
          {@const isEditing = editingNodeId === message.node_id}
          <div
            class="group msg-enter mb-6"
            style="animation-delay: {Math.min(i * 30, 300)}ms"
            role="article"
            onmouseenter={() => { hoveredMessageId = message.version_id; void loadVersions(message.node_id); }}
            onmouseleave={() => { hoveredMessageId = ""; }}
          >
            {#if message.role === "user"}
              <div class="flex justify-end gap-3">
                <div class="max-w-[80%]">
                  {#if isEditing}
                    <div class="rounded-[var(--radius-lg)] border border-[var(--brand)] bg-[var(--bg-app)] p-2 shadow-[0_0_0_2px_var(--brand-glow)]">
                      <textarea class="block w-full resize-none bg-transparent px-2 py-1 text-sm leading-relaxed text-[var(--ink-body)] outline-none" bind:value={editText} onkeydown={handleEditKeydown} use:editAutoResize></textarea>
                      <div class="mt-1.5 flex items-center justify-end gap-1.5">
                        <button type="button" class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] px-2 text-xs text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]" onclick={cancelEdit}>
                          <X size={12} /> {i18n.t("chat.cancel")}
                        </button>
                        <button type="button" class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--brand)] px-2.5 text-xs text-white hover:bg-[var(--brand-strong)] disabled:opacity-50" onclick={() => void submitEdit()} disabled={editSaving || !editText.trim()}>
                          {#if editSaving}<Loader2 size={12} class="animate-spin" />{:else}<Check size={12} />{/if}
                          {i18n.t("chat.save")} <kbd class="ml-1 text-[10px] opacity-60">Ctrl+↵</kbd>
                        </button>
                      </div>
                    </div>
                  {:else}
                    <div class="user-bubble rounded-2xl rounded-br-md px-4 py-2.5 text-sm leading-relaxed shadow-[var(--shadow-sm)]">
                      <p class="whitespace-pre-wrap">{text}</p>
                    </div>
                  {/if}
                  {#if !isEditing}
                    <div class="mt-1 flex items-center justify-end gap-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100">
                      <span class="mr-auto text-[10px] text-[var(--ink-faint)]">{formatTimestamp(message.created_at)}</span>
                      <button type="button" title={i18n.t("chat.edit")} class="msg-action-btn" onclick={() => startEdit(message)}><Edit3 size={13} /></button>
                      <button type="button" title={i18n.t("chat.copy")} class="msg-action-btn" onclick={() => copyText(text, message.version_id)}>
                        {#if copiedVersionId === message.version_id}<Check size={13} class="text-[var(--success)]" />{:else}<Copy size={13} />{/if}
                      </button>
                      <button type="button" title={i18n.t("chat.delete")} class="msg-action-btn hover:!text-[var(--danger)]" onclick={() => handleDelete(message.node_id)}><Trash2 size={13} /></button>
                    </div>
                  {/if}
                </div>
                <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-gray-700 to-gray-900 text-xs font-bold text-white shadow-sm">{i18n.t("chat.user_avatar")}</div>
              </div>
            {:else}
              <div class="flex gap-3">
                <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-[var(--brand-soft)] to-blue-100 text-xs font-bold text-[var(--brand)] shadow-sm">
                  {message.role === "assistant" ? "步" : "系"}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="mb-1.5 flex items-center gap-2">
                    <span class="text-sm font-semibold text-[var(--ink-strong)]">{message.role === "assistant" ? i18n.t("chat.assistant") : i18n.t("chat.system")}</span>
                    {#if message.api_channel_model_id}
                      <span class="rounded-[var(--radius-full)] bg-[var(--bg-hover)] px-2 py-0.5 text-[10px] font-medium text-[var(--ink-faint)]">{message.api_channel_model_id}</span>
                    {/if}
                    {#if message.prompt_tokens || message.completion_tokens}
                      <span class="text-[10px] text-[var(--ink-faint)]">{message.prompt_tokens ?? 0} → {message.completion_tokens ?? 0} tokens</span>
                    {/if}
                  </div>
                  {#if isEditing}
                    <div class="rounded-[var(--radius-md)] border border-[var(--brand)] bg-[var(--bg-app)] p-2 shadow-[0_0_0_2px_var(--brand-glow)]">
                      <textarea class="block w-full resize-none bg-transparent px-2 py-1 text-sm leading-relaxed text-[var(--ink-body)] outline-none" bind:value={editText} onkeydown={handleEditKeydown} use:editAutoResize></textarea>
                      <div class="mt-1.5 flex items-center justify-end gap-1.5">
                        <button type="button" class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] px-2 text-xs text-[var(--ink-muted)] hover:bg-[var(--bg-hover)]" onclick={cancelEdit}>
                          <X size={12} /> {i18n.t("chat.cancel")}
                        </button>
                        <button type="button" class="inline-flex h-7 items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--brand)] px-2.5 text-xs text-white hover:bg-[var(--brand-strong)] disabled:opacity-50" onclick={() => void submitEdit()} disabled={editSaving || !editText.trim()}>
                          {#if editSaving}<Loader2 size={12} class="animate-spin" />{:else}<Check size={12} />{/if}
                          {i18n.t("chat.save")} <kbd class="ml-1 text-[10px] opacity-60">Ctrl+↵</kbd>
                        </button>
                      </div>
                    </div>
                  {:else}
                    <div class="prose-chat text-sm leading-relaxed text-[var(--ink-body)]">{@html renderMarkdown(text)}</div>
                  {/if}
                  {#if !isEditing}
                    <div class="mt-2 flex items-center gap-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100">
                      <span class="mr-1 text-[10px] text-[var(--ink-faint)]">{formatTimestamp(message.created_at)}</span>
                      <button type="button" title={i18n.t("chat.copy")} class="msg-action-btn" onclick={() => copyText(text, message.version_id)}>
                        {#if copiedVersionId === message.version_id}<Check size={13} class="text-[var(--success)]" />{:else}<Copy size={13} />{/if}
                      </button>
                      {#if message.role === "assistant"}
                        <button type="button" title={i18n.t("chat.regenerate")} class="msg-action-btn" onclick={() => void handleRegenerate(message.node_id)} disabled={streaming}><RefreshCw size={13} /></button>
                      {/if}
                      <button type="button" title={i18n.t("chat.edit")} class="msg-action-btn" onclick={() => startEdit(message)}><Edit3 size={13} /></button>
                      <button type="button" title={i18n.t("chat.delete")} class="msg-action-btn hover:!text-[var(--danger)]" onclick={() => handleDelete(message.node_id)}><Trash2 size={13} /></button>
                      {#if versionInfo}
                        <div class="ml-2 flex items-center gap-0.5 rounded-[var(--radius-full)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-1.5 py-0.5 shadow-sm">
                          <button type="button" title={i18n.t("chat.prev_version")} class="inline-flex h-5 w-5 items-center justify-center rounded-full text-[var(--ink-faint)] hover:text-[var(--ink-muted)] disabled:opacity-30" disabled={versionInfo.current <= 1} onclick={() => { const versions = versionsByNode[message.node_id]; const idx = versions.findIndex(v => v.version_id === message.version_id); if (idx > 0) void handleSwitchVersion(message.node_id, versions[idx - 1].version_id); }}>
                            <ChevronLeft size={12} />
                          </button>
                          <span class="min-w-[28px] text-center text-[10px] font-medium text-[var(--ink-faint)]">{versionInfo.current}/{versionInfo.total}</span>
                          <button type="button" title={i18n.t("chat.next_version")} class="inline-flex h-5 w-5 items-center justify-center rounded-full text-[var(--ink-faint)] hover:text-[var(--ink-muted)] disabled:opacity-30" disabled={versionInfo.current >= versionInfo.total} onclick={() => { const versions = versionsByNode[message.node_id]; const idx = versions.findIndex(v => v.version_id === message.version_id); if (idx < versions.length - 1) void handleSwitchVersion(message.node_id, versions[idx + 1].version_id); }}>
                            <ChevronRight size={12} />
                          </button>
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/each}

        {#if streaming}
          <div class="msg-enter mb-6 flex gap-3">
            <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-[var(--brand-soft)] to-blue-100 text-xs font-bold text-[var(--brand)] shadow-sm">步</div>
            <div class="min-w-0 flex-1">
              <div class="mb-1.5 flex items-center gap-2">
                <span class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("chat.assistant")}</span>
                <span class="inline-flex items-center gap-1 rounded-[var(--radius-full)] bg-[var(--brand-soft)] px-2 py-0.5 text-[10px] font-medium text-[var(--brand)]">
                  <Loader2 size={10} class="animate-spin" />
                  {i18n.t("chat.generating")}
                </span>
              </div>
              {#if streamingText}
                <div class="prose-chat text-sm leading-relaxed text-[var(--ink-body)]">{@html renderMarkdown(streamingText)}</div>
              {:else}
                <div class="flex items-center gap-1.5 py-3">
                  <span class="typing-dot h-2 w-2 rounded-full bg-[var(--brand)]"></span>
                  <span class="typing-dot h-2 w-2 rounded-full bg-[var(--brand)]"></span>
                  <span class="typing-dot h-2 w-2 rounded-full bg-[var(--brand)]"></span>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- ─── Floating Composer ─── -->
  <div class="px-4 pb-4 pt-2">
    <div class="composer-card mx-auto max-w-3xl px-4 pb-2 pt-3">
      <textarea
        class="block w-full resize-none bg-transparent text-sm leading-relaxed text-[var(--ink-body)] outline-none placeholder:text-[var(--ink-faint)]"
        placeholder={i18n.t("chat.input_placeholder")}
        rows="3"
        bind:this={composerTextarea}
        bind:value={composerText}
        onkeydown={handleKeydown}
        disabled={sending}
        style="overflow-y: hidden;"
      ></textarea>
      <div class="mt-1 flex items-center justify-between">
        <div class="flex items-center gap-1">
          <button type="button" title={i18n.t("chat.attachment")} class="flex h-8 w-8 items-center justify-center rounded-[var(--radius-md)] text-[var(--ink-faint)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--ink-muted)]">
            <Paperclip size={16} />
          </button>
        </div>
        <div class="flex items-center gap-2">
          {#if streaming}
            <button type="button" title={i18n.t("chat.stop")} class="flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--danger)] px-3 text-xs font-medium text-white shadow-sm transition-all hover:bg-red-700" onclick={handleStopGeneration}>
              <Square size={12} /> {i18n.t("chat.stop")}
            </button>
          {:else}
            <button type="button" title="{i18n.t('chat.send')} (Enter)" class="flex h-8 items-center gap-1.5 rounded-[var(--radius-md)] bg-[var(--brand)] px-3 text-xs font-medium text-white shadow-sm transition-all hover:bg-[var(--brand-strong)] disabled:opacity-40" onclick={() => void handleSend()} disabled={!composerText.trim() || sending}>
              {#if sending}<Loader2 size={14} class="animate-spin" />{:else}<SendHorizontal size={14} />{/if}
              {i18n.t("chat.send")}
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .msg-action-btn {
    display: inline-flex;
    height: 24px;
    width: 24px;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    color: var(--ink-faint);
    transition: all 120ms ease;
  }
  .msg-action-btn:hover {
    background: var(--bg-hover);
    color: var(--ink-muted);
  }

  :global(.prose-chat) { word-wrap: break-word; overflow-wrap: break-word; }
  :global(.prose-chat p) { margin: 0.25em 0; }
  :global(.prose-chat p:first-child) { margin-top: 0; }
  :global(.prose-chat p:last-child) { margin-bottom: 0; }
  :global(.prose-chat h1), :global(.prose-chat h2), :global(.prose-chat h3), :global(.prose-chat h4) { font-weight: 600; margin: 0.75em 0 0.25em; color: var(--ink-strong); }
  :global(.prose-chat h1) { font-size: 1.25em; }
  :global(.prose-chat h2) { font-size: 1.125em; }
  :global(.prose-chat h3) { font-size: 1em; }
  :global(.prose-chat ul), :global(.prose-chat ol) { margin: 0.5em 0; padding-left: 1.5em; }
  :global(.prose-chat li) { margin: 0.15em 0; }
  :global(.prose-chat code) { font-family: ui-monospace, "SFMono-Regular", "Cascadia Code", "Consolas", monospace; font-size: 0.9em; padding: 0.15em 0.35em; background: var(--bg-hover); border-radius: var(--radius-sm); color: var(--ink-strong); }
  :global(.prose-chat pre) { position: relative; margin: 0.5em 0; padding: 0.75em 1em; background: #1e1e2e; border-radius: var(--radius-md); overflow-x: auto; color: #cdd6f4; }
  :global(.prose-chat pre code) { padding: 0; background: transparent; border-radius: 0; color: inherit; font-size: 0.85em; }
  :global(.prose-chat blockquote) { margin: 0.5em 0; padding: 0.25em 0.75em; border-left: 3px solid var(--brand); color: var(--ink-muted); background: var(--brand-soft); border-radius: 0 var(--radius-sm) var(--radius-sm) 0; }
  :global(.prose-chat a) { color: var(--brand); text-decoration: underline; }
  :global(.prose-chat table) { margin: 0.5em 0; border-collapse: collapse; width: 100%; font-size: 0.9em; }
  :global(.prose-chat th), :global(.prose-chat td) { border: 1px solid var(--border-soft); padding: 0.35em 0.5em; text-align: left; }
  :global(.prose-chat th) { background: var(--bg-hover); font-weight: 600; }
  :global(.prose-chat hr) { margin: 0.75em 0; border: none; border-top: 1px solid var(--border-soft); }
  :global(.prose-chat img) { max-width: 100%; border-radius: var(--radius-md); margin: 0.5em 0; }
</style>
