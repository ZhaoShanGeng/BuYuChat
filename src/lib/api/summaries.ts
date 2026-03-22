import { tauriInvoke } from "$lib/api/client";

export type SummaryLog = {
  id: string;
  conversation_id: string;
  trigger_reason: string;
  range_start_index: number;
  range_end_index: number;
  original_token_count: number;
  summary_token_count: number;
  summary_content: string;
  created_at: number;
};

export function listSummaries(conversationId: string) {
  return tauriInvoke<SummaryLog[]>("list_summaries", { conversationId });
}

export function generateManualSummary(conversationId: string, instruction?: string) {
  return tauriInvoke<SummaryLog>("generate_manual_summary", { conversationId, instruction });
}
