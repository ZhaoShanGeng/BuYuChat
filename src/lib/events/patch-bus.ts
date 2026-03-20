import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const INCREMENTAL_PATCH_EVENT = "incremental_patch_event";

export type IncrementalPatchOp = "upsert" | "delete" | "replace";

export type IncrementalPatchEvent = {
  patch_id: string;
  emitted_at: number;
  scope_kind: string;
  scope_id: string | null;
  resource_kind: string;
  resource_id: string | null;
  op: IncrementalPatchOp;
  data: unknown;
};

export async function listenIncrementalPatches(
  handler: (event: IncrementalPatchEvent) => void
): Promise<UnlistenFn> {
  return listen<IncrementalPatchEvent>(INCREMENTAL_PATCH_EVENT, (event) => {
    handler(event.payload);
  });
}
