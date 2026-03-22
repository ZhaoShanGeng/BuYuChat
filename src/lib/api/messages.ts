import { tauriInvoke } from "$lib/api/client";

export type ContentWriteInput = {
  content_type: string;
  mime_type: string | null;
  text_content: string | null;
  source_file_path: string | null;
  primary_storage_uri: string | null;
  size_bytes_hint: number | null;
  preview_text: string | null;
  config_json: Record<string, unknown>;
};

export type StoredContent = {
  content_id: string;
  content_type: string;
  storage_kind: string;
  mime_type: string | null;
  size_bytes: number;
  preview_text: string | null;
  primary_storage_uri: string | null;
  text_content: string | null;
  chunk_count: number;
  sha256: string | null;
  config_json: Record<string, unknown>;
};

export type MessageContentRefView = {
  ref_id: string;
  ref_role: string;
  plugin_id: string | null;
  sort_order: number;
  content: StoredContent;
  config_json: Record<string, unknown>;
};

export type MessageVersionView = {
  node_id: string;
  version_id: string;
  conversation_id: string;
  author_participant_id: string;
  role: "system" | "user" | "assistant" | "tool";
  reply_to_node_id: string | null;
  order_key: string;
  version_index: number;
  is_active: boolean;
  primary_content: StoredContent;
  content_refs: MessageContentRefView[];
  context_policy: string;
  viewer_policy: string;
  api_channel_id: string | null;
  api_channel_model_id: string | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  total_tokens: number | null;
  finish_reason: string | null;
  generation_run_id: string | null;
  created_at: number;
};

// ─── Queries ───

export function listVisibleMessages(conversationId: string) {
  return tauriInvoke<MessageVersionView[]>("list_visible_messages", {
    conversationId
  });
}

export function listMessageVersions(nodeId: string) {
  return tauriInvoke<MessageVersionView[]>("list_message_versions", {
    nodeId
  });
}

export function getMessageBody(versionId: string) {
  return tauriInvoke<StoredContent>("get_message_body", { versionId });
}

// ─── Mutations ───

export type CreateMessageInput = {
  conversation_id: string;
  author_participant_id: string;
  text: string;
  reply_to_node_id?: string | null;
};

export function createUserMessage(input: CreateMessageInput) {
  const primaryContent: ContentWriteInput = {
    content_type: "text",
    mime_type: "text/plain",
    text_content: input.text,
    source_file_path: null,
    primary_storage_uri: null,
    size_bytes_hint: null,
    preview_text: input.text.trim().slice(0, 240) || null,
    config_json: {}
  };

  return tauriInvoke<MessageVersionView>("create_user_message", {
    input: {
      conversation_id: input.conversation_id,
      author_participant_id: input.author_participant_id,
      role: "user",
      reply_to_node_id: input.reply_to_node_id ?? null,
      order_after_node_id: null,
      primary_content: primaryContent,
      context_policy: "full",
      viewer_policy: "full",
      config_json: {}
    }
  });
}

export function createSystemMessage(input: CreateMessageInput) {
  const primaryContent: ContentWriteInput = {
    content_type: "text",
    mime_type: "text/plain",
    text_content: input.text,
    source_file_path: null,
    primary_storage_uri: null,
    size_bytes_hint: null,
    preview_text: input.text.trim().slice(0, 240) || null,
    config_json: {}
  };

  return tauriInvoke<MessageVersionView>("create_system_message", {
    input: {
      conversation_id: input.conversation_id,
      author_participant_id: input.author_participant_id,
      role: "system",
      reply_to_node_id: input.reply_to_node_id ?? null,
      order_after_node_id: null,
      primary_content: primaryContent,
      context_policy: "full",
      viewer_policy: "full",
      config_json: {}
    }
  });
}

export type GenerateReplyInput = {
  conversation_id: string;
  responder_participant_id: string;
  trigger_message_version_id?: string | null;
  override_api_channel_id?: string | null;
  override_api_channel_model_id?: string | null;
  request_parameters_json?: Record<string, unknown> | null;
  create_hidden_message?: boolean;
};

export type GenerateReplyStreamInput = {
  request: GenerateReplyInput;
  stream_id: string;
};

export function generateReply(input: GenerateReplyInput) {
  return tauriInvoke<MessageVersionView>("generate_reply", {
    input: {
      conversation_id: input.conversation_id,
      responder_participant_id: input.responder_participant_id,
      trigger_message_version_id: input.trigger_message_version_id ?? null,
      override_api_channel_id: input.override_api_channel_id ?? null,
      override_api_channel_model_id: input.override_api_channel_model_id ?? null,
      request_parameters_json: input.request_parameters_json ?? null,
      create_hidden_message: input.create_hidden_message ?? false
    }
  });
}

export function generateReplyStream(input: GenerateReplyStreamInput) {
  return tauriInvoke<MessageVersionView>("generate_reply_stream", {
    input: {
      stream_id: input.stream_id,
      request: {
        conversation_id: input.request.conversation_id,
        responder_participant_id: input.request.responder_participant_id,
        trigger_message_version_id: input.request.trigger_message_version_id ?? null,
        override_api_channel_id: input.request.override_api_channel_id ?? null,
        override_api_channel_model_id: input.request.override_api_channel_model_id ?? null,
        request_parameters_json: input.request.request_parameters_json ?? null,
        create_hidden_message: input.request.create_hidden_message ?? false
      }
    }
  });
}

export type RegenerateReplyInput = {
  conversation_id: string;
  responder_participant_id: string;
  trigger_message_version_id: string;
  override_api_channel_id?: string | null;
  override_api_channel_model_id?: string | null;
  request_parameters_json?: Record<string, unknown> | null;
};

export type RegenerateReplyStreamInput = {
  request: RegenerateReplyInput;
  stream_id: string;
};

export function regenerateReply(input: RegenerateReplyInput) {
  return tauriInvoke<MessageVersionView>("regenerate_reply", {
    input: {
      conversation_id: input.conversation_id,
      responder_participant_id: input.responder_participant_id,
      trigger_message_version_id: input.trigger_message_version_id,
      override_api_channel_id: input.override_api_channel_id ?? null,
      override_api_channel_model_id: input.override_api_channel_model_id ?? null,
      request_parameters_json: input.request_parameters_json ?? null
    }
  });
}

export function regenerateReplyStream(input: RegenerateReplyStreamInput) {
  return tauriInvoke<MessageVersionView>("regenerate_reply_stream", {
    input: {
      stream_id: input.stream_id,
      request: {
        conversation_id: input.request.conversation_id,
        responder_participant_id: input.request.responder_participant_id,
        trigger_message_version_id: input.request.trigger_message_version_id,
        override_api_channel_id: input.request.override_api_channel_id ?? null,
        override_api_channel_model_id: input.request.override_api_channel_model_id ?? null,
        request_parameters_json: input.request.request_parameters_json ?? null
      }
    }
  });
}

export type EditMessageVersionInput = {
  node_id: string;
  base_version_id: string;
  text: string;
  context_policy?: MessageVersionView["context_policy"];
  viewer_policy?: MessageVersionView["viewer_policy"];
  config_json?: Record<string, unknown>;
};

export function editMessageVersion(input: EditMessageVersionInput) {
  return tauriInvoke<MessageVersionView>("edit_message_version", {
    input: {
      node_id: input.node_id,
      base_version_id: input.base_version_id,
      primary_content: {
        content_type: "text",
        mime_type: "text/plain",
        text_content: input.text,
        source_file_path: null,
        primary_storage_uri: null,
        size_bytes_hint: null,
        preview_text: input.text.trim().slice(0, 240) || null,
        config_json: {}
      },
      context_policy: input.context_policy ?? "full",
      viewer_policy: input.viewer_policy ?? "full",
      config_json: input.config_json ?? {}
    }
  });
}

export function switchMessageVersion(nodeId: string, versionId: string) {
  return tauriInvoke<MessageVersionView>("switch_message_version", {
    nodeId,
    versionId
  });
}

export function deleteMessageVersion(nodeId: string, versionId: string) {
  return tauriInvoke<void>("delete_message_version", { nodeId, versionId });
}

export function deleteMessageNode(nodeId: string) {
  return tauriInvoke<void>("delete_message_node", { nodeId });
}

export type AppendMessageAttachmentInput = {
  message_version_id: string;
  plugin_id?: string | null;
  ref_role: string;
  sort_order: number;
  content: ContentWriteInput;
  config_json?: Record<string, unknown>;
};

export function appendMessageAttachment(input: AppendMessageAttachmentInput) {
  return tauriInvoke<MessageContentRefView>("append_message_attachment", {
    input: {
      message_version_id: input.message_version_id,
      plugin_id: input.plugin_id ?? null,
      ref_role: input.ref_role,
      sort_order: input.sort_order,
      content: input.content,
      config_json: input.config_json ?? {}
    }
  });
}
