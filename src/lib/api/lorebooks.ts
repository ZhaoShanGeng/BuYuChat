import { tauriInvoke } from "$lib/api/client";
import type { StoredContent } from "$lib/api/messages";
import type { ContentWriteInput } from "$lib/api/agents";

export type LorebookSummary = {
  id: string;
  name: string;
  description: string | null;
  scan_depth: number;
  token_budget: number | null;
  insertion_strategy: string;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type LorebookEntryKeyDetail = {
  id: string;
  entry_id: string;
  key_type: string;
  match_type: string;
  pattern_text: string;
  case_sensitive: boolean;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type LorebookEntryDetail = {
  id: string;
  lorebook_id: string;
  title: string | null;
  primary_content: StoredContent;
  activation_strategy: string;
  keyword_logic: string;
  insertion_position: string;
  insertion_order: number;
  insertion_depth: number | null;
  insertion_role: string | null;
  outlet_name: string | null;
  entry_scope: string;
  enabled: boolean;
  sort_order: number;
  keys: LorebookEntryKeyDetail[];
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type LorebookDetail = {
  lorebook: LorebookSummary;
  entries: LorebookEntryDetail[];
};

export type CreateLorebookInput = {
  name: string;
  description: string | null;
  scan_depth: number;
  token_budget: number | null;
  insertion_strategy: string;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type UpdateLorebookInput = CreateLorebookInput;

export type CreateLorebookEntryInput = {
  lorebook_id: string;
  title: string | null;
  primary_content: ContentWriteInput;
  activation_strategy: string;
  keyword_logic: string;
  insertion_position: string;
  insertion_order: number;
  insertion_depth: number | null;
  insertion_role: "system" | "user" | "assistant" | "tool" | null;
  outlet_name: string | null;
  entry_scope: string;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type UpdateLorebookEntryInput = Omit<CreateLorebookEntryInput, "lorebook_id">;

export function listLorebooks() {
  return tauriInvoke<LorebookSummary[]>("list_lorebooks");
}

export function getLorebookDetail(id: string) {
  return tauriInvoke<LorebookDetail>("get_lorebook_detail", { id });
}

export function createLorebook(input: CreateLorebookInput) {
  return tauriInvoke<LorebookDetail>("create_lorebook", { input });
}

export function updateLorebook(id: string, input: UpdateLorebookInput) {
  return tauriInvoke<LorebookDetail>("update_lorebook", { id, input });
}

export function deleteLorebook(id: string) {
  return tauriInvoke<void>("delete_lorebook", { id });
}

export function createLorebookEntry(input: CreateLorebookEntryInput) {
  return tauriInvoke<LorebookEntryDetail>("create_lorebook_entry", { input });
}

export function updateLorebookEntry(id: string, input: UpdateLorebookEntryInput) {
  return tauriInvoke<LorebookEntryDetail>("update_lorebook_entry", { id, input });
}

export function deleteLorebookEntry(id: string) {
  return tauriInvoke<void>("delete_lorebook_entry", { id });
}

export function replaceLorebookEntryKeys(entryId: string, keys: string[]) {
  return tauriInvoke<string[]>("replace_lorebook_entry_keys", { entryId, keys });
}
