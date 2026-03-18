# IPC 命令参考

> 所有命令定义在 `src-tauri/src/commands/` 下，按功能分文件。
> 前端通过 `invoke("command_name", {args})` 调用，事件通过 `listen("event_name", handler)` 订阅。
> 所有命令成功时返回 `T`，失败时返回 `{ error: string }`（Tauri 自动包装）。

本文档只保留当前版本要真正落地的命令。不要再生成收藏、Prompt 历史、全局 Prompt 库等已移除接口。

---

## 对话管理（conversation.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `list_conversations` | `{page, per_page}` | `PageResponse<ConversationRow>` | 置顶优先，按更新时间倒序 |
| `get_conversation` | `{id}` | `ConversationRow` | 按 ID 获取 |
| `create_conversation` | `{model_id, provider, assistant_id?}` | `ConversationRow` | 新建对话 |
| `update_conversation_title` | `{id, title}` | `()` | 手动设置标题 |
| `update_conversation_model` | `{id, model_id, provider}` | `()` | 切换模型（对话中） |
| `update_conversation_system_prompt` | `{id, system_prompt: null\|string}` | `()` | 覆盖 system prompt |
| `toggle_pin_conversation` | `{id}` | `bool` | 返回新的 pinned 状态 |
| `delete_conversation` | `{id}` | `()` | 删除对话及所有消息 |
| `clear_conversation_messages` | `{id}` | `()` | 清空消息，保留对话 |
| `export_conversation` | `{id, format}` | `string` | format: "json"\|"markdown"，返回文件内容字符串 |

---

## 消息（message.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `list_messages` | `{conv_id}` | `Vec<MessageRow>` | 只返回 is_active=true 的消息 |
| `get_message_versions` | `{version_group_id}` | `Vec<MessageRow>` | 返回所有版本列表 |
| `switch_message_version` | `{version_group_id, target_index}` | `MessageRow` | 切换活跃版本 |

---

## 对话流式控制（chat.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `send_message` | `{conv_id, content, content_parts?, override_model?}` | `{user_msg_id, assistant_msg_id}` | 启动流式生成 |
| `regenerate_message` | `{conv_id}` | `{assistant_msg_id}` | 重新生成最后一条 |
| `edit_user_message` | `{conv_id, message_id, new_content}` | `{user_msg_id, assistant_msg_id}` | 编辑并重新生成 |
| `stop_generation` | `{conv_id}` | `()` | 取消当前生成 |

**说明：**
- `send_message` / `regenerate_message` / `edit_user_message` 立即返回消息 ID，流式内容通过事件推送
- 前端收到 ID 后立即乐观渲染占位消息，再订阅事件更新内容

---

## 流式事件（前端 listen）

```typescript
// 事件名格式：chat:stream:{conv_id}
// 事件载荷：StreamEvent（JSON）
type StreamEvent =
  | { type: "delta"; text: string }
  | { type: "tool_call"; call: ToolCall }
  | { type: "done"; usage?: TokenUsage; finish_reason: string }
  | { type: "error"; message: string };

// 其他事件
"chat:title_updated"     → { conv_id: string, title: string }  // AI 命名完成
"rag:document:status"    → { doc_id: string, status: "ready"|"error", error_msg?: string }
"mcp:server:status"      → { server_id: string, status: "connected"|"disconnected"|"error" }
```

---

## Provider 与模型（provider.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `list_provider_configs` | `{}` | `Vec<ProviderConfigRow>` | 列出所有 Provider 配置 |
| `save_provider_config` | `{provider, api_key?, base_url?}` | `()` | 新建或更新 Provider 配置 |
| `test_provider_connection` | `{provider}` | `{ok: bool, message: string}` | 调用 health_check() |
| `list_models` | `{provider}` | `Vec<ModelInfo>` | 按 Provider 列出模型 |
| `save_api_key` | `{key_id, value}` | `()` | 写入 keyring |
| `delete_api_key` | `{key_id}` | `()` | 从 keyring 删除 |
| `list_custom_channels` | `{}` | `Vec<CustomChannelRow>` | |
| `create_custom_channel` | `CustomChannelRow` | `CustomChannelRow` | 同时注册到 ProviderRegistry |
| `update_custom_channel` | `{id, ...fields}` | `()` | 同时更新 ProviderRegistry |
| `delete_custom_channel` | `{id}` | `()` | 同时从 ProviderRegistry 移除 |

---

## 助手（assistant.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `list_assistants` | `{}` | `Vec<AssistantRow>` | 内置在前 |
| `get_assistant` | `{id}` | `AssistantRow` | |
| `create_assistant` | `CreateAssistantReq` | `AssistantRow` | |
| `update_assistant` | `{id, ...UpdateAssistantReq}` | `AssistantRow` | 更新助手配置 |
| `delete_assistant` | `{id}` | `()` | 内置助手不可删除 |
| `duplicate_assistant` | `{id}` | `AssistantRow` | 复制助手 |

---

## 参数预设（param.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `list_param_presets` | `{provider?}` | `Vec<ParamPresetRow>` | provider=null 返回通用预设 |
| `create_param_preset` | `{name, params, provider?}` | `ParamPresetRow` | |
| `delete_param_preset` | `{id}` | `()` | |

---

## RAG 知识库（rag.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `import_document_file` | `{path, name}` | `{doc_id}` | 异步启动，通过事件返回状态 |
| `import_document_url` | `{url}` | `{doc_id}` | 同上 |
| `list_documents` | `{}` | `Vec<DocumentRow>` | |
| `delete_document` | `{id}` | `()` | |
| `scrape_url_preview` | `{url}` | `{text: string}` | 爬取正文，不存入知识库 |

---

## 工具 & MCP（tool.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `list_tools` | `{}` | `Vec<RegisteredTool>` | 含来源(builtin/mcp/user) |
| `set_tool_enabled` | `{name, enabled}` | `()` | 启用/禁用工具 |
| `list_mcp_servers` | `{}` | `Vec<McpServerRow>` | 可选增强 |
| `create_mcp_server` | `McpServerRow` | `McpServerRow` | 可选增强 |
| `update_mcp_server` | `{id, ...fields}` | `()` | 可选增强 |
| `delete_mcp_server` | `{id}` | `()` | 可选增强 |
| `toggle_mcp_server` | `{id, enabled}` | `()` | 可选增强 |
| `get_mcp_server_status` | `{id}` | `McpStatus` | 可选增强 |

---

## 增强功能（enhancement.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `start_multi_chat` | `{conv_id, content, model_ids: string[]}` | `{request_id}` | 并发发给多个模型 |
| `adopt_multi_chat_response` | `{conv_id, model_id, content}` | `{msg_id}` | 选中某模型回复作为正式历史 |
| `tts_speak` | `{text, voice?, rate?}` | `()` | 朗读文本 |
| `tts_stop` | `{}` | `()` | 停止朗读 |
| `translate_text` | `{text, src_lang, tgt_lang, provider?, model?}` | `{translated}` | 流式不需要，直接返回 |

---

## 数据导入导出（export.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `export_all_data` | `{path}` | `()` | 整库导出为 JSON（含对话/消息/助手/知识库） |
| `import_all_data` | `{path}` | `{imported_count}` | 从 JSON 文件导入（合并模式，不覆盖同 ID） |

---

## 命令注册（lib.rs）

```rust
// src-tauri/src/lib.rs（Tauri 插件入口）
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // conversation
        commands::conversation::list_conversations,
        commands::conversation::get_conversation,
        commands::conversation::create_conversation,
        commands::conversation::update_conversation_title,
        commands::conversation::update_conversation_model,
        commands::conversation::update_conversation_system_prompt,
        commands::conversation::toggle_pin_conversation,
        commands::conversation::delete_conversation,
        commands::conversation::clear_conversation_messages,
        commands::conversation::export_conversation,
        // message
        commands::message::list_messages,
        commands::message::get_message_versions,
        commands::message::switch_message_version,
        // chat
        commands::chat::send_message,
        commands::chat::regenerate_message,
        commands::chat::edit_user_message,
        commands::chat::stop_generation,
        // provider
        commands::provider::list_provider_configs,
        commands::provider::save_provider_config,
        commands::provider::test_provider_connection,
        commands::provider::list_models,
        commands::provider::save_api_key,
        commands::provider::delete_api_key,
        commands::provider::list_custom_channels,
        commands::provider::create_custom_channel,
        commands::provider::update_custom_channel,
        commands::provider::delete_custom_channel,
        // assistant
        commands::assistant::list_assistants,
        commands::assistant::get_assistant,
        commands::assistant::create_assistant,
        commands::assistant::update_assistant,
        commands::assistant::delete_assistant,
        commands::assistant::duplicate_assistant,
        // param
        commands::param::list_param_presets,
        commands::param::create_param_preset,
        commands::param::delete_param_preset,
        // rag
        commands::rag::import_document_file,
        commands::rag::import_document_url,
        commands::rag::list_documents,
        commands::rag::delete_document,
        commands::rag::scrape_url_preview,
        // tool
        commands::tool::list_tools,
        commands::tool::set_tool_enabled,
        commands::tool::list_mcp_servers,
        commands::tool::create_mcp_server,
        commands::tool::update_mcp_server,
        commands::tool::delete_mcp_server,
        commands::tool::toggle_mcp_server,
        commands::tool::get_mcp_server_status,
        // enhancement
        commands::enhancement::start_multi_chat,
        commands::enhancement::adopt_multi_chat_response,
        commands::enhancement::tts_speak,
        commands::enhancement::tts_stop,
        commands::enhancement::translate_text,
        // export
        commands::export::export_all_data,
        commands::export::import_all_data,
    ])
```

---

## 生成约束

- 命令名、参数名、返回结构必须与本文档一致
- 没出现在本文档里的命令，不要擅自新增
- 增强命令可以后实现，但命令名不要提前漂移
