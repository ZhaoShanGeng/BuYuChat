# 系统架构

本文档按“核心闭环 + 可选增强”理解即可。毕业设计版本先保证主链路稳定，再逐步挂接多模型对比、自定义渠道、语音等增强模块。

## 架构总览

```
┌────────────────────────────────────────────────────────────────┐
│                        Frontend (React)                        │
│  Chat View  Conversation List  Settings  Assistant  RAG UI     │
│  Optional: Multi-Compare / Translation / Voice / Channel UI    │
│                        │ Tauri IPC                             │
├────────────────────────┼───────────────────────────────────────┤
│                   Rust Backend (Tauri Core)                     │
│                        │                                        │
│  Command Router → ChatService → Provider Adapter               │
│                  │             │                                │
│                  │             └─ OpenAI / Claude / Gemini      │
│                  │                / Ollama / CustomChannel      │
│                  │                                              │
│                  ├─ ConversationStore (SQLite)                  │
│                  ├─ PromptService / AssistantService            │
│                  ├─ ParamService                                │
│                  ├─ ToolService / WebSearch                     │
│                  ├─ RagService                                  │
│                  └─ Optional: MultiChat / TTS / Translation     │
│                                                                │
│  Infrastructure: SQLite / Keyring / Embedding / Vector Store   │
└────────────────────────────────────────────────────────────────┘
```

---

## 层次职责

| 层 | 技术 | 职责 |
|---|------|------|
| **Frontend** | React 18 + TypeScript + Vite | UI 渲染、用户交互、事件订阅、消息列表与设置界面 |
| **IPC Layer** | Tauri `#[tauri::command]` + Event System | 前端 `invoke()` 请求后端，后端 `emit()` 推送流式结果 |
| **Core Services** | Rust async (tokio) | 对话编排、会话存储、提示词管理、参数管理 |
| **AI Services** | Rust | Provider 适配、工具调用、RAG 检索、命名等 AI 相关服务 |
| **Enhancement Services** | Rust | 多模型对比、TTS、翻译、自定义渠道等增强能力 |
| **Infrastructure** | SQLite + keyring + embedding/vector store | 持久化、密钥管理、向量检索、本地资源管理 |

---

## 毕业设计主链路

推荐把系统理解成下面这条主链路，其他模块都围绕它扩展：

```
设置模型与密钥
    ↓
创建会话并发送消息
    ↓
ChatService 组装上下文
    ↓
Provider 流式返回
    ↓
前端实时渲染与落库
    ↓
可选叠加 Assistant / Param / RAG / Tool
```

只要这条链路稳定，项目就已经具备完整可演示性。多模型对比、TTS、自定义渠道等都属于在主链路之上的加分模块。

---

## IPC 数据流

```
用户操作（前端）
    │ invoke("send_message", {...})
    ▼
Command Router (Tauri)
    │
    ▼
ChatService.send()
    ├─ PromptService.compose()        → 注入 System Prompt + 可选上下文增强 + RAG 上下文
    ├─ Provider.chat_stream()         → SSE / HTTP 流式请求
    └─ emit("chat:stream:{conv_id}")  → 逐 token 推送到前端
        └─ 前端 listen() 接收 → 渲染
```

---

## 关键数据流示例

### 带工具的多轮对话

```
用户输入 → ChatService → LLM 返回 tool_calls
    → ToolService.execute()
        ├─ 内置工具 → 直接执行（网页搜索/计算器）
        └─ MCP 工具 → McpClient.call_tool() → MCP Server
    → 结果格式化为 tool message
    → 再次发送给 LLM → 最终回复 → 推送前端
```

### RAG 增强对话

```
用户输入 → Embedder(query) → VectorStore.search(top-k)
    → 检索片段 + 来源标记
    → PromptComposer: system + RAG上下文 + 对话历史
    → LLM → 回复 + CitationEngine 解析引用标注
    → 前端展示带来源标注的回复
```

说明：毕业设计版本可先不做完整记忆系统，先把“文件入库 -> 检索 -> 引用展示”这条 RAG 链路做扎实。
