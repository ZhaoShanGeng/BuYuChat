import { tauriInvoke } from "$lib/api/client";

export type PluginSummary = {
  id: string;
  name: string;
  plugin_key: string;
  version: string | null;
  runtime: string;
  entry_point: string | null;
  enabled: boolean;
  capabilities: string[]; // keys, like "tool", "filter"
  config_json: Record<string, unknown>;
  created_at: number;
  updated_at: number;
};

export type PluginDetail = PluginSummary & {
  permissions: Record<string, unknown>;
};

export type CreatePluginInput = {
  name: string;
  plugin_key: string;
  version: string | null;
  runtime: string;
  entry_point: string | null;
  capabilities: string[];
  permissions: Record<string, unknown>;
  enabled: boolean;
  config_json: Record<string, unknown>;
};

export function listPlugins() {
  return tauriInvoke<PluginSummary[]>("list_plugins");
}

export function getPluginDetail(id: string) {
  return tauriInvoke<PluginDetail>("get_plugin", { id });
}

export function createPlugin(input: CreatePluginInput) {
  return tauriInvoke<PluginDetail>("create_plugin", { input });
}

export function updatePlugin(id: string, input: CreatePluginInput) {
  return tauriInvoke<PluginDetail>("update_plugin", { id, input });
}

export function deletePlugin(id: string) {
  return tauriInvoke<void>("delete_plugin", { id });
}
