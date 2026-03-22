import { tauriInvoke } from "$lib/api/client";

export type ConversationVariable = {
  key: string;
  value: string;
  source: string; // "user", "workflow", "plugin"
  updated_at: number;
};

export function listVariables(conversationId: string) {
  return tauriInvoke<ConversationVariable[]>("list_variables", { conversationId });
}

export function updateVariable(conversationId: string, key: string, value: string) {
  return tauriInvoke<void>("update_variable", { conversationId, key, value });
}

export function deleteVariable(conversationId: string, key: string) {
  return tauriInvoke<void>("delete_variable", { conversationId, key });
}
