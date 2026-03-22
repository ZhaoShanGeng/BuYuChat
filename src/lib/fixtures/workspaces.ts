import type { SidebarItem, WorkspaceId } from "$lib/state/app-shell.svelte";

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

export const mockAgents = [
  {
    id: "1",
    name: "小助手",
    description: "通用聊天助手，擅长回答各种问题",
    greetings: 2,
    media: 0,
    presets: 1,
    color: "from-blue-400 to-blue-600"
  },
  {
    id: "2",
    name: "代码专家",
    description: "专注于编程和技术问题的 AI 助手",
    greetings: 1,
    media: 0,
    presets: 2,
    color: "from-violet-400 to-violet-600"
  },
  {
    id: "3",
    name: "创意写手",
    description: "擅长创意写作、故事生成和文案创作",
    greetings: 3,
    media: 1,
    presets: 1,
    color: "from-emerald-400 to-emerald-600"
  }
] as const;

export const mockPresets = [
  {
    id: "1",
    name: "默认预设",
    description: "基础对话预设，包含标准系统提示",
    entries: 5,
    channels: 1
  },
  {
    id: "2",
    name: "角色扮演",
    description: "适用于角色扮演场景的提示词编排",
    entries: 8,
    channels: 2
  },
  {
    id: "3",
    name: "代码助手",
    description: "优化了代码生成和分析能力的预设",
    entries: 4,
    channels: 1
  }
] as const;

export const mockPresetEntries = [
  {
    id: "e1",
    role: "system",
    position: "before_chat",
    label: "系统提示",
    enabled: true,
    text: "你是一个有用的AI助手。"
  },
  {
    id: "e2",
    role: "system",
    position: "after_char",
    label: "角色增强",
    enabled: true,
    text: "请保持角色一致性。"
  },
  {
    id: "e3",
    role: "user",
    position: "depth_4",
    label: "上下文提醒",
    enabled: false,
    text: "[重要：请记住以上设定]"
  },
  {
    id: "e4",
    role: "system",
    position: "before_chat",
    label: "输出格式",
    enabled: true,
    text: "请使用 Markdown 格式回复。"
  },
  {
    id: "e5",
    role: "system",
    position: "after_chat",
    label: "安全提示",
    enabled: true,
    text: "请遵守内容安全规范。"
  }
] as const;

export const mockLorebooks = [
  {
    id: "1",
    name: "世界观设定",
    description: "包含世界的基础背景、规则和历史",
    entries: 12,
    enabled: true
  },
  {
    id: "2",
    name: "角色百科",
    description: "所有角色的详细信息和关系",
    entries: 25,
    enabled: true
  },
  {
    id: "3",
    name: "地理位置",
    description: "世界中的重要地点和场景描述",
    entries: 8,
    enabled: false
  }
] as const;

export const mockLorebookEntries = [
  {
    id: "e1",
    name: "魔法体系",
    keys: ["魔法", "法术", "元素"],
    enabled: true,
    text: "这个世界的魔法基于五种元素：火、水、风、土、雷。",
    position: "before_char",
    depth: 4
  },
  {
    id: "e2",
    name: "精灵族",
    keys: ["精灵", "精灵族", "艾达"],
    enabled: true,
    text: "精灵族是世界上最古老的种族之一，拥有超长的寿命和与自然的亲和力。",
    position: "after_char",
    depth: 4
  },
  {
    id: "e3",
    name: "黑暗森林",
    keys: ["黑暗森林", "禁地"],
    enabled: false,
    text: "黑暗森林是大陆中央被诅咒的区域，充满了危险的魔物。",
    position: "before_char",
    depth: 8
  },
  {
    id: "e4",
    name: "王国历史",
    keys: ["王国", "历史", "建国"],
    enabled: true,
    text: "莱恩王国由第一代国王亚瑟于 500 年前建立，经历了三次大战。",
    position: "before_char",
    depth: 4
  }
] as const;

export const mockWorkflows = [
  {
    id: "1",
    name: "多模型辩论",
    description: "让两个 AI 模型对同一问题进行辩论，合成最终回答",
    nodes: 5,
    edges: 4,
    lastRun: "2分钟前",
    status: "success" as const
  },
  {
    id: "2",
    name: "RAG 增强生成",
    description: "先检索相关文档，再基于上下文生成回答",
    nodes: 4,
    edges: 3,
    lastRun: "1小时前",
    status: "success" as const
  },
  {
    id: "3",
    name: "自动摘要链",
    description: "将长文本拆分后逐段摘要，最终合成摘要",
    nodes: 3,
    edges: 2,
    lastRun: "未执行",
    status: "idle" as const
  }
] as const;

export const mockWorkflowNodes = [
  { id: "n1", type: "input", label: "用户输入", x: 50, y: 100 },
  { id: "n2", type: "llm", label: "正方 (GPT-4o)", x: 250, y: 50 },
  { id: "n3", type: "llm", label: "反方 (Claude)", x: 250, y: 150 },
  { id: "n4", type: "merge", label: "观点合并", x: 450, y: 100 },
  { id: "n5", type: "output", label: "最终回答", x: 650, y: 100 }
] as const;

export const mockWorkflowRuns = [
  { id: "r1", status: "success" as const, duration: "12.3s", startedAt: "2分钟前", tokens: 2450 },
  { id: "r2", status: "failed" as const, duration: "3.1s", startedAt: "1小时前", tokens: 820 },
  { id: "r3", status: "success" as const, duration: "8.7s", startedAt: "3小时前", tokens: 1890 }
] as const;

export const mockChannels = [
  {
    id: "ch1",
    name: "OpenAI",
    provider: "openai",
    baseUrl: "https://api.openai.com/v1",
    apiKey: "sk-***...***abc",
    models: ["gpt-4o", "gpt-4o-mini", "o1-preview"],
    enabled: true
  },
  {
    id: "ch2",
    name: "Anthropic",
    provider: "anthropic",
    baseUrl: "https://api.anthropic.com",
    apiKey: "sk-ant-***...***xyz",
    models: ["claude-3.5-sonnet", "claude-3-haiku"],
    enabled: true
  },
  {
    id: "ch3",
    name: "本地 Ollama",
    provider: "ollama",
    baseUrl: "http://localhost:11434",
    apiKey: "",
    models: ["llama3", "qwen2.5"],
    enabled: false
  }
] as const;

export const mockSettingsSections = [
  {
    title: "数据管理",
    desc: "管理应用数据、导入导出和备份",
    icon: "database",
    items: ["导出所有数据", "导入数据", "清除缓存"]
  },
  {
    title: "通知",
    desc: "管理应用通知和提醒",
    icon: "bell",
    items: ["生成完成通知", "错误通知", "更新提醒"]
  },
  {
    title: "安全",
    desc: "安全和隐私设置",
    icon: "shield",
    items: ["API 密钥加密", "对话历史保留期", "匿名使用统计"]
  },
  {
    title: "插件",
    desc: "管理已安装的插件和扩展",
    icon: "plug",
    items: ["MCP 服务器", "RAG 检索引擎", "TTS 语音合成"]
  }
] as const;
