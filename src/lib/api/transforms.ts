import { tauriInvoke } from "$lib/api/client";

export type TransformPipelineSummary = {
  id: string;
  name: string;
  pipeline_key: string;
  pipeline_type: string; // "regex", "script"
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type TransformStep = {
  step_type: string;
  pattern: string;
  replacement: string;
  sort_order: number;
};

export type TransformPipelineDetail = {
  summary: TransformPipelineSummary;
  steps: TransformStep[];
};

export type CreateTransformPipelineInput = {
  name: string;
  pipeline_key: string;
  pipeline_type: string;
  steps: TransformStep[];
  enabled: boolean;
  sort_order: number;
  config_json: Record<string, unknown>;
};

export function listTransformPipelines() {
  return tauriInvoke<TransformPipelineSummary[]>("list_transform_pipelines");
}

export function getTransformPipelineDetail(id: string) {
  return tauriInvoke<TransformPipelineDetail>("get_transform_pipeline", { id });
}

export function createTransformPipeline(input: CreateTransformPipelineInput) {
  return tauriInvoke<TransformPipelineDetail>("create_transform_pipeline", { input });
}

export function updateTransformPipeline(id: string, input: CreateTransformPipelineInput) {
  return tauriInvoke<TransformPipelineDetail>("update_transform_pipeline", { id, input });
}

export function deleteTransformPipeline(id: string) {
  return tauriInvoke<void>("delete_transform_pipeline", { id });
}

export function testTransformPipeline(pipelineId: string, inputText: string) {
  return tauriInvoke<{ output_text: string }>("test_transform_pipeline", { pipelineId, inputText });
}
