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
    "chat.start_hint": "与 AI 助手开始对话，你可以问任何问题",
    "chat.send": "发送",
    "chat.stop": "停止",
    "chat.sending": "发送中…",
    "chat.generating": "生成中…",
    "chat.input_placeholder": "输入消息… (Shift+Enter 换行)",
    "chat.attachment": "添加附件",
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
    "chat.start_hint": "Start a conversation with the AI assistant — ask anything",
    "chat.send": "Send",
    "chat.stop": "Stop",
    "chat.sending": "Sending…",
    "chat.generating": "Generating…",
    "chat.input_placeholder": "Type a message… (Shift+Enter for new line)",
    "chat.attachment": "Attach file",
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
  },
};

class I18nState {
  locale = $state<Locale>("zh-CN");

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
  }

  toggleLocale() {
    this.locale = this.locale === "zh-CN" ? "en" : "zh-CN";
  }
}

export const i18n = new I18nState();
