import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const GENERATION_STREAM_EVENT = "generation_stream_event";

export type GenerationStreamEventKind = "started" | "delta" | "completed" | "failed";

export type GenerationStreamEvent = {
  stream_id: string;
  kind: GenerationStreamEventKind;
  delta_text: string | null;
  accumulated_text: string | null;
  message_version_id: string | null;
  finish_reason: string | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  total_tokens: number | null;
  error_text: string | null;
};

export async function listenGenerationStream(
  handler: (event: GenerationStreamEvent) => void
): Promise<UnlistenFn> {
  return listen<GenerationStreamEvent>(GENERATION_STREAM_EVENT, (event) => {
    handler(event.payload);
  });
}
