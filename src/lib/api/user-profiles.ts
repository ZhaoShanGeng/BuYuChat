import { tauriInvoke } from "$lib/api/client";
import type { ContentWriteInput } from "$lib/api/agents";
import type { StoredContent } from "$lib/api/messages";

export type UserProfileSummary = {
  id: string;
  name: string;
  title: string | null;
  avatar_uri: string | null;
  insertion_position: string | null;
  insertion_depth: number | null;
  insertion_role: string | null;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type UserProfileDetail = {
  summary: UserProfileSummary;
  description_content: StoredContent | null;
};

export type CreateUserProfileInput = {
  name: string;
  title: string | null;
  avatar_uri: string | null;
  description_content: ContentWriteInput | null;
  insertion_position: string | null;
  insertion_depth: number | null;
  insertion_role: string | null;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export function listUserProfiles() {
  return tauriInvoke<UserProfileSummary[]>("list_user_profiles");
}

export function getUserProfileDetail(id: string) {
  return tauriInvoke<UserProfileDetail>("get_user_profile", { id });
}

export function createUserProfile(input: CreateUserProfileInput) {
  return tauriInvoke<UserProfileDetail>("create_user_profile", { input });
}

export function updateUserProfile(id: string, input: CreateUserProfileInput) {
  return tauriInvoke<UserProfileDetail>("update_user_profile", { id, input });
}

export function deleteUserProfile(id: string) {
  return tauriInvoke<void>("delete_user_profile", { id });
}
