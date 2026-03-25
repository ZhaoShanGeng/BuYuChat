# 步语 BuYu — 前端设计

**版本：** 0.1
**阶段：** MVP（P0）
**技术栈：** Svelte 5 (runes) + TypeScript + Tailwind CSS 4 + bits-ui + lucide-svelte

---

## 1. 页面布局

MVP 为**单页应用**（无路由），整体采用经典 IM 布局：

```
┌──────────────────────────────────────────────────────────┐
│  Titlebar (Tauri custom titlebar, 可选)                   │
├────────────┬─────────────────────────────────────────────┤
│            │  ChatHeader                                  │
│            │  ┌─────────────────────────────────────────┐│
│  Sidebar   │  │  会话标题 | Agent/模型标签 | ⚙️ 设置     ││
│            │  └─────────────────────────────────────────┘│
│  ┌──────┐  │  ChatMessages (虚拟滚动)                     │
│  │新建   │  │  ┌─────────────────────────────────────────┐│
│  │会话   │  │  │  UserBubble                             ││
│  ├──────┤  │  │  AssistantBubble (流式渲染)               ││
│  │📌置顶 │  │  │    └─ VersionSwitcher < [1] 2 3 >       ││
│  │会话A  │  │  │  UserBubble                             ││
│  │会话B  │  │  │  AssistantBubble (generating...)         ││
│  │会话C  │  │  │    └─ 取消按钮 / Reroll 按钮             ││
│  │ ...   │  │  └─────────────────────────────────────────┘│
│  ├──────┤  │  ChatInput                                   │
│  │归档 ▸ │  │  ┌─────────────────────────────────────────┐│
│  │设置 ▸ │  │  │  [输入框 (textarea)]        [发送按钮]   ││
│  └──────┘  │  └─────────────────────────────────────────┘│
├────────────┴─────────────────────────────────────────────┤
│  StatusBar (可选: token 统计, 连接状态)                     │
└──────────────────────────────────────────────────────────┘
```

### 尺寸约定

| 区域 | 宽度 | 最小宽度 |
|------|------|---------|
| Sidebar | 280px（可拖拽调整） | 200px |
| ChatArea | 剩余空间 | 480px |
| 整体窗口 | — | 960×640 (tauri.conf.json) |

---

## 2. 组件树

```
App.svelte
├── Sidebar.svelte
│   ├── SidebarHeader.svelte          # 新建会话按钮、搜索框(P1)
│   ├── ConversationList.svelte       # 会话列表（虚拟滚动）
│   │   └── ConversationItem.svelte   # 单条会话（标题、右键菜单）
│   └── SidebarFooter.svelte          # 归档入口、设置入口
│
├── ChatArea.svelte
│   ├── ChatHeader.svelte             # 会话标题、Agent/模型标签、设置按钮
│   ├── ChatMessages.svelte           # 消息列表容器（虚拟滚动）
│   │   └── MessageBubble.svelte      # 单条消息楼层
│   │       ├── BubbleContent.svelte  # Markdown 渲染 + 流式光标
│   │       ├── BubbleActions.svelte  # Reroll、复制、删除（hover 显示）
│   │       └── VersionSwitcher.svelte # < [1] 2 3 > （hover 显示）
│   ├── ChatInput.svelte              # 输入框 + 发送按钮
│   └── EmptyState.svelte             # 空会话引导（未配置 Agent 时）
│
├── SettingsPanel.svelte              # 侧边滑出面板
│   ├── ChannelSettings.svelte        # 渠道 CRUD
│   ├── ModelSettings.svelte          # 模型管理
│   ├── AgentSettings.svelte          # Agent CRUD
│   └── ConversationSettings.svelte   # 当前会话配置（绑定 Agent/渠道/模型）
│
└── shared/                           # 通用组件
    ├── Dialog.svelte                 # 确认对话框（bits-ui Dialog）
    ├── Toast.svelte                  # 通知提示
    ├── Dropdown.svelte               # 下拉菜单（bits-ui）
    └── Spinner.svelte                # 加载指示器
```

---

## 3. 路由规划

### MVP：无路由

单页，所有内容在 `App.svelte` 中通过条件渲染切换：
- 主视图：Sidebar + ChatArea
- 设置面板：侧边滑出 overlay（不替换主视图）

### P1：引入路由

```
/                       → 重定向到最近会话或空状态
/chat/:conversationId   → 聊天视图
/settings               → 全局设置
/settings/channels      → 渠道管理
/settings/agents        → Agent 管理
/archived               → 归档会话列表
```

P1 路由工具：`svelte-spa-router` 或 Svelte 5 原生 `{#snippet}` 条件路由。

---

## 4. 状态管理

### 4.1 Store 结构

所有 store 使用 Svelte 5 runes（`.svelte.ts` 文件），不使用 Svelte 4 stores。

```typescript
// stores/app.svelte.ts — 全局应用状态
let sidebarWidth = $state(280);
let settingsPanelOpen = $state(false);
let settingsTab = $state<"channels" | "agents" | "conversation">("channels");

// stores/conversations.svelte.ts — 会话状态
let conversations = $state<Map<string, Conversation>>(new Map());
let activeConversationId = $state<string | null>(null);
let activeConversation = $derived(
  activeConversationId ? conversations.get(activeConversationId) : null
);

// stores/messages.svelte.ts — 消息状态
let messagesByConversation = $state<Map<string, MessageNode[]>>(new Map());
let generatingVersions = $state<Set<string>>(new Set());
let pendingInput = $state<Map<string, string>>(new Map()); // 失败恢复用

// stores/settings.svelte.ts — 配置数据
let channels = $state<Channel[]>([]);
let agents = $state<Agent[]>([]);
```

### 4.2 Store 依赖关系

```
app.svelte.ts（UI 状态，无依赖）
     │
conversations.svelte.ts ──► messages.svelte.ts
     │                         │
     └── settings.svelte.ts ◄──┘  （channels/agents 供选择器使用）
```

### 4.3 缓存策略

| 数据 | 缓存方式 | 失效时机 |
|------|---------|---------|
| 会话列表 | 启动时全量加载，内存保持 | CRUD 操作后局部更新 |
| 消息列表 | 按会话懒加载，切换会话时加载 | 不主动清空（LRU 淘汰 P1） |
| 版本 content | active 版本随消息列表加载 | 切换版本时按需加载 |
| channels/agents | 启动时全量加载 | CRUD 后刷新 |
| generating 状态 | 纯内存，不持久化 | 启动时清空 |

### 4.4 不持久化到 localStorage

MVP 不将任何 store 数据写入 localStorage。所有数据来源为后端 SQLite，前端是纯缓存。

---

## 5. 主题与样式系统

### 5.1 Tailwind CSS 4 配置

CSS-first 配置（无 `tailwind.config.js`），通过 `app.css` 中 `@theme` 定义：

```css
@import "tailwindcss";

@theme {
  /* 主色 */
  --color-primary: #0f172a;        /* slate-900 */
  --color-primary-hover: #1e293b;  /* slate-800 */
  --color-accent: #0ea5e9;         /* sky-500 */
  --color-accent-light: #e0f2fe;   /* sky-100 */

  /* 背景 */
  --color-bg-main: #f8fafc;        /* slate-50 */
  --color-bg-sidebar: #ffffff;
  --color-bg-bubble-user: #f1f5f9; /* slate-100 */
  --color-bg-bubble-ai: #ffffff;

  /* 文字 */
  --color-text-primary: #0f172a;
  --color-text-secondary: #64748b; /* slate-500 */
  --color-text-muted: #94a3b8;     /* slate-400 */

  /* 边框 */
  --color-border: #e2e8f0;         /* slate-200 */
  --color-border-light: #f1f5f9;

  /* 圆角 */
  --radius-sm: 0.5rem;
  --radius-md: 1rem;
  --radius-lg: 1.5rem;
  --radius-xl: 2rem;

  /* 侧边栏 */
  --sidebar-width: 280px;
  --sidebar-min-width: 200px;
  --sidebar-max-width: 400px;
}
```

### 5.2 暗色模式（P1）

MVP 仅浅色模式。P1 通过 `@theme dark` 覆盖变量实现暗色模式：

```css
@theme dark {
  --color-bg-main: #0f172a;
  --color-text-primary: #f8fafc;
  /* ... */
}
```

### 5.3 组件样式原则

- 使用 Tailwind utility classes，不写自定义 CSS（特殊动画除外）
- bits-ui 组件只提供行为，样式完全由 Tailwind 控制
- 圆角统一使用 theme 变量（`rounded-lg` → `--radius-lg`）
- 不使用 `!important`

---

## 6. 核心交互规范

### 6.1 消息气泡

| 角色 | 对齐 | 背景色 | 头像 |
|------|------|--------|------|
| user | 右对齐 | `bg-bubble-user` | 无（MVP） |
| assistant | 左对齐 | `bg-bubble-ai` | Agent 头像 / 默认图标 |

### 6.2 流式渲染

- AI 回复 `status=generating` 时，content 逐步追加
- 末尾显示闪烁光标 `▊`（CSS animation）
- 使用 Markdown 渲染（P1 引入 `marked` 或 `mdsvex`）
- MVP 阶段纯文本渲染（保留换行）

### 6.3 版本切换器

```
┌─────────────────────────────────────┐
│  AI 回复内容...                      │
│                                      │
│  ◄  [1]  2   3  ►     🔄 ✂️ 📋      │
│     ^^^ active高亮   Reroll 取消 复制  │
└─────────────────────────────────────┘
```

- 仅 hover 楼层时显示（CSS `group-hover`）
- 当前 active 版本数字高亮（`bg-primary text-white rounded`）
- 左右箭头：相邻切换
- 数字：直接跳转
- 切换时：立即写库 + 如果目标版本 content 为 null 则按需加载

### 6.4 输入框

- `<textarea>` 自动高度（min 1 行，max 8 行）
- Enter 发送，Shift+Enter 换行
- 发送中（generating）：发送按钮变为停止按钮
- 发送失败：输入框恢复之前的内容

### 6.5 会话列表项

```
┌──────────────────────────┐
│ 📌 关于 Rust 的讨论       │  ← 置顶图标
│    3 分钟前               │
├──────────────────────────┤
│ 新会话                    │  ← 普通
│    1 小时前               │
└──────────────────────────┘
```

- 当前活跃会话高亮背景
- 右键菜单：重命名、置顶/取消置顶、归档、删除
- 双击标题：进入内联编辑模式

---

## 7. 错误展示规范

### 7.1 错误类型与展示方式

| 错误类型 | 展示方式 | 示例 |
|----------|---------|------|
| 配置缺失（前端拦截） | 内联提示 + 引导按钮 | "请先配置 Agent" [去配置] |
| 业务错误（422） | Toast 通知 | "渠道已禁用" |
| 发送失败 | Toast + 恢复输入 | "发送失败：未配置渠道" |
| 生成失败 | 楼层内标签 | 气泡底部红色 "生成失败" 标签 |
| 生成取消 | 楼层内标签 | 气泡底部灰色 "已取消" 标签 |
| 网络错误 | Toast 通知 | "网络连接失败" |
| 内部错误 | Toast 通知 | "系统错误，请重试" |

### 7.2 Toast 规范

- 位置：右上角
- 自动消失：3 秒（错误类 5 秒）
- 最多同时显示 3 条，超出队列
- 类型：`success`（绿）、`error`（红）、`warning`（黄）、`info`（蓝）

### 7.3 i18n 错误文案表

后端返回 `error_code`，前端查表翻译：

```typescript
const ERROR_MESSAGES: Record<string, string> = {
  // 配置相关
  NO_AGENT: "请先为会话配置 Agent",
  AGENT_DISABLED: "当前 Agent 已禁用，请启用或更换",
  NO_CHANNEL: "请先配置 AI 服务渠道",
  CHANNEL_DISABLED: "当前渠道已禁用，请启用或更换",
  NO_MODEL: "请先选择模型",

  // 验证相关
  VALIDATION_ERROR: "输入不合法，请检查",
  INVALID_URL: "请输入有效的 URL（以 http:// 或 https:// 开头）",
  NAME_EMPTY: "名称不能为空",
  CONTENT_EMPTY: "消息内容不能为空",

  // 冲突
  MODEL_ID_CONFLICT: "该渠道下已存在此模型 ID",
  NOT_LAST_USER_NODE: "只能对最后一条用户消息执行此操作",
  VERSION_NOT_IN_NODE: "版本不属于该消息",

  // 服务端
  NOT_FOUND: "资源不存在",
  CHANNEL_UNREACHABLE: "无法连接到 AI 服务，请检查渠道配置",
  AI_REQUEST_FAILED: "AI 服务返回错误，请稍后重试",
  INTERNAL_ERROR: "系统内部错误，请重试",
};
```

---

## 8. 响应式策略

MVP 窗口最小尺寸 960×640，不做移动端适配。

| 窗口宽度 | 行为 |
|----------|------|
| ≥ 1280px | Sidebar 280px + ChatArea 填充 |
| 960-1280px | Sidebar 可折叠（点击按钮收起为图标栏 60px） |
| < 960px | 不支持（tauri.conf.json 已设 minWidth=960） |

### 折叠 Sidebar

```
┌────┬───────────────────────────────┐
│ ☰  │  ChatArea                     │
│ +  │                               │
│ 💬 │                               │  ← 图标栏模式
│ 💬 │                               │
│ ⚙️ │                               │
└────┴───────────────────────────────┘
```

---

## 9. 动画与过渡

| 场景 | 效果 | 实现 |
|------|------|------|
| 设置面板开关 | 从右侧滑入/滑出 | Svelte `transition:slide` |
| Toast 出现/消失 | 淡入 + 上滑 / 淡出 | Svelte `transition:fly` |
| 消息出现 | 淡入 | Svelte `transition:fade` |
| 流式光标 | 闪烁 | CSS `@keyframes blink` |
| 版本切换 | 内容淡入 | CSS `transition: opacity 150ms` |
| Sidebar 折叠 | 宽度过渡 | CSS `transition: width 200ms ease` |

---

## 10. 无障碍（基础）

- 所有按钮有 `aria-label`
- 输入框有 `<label>` 关联
- 颜色对比度 ≥ 4.5:1（WCAG AA）
- 键盘导航：Tab 切换焦点、Enter 激活按钮
- bits-ui 组件自带 ARIA 属性
