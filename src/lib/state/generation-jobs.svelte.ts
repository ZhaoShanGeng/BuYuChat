import type { MessageVersionView } from "$lib/api/messages";
import type { GenerationStreamEvent } from "$lib/events/generation-stream";

export type GenerationJobStatus =
  | "queued"
  | "running"
  | "completed"
  | "failed";

export type GenerationJob = {
  stream_id: string;
  conversation_id: string;
  responder_participant_id: string;
  status: GenerationJobStatus;
  accumulated_text: string;
  message_version_id: string | null;
  error_text: string | null;
  started_at: number;
  finished_at: number | null;
};

class GenerationJobsState {
  jobsByStreamId = $state<Record<string, GenerationJob>>({});
  unreadByConversationId = $state<Record<string, number>>({});
  activeConversationId = $state<string | null>(null);

  registerJob(input: {
    streamId: string;
    conversationId: string;
    responderParticipantId: string;
  }) {
    this.jobsByStreamId = {
      ...this.jobsByStreamId,
      [input.streamId]: {
        stream_id: input.streamId,
        conversation_id: input.conversationId,
        responder_participant_id: input.responderParticipantId,
        status: "queued",
        accumulated_text: "",
        message_version_id: null,
        error_text: null,
        started_at: Date.now(),
        finished_at: null
      }
    };
  }

  setActiveConversation(conversationId: string | null) {
    this.activeConversationId = conversationId;
    if (!conversationId) return;

    if (this.unreadByConversationId[conversationId]) {
      const next = { ...this.unreadByConversationId };
      delete next[conversationId];
      this.unreadByConversationId = next;
    }
  }

  applyEvent(event: GenerationStreamEvent) {
    const existing = this.jobsByStreamId[event.stream_id];
    if (!existing) {
      return;
    }

    const accumulatedText = event.accumulated_text ?? existing.accumulated_text;
    const base: GenerationJob = {
      ...existing,
      accumulated_text: accumulatedText,
      message_version_id: event.message_version_id ?? existing.message_version_id,
      error_text: event.error_text ?? existing.error_text
    };

    switch (event.kind) {
      case "started":
        this.jobsByStreamId = {
          ...this.jobsByStreamId,
          [event.stream_id]: {
            ...base,
            status: "running",
            accumulated_text: ""
          }
        };
        return;
      case "delta":
        this.jobsByStreamId = {
          ...this.jobsByStreamId,
          [event.stream_id]: {
            ...base,
            status: "running"
          }
        };
        return;
      case "completed":
        this.jobsByStreamId = {
          ...this.jobsByStreamId,
          [event.stream_id]: {
            ...base,
            status: "completed",
            finished_at: Date.now()
          }
        };
        this.markConversationUnreadIfNeeded(existing.conversation_id);
        return;
      case "failed":
        this.failJob(event.stream_id, base.error_text);
        return;
    }
  }

  failJob(streamId: string, errorText: string | null | undefined) {
    const existing = this.jobsByStreamId[streamId];
    if (!existing) {
      return;
    }

    const alreadyFailed = existing.status === "failed";
    this.jobsByStreamId = {
      ...this.jobsByStreamId,
      [streamId]: {
        ...existing,
        status: "failed",
        error_text: errorText ?? existing.error_text,
        finished_at: existing.finished_at ?? Date.now()
      }
    };

    if (!alreadyFailed) {
      this.markConversationUnreadIfNeeded(existing.conversation_id);
    }
  }

  resolveMessage(message: MessageVersionView) {
    const targetEntry = Object.entries(this.jobsByStreamId).find(
      ([, job]) => job.message_version_id === message.version_id
    );
    if (!targetEntry) {
      return;
    }

    const [streamId] = targetEntry;
    const next = { ...this.jobsByStreamId };
    delete next[streamId];
    this.jobsByStreamId = next;
  }

  dismissConversationFailures(conversationId: string) {
    const next = { ...this.jobsByStreamId };
    let changed = false;
    for (const [streamId, job] of Object.entries(next)) {
      if (job.conversation_id === conversationId && job.status === "failed") {
        delete next[streamId];
        changed = true;
      }
    }
    if (changed) {
      this.jobsByStreamId = next;
    }
  }

  inFlightCountForConversation(conversationId: string) {
    return Object.values(this.jobsByStreamId).filter(
      (job) =>
        job.conversation_id === conversationId &&
        (job.status === "queued" || job.status === "running")
    ).length;
  }

  unreadCountForConversation(conversationId: string) {
    return this.unreadByConversationId[conversationId] ?? 0;
  }

  visibleJobsForConversation(conversationId: string) {
    return Object.values(this.jobsByStreamId)
      .filter((job) => {
        if (job.conversation_id !== conversationId) return false;
        return job.status === "queued" || job.status === "running" || job.status === "failed";
      })
      .sort((a, b) => a.started_at - b.started_at);
  }

  private markConversationUnreadIfNeeded(conversationId: string) {
    if (!conversationId || conversationId === this.activeConversationId) {
      return;
    }

    this.unreadByConversationId = {
      ...this.unreadByConversationId,
      [conversationId]: (this.unreadByConversationId[conversationId] ?? 0) + 1
    };
  }
}

export const generationJobsState = new GenerationJobsState();
