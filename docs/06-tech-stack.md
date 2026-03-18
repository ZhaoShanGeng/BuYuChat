# 技术选型

## 核心后端依赖

| 用途 | 库 | 说明 |
|------|----|------|
| 桌面框架 | `tauri v2` | 应用框架，权限声明制安全模型 |
| 异步运行时 | `tokio` | 多线程异步运行时 |
| 取消控制 | `tokio-util` | `CancellationToken` |
| HTTP 客户端 | `reqwest` | 异步 HTTP，支持流式响应 |
| SSE 解析 | `eventsource-stream` / `tokio-stream` | 流式 SSE 解析 |
| 序列化 | `serde` + `serde_json` | JSON 处理 |
| 数据库 | `sqlx` (SQLite) | 异步 SQLite，支持版本化迁移 |
| 密钥管理 | `keyring` | OS 原生密钥管理（Keychain/Credential Manager） |
| 错误处理 | `thiserror` + `anyhow` | 定义错误类型，传播 |
| UUID | `uuid` | 主键生成 |
| 时间 | `chrono` | 日期时间与 Prompt 变量 |

## AI 与知识库相关依赖

| 用途 | 库 | 说明 |
|------|----|------|
| 向量检索 | `usearch` | HNSW 检索索引 |
| 嵌入推理 | `ort` | 本地 ONNX 嵌入模型 |
| PDF 解析 | `lopdf` | PDF 文本提取 |
| DOCX 解析 | `docx-rs` | Word 文档解析 |
| 网页抓取 | `scraper` | HTML 正文提取 |
| 模板引擎 | `handlebars` | 自定义渠道请求模板 |
| JSONPath | `jsonpath-rust` | 自定义渠道响应映射 |
| MCP 协议 | 自实现 | JSON-RPC、stdio / SSE |

## 前端依赖

| 用途 | 库 | 说明 |
|------|----|------|
| UI 框架 | React 18 + TypeScript | 组件化开发 |
| 构建工具 | Vite | 快速构建与 HMR |
| 状态管理 | Zustand | 轻量 store |
| 组件基础 | Tailwind CSS + shadcn/ui | 快速搭建桌面 UI |
| Markdown | `react-markdown` + `rehype-highlight` | 消息渲染 |
| 图标 | `lucide-react` | 图标库 |

## 可选增强依赖

只有在主链路稳定后再引入：

| 用途 | 库 | 说明 |
|------|----|------|
| 数学公式 | `rehype-katex` + `remark-math` | 可选 |
| Mermaid | `mermaid` | 可选 |
| 国际化 | `react-i18next` | 可选 |
| 拖拽排序 | `dnd-kit` | 可选 |
| 动效 | `framer-motion` | 可选 |

## 版本约束

| 组件 | 最低版本 | 说明 |
|------|---------|------|
| Rust | 1.75+ | 必须支持 async trait |
| Tauri | 2.x | v2 权限系统 |
| Node.js | 18+ | 前端构建 |
| SQLite | 3.35+ | 用于 JSON 函数 |

## 选型原则

- 先用最少依赖跑通主链路，再补增强功能
- 不为“以后可能会用”提前引入库
- 文档里没有明确需要的库，不要让 AI 擅自加
