import {
  getConversationDetail,
  listConversations,
  type ConversationDetail,
  type ConversationSummary
} from "$lib/api/conversations";
import { listVisibleMessages, type MessageVersionView } from "$lib/api/messages";
import type { IncrementalPatchEvent } from "$lib/events/patch-bus";

function sortMessages(items: MessageVersionView[]) {
  return [...items].sort((a, b) => a.order_key.localeCompare(b.order_key));
}

class ConversationsState {
  summaries = $state<ConversationSummary[]>([]);
  detailsById = $state<Record<string, ConversationDetail>>({});
  visibleMessagesByConversationId = $state<Record<string, MessageVersionView[]>>({});
  activeConversationId = $state<string | null>(null);
  loadingList = $state(false);
  loadingConversation = $state(false);

  get activeSummary() {
    return this.summaries.find((item) => item.id === this.activeConversationId) ?? null;
  }

  get activeDetail() {
    return this.activeConversationId ? this.detailsById[this.activeConversationId] ?? null : null;
  }

  get activeMessages() {
    return this.activeConversationId
      ? this.visibleMessagesByConversationId[this.activeConversationId] ?? []
      : [];
  }

  async bootstrap() {
    await this.loadList();

    if (!this.activeConversationId && this.summaries.length > 0) {
      await this.selectConversation(this.summaries[0].id);
    }
  }

  async loadList() {
    this.loadingList = true;
    try {
      const summaries = await listConversations();
      this.summaries = summaries.sort((a, b) => b.updated_at - a.updated_at);

      if (
        this.activeConversationId &&
        !this.summaries.some((item) => item.id === this.activeConversationId)
      ) {
        this.activeConversationId = this.summaries[0]?.id ?? null;
      }
    } finally {
      this.loadingList = false;
    }
  }

  async selectConversation(id: string) {
    this.activeConversationId = id;
    await this.loadConversation(id);
  }

  async loadConversation(id: string) {
    this.loadingConversation = true;

    try {
      const [detail, messages] = await Promise.all([
        getConversationDetail(id),
        listVisibleMessages(id)
      ]);

      this.detailsById = {
        ...this.detailsById,
        [id]: detail
      };

      this.visibleMessagesByConversationId = {
        ...this.visibleMessagesByConversationId,
        [id]: sortMessages(messages)
      };

      if (!this.summaries.some((item) => item.id === detail.summary.id)) {
        this.summaries = [detail.summary, ...this.summaries];
      } else {
        this.summaries = this.summaries.map((item) =>
          item.id === detail.summary.id ? detail.summary : item
        );
      }
    } finally {
      this.loadingConversation = false;
    }
  }

  applyPatch(event: IncrementalPatchEvent) {
    if (event.resource_kind === "conversation") {
      if (event.op === "delete" && event.resource_id) {
        this.summaries = this.summaries.filter((item) => item.id !== event.resource_id);
        if (this.activeConversationId === event.resource_id) {
          this.activeConversationId = this.summaries[0]?.id ?? null;
        }
        return;
      }

      if (event.op === "upsert" && event.data && typeof event.data === "object") {
        const detail = event.data as ConversationDetail;
        this.detailsById = {
          ...this.detailsById,
          [detail.summary.id]: detail
        };
        this.summaries = [
          detail.summary,
          ...this.summaries.filter((item) => item.id !== detail.summary.id)
        ].sort((a, b) => b.updated_at - a.updated_at);
      }
    }

    if (
      event.resource_kind === "message_version" &&
      event.op === "upsert" &&
      event.scope_kind === "conversation" &&
      event.scope_id &&
      event.data &&
      typeof event.data === "object"
    ) {
      const message = event.data as MessageVersionView;
      const current = this.visibleMessagesByConversationId[event.scope_id] ?? [];
      const next = sortMessages([
        message,
        ...current.filter((item) => item.version_id !== message.version_id)
      ]);

      this.visibleMessagesByConversationId = {
        ...this.visibleMessagesByConversationId,
        [event.scope_id]: next
      };
    }

    if (
      event.resource_kind === "message_node" &&
      event.op === "delete" &&
      event.scope_kind === "conversation" &&
      event.scope_id &&
      event.resource_id
    ) {
      const current = this.visibleMessagesByConversationId[event.scope_id] ?? [];
      this.visibleMessagesByConversationId = {
        ...this.visibleMessagesByConversationId,
        [event.scope_id]: current.filter((item) => item.node_id !== event.resource_id)
      };
    }
  }
}

export const conversationsState = new ConversationsState();
