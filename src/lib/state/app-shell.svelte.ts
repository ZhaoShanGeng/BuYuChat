export type WorkspaceId =
  | "chat"
  | "agents"
  | "presets"
  | "lorebooks"
  | "workflows"
  | "settings";

export type InspectorTabId =
  | "context"
  | "versions"
  | "summaries"
  | "variables"
  | "bindings"
  | "workflow";

export type SidebarItem = {
  id: string;
  title: string;
  meta: string;
};

export type NavItem = {
  id: WorkspaceId;
  label: string;
};

export const navItems: NavItem[] = [
  { id: "chat", label: "Chat" },
  { id: "agents", label: "Agents" },
  { id: "presets", label: "Presets" },
  { id: "lorebooks", label: "Lorebooks" },
  { id: "workflows", label: "Workflows" },
  { id: "settings", label: "Settings" }
];

export const workspaceSidebarItems: Record<WorkspaceId, SidebarItem[]> = {
  chat: [
    { id: "conversation-schema", title: "Schema Review", meta: "3 summaries · 12 nodes" },
    { id: "conversation-workflow", title: "Workflow Design", meta: "2 agents · running" },
    { id: "conversation-storage", title: "Media Storage", meta: "4 files · 1 image" }
  ],
  agents: [
    { id: "agent-orbit", title: "Orbit", meta: "Default workspace operator" },
    { id: "agent-lantern", title: "Lantern", meta: "Research and lore matching" },
    { id: "agent-signal", title: "Signal", meta: "Workflow routing specialist" }
  ],
  presets: [
    { id: "preset-story", title: "Story Board", meta: "12 prompt entries" },
    { id: "preset-ops", title: "Ops Review", meta: "8 prompt entries" },
    { id: "preset-terse", title: "Terse Debug", meta: "5 prompt entries" }
  ],
  lorebooks: [
    { id: "lore-product", title: "Product Lore", meta: "31 entries · 104 keys" },
    { id: "lore-world", title: "World Rules", meta: "18 entries · 42 keys" },
    { id: "lore-memory", title: "Memory Notes", meta: "9 entries · rolling" }
  ],
  workflows: [
    { id: "workflow-agent-loop", title: "Agent Loop", meta: "7 nodes · 9 edges" },
    { id: "workflow-brief", title: "Brief to Summary", meta: "5 nodes · deterministic" },
    { id: "workflow-moderate", title: "Moderation Pass", meta: "4 nodes · guarded" }
  ],
  settings: [
    { id: "settings-api", title: "API Channels", meta: "3 providers configured" },
    { id: "settings-plugins", title: "Plugins", meta: "5 active extensions" },
    { id: "settings-display", title: "Appearance", meta: "Light workspace theme" }
  ]
};

export const inspectorTabs: { id: InspectorTabId; label: string }[] = [
  { id: "context", label: "Context" },
  { id: "versions", label: "Versions" },
  { id: "summaries", label: "Summaries" },
  { id: "variables", label: "Variables" },
  { id: "bindings", label: "Bindings" },
  { id: "workflow", label: "Workflow" }
];

class AppShellState {
  activeWorkspace = $state<WorkspaceId>("chat");
  activeInspectorTab = $state<InspectorTabId>("context");
  activeSidebarItemId = $state<string>("conversation-schema");
  mobileSidebarOpen = $state(false);
  mobileInspectorOpen = $state(false);

  setWorkspace(id: WorkspaceId) {
    this.activeWorkspace = id;
    this.activeSidebarItemId = workspaceSidebarItems[id][0]?.id ?? "";
    this.mobileSidebarOpen = false;
    this.mobileInspectorOpen = false;
  }

  setInspectorTab(id: InspectorTabId) {
    this.activeInspectorTab = id;
  }

  setSidebarItem(id: string) {
    this.activeSidebarItemId = id;
    this.mobileSidebarOpen = false;
  }

  closeMobileSidebar() {
    this.mobileSidebarOpen = false;
  }

  closeMobileInspector() {
    this.mobileInspectorOpen = false;
  }

  toggleMobileSidebar() {
    this.mobileSidebarOpen = !this.mobileSidebarOpen;
    if (this.mobileSidebarOpen) {
      this.mobileInspectorOpen = false;
    }
  }

  toggleMobileInspector() {
    this.mobileInspectorOpen = !this.mobileInspectorOpen;
    if (this.mobileInspectorOpen) {
      this.mobileSidebarOpen = false;
    }
  }
}

export const appShell = new AppShellState();
