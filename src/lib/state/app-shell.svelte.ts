import { workspaceSidebarItems } from "$lib/fixtures/workspaces";

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
  meta?: string;
  updatedAt?: number | null;
  busyCount?: number;
  unreadCount?: number;
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
  inspectorVisible = $state(true);
  mobileSidebarOpen = $state(false);
  mobileInspectorOpen = $state(false);
  desktopWide = $state(typeof window === "undefined" ? true : window.innerWidth >= 1280);

  constructor() {
    if (typeof window !== "undefined") {
      const syncDesktopWide = () => {
        this.desktopWide = window.innerWidth >= 1280;
      };

      syncDesktopWide();
      window.addEventListener("resize", syncDesktopWide);
    }
  }

  private useDesktopInspector() {
    return this.desktopWide;
  }

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

  closeInspector() {
    if (this.useDesktopInspector()) {
      this.inspectorVisible = false;
      return;
    }

    this.mobileInspectorOpen = false;
  }

  openInspector() {
    if (this.useDesktopInspector()) {
      this.inspectorVisible = true;
      return;
    }

    this.mobileInspectorOpen = true;
    this.mobileSidebarOpen = false;
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

  toggleInspector() {
    if (this.useDesktopInspector()) {
      this.inspectorVisible = !this.inspectorVisible;
      return;
    }

    this.toggleMobileInspector();
  }
}

export const appShell = new AppShellState();
