import { tauriInvoke } from "$lib/api/client";
import type { ChannelBindingDetail } from "$lib/api/conversations";
import type { StoredContent } from "$lib/api/messages";
import type { ContentWriteInput } from "$lib/api/agents";

export type PresetSummary = {
  id: string;
  name: string;
  description: string | null;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type PresetEntryDetail = {
  id: string;
  preset_id: string;
  name: string;
  role: "system" | "user" | "assistant" | "tool";
  primary_content: StoredContent;
  position_type: string;
  list_order: number;
  depth: number | null;
  depth_order: number;
  triggers_json: Record<string, unknown>;
  enabled: boolean;
  is_pinned: boolean;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type PresetDetail = {
  preset: PresetSummary;
  entries: PresetEntryDetail[];
  channel_bindings: ChannelBindingDetail[];
};

export type CreatePresetInput = {
  name: string;
  description: string | null;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type UpdatePresetInput = CreatePresetInput;

export type CreatePresetEntryInput = {
  preset_id: string;
  name: string;
  role: "system" | "user" | "assistant" | "tool";
  primary_content: ContentWriteInput;
  position_type: string;
  list_order: number;
  depth: number | null;
  depth_order: number;
  triggers_json: Record<string, unknown>;
  enabled: boolean;
  is_pinned: boolean;
  config_json: Record<string, unknown>;
};

export type UpdatePresetEntryInput = Omit<CreatePresetEntryInput, "preset_id">;

export function listPresets() {
  return tauriInvoke<PresetSummary[]>("list_presets");
}

export function getPresetDetail(id: string) {
  return tauriInvoke<PresetDetail>("get_preset_detail", { id });
}

export function createPreset(input: CreatePresetInput) {
  return tauriInvoke<PresetDetail>("create_preset", { input });
}

export function updatePreset(id: string, input: UpdatePresetInput) {
  return tauriInvoke<PresetDetail>("update_preset", { id, input });
}

export function deletePreset(id: string) {
  return tauriInvoke<void>("delete_preset", { id });
}

export function createPresetEntry(input: CreatePresetEntryInput) {
  return tauriInvoke<PresetEntryDetail>("create_preset_entry", { input });
}

export function updatePresetEntry(id: string, input: UpdatePresetEntryInput) {
  return tauriInvoke<PresetEntryDetail>("update_preset_entry", { id, input });
}

export function deletePresetEntry(id: string) {
  return tauriInvoke<void>("delete_preset_entry", { id });
}

export function reorderPresetEntries(presetId: string, entryIds: string[]) {
  return tauriInvoke<PresetEntryDetail[]>("reorder_preset_entries", { presetId, entryIds });
}
