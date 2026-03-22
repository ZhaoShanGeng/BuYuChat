import { tauriInvoke } from "$lib/api/client";

export type WorkflowSummary = {
  id: string;
  name: string;
  description: string | null;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type WorkflowNode = {
  id: string;
  type: string;
  position: { x: number; y: number };
  data: Record<string, unknown>;
};

export type WorkflowEdge = {
  id: string;
  source: string;
  target: string;
  source_handle?: string;
  target_handle?: string;
};

export type WorkflowDetail = {
  summary: WorkflowSummary;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
};

export type CreateWorkflowInput = {
  name: string;
  description: string | null;
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export type UpdateWorkflowInput = CreateWorkflowInput;

export type SaveWorkflowGraphInput = {
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
};

export function listWorkflows() {
  return tauriInvoke<WorkflowSummary[]>("list_workflows");
}

export function getWorkflowDetail(id: string) {
  return tauriInvoke<WorkflowDetail>("get_workflow_detail", { id });
}

export function createWorkflow(input: CreateWorkflowInput) {
  return tauriInvoke<WorkflowDetail>("create_workflow_def", { input });
}

export function updateWorkflow(id: string, input: UpdateWorkflowInput) {
  return tauriInvoke<WorkflowDetail>("update_workflow", { id, input });
}

export function deleteWorkflow(id: string) {
  return tauriInvoke<void>("delete_workflow", { id });
}

export function saveWorkflowGraph(id: string, input: SaveWorkflowGraphInput) {
  return tauriInvoke<WorkflowDetail>("save_workflow_graph", { id, input });
}
