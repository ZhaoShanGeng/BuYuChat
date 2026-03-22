<script lang="ts">
  import { Loader2 } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import { tick } from "svelte";
  import type { AgentSummary } from "$lib/api/agents";
  import {
    updateConversationMeta,
    type ConversationDetail
  } from "$lib/api/conversations";
  import type {
    ContentWriteInput,
    MessageVersionView
  } from "$lib/api/messages";
  import type { InspectorTabId, SidebarItem } from "$lib/state/app-shell.svelte";
  import {
    appendMessageAttachment,
    createUserMessage,
    deleteMessageNode,
    editMessageVersion,
    generateReplyStream,
    listMessageVersions,
    regenerateReplyStream,
    switchMessageVersion
  } from "$lib/api/messages";
  import {
    getAgentParticipants,
    mergeConversationChatConfig,
    resolvePreferredResponderParticipantIds,
    resolvePrimaryResponderParticipantId
  } from "$lib/chat/conversation-preferences";
  import { i18n } from "$lib/i18n.svelte";
  import { generationJobsState } from "$lib/state/generation-jobs.svelte";
  import ChatComposer from "$components/chat/chat-composer.svelte";
  import ChatEmptyState from "$components/chat/chat-empty-state.svelte";
  import ChatHeader from "$components/chat/chat-header.svelte";
  import ChatShell from "$components/chat/chat-shell.svelte";
  import ChatMessageItem from "$components/chat/chat-message-item.svelte";
  import ChatStreamingMessage from "$components/chat/chat-streaming-message.svelte";
  import ResourceSidebar from "$components/layout/resource-sidebar.svelte";
  import InspectorPanel from "$components/layout/inspector-panel.svelte";

  type PendingAttachment = {
    id: string;
    name: string;
    mimeType: string | null;
    sizeBytes: number;
    contentType: ContentWriteInput["content_type"];
    refRole: string;
    textContent: string | null;
    sourceFilePath: string | null;
    primaryStorageUri: string | null;
    previewText: string | null;
  };

  let {
    conversationTitle = "Conversation",
    conversationId = "",
    conversationDetail = null,
    loading = false,
    messages = [],
    editable = false,
    onEnsureConversation = undefined,
    availableAgents = [],
    onStartConversationWithAgent = undefined,
    desktopWide = true,
    sidebarItems = [],
    activeSidebarId = "",
    sidebarOpen = false,
    inspectorVisible = true,
    inspectorOpen = false,
    inspectorTabs = [],
    activeInspectorTab = "context" as InspectorTabId,
    onRename = undefined,
    onSelectSidebar = () => {},
    onCreateSidebarItem = undefined,
    onRenameSidebarItem = undefined,
    onDeleteSidebarItem = undefined,
    onToggleSidebar = () => {},
    onToggleInspector = () => {},
    onOpenInspector = () => {},
    onCloseSidebar = () => {},
    onCloseInspector = () => {},
    onSelectInspectorTab = () => {}
  }: {
    conversationTitle?: string;
    conversationId?: string;
    conversationDetail?: ConversationDetail | null;
    loading?: boolean;
    messages?: MessageVersionView[];
    editable?: boolean;
    onEnsureConversation?: ((preferredAgentId?: string) => Promise<ConversationDetail | null>) | undefined;
    availableAgents?: AgentSummary[];
    onStartConversationWithAgent?: ((agentId: string) => Promise<string | null>) | undefined;
    desktopWide?: boolean;
    sidebarItems?: SidebarItem[];
    activeSidebarId?: string;
    sidebarOpen?: boolean;
    inspectorVisible?: boolean;
    inspectorOpen?: boolean;
    inspectorTabs?: { id: InspectorTabId; label: string }[];
    activeInspectorTab?: InspectorTabId;
    onRename?: ((title: string) => void) | undefined;
    onSelectSidebar?: (id: string) => void;
    onCreateSidebarItem?: (() => void) | undefined;
    onRenameSidebarItem?: ((id: string, title: string) => void) | undefined;
    onDeleteSidebarItem?: ((id: string) => void) | undefined;
    onToggleSidebar?: () => void;
    onToggleInspector?: () => void;
    onOpenInspector?: () => void;
    onCloseSidebar?: () => void;
    onCloseInspector?: () => void;
    onSelectInspectorTab?: (id: InspectorTabId) => void;
  } = $props();

  let composerText = $state("");
  let sending = $state(false);
  let pendingAttachments = $state<PendingAttachment[]>([]);
  let versionsByNode = $state<Record<string, MessageVersionView[]>>({});
  let loadingVersions = $state<Record<string, boolean>>({});
  let editingNodeId = $state("");
  let editingVersionId = $state("");
  let editText = $state("");
  let editSaving = $state(false);
  let copiedVersionId = $state("");
  let selectedNodeId = $state("");
  let startingAgentId = $state("");
  let scrollContainer = $state<HTMLDivElement | undefined>(undefined);
  let selectedResponderParticipantIds = $state<string[]>([]);
  let loadedResponderConversationId = $state<string | null>(null);

  const activeGenerationJobs = $derived(
    conversationId ? generationJobsState.visibleJobsForConversation(conversationId) : []
  );
  const inFlightCountForConversation = $derived(
    conversationId ? generationJobsState.inFlightCountForConversation(conversationId) : 0
  );
  const canSendNewTurn = $derived(inFlightCountForConversation === 0);
  const agentParticipants = $derived(getAgentParticipants(conversationDetail));
  const availableAgentById = $derived(new Map(availableAgents.map((agent) => [agent.id, agent])));
  const participantById = $derived(
    new Map((conversationDetail?.participants ?? []).map((participant) => [participant.id, participant]))
  );
  const recipientOptions = $derived(
    agentParticipants.map((participant) => {
      const agent = participant.agent_id ? availableAgentById.get(participant.agent_id) : null;
      return {
        id: participant.id,
        label: participant.display_name ?? agent?.name ?? i18n.t("chat.assistant"),
        secondaryLabel: agent?.title ?? null
      };
    })
  );
  const attachmentChips = $derived(
    pendingAttachments.map((attachment) => ({
      id: attachment.id,
      name: attachment.name,
      meta: formatAttachmentSize(attachment.sizeBytes, attachment.mimeType)
    }))
  );

  $effect(() => {
    const nextConversationId = conversationDetail?.summary.id ?? conversationId ?? null;
    const nextDefault = resolvePreferredResponderParticipantIds(conversationDetail);
    const validCurrent = selectedResponderParticipantIds.filter((participantId) =>
      agentParticipants.some((participant) => participant.id === participantId)
    );

    if (loadedResponderConversationId !== nextConversationId) {
      selectedResponderParticipantIds = nextDefault;
      loadedResponderConversationId = nextConversationId;
      return;
    }

    if (validCurrent.length !== selectedResponderParticipantIds.length) {
      selectedResponderParticipantIds = validCurrent.length > 0 ? validCurrent : nextDefault;
      return;
    }

    if (selectedResponderParticipantIds.length === 0 && nextDefault.length > 0) {
      selectedResponderParticipantIds = nextDefault;
    }
  });

  $effect(() => {
    const jobSignature = activeGenerationJobs
      .map((job) => `${job.stream_id}:${job.status}:${job.accumulated_text.length}`)
      .join("|");
    if (messages.length || jobSignature) {
      void scrollToBottom();
    }
  });

  $effect(() => {
    if (messages.length === 0) {
      selectedNodeId = "";
      return;
    }

    if (!selectedNodeId || !messages.some((message) => message.node_id === selectedNodeId)) {
      selectedNodeId = messages[messages.length - 1]?.node_id ?? "";
    }
  });

  function describeChatError(error: unknown) {
    const message = error instanceof Error ? error.message : i18n.t("chat.generic_error");

    if (
      message.includes("no active api channel/model could be resolved") ||
      message.includes("no models found for channel")
    ) {
      return i18n.t("chat.no_channel_desc");
    }

    return message;
  }

  async function scrollToBottom() {
    await tick();
    if (scrollContainer) {
      scrollContainer.scrollTop = scrollContainer.scrollHeight;
    }
  }

  function getMessageText(message: MessageVersionView): string {
    return message.primary_content.text_content ?? message.primary_content.preview_text ?? "";
  }

  function getVersionInfo(message: MessageVersionView) {
    const versions = versionsByNode[message.node_id];
    if (!versions || versions.length <= 1) return null;
    const index = versions.findIndex((item) => item.version_id === message.version_id);
    return { current: index + 1, total: versions.length };
  }

  function getDefaultAgentId() {
    return availableAgents[0]?.id ?? null;
  }

  function handleSuggestion(text: string) {
    composerText = text;
  }

  function getHumanParticipantId(detail: ConversationDetail | null | undefined) {
    return (
      detail?.participants.find(
        (participant) => participant.enabled && participant.participant_type === "human"
      )?.id ?? null
    );
  }

  function getParticipantMeta(participantId: string | null | undefined, role: MessageVersionView["role"]) {
    if (!participantId) {
      return {
        name:
          role === "user"
            ? i18n.t("chat.user_label")
            : role === "system"
              ? i18n.t("chat.system")
              : i18n.t("chat.assistant"),
        avatarUri: null,
        kind: role === "user" ? "human" : role === "system" ? "system" : "agent"
      } as const;
    }

    const participant = participantById.get(participantId);
    const agent = participant?.agent_id ? availableAgentById.get(participant.agent_id) : null;

    if (participant?.participant_type === "human" || role === "user") {
      return {
        name: participant?.display_name ?? i18n.t("chat.user_label"),
        avatarUri: null,
        kind: "human"
      } as const;
    }

    if (participant?.participant_type === "agent" || role === "assistant") {
      return {
        name: participant?.display_name ?? agent?.name ?? i18n.t("chat.assistant"),
        avatarUri: agent?.avatar_uri ?? null,
        kind: "agent"
      } as const;
    }

    return {
      name: participant?.display_name ?? i18n.t("chat.system"),
      avatarUri: null,
      kind: "system"
    } as const;
  }

  function hasRequiredParticipants(detail: ConversationDetail | null | undefined) {
    return !!getHumanParticipantId(detail) && getAgentParticipants(detail).length > 0;
  }

  function getSelectedResponderParticipantIds(detail: ConversationDetail | null | undefined) {
    const responderIds = selectedResponderParticipantIds.filter((participantId) =>
      getAgentParticipants(detail).some((participant) => participant.id === participantId)
    );
    if (responderIds.length > 0) {
      return responderIds;
    }
    return resolvePreferredResponderParticipantIds(detail);
  }

  async function persistResponderSelection(
    detail: ConversationDetail,
    responderParticipantIds: string[]
  ) {
    const primaryResponderId = resolvePrimaryResponderParticipantId(detail);
    const nextPrimaryResponderId =
      primaryResponderId && responderParticipantIds.includes(primaryResponderId)
        ? primaryResponderId
        : responderParticipantIds[0] ?? null;

    return updateConversationMeta(detail.summary.id, {
      title: detail.summary.title,
      description: detail.summary.description,
      archived: detail.summary.archived,
      pinned: detail.summary.pinned,
      config_json: mergeConversationChatConfig(detail.summary, {
        primary_responder_participant_id: nextPrimaryResponderId,
        preferred_responder_participant_ids: responderParticipantIds
      })
    });
  }

  async function loadVersions(nodeId: string) {
    if (versionsByNode[nodeId] || loadingVersions[nodeId]) return;

    loadingVersions = { ...loadingVersions, [nodeId]: true };
    try {
      const versions = await listMessageVersions(nodeId);
      versionsByNode = { ...versionsByNode, [nodeId]: versions };
    } finally {
      loadingVersions = { ...loadingVersions, [nodeId]: false };
    }
  }

  async function handleSwitchVersion(nodeId: string, versionId: string) {
    try {
      await switchMessageVersion(nodeId, versionId);
      const versions = await listMessageVersions(nodeId);
      versionsByNode = { ...versionsByNode, [nodeId]: versions };
    } catch (error) {
      console.error("Version switch failed:", error);
      toast.error(i18n.t("chat.version_switch_failed"), {
        description: error instanceof Error ? error.message : i18n.t("chat.generic_error")
      });
    }
  }

  function startEdit(message: MessageVersionView) {
    editingNodeId = message.node_id;
    editingVersionId = message.version_id;
    editText = getMessageText(message);
  }

  function cancelEdit() {
    editingNodeId = "";
    editingVersionId = "";
    editText = "";
  }

  async function submitEdit() {
    const editingMessage = messages.find((item) => item.node_id === editingNodeId);
    if ((!editText.trim() && (editingMessage?.content_refs.length ?? 0) === 0) || editSaving) {
      return;
    }

    editSaving = true;
    try {
      await editMessageVersion({
        node_id: editingNodeId,
        base_version_id: editingVersionId,
        text: editText.trim()
      });
      cancelEdit();
    } catch (error) {
      console.error("Edit failed:", error);
      toast.error(i18n.t("chat.edit_failed"), {
        description: error instanceof Error ? error.message : i18n.t("chat.generic_error")
      });
    } finally {
      editSaving = false;
    }
  }

  async function handleDelete(nodeId: string) {
    try {
      await deleteMessageNode(nodeId);
    } catch (error) {
      console.error("Delete failed:", error);
      toast.error(i18n.t("chat.delete_failed"), {
        description: error instanceof Error ? error.message : i18n.t("chat.generic_error")
      });
    }
  }

  function copyText(text: string, versionId: string) {
    void navigator.clipboard.writeText(text);
    copiedVersionId = versionId;
    setTimeout(() => {
      copiedVersionId = "";
    }, 1500);
  }

  async function handleRegenerate(message: MessageVersionView) {
    if (inFlightCountForConversation > 0) {
      return;
    }

    let targetConversationId = conversationId;
    let targetDetail = conversationDetail;

    const ensuredDetail = onEnsureConversation ? await onEnsureConversation() : null;
    if (ensuredDetail) {
      targetConversationId = ensuredDetail.summary.id;
      targetDetail = ensuredDetail;
    }

    if (!targetConversationId || !targetDetail) {
      return;
    }

    const responderParticipantId = message.author_participant_id;
    const streamId = `regen-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    generationJobsState.registerJob({
      streamId,
      conversationId: targetConversationId,
      responderParticipantId
    });

    try {
      await regenerateReplyStream({
        request: {
          conversation_id: targetConversationId,
          responder_participant_id: responderParticipantId,
          trigger_message_version_id: message.version_id
        },
        stream_id: streamId
      });
    } catch (error) {
      generationJobsState.failJob(streamId, describeChatError(error));
      console.error("Regenerate failed:", error);
      toast.error(i18n.t("chat.regenerate_failed"), {
        description: describeChatError(error)
      });
    }
  }

  function toggleResponder(participantId: string) {
    if (selectedResponderParticipantIds.includes(participantId)) {
      if (selectedResponderParticipantIds.length === 1) {
        return;
      }

      selectedResponderParticipantIds = selectedResponderParticipantIds.filter(
        (item) => item !== participantId
      );
      return;
    }

    selectedResponderParticipantIds = [...selectedResponderParticipantIds, participantId];
  }

  async function handleSend() {
    const trimmedText = composerText.trim();
    const attachmentsToSend = [...pendingAttachments];
    if ((!trimmedText && attachmentsToSend.length === 0) || sending || !canSendNewTurn) {
      return;
    }

    let targetConversationId = conversationId;
    let targetDetail = conversationDetail;

    const defaultResponderAgentId =
      agentParticipants.find((participant) => participant.id === selectedResponderParticipantIds[0])
        ?.agent_id ??
      getDefaultAgentId() ??
      undefined;
    const ensuredDetail = onEnsureConversation
      ? await onEnsureConversation(
          !targetConversationId || !targetDetail || !hasRequiredParticipants(targetDetail)
            ? defaultResponderAgentId
            : undefined
        )
      : null;

    if (ensuredDetail) {
      targetConversationId = ensuredDetail.summary.id;
      targetDetail = ensuredDetail;
    }

    if (!targetConversationId || !targetDetail) {
      toast.error(i18n.t("chat.send_failed"), {
        description: i18n.t("chat.create_conversation_failed")
      });
      return;
    }

    const authorParticipantId = getHumanParticipantId(targetDetail);
    const responderParticipantIds = getSelectedResponderParticipantIds(targetDetail);

    if (!authorParticipantId || responderParticipantIds.length === 0) {
      toast.error(i18n.t("chat.no_agent_title"), {
        description: i18n.t("chat.no_participant_desc")
      });
      return;
    }

    composerText = "";
    pendingAttachments = [];
    sending = true;

    try {
      const userMessage = await createUserMessage({
        conversation_id: targetConversationId,
        author_participant_id: authorParticipantId,
        text: trimmedText
      });

      if (attachmentsToSend.length > 0) {
        const failures = await appendAttachmentsToMessage(userMessage.version_id, attachmentsToSend);
        if (failures.length > 0) {
          toast.warning(i18n.t("chat.attach_failed"), {
            description: failures.join("，")
          });
        }
      }

      try {
        targetDetail = await persistResponderSelection(targetDetail, responderParticipantIds);
      } catch (error) {
        console.error("Failed to persist responder selection:", error);
      }

      selectedResponderParticipantIds = responderParticipantIds;

      for (const [index, responderParticipantId] of responderParticipantIds.entries()) {
        const streamId = `stream-${Date.now()}-${index}-${Math.random().toString(36).slice(2, 8)}`;
        generationJobsState.registerJob({
          streamId,
          conversationId: targetConversationId,
          responderParticipantId
        });

        void generateReplyStream({
          request: {
            conversation_id: targetConversationId,
            responder_participant_id: responderParticipantId,
            trigger_message_version_id: userMessage.version_id
          },
          stream_id: streamId
        }).catch((error) => {
          generationJobsState.failJob(streamId, describeChatError(error));
          console.error("Generation failed:", error);
          if (conversationId === targetConversationId) {
            toast.error(i18n.t("chat.send_failed"), {
              description: describeChatError(error)
            });
          }
        });
      }
    } catch (error) {
      console.error("Send failed:", error);
      composerText = trimmedText;
      pendingAttachments = [...attachmentsToSend, ...pendingAttachments];
      toast.error(i18n.t("chat.send_failed"), {
        description: describeChatError(error)
      });
    } finally {
      sending = false;
    }
  }

  async function appendAttachmentsToMessage(
    messageVersionId: string,
    attachments: PendingAttachment[]
  ) {
    const results = await Promise.allSettled(
      attachments.map((attachment, index) =>
        appendMessageAttachment({
          message_version_id: messageVersionId,
          ref_role: attachment.refRole,
          sort_order: index,
          content: {
            content_type: attachment.contentType,
            mime_type: attachment.mimeType,
            text_content: attachment.textContent,
            source_file_path: attachment.sourceFilePath,
            primary_storage_uri: attachment.primaryStorageUri,
            size_bytes_hint: attachment.sizeBytes,
            preview_text: attachment.previewText,
            config_json: {
              file_name: attachment.name
            }
          },
          config_json: {
            file_name: attachment.name
          }
        })
      )
    );

    return results
      .map((result, index) =>
        result.status === "rejected" ? attachments[index]?.name ?? i18n.t("chat.attachment") : null
      )
      .filter((value): value is string => !!value);
  }

  async function handleAttachFiles(files: FileList | null) {
    if (!files || files.length === 0) {
      return;
    }

    const prepared = await Promise.allSettled(
      Array.from(files).map((file, index) => prepareAttachment(file, pendingAttachments.length + index))
    );

    const nextAttachments = prepared
      .filter((result): result is PromiseFulfilledResult<PendingAttachment> => result.status === "fulfilled")
      .map((result) => result.value);
    const failed = prepared.filter((result) => result.status === "rejected");

    if (nextAttachments.length > 0) {
      pendingAttachments = [...pendingAttachments, ...nextAttachments];
    }

    if (failed.length > 0) {
      toast.warning(i18n.t("chat.attach_unsupported"), {
        description: failed
          .map((result) =>
            result.status === "rejected" && result.reason instanceof Error
              ? result.reason.message
              : i18n.t("chat.generic_error")
          )
          .join("；")
      });
    }
  }

  function removeAttachment(attachmentId: string) {
    pendingAttachments = pendingAttachments.filter((attachment) => attachment.id !== attachmentId);
  }

  async function prepareAttachment(file: File, index: number): Promise<PendingAttachment> {
    const mimeType = file.type || null;
    const extension = file.name.split(".").pop()?.toLowerCase() ?? "";
    const path = (file as File & { path?: string }).path ?? null;
    const textLike =
      (mimeType?.startsWith("text/") ?? false) ||
      ["md", "markdown", "txt", "json", "csv", "xml", "yaml", "yml", "html", "htm", "svg"].includes(extension);

    if (path) {
      const descriptor = describeAttachment(mimeType, extension);
      return {
        id: `attachment-${Date.now()}-${index}-${Math.random().toString(36).slice(2, 7)}`,
        name: file.name,
        mimeType,
        sizeBytes: file.size,
        contentType: descriptor.contentType,
        refRole: descriptor.refRole,
        textContent: null,
        sourceFilePath: path,
        primaryStorageUri: null,
        previewText: file.name
      };
    }

    if (textLike) {
      const textContent = await file.text();
      return {
        id: `attachment-${Date.now()}-${index}-${Math.random().toString(36).slice(2, 7)}`,
        name: file.name,
        mimeType,
        sizeBytes: file.size,
        contentType: extension === "md" || extension === "markdown" ? "markdown" : extension === "json" ? "json" : "text",
        refRole: "attachment",
        textContent,
        sourceFilePath: null,
        primaryStorageUri: null,
        previewText: file.name
      };
    }

    if (mimeType?.startsWith("image/")) {
      const dataUrl = await readFileAsDataUrl(file);
      return {
        id: `attachment-${Date.now()}-${index}-${Math.random().toString(36).slice(2, 7)}`,
        name: file.name,
        mimeType,
        sizeBytes: file.size,
        contentType: "image",
        refRole: "image",
        textContent: null,
        sourceFilePath: null,
        primaryStorageUri: dataUrl,
        previewText: file.name
      };
    }

    throw new Error(`${file.name}: ${i18n.t("chat.attachment_missing_path")}`);
  }

  function describeAttachment(mimeType: string | null, extension: string) {
    if (mimeType?.startsWith("image/")) {
      return { contentType: "image" as const, refRole: "image" };
    }
    if (mimeType?.startsWith("audio/")) {
      return { contentType: "audio" as const, refRole: "audio" };
    }
    if (mimeType?.startsWith("video/")) {
      return { contentType: "video" as const, refRole: "video" };
    }
    if (
      (mimeType?.startsWith("text/") ?? false) ||
      ["md", "markdown", "txt", "json", "csv", "xml", "yaml", "yml", "html", "htm", "svg"].includes(extension)
    ) {
      if (extension === "md" || extension === "markdown") {
        return { contentType: "markdown" as const, refRole: "attachment" };
      }
      if (extension === "json") {
        return { contentType: "json" as const, refRole: "attachment" };
      }
      return { contentType: "text" as const, refRole: "attachment" };
    }
    return { contentType: "file" as const, refRole: "file" };
  }

  function readFileAsDataUrl(file: File) {
    return new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(reader.error ?? new Error("Failed to read attachment"));
      reader.onload = () => {
        if (typeof reader.result === "string") {
          resolve(reader.result);
          return;
        }
        reject(new Error("Failed to read attachment"));
      };
      reader.readAsDataURL(file);
    });
  }

  function formatAttachmentSize(sizeBytes: number, mimeType: string | null) {
    const sizeLabel =
      sizeBytes >= 1024 * 1024
        ? `${(sizeBytes / (1024 * 1024)).toFixed(1)} MB`
        : `${Math.max(1, Math.round(sizeBytes / 1024))} KB`;
    return mimeType ? `${mimeType} · ${sizeLabel}` : sizeLabel;
  }

  async function handleStartConversationWithAgent(agentId: string) {
    if (!onStartConversationWithAgent || startingAgentId) return;

    startingAgentId = agentId;
    try {
      const id = await onStartConversationWithAgent(agentId);
      if (!id) {
        toast.error(i18n.t("chat.create_conversation_failed"), {
          description: i18n.t("chat.generic_error")
        });
      }
    } finally {
      startingAgentId = "";
    }
  }

  const selectedMessage = $derived(
    selectedNodeId ? messages.find((message) => message.node_id === selectedNodeId) ?? null : null
  );
  const selectedVersionCount = $derived(
    selectedMessage ? (versionsByNode[selectedMessage.node_id]?.length ?? 1) : 0
  );
</script>

<ChatShell
  {desktopWide}
  {sidebarOpen}
  {inspectorVisible}
  {inspectorOpen}
  {onCloseSidebar}
  {onCloseInspector}
  {onOpenInspector}
>
  {#snippet rail()}
    <ResourceSidebar
      workspace="chat"
      items={sidebarItems}
      activeId={activeSidebarId}
      onSelect={onSelectSidebar}
      onCreateNew={onCreateSidebarItem}
      onRename={onRenameSidebarItem}
      onDelete={onDeleteSidebarItem}
    />
  {/snippet}

  {#snippet header()}
    <ChatHeader
      {conversationTitle}
      {editable}
      {onRename}
      {onToggleSidebar}
      {onToggleInspector}
    />
  {/snippet}

  {#snippet body()}
    <div bind:this={scrollContainer} class="app-scrollbar min-h-0 flex-1 overflow-y-auto">
      {#if loading}
        <div class="flex h-full items-center justify-center px-4 py-6">
          <div class="flex flex-col items-center gap-3">
            <Loader2 size={28} class="animate-spin text-[var(--brand)]" />
            <span class="text-sm text-[var(--ink-faint)]">{i18n.t("chat.loading")}</span>
          </div>
        </div>
      {:else if messages.length === 0 && activeGenerationJobs.length === 0}
        <div class="h-full">
          <ChatEmptyState
            {conversationTitle}
            {availableAgents}
            {startingAgentId}
            onSelectSuggestion={handleSuggestion}
            onStartWithAgent={handleStartConversationWithAgent}
          />
        </div>
      {:else}
        <div class="mx-auto w-full max-w-[var(--message-max-width)] px-4 py-6">
          {#each messages as message, index (message.version_id)}
            {@const text = getMessageText(message)}
            {@const versionInfo = getVersionInfo(message)}
            {@const author = getParticipantMeta(message.author_participant_id, message.role)}

            <ChatMessageItem
              message={message}
              {text}
              authorName={author.name}
              avatarUri={author.avatarUri}
              authorKind={author.kind}
              {versionInfo}
              isEditing={editingNodeId === message.node_id}
              selected={selectedNodeId === message.node_id}
              bind:editText
              {editSaving}
              copied={copiedVersionId === message.version_id}
              generationLocked={inFlightCountForConversation > 0}
              animationDelay={`${Math.min(index * 30, 300)}ms`}
              onLoadVersions={() => void loadVersions(message.node_id)}
              onStartEdit={() => startEdit(message)}
              onCancelEdit={cancelEdit}
              onSubmitEdit={() => void submitEdit()}
              onCopy={() => copyText(text, message.version_id)}
              onDelete={() => void handleDelete(message.node_id)}
              onRegenerate={() => void handleRegenerate(message)}
              onSelect={() => {
                selectedNodeId = message.node_id;
              }}
              onPrevVersion={() => {
                const versions = versionsByNode[message.node_id] ?? [];
                const currentIndex = versions.findIndex((item) => item.version_id === message.version_id);
                if (currentIndex > 0) {
                  void handleSwitchVersion(message.node_id, versions[currentIndex - 1].version_id);
                }
              }}
              onNextVersion={() => {
                const versions = versionsByNode[message.node_id] ?? [];
                const currentIndex = versions.findIndex((item) => item.version_id === message.version_id);
                if (currentIndex < versions.length - 1) {
                  void handleSwitchVersion(message.node_id, versions[currentIndex + 1].version_id);
                }
              }}
            />
          {/each}

          {#each activeGenerationJobs as job (job.stream_id)}
            {@const author = getParticipantMeta(job.responder_participant_id, "assistant")}
            <ChatStreamingMessage
              text={job.accumulated_text}
              authorName={author.name}
              avatarUri={author.avatarUri}
              status={job.status}
              errorText={job.error_text}
              onDismiss={() => generationJobsState.dismissConversationFailures(conversationId)}
            />
          {/each}
        </div>
      {/if}
    </div>
  {/snippet}

  {#snippet composer()}
    <ChatComposer
      bind:value={composerText}
      {sending}
      canSend={canSendNewTurn && (!!composerText.trim() || pendingAttachments.length > 0)}
      busy={inFlightCountForConversation > 0}
      attachments={attachmentChips}
      availableRecipients={recipientOptions}
      selectedRecipientIds={selectedResponderParticipantIds}
      onSend={() => void handleSend()}
      onAttachFiles={(files) => void handleAttachFiles(files)}
      onRemoveAttachment={removeAttachment}
      onToggleRecipient={toggleResponder}
    />
  {/snippet}

  {#snippet inspector()}
    <InspectorPanel
      tabs={inspectorTabs}
      activeTab={activeInspectorTab}
      {conversationTitle}
      {conversationDetail}
      {availableAgents}
      {messages}
      selectedMessage={selectedMessage}
      selectedVersionCount={selectedVersionCount}
      onSelectTab={onSelectInspectorTab}
      onClose={onCloseInspector}
    />
  {/snippet}
</ChatShell>
