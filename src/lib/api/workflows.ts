import { tauriInvoke } from "$lib/api/client";

export type WorkflowDefSummary = {
  id: string;
  name: string;
  description: string | null;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type WorkflowNodeDetail = {
  id: string;
  workflow_def_id: string;
  node_key: string;
  name: string | null;
  node_type: string;
  agent_id: string | null;
  plugin_id: string | null;
  preset_id: string | null;
  lorebook_id: string | null;
  user_profile_id: string | null;
  api_channel_id: string | null;
  api_channel_model_id: string | null;
  workspace_mode: string;
  history_read_mode: string;
  summary_write_mode: string;
  message_write_mode: string;
  visible_output_mode: string;
  config_json: Record<string, unknown>;
};

export type WorkflowEdgeDetail = {
  id: string;
  workflow_def_id: string;
  from_node_id: string;
  to_node_id: string;
  edge_type: string;
  priority: number;
  condition_expr: string | null;
  label: string | null;
  enabled: boolean;
  config_json: Record<string, unknown>;
};

export type WorkflowDefDetail = {
  summary: WorkflowDefSummary;
  nodes: WorkflowNodeDetail[];
  edges: WorkflowEdgeDetail[];
};

export function listWorkflowDefs() {
  return tauriInvoke<WorkflowDefSummary[]>("list_workflow_defs");
}

export function getWorkflowDefDetail(id: string) {
  return tauriInvoke<WorkflowDefDetail>("get_workflow_def_detail", { id });
}
