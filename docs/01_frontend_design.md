# 步语 BuYu — 前端设计

**版本：** 0.2
**阶段：** 当前实现
**最后更新：** 2026-04-08
**技术栈：** Svelte 5 (runes) + TypeScript + Tailwind CSS 4 + bits-ui + lucide-svelte

本文只描述仓库当前代码已经落地的前端结构。

## 1. 当前页面骨架

前端仍然是**单页应用**，但主界面已经演进为“工作台壳 + 多 section”结构：

```text
WorkspaceShell
├─ 桌面端
│  ├─ IconRail                左侧图标栏，切换 chat / agents / settings
│  ├─ Context Sidebar         随 section 切换的上下文侧边栏
│  │  ├─ chat      -> ConversationSidebarPanel
│  │  ├─ agents    -> AgentSidebarPanel
│  │  └─ settings  -> SettingsChannelSidebar
│  └─ Main Content
│     ├─ chat      -> ChatPanel
│     ├─ agents    -> AgentSettingsPanel
│     └─ settings  -> SettingsPage
└─ 移动端
   ├─ 顶部菜单按钮
   ├─ Drawer 抽屉承载上下文侧边栏
   └─ 主内容区沿用桌面端的三个 section
```

当前布局是：

- 左侧先选工作区类型
- 中间侧边栏展示当前工作区的上下文内容
- 右侧主区展示聊天、Agent 编辑或设置页

## 2. 当前目录组织

前端组件已经不再全部平铺在一个目录里，而是按功能拆分：

```text
src/components/
├─ app-shell/        工作台外壳、窗口控件、工作台总状态
├─ chat/             聊天主区、消息列表、输入区、聊天状态
├─ conversations/    会话侧边栏
├─ agents/           Agent 侧边栏与 Agent 编辑
├─ settings/         设置页、渠道编辑、模型管理、设置状态
├─ chat-ui/          复用型聊天 UI 小组件
└─ legacy/           仍保留在仓库中的旧组件与旧状态实现
```

当前主要工作流已经使用 `app-shell / chat / conversations / agents / settings` 这五组目录。
`legacy/` 目录仅表示“文件还在仓库里”，不代表它仍是当前主界面入口。

## 3. 真实入口文件

### 3.1 启动入口

- `src/main.ts`
  - 挂载 `App.svelte`
  - 初始化主题模块
  - 管理启动 splash 的隐藏时机
- `src/App.svelte`
  - 只负责挂载 `WorkspaceShell.svelte`

### 3.2 工作台入口

- `src/components/app-shell/WorkspaceShell.svelte`
  - 整个前端的主布局
  - 负责桌面端 / 移动端分支渲染
  - 负责 section 切换、抽屉开关、窗口拖拽区等

## 4. 当前组件树

```text
App.svelte
└─ WorkspaceShell.svelte
   ├─ app-shell/IconRail.svelte
   ├─ conversations/ConversationSidebarPanel.svelte
   │  └─ conversations/ConversationSidebarItem.svelte
   ├─ agents/AgentSidebarPanel.svelte
   │  └─ agents/AgentSidebarItem.svelte
   ├─ settings/SettingsChannelSidebar.svelte
   ├─ chat/ChatPanel.svelte
   │  ├─ chat/ChatHeader.svelte
   │  ├─ chat/MessageTimeline.svelte
   │  │  └─ chat/MessageCard.svelte
   │  └─ chat/ChatComposer.svelte
   ├─ agents/AgentSettingsPanel.svelte
   └─ settings/SettingsPage.svelte
      ├─ settings/SettingsUtilityPanel.svelte
      ├─ settings/SettingsChannelEditor.svelte
      ├─ settings/SettingsModelManager.svelte
      └─ settings/SettingsNoticeBanner.svelte
```

补充说明：

- `ConversationSidebarPanel.svelte`
  - 当前实现里带有搜索框
  - 会话按 Agent 分组显示
- `ChatPanel.svelte`
  - 由 `ChatHeader + MessageTimeline + ChatComposer` 组成
- `SettingsPage.svelte`
  - 主内容区只负责“实用工具面板 + 渠道编辑 + 模型管理”
  - 设置页左侧的渠道列表由 `WorkspaceShell` 统一渲染

## 5. 当前状态管理

### 5.1 页面级状态工厂

当前以前端状态工厂为核心：

| 文件 | 作用 |
|------|------|
| `src/components/app-shell/workspace-shell.svelte.ts` | 聊天工作台总控：启动、会话切换、消息流、Agent 编辑、会话快速绑定 |
| `src/components/settings/settings-page-state.svelte.ts` | 设置页总控：渠道 CRUD、模型 CRUD、远程模型拉取、配置导入导出 |

### 5.2 `workspace-shell.svelte.ts` 负责什么

- 首屏 bootstrap
- 加载 channels / agents / conversations / messages / models
- 发送消息、reroll、编辑消息、取消生成
- 会话标题、Agent、渠道、模型的快速切换
- 消息版本切换和按需加载
- 流式事件接收和本地即时合并
- 设置页变更后的工作台刷新

### 5.3 `settings-page-state.svelte.ts` 负责什么

- 渠道列表加载与筛选
- 渠道创建、更新、删除、连通性测试
- thinking tags 输入与序列化
- 模型创建、删除、远程拉取、导入
- 导出配置、导入配置
- 打开数据目录、日志目录

## 6. 前端与后端的通信层

当前所有 Tauri 调用集中在 `src/lib/transport/`：

```text
transport/
├─ agents.ts
├─ channels.ts
├─ conversations.ts
├─ messages.ts
├─ models.ts
└─ common.ts
```

它们的职责是：

1. 调用 `invoke()`
2. 处理 snake_case / camelCase 转换
3. 把某些原始字段转换成更好用的前端结构

当前已经存在的转换例子：

- `enabled_tools: string | null` -> `enabledTools: string[]`
- `message` 相关事件 -> 前端联合类型 `GenerationEvent`
- `send_message` / `dry_run` 原始返回 -> 前端区分为 `kind: "started"` 或 `kind: "dryRun"`

## 7. 当前聊天区交互

### 7.1 ChatHeader

`ChatHeader.svelte` 当前支持：

- 会话标题快速编辑
- 当前 Agent / 渠道 / 模型状态展示
- 快速切换 Agent
- 快速切换渠道
- 快速切换模型
- 移动端菜单按钮

### 7.2 MessageTimeline

`MessageTimeline.svelte` 当前支持：

- 消息列表渲染
- 继续加载更旧消息
- dry run 摘要显示
- 顶部通知展示
- 版本切换
- 版本正文按需加载
- 编辑消息 / 重新发送
- 删除版本
- 取消生成
- reroll

### 7.3 MessageCard

单条消息当前支持：

- user / assistant 两种视觉样式
- Markdown 富文本渲染
- thinking 内容展示
- 图片 / 文件 / 工具调用结果展示
- generating / failed / cancelled 状态显示
- 版本切换器

### 7.4 ChatComposer

输入区当前支持：

- 文本输入
- 图片附件
- 文件附件
- `dry run`
- 发送中显示取消按钮
- 会话级启用工具列表切换

## 8. 当前设置页

设置页不再是一个纯“全局设置抽屉”，而是完整工作区的一部分。

### 8.1 侧边栏

`SettingsChannelSidebar.svelte` 当前负责：

- 渠道搜索
- 渠道列表
- 新建渠道入口
- 当前选中状态展示

### 8.2 主内容区

`SettingsPage.svelte` 当前包含三块：

- `SettingsUtilityPanel`
  - 导出配置
  - 导入配置
  - 打开数据目录
  - 打开日志目录
- `SettingsChannelEditor`
  - 渠道基础信息编辑
  - API Key / auth / endpoint / thinking tags
  - 渠道启用开关
  - 渠道连通性测试
- `SettingsModelManager`
  - 本地模型列表
  - 新建模型
  - 删除模型
  - 从远程拉取候选模型
  - 导入远程模型

## 9. 当前样式系统

### 9.1 主题

`src/app.css` 已经定义了两套主题：

- 浅色主题：Notebook 风格纸张色背景
- 深色主题：暗色纸张风格背景

当前代码里已经有：

- `.dark` 变量覆盖
- safe area 适配
- 抽屉动画
- 消息入场动画
- thinking 脉冲动画
- resize 期间禁用动画

### 9.2 布局变量

当前关键 CSS 变量：

- `--workspace-rail-width`
- `--workspace-sidebar-width`
- `--settings-sidebar-width`
- `--settings-content-max-width`

### 9.3 字体与视觉方向

当前主题使用：

- 标题字体：`Playfair Display` / `Noto Serif SC`
- 正文字体：`Inter` / `Noto Sans SC`
- 整体视觉：纸张感、笔记本感、暖色强调色

## 10. 当前响应式策略

### 10.1 桌面端

- 显示左侧 `IconRail`
- 显示上下文侧边栏
- 主内容区占剩余空间

### 10.2 移动端

通过 `src/lib/hooks/is-mobile.svelte.ts` 判断后：

- 隐藏左侧固定栏
- 使用 Drawer 承载上下文侧边栏
- 在非聊天页显示统一移动端顶栏
- 使用 safe-area padding

所以当前代码已经**不是“不支持移动端”**，而是有基础移动适配。

## 11. 当前前端边界

以下内容没有在代码里形成稳定事实，不应继续写入设计文档：

- 独立路由系统
- 虚拟滚动消息列表
- 单独的 Settings 侧滑 overlay
- 固定宽度可拖拽 Sidebar
- 仅浅色模式

这些都不是当前仓库的真实实现。
