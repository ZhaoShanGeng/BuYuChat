# 目录结构

本文档是给 AI 生成代码时用的目录约束。没有写在这里的目录，不要擅自发散创建。

---

## 后端目录

```text
src-tauri/
├── Cargo.toml
├── build.rs
├── tauri.conf.json
├── migrations/
│   ├── 001_init.sql
│   └── 002_seed.sql
└── src/
    ├── lib.rs
    ├── main.rs
    ├── error.rs
    ├── types.rs
    ├── state.rs
    ├── db/
    │   ├── mod.rs
    │   ├── models.rs
    │   ├── conversation.rs
    │   ├── message.rs
    │   ├── provider_config.rs
    │   ├── assistant.rs
    │   ├── param_preset.rs
    │   ├── document.rs
    │   ├── tool.rs
    │   ├── mcp_server.rs
    │   └── custom_channel.rs
    ├── providers/
    │   ├── mod.rs
    │   ├── openai.rs
    │   ├── openai_compat.rs
    │   ├── claude.rs
    │   ├── gemini.rs
    │   ├── ollama.rs
    │   └── custom.rs
    ├── services/
    │   ├── chat.rs
    │   ├── context_builder.rs
    │   ├── assistant.rs
    │   ├── prompt.rs
    │   ├── param.rs
    │   ├── naming.rs
    │   ├── versioning.rs
    │   ├── rag/
    │   │   ├── mod.rs
    │   │   ├── processor.rs
    │   │   ├── embedder.rs
    │   │   ├── vector_store.rs
    │   │   └── citation.rs
    │   └── tool/
    │       ├── mod.rs
    │       ├── builtin/
    │       │   ├── calculator.rs
    │       │   └── web_search.rs
    │       └── mcp/
    │           ├── client.rs
    │           ├── registry.rs
    │           └── transport.rs
    ├── commands/
    │   ├── mod.rs
    │   ├── conversation.rs
    │   ├── message.rs
    │   ├── chat.rs
    │   ├── provider.rs
    │   ├── assistant.rs
    │   ├── param.rs
    │   ├── rag.rs
    │   ├── tool.rs
    │   ├── enhancement.rs
    │   └── export.rs
    └── utils/
        ├── time.rs
        └── json.rs
```

---

## 前端目录

```text
src/
├── main.tsx
├── App.tsx
├── app/
│   ├── router.tsx
│   └── providers.tsx
├── api/
│   ├── tauri.ts
│   ├── conversation.ts
│   ├── chat.ts
│   ├── provider.ts
│   ├── assistant.ts
│   ├── rag.ts
│   ├── tool.ts
│   └── enhancement.ts
├── stores/
│   ├── conversation-store.ts
│   ├── chat-store.ts
│   ├── settings-store.ts
│   ├── assistant-store.ts
│   └── rag-store.ts
├── components/
│   ├── layout/
│   ├── chat/
│   ├── assistant/
│   ├── rag/
│   ├── settings/
│   └── common/
├── pages/
│   ├── chat-page.tsx
│   ├── settings-page.tsx
│   └── knowledge-page.tsx
├── hooks/
│   ├── use-chat-stream.ts
│   └── use-tauri-event.ts
├── lib/
│   ├── markdown.ts
│   └── utils.ts
└── styles/
    └── globals.css
```

---

## 目录规则

1. `db/` 只放数据访问，不放业务编排。
2. `services/` 只放业务逻辑，不直接暴露给前端。
3. `commands/` 只做参数接收、调用 service、返回结果。
4. `providers/` 只处理第三方 API 适配，不掺杂数据库逻辑。
5. 前端 `api/` 只封装 Tauri `invoke`，状态逻辑放在 `stores/`。

---

## 生成顺序建议

1. `error.rs`
2. `types.rs`
3. `db/mod.rs` + `migrations/`
4. `db/*.rs`
5. `providers/*.rs`
6. `services/chat.rs` / `services/prompt.rs` / `services/tool/`
7. `commands/*.rs`
8. 前端 `api/` 和 `stores/`
9. 前端页面与组件
