import type {
  ConversationDetail,
  ConversationParticipantDetail,
  ConversationSummary
} from "$lib/api/conversations";

export type ChatConversationConfig = {
  primary_responder_participant_id: string | null;
  preferred_responder_participant_ids: string[];
};

type ConversationConfigShape = {
  chat?: Partial<ChatConversationConfig>;
  [key: string]: unknown;
};

export function getAgentParticipants(detail: ConversationDetail | null | undefined) {
  return (
    detail?.participants.filter(
      (participant) =>
        participant.enabled &&
        participant.participant_type === "agent" &&
        !!participant.agent_id
    ) ?? []
  );
}

export function readChatConversationConfig(
  summary: ConversationSummary | null | undefined
): ChatConversationConfig {
  const config = (summary?.config_json ?? {}) as ConversationConfigShape;
  const chat = config.chat ?? {};

  return {
    primary_responder_participant_id:
      typeof chat.primary_responder_participant_id === "string"
        ? chat.primary_responder_participant_id
        : null,
    preferred_responder_participant_ids: Array.isArray(chat.preferred_responder_participant_ids)
      ? chat.preferred_responder_participant_ids.filter(
          (value): value is string => typeof value === "string"
        )
      : []
  };
}

export function resolvePrimaryResponderParticipantId(
  detail: ConversationDetail | null | undefined
): string | null {
  const responders = getAgentParticipants(detail);
  if (responders.length === 0) return null;

  const config = readChatConversationConfig(detail?.summary);
  if (
    config.primary_responder_participant_id &&
    responders.some((participant) => participant.id === config.primary_responder_participant_id)
  ) {
    return config.primary_responder_participant_id;
  }

  return responders[0]?.id ?? null;
}

export function resolvePreferredResponderParticipantIds(
  detail: ConversationDetail | null | undefined
): string[] {
  const responders = getAgentParticipants(detail);
  if (responders.length === 0) return [];

  const config = readChatConversationConfig(detail?.summary);
  const validPreferred = config.preferred_responder_participant_ids.filter((participantId) =>
    responders.some((participant) => participant.id === participantId)
  );

  if (validPreferred.length > 0) {
    return validPreferred;
  }

  const primaryResponderId = resolvePrimaryResponderParticipantId(detail);
  return primaryResponderId ? [primaryResponderId] : [];
}

export function mergeConversationChatConfig(
  summary: ConversationSummary,
  nextChatConfig: Partial<ChatConversationConfig>
) {
  const currentConfig = (summary.config_json ?? {}) as ConversationConfigShape;
  const currentChat = readChatConversationConfig(summary);

  return {
    ...currentConfig,
    chat: {
      ...currentChat,
      ...nextChatConfig
    }
  };
}

export function buildConversationChatConfigFromParticipants(
  participants: ConversationParticipantDetail[],
  primaryAgentId: string | null,
  preferredAgentIds: string[]
) {
  const enabledAgentParticipants = participants.filter(
    (participant) =>
      participant.enabled &&
      participant.participant_type === "agent" &&
      !!participant.agent_id
  );

  const preferredResponderParticipantIds = preferredAgentIds
    .map(
      (agentId) =>
        enabledAgentParticipants.find((participant) => participant.agent_id === agentId)?.id ?? null
    )
    .filter((value): value is string => !!value);

  const primaryResponderParticipantId =
    (primaryAgentId
      ? enabledAgentParticipants.find((participant) => participant.agent_id === primaryAgentId)?.id
      : null) ?? preferredResponderParticipantIds[0] ?? enabledAgentParticipants[0]?.id ?? null;

  return {
    primary_responder_participant_id: primaryResponderParticipantId,
    preferred_responder_participant_ids:
      preferredResponderParticipantIds.length > 0
        ? preferredResponderParticipantIds
        : primaryResponderParticipantId
          ? [primaryResponderParticipantId]
          : []
  };
}
