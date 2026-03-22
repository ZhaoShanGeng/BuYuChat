export type Locale = "zh-CN" | "en";

const translations: Record<Locale, Record<string, string>> = {
  "zh-CN": {
    // Nav
    "nav.chat": "对话",
    "nav.agents": "智能体",
    "nav.presets": "预设",
    "nav.lorebooks": "世界书",
    "nav.workflows": "工作流",
    "nav.settings": "设置",

    // Sidebar
    "sidebar.search": "搜索",
    "sidebar.new": "新建",
    "sidebar.rename": "重命名",
    "sidebar.delete": "删除",
    "sidebar.no_match": "无匹配结果",
    "sidebar.empty": "暂无内容",

    // Chat
    "chat.new_conversation": "新对话",
    "chat.user_label": "你",
    "chat.start_hint": "与 AI 助手开始对话，你可以问任何问题",
    "chat.no_agent_title": "没有可用智能体",
    "chat.no_agent_desc": "请先创建并启用至少一个智能体，然后再开始会话。",
    "chat.no_participant_desc": "当前会话缺少可用的用户或智能体参与者。",
    "chat.no_channel_desc": "请先在设置中启用至少一个 API 渠道，并添加至少一个模型。",
    "chat.agent_section_title": "选择智能体开始",
    "chat.agent_section_desc": "先选择一个智能体，再创建会话并开始发送消息。",
    "chat.send": "发送",
    "chat.send_failed": "发送失败",
    "chat.regenerate_failed": "重新生成失败",
    "chat.edit_failed": "编辑失败",
    "chat.delete_failed": "删除失败",
    "chat.attach_failed": "部分附件添加失败",
    "chat.attach_unsupported": "部分附件无法读取，已跳过。",
    "chat.bindings_save_failed": "保存会话绑定失败",
    "chat.version_switch_failed": "切换版本失败",
    "chat.create_conversation_failed": "无法创建新会话",
    "chat.generic_error": "操作失败，请稍后重试",
    "chat.sending": "发送中…",
    "chat.generating": "生成中…",
    "chat.failed": "生成失败",
    "chat.input_placeholder": "输入消息… (Shift+Enter 换行)",
    "chat.attachment": "添加附件",
    "chat.attachment_empty": "这条消息只有附件",
    "chat.attachment_remove": "移除附件",
    "chat.attachment_missing_path": "当前环境无法读取该文件路径，请改用可访问的本地文件。",
    "chat.edit": "编辑",
    "chat.copy": "复制",
    "chat.regenerate": "重新生成",
    "chat.delete": "删除",
    "chat.save": "保存",
    "chat.cancel": "取消",
    "chat.loading": "加载中…",
    "chat.assistant": "助手",
    "chat.system": "系统",
    "chat.user_avatar": "你",
    "chat.clear": "清空对话",
    "chat.export": "导出对话",
    "chat.edit_title": "双击编辑标题",
    "chat.prev_version": "上一个版本",
    "chat.next_version": "下一个版本",
    "chat.recipient_title": "回复目标",
    "chat.recipient_desc": "在同一轮中选择要同时回复的智能体。主智能体会作为默认单选目标被记忆。",
    "chat.select_responder_desc": "至少选择一个智能体参与回复。",
    "chat.primary_agent": "主智能体",
    "chat.recipient_selector": "选择回复智能体",
    "chat.in_progress": "生成中",
    "chat.dismiss_failed": "清除失败提示",

    // Suggestions
    "suggest.chat": "开始一段对话",
    "suggest.chat_desc": "与 AI 智能体自由交流",
    "suggest.brainstorm": "头脑风暴",
    "suggest.brainstorm_desc": "探索创意想法",
    "suggest.write": "创意写作",
    "suggest.write_desc": "生成故事或文案",

    // Time
    "time.just_now": "刚刚",
    "time.minutes_ago": "{n}分钟前",
    "time.hours_ago": "{n}小时前",
    "time.days_ago": "{n}天前",
    "time.yesterday": "昨天",

    // Inspector
    "inspector.title": "检查器",
    "inspector.tab.context": "Context",
    "inspector.tab.versions": "Versions",
    "inspector.tab.summaries": "Summaries",
    "inspector.tab.variables": "Variables",
    "inspector.tab.bindings": "Bindings",
    "inspector.tab.workflow": "Workflow",
    "inspector.select_msg": "选择一条消息以查看详细信息",
    "inspector.context_desc": "查看当前会话的完整上下文构建结果，包括系统提示、角色设定和对话历史。",
    "inspector.versions_desc": "浏览和切换选中消息的所有版本，对比不同生成结果。",
    "inspector.summaries_desc": "查看自动生成的对话摘要，了解 AI 如何理解对话进程。",
    "inspector.variables_desc": "监控和编辑对话中的变量状态，调试变量驱动的行为。",
    "inspector.bindings_desc": "管理当前会话绑定的预设、世界书和用户配置文件。",
    "inspector.workflow_desc": "查看工作流执行状态、节点图和运行轨迹。",

    // Workspace placeholders
    "ws.agents.title": "角色卡与绑定管理",
    "ws.agents.desc": "管理 AI 智能体的角色设定、性格描述、问候语和资源绑定。",
    "ws.presets.title": "提示词编排",
    "ws.presets.desc": "管理有序的提示词条目，控制角色、位置、深度和启用状态。",
    "ws.lorebooks.title": "知识规则管理",
    "ws.lorebooks.desc": "管理世界书条目、匹配关键词和插入策略。",
    "ws.workflows.title": "执行图设计",
    "ws.workflows.desc": "设计和管理工作流节点图，查看执行轨迹和结果。",
    "ws.settings.title": "全局配置",
    "ws.settings.desc": "管理 API 渠道、插件和外观设置。",

    // Theme
    "theme.light": "浅色模式",
    "theme.dark": "深色模式",
    "theme.system": "跟随系统",
    "settings.section.channels": "API 渠道",
    "settings.section.appearance": "外观",
    "settings.section.general": "通用",
    "settings.channels.title": "API 渠道管理",
    "settings.channels.desc": "配置和管理 AI 服务的 API 连接",
    "settings.channels.add": "添加渠道",
    "settings.channels.add_model": "添加模型",
    "settings.channels.edit": "编辑渠道",
    "settings.channels.delete": "删除渠道",
    "settings.channels.disabled": "已禁用",
    "settings.channels.unset": "未设置",
    "settings.appearance.theme_title": "主题",
    "settings.appearance.theme_desc": "选择你喜欢的颜色主题",
    "settings.appearance.language_title": "语言",
    "settings.appearance.language_desc": "切换界面显示语言",
    "settings.general.database": "数据与存储",
    "settings.general.notifications": "通知",
    "settings.general.security": "安全",
    "settings.general.integrations": "集成",
    "agents.create": "新建智能体",
    "agents.search": "搜索智能体…",
    "agents.create_card": "创建新智能体",
    "presets.create": "新建预设",
    "presets.search": "搜索预设…",
    "lorebooks.create": "新建世界书",
    "lorebooks.search": "搜索世界书…",
    "workflows.create": "新建工作流",
  },

  en: {
    // Nav
    "nav.chat": "Chat",
    "nav.agents": "Agents",
    "nav.presets": "Presets",
    "nav.lorebooks": "Lorebooks",
    "nav.workflows": "Workflows",
    "nav.settings": "Settings",

    // Sidebar
    "sidebar.search": "Search",
    "sidebar.new": "New",
    "sidebar.rename": "Rename",
    "sidebar.delete": "Delete",
    "sidebar.no_match": "No matches",
    "sidebar.empty": "Nothing here",

    // Chat
    "chat.new_conversation": "New Chat",
    "chat.user_label": "You",
    "chat.start_hint": "Start a conversation with the AI assistant — ask anything",
    "chat.no_agent_title": "No available agents",
    "chat.no_agent_desc": "Create and enable at least one agent before starting a conversation.",
    "chat.no_participant_desc": "The current conversation is missing a usable human or agent participant.",
    "chat.no_channel_desc": "Enable at least one API channel in Settings and add at least one model first.",
    "chat.agent_section_title": "Start with an agent",
    "chat.agent_section_desc": "Choose an agent first, then create the conversation and send your first message.",
    "chat.send": "Send",
    "chat.send_failed": "Send failed",
    "chat.regenerate_failed": "Regenerate failed",
    "chat.edit_failed": "Edit failed",
    "chat.delete_failed": "Delete failed",
    "chat.attach_failed": "Some attachments could not be added",
    "chat.attach_unsupported": "Some attachments could not be read and were skipped.",
    "chat.bindings_save_failed": "Failed to save conversation bindings",
    "chat.version_switch_failed": "Version switch failed",
    "chat.create_conversation_failed": "Unable to create a conversation",
    "chat.generic_error": "Operation failed. Please try again.",
    "chat.sending": "Sending…",
    "chat.generating": "Generating…",
    "chat.failed": "Generation failed",
    "chat.input_placeholder": "Type a message… (Shift+Enter for new line)",
    "chat.attachment": "Attach file",
    "chat.attachment_empty": "This message only contains attachments",
    "chat.attachment_remove": "Remove attachment",
    "chat.attachment_missing_path": "This environment could not access the local file path for that attachment.",
    "chat.edit": "Edit",
    "chat.copy": "Copy",
    "chat.regenerate": "Regenerate",
    "chat.delete": "Delete",
    "chat.save": "Save",
    "chat.cancel": "Cancel",
    "chat.loading": "Loading…",
    "chat.assistant": "Assistant",
    "chat.system": "System",
    "chat.user_avatar": "U",
    "chat.clear": "Clear chat",
    "chat.export": "Export chat",
    "chat.edit_title": "Double-click to edit title",
    "chat.prev_version": "Previous version",
    "chat.next_version": "Next version",
    "chat.recipient_title": "Responders",
    "chat.recipient_desc": "Choose which agents should answer this turn in parallel. The primary agent stays as the default single target.",
    "chat.select_responder_desc": "Select at least one agent to reply.",
    "chat.primary_agent": "Primary agent",
    "chat.recipient_selector": "Choose responders",
    "chat.in_progress": "In progress",
    "chat.dismiss_failed": "Dismiss failures",

    // Suggestions
    "suggest.chat": "Start a chat",
    "suggest.chat_desc": "Free conversation with AI",
    "suggest.brainstorm": "Brainstorm",
    "suggest.brainstorm_desc": "Explore creative ideas",
    "suggest.write": "Creative writing",
    "suggest.write_desc": "Generate stories or copy",

    // Time
    "time.just_now": "Just now",
    "time.minutes_ago": "{n}m ago",
    "time.hours_ago": "{n}h ago",
    "time.days_ago": "{n}d ago",
    "time.yesterday": "Yesterday",

    // Inspector
    "inspector.title": "Inspector",
    "inspector.tab.context": "Context",
    "inspector.tab.versions": "Versions",
    "inspector.tab.summaries": "Summaries",
    "inspector.tab.variables": "Variables",
    "inspector.tab.bindings": "Bindings",
    "inspector.tab.workflow": "Workflow",
    "inspector.select_msg": "Select a message to view details",
    "inspector.context_desc": "View the full context build for the current session, including system prompts, character settings, and chat history.",
    "inspector.versions_desc": "Browse and switch between all versions of a selected message.",
    "inspector.summaries_desc": "View auto-generated conversation summaries.",
    "inspector.variables_desc": "Monitor and edit variable states in the conversation.",
    "inspector.bindings_desc": "Manage presets, lorebooks, and user profiles bound to this session.",
    "inspector.workflow_desc": "View workflow execution status, node graph, and run traces.",

    // Workspace placeholders
    "ws.agents.title": "Character Cards & Bindings",
    "ws.agents.desc": "Manage AI agent personas, personality descriptions, greetings, and resource bindings.",
    "ws.presets.title": "Prompt Orchestration",
    "ws.presets.desc": "Manage ordered prompt entries — control role, position, depth, and enable state.",
    "ws.lorebooks.title": "Knowledge Rules",
    "ws.lorebooks.desc": "Manage lorebook entries, matching keywords, and insertion strategies.",
    "ws.workflows.title": "Execution Graph Design",
    "ws.workflows.desc": "Design and manage workflow node graphs, view execution traces and results.",
    "ws.settings.title": "Global Configuration",
    "ws.settings.desc": "Manage API channels, plugins, and appearance settings.",

    // Theme
    "theme.light": "Light mode",
    "theme.dark": "Dark mode",
    "theme.system": "Follow system",
    "settings.section.channels": "API Channels",
    "settings.section.appearance": "Appearance",
    "settings.section.general": "General",
    "settings.channels.title": "API Channel Management",
    "settings.channels.desc": "Configure and manage API connections for AI providers",
    "settings.channels.add": "Add channel",
    "settings.channels.add_model": "Add model",
    "settings.channels.edit": "Edit channel",
    "settings.channels.delete": "Delete channel",
    "settings.channels.disabled": "Disabled",
    "settings.channels.unset": "Unset",
    "settings.appearance.theme_title": "Theme",
    "settings.appearance.theme_desc": "Choose the interface theme you prefer",
    "settings.appearance.language_title": "Language",
    "settings.appearance.language_desc": "Switch the interface language",
    "settings.general.database": "Data & Storage",
    "settings.general.notifications": "Notifications",
    "settings.general.security": "Security",
    "settings.general.integrations": "Integrations",
    "agents.create": "New agent",
    "agents.search": "Search agents…",
    "agents.create_card": "Create new agent",
    "presets.create": "New preset",
    "presets.search": "Search presets…",
    "lorebooks.create": "New lorebook",
    "lorebooks.search": "Search lorebooks…",
    "workflows.create": "New workflow",
  },
};

class I18nState {
  locale = $state<Locale>("zh-CN");

  constructor() {
    if (typeof window !== "undefined") {
      const stored = localStorage.getItem("buyu-locale") as Locale | null;
      if (stored === "zh-CN" || stored === "en") {
        this.locale = stored;
      }
    }
  }

  t(key: string, params?: Record<string, string | number>): string {
    let text = translations[this.locale][key] ?? key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        text = text.replace(`{${k}}`, String(v));
      }
    }
    return text;
  }

  setLocale(locale: Locale) {
    this.locale = locale;
    if (typeof window !== "undefined") {
      localStorage.setItem("buyu-locale", locale);
    }
  }

  toggleLocale() {
    this.locale = this.locale === "zh-CN" ? "en" : "zh-CN";
    if (typeof window !== "undefined") {
      localStorage.setItem("buyu-locale", this.locale);
    }
  }
}

export const i18n = new I18nState();
