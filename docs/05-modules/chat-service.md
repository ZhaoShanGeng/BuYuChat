# ChatService — 对话编排服务

> 路径：`src-tauri/src/services/chat.rs`

ChatService 是所有对话请求的中枢，负责：上下文组装 → Provider 调用 → 结果落库 → 流式推送前端。

---

## 结构体定义

```rust
pub struct ChatService {
    db: SqlitePool,
    providers: Arc<ProviderRegistry>,
    prompt_svc: Arc<PromptService>,
    rag_svc: Arc<RagService>,
    tool_svc: Arc<ToolService>,
    naming_svc: Arc<NamingService>,
}
```

---

## 公开方法

### send_message

```rust
/// 用户发送消息，启动流式对话
///
/// # 参数
/// - `conv_id`：目标对话 UUID
/// - `user_content`：用户输入（文本或多模态 ContentPart 列表）
/// - `override_model`：可选，本次覆盖使用的模型（None = 使用对话绑定模型）
/// - `cancel_token`：前端取消时 drop 触发
/// - `app_handle`：用于 emit 流式事件
///
/// # 返回
/// 返回新建的 user message id 和 assistant message id（用于前端乐观更新）
///
/// # 副作用
/// - 持久化 user message
/// - 创建 assistant 占位消息，并在流结束后补全内容
/// - 通过 Tauri emit 推送 StreamEvent（事件名 "chat:stream:{conv_id}"）
/// - 触发后台 NamingService（首轮对话）
pub async fn send_message(
    &self,
    conv_id: &str,
    user_content: MessageContent,
    override_model: Option<String>,
    cancel_token: CancellationToken,
    app_handle: &AppHandle,
) -> Result<(String, String)>;
```

**内部步骤（按序）：**

1. **加载对话配置**
   ```rust
   let conv = db::conversation::get(&self.db, conv_id).await?;
   let model = override_model.unwrap_or_else(|| conv.model_id.clone());
   let provider_name = conv.provider.clone();
   ```

2. **持久化 user message**
   ```rust
   let user_msg_id = uuid::Uuid::new_v4().to_string();
   let version_group_id = uuid::Uuid::new_v4().to_string();
   db::message::insert(&self.db, MessageRow {
       id: user_msg_id.clone(),
       conversation_id: conv_id.to_string(),
       parent_message_id: db::message::find_last_active_message_id(&self.db, conv_id).await?,
       version_group_id: version_group_id.clone(),
       version_index: 1,
       is_active: true,
       role: "user".to_string(),
       content: user_content.as_text().into(),
       ..Default::default()
   }).await?;
   ```

3. **组装历史消息（上下文裁剪）**
   ```rust
   let history = db::message::list_active(&self.db, conv_id).await?;
   let messages = context_builder::build(history, &model, max_tokens_budget);
   ```

4. **RAG 检索（如对话关联知识库）**
   ```rust
   let rag_ctx = if !conv_knowledge_bases.is_empty() {
       Some(self.rag_svc.retrieve(user_content.as_text(), &conv_knowledge_bases, 5, 0.5).await?)
   } else { None };
   ```

5. **注入 System Prompt**
   ```rust
   let system = self.prompt_svc.compose_system(&conv, rag_ctx.clone()).await?;
   ```

6. **加载工具定义**
   ```rust
   let tools = self.tool_svc.get_tools_for_conv(&conv).await?;
   ```

7. **构造 ChatRequest**
   ```rust
   let req = ChatRequest {
       model: model.clone(),
       messages,
       system_prompt: system,
       params: resolve_model_params(&conv).await?,
       tools: if tools.is_empty() { None } else { Some(tools) },
       stream: true,
   };
   ```

   这里涉及 3 个辅助来源：

   - `conv_knowledge_bases`：来自当前 assistant 绑定的 `knowledge_base_ids`
   - `resolve_model_params(&conv)`：按“对话覆盖 > assistant.params > 默认值”合并参数
   - `max_tokens_budget`：从模型上下文长度中预留回复空间后得到的可用历史预算

8. **创建 assistant 占位消息**
   ```rust
   let assistant_msg_id = uuid::Uuid::new_v4().to_string();
   db::message::insert(&self.db, MessageRow {
       id: assistant_msg_id.clone(),
       conversation_id: conv_id.to_string(),
       parent_message_id: Some(user_msg_id.clone()),
       version_group_id: uuid::Uuid::new_v4().to_string(),
       version_index: 1,
       is_active: true,
       role: "assistant".to_string(),
       content: Some(String::new()),
       ..Default::default()
   }).await?;
   ```

   `send_message` 返回前，`assistant_msg_id` 对应的消息必须已经在数据库里。

9. **获取 Provider**
   ```rust
   let provider = self.providers.get(&provider_name).await?;
   let (tx, mut rx) = mpsc::channel::<StreamEvent>(128);
   ```

10. **执行主流程**

   - 无工具或 provider 不支持工具调用：直接走 `provider.chat_stream`
   - 有工具且 provider 支持工具调用：走 `run_tool_loop`

11. **转发流式事件到前端**
    ```rust
    while let Some(event) = rx.recv().await {
        if cancel_token.is_cancelled() { break; }
        app_handle.emit(&format!("chat:stream:{}", conv_id), &event)?;
        if let StreamEvent::Delta { text } = &event {
            accumulated_text.push_str(text);
        }
    }
    ```

12. **更新 assistant 占位消息**
    ```rust
    db::message::update_assistant_result(
        &self.db,
        &assistant_msg_id,
        &accumulated_text,
        serialized_tool_calls.as_deref(),
        citations_json.as_deref(),
        usage.map(|u| u.total_tokens as i64),
    ).await?;
    db::conversation::touch(&self.db, conv_id).await?;
    ```

13. **触发自动命名（首轮）**
    ```rust
    let msg_count = db::message::count_active(&self.db, conv_id).await?;
    if msg_count == 2 {  // user + assistant 各一条
        let naming_svc_clone = self.naming_svc.clone();
        tokio::spawn(async move {
            naming_svc_clone.auto_name(conv_id, app_handle).await;
        });
    }
    ```

---

### regenerate

```rust
/// 重新生成对话中最后一条 assistant 消息
///
/// - 找到最后一条 is_active=true 的 assistant 消息的 version_group_id
/// - 将同组所有消息设为 is_active=false
/// - 新建 version_index+1 的 assistant 消息（走与 send_message 相同的生成流程）
/// - 历史上下文不包含最后一条 assistant 消息（重新生成，从 user 消息截止）
pub async fn regenerate(
    &self,
    conv_id: &str,
    cancel_token: CancellationToken,
    app_handle: &AppHandle,
) -> Result<String>;
```

实现约束：

- 只允许重新生成最后一条活跃 assistant 消息
- 新版本沿用原 `version_group_id`
- 重新生成前先把该组旧版本全部设为 `is_active=false`
- 历史上下文截止到对应的上一条 user 消息，不包含旧 assistant 回复

---

### edit_user_message

```rust
pub async fn edit_user_message(
    &self,
    conv_id: &str,
    message_id: &str,
    new_content: MessageContent,
    cancel_token: CancellationToken,
    app_handle: &AppHandle,
) -> Result<(String, String)>;
```

实现约束：

1. 只允许编辑 `role='user'` 的消息
2. 在事务里删除该消息及其后续全部消息
3. 插入新的 user 消息
4. 复用 `send_message` 的生成流程创建新的 assistant 回复

---

### stop_generation

```rust
pub fn stop_generation(&self, cancel_token: CancellationToken) {
    cancel_token.cancel();
}
```

---

## 上下文裁剪（context_builder）

> 路径：`src-tauri/src/services/context_builder.rs`

```rust
/// 将数据库消息列表裁剪为 Provider 可接受的 messages 数组
///
pub fn build(
    history: Vec<MessageRow>,
    model: &str,
    max_context_tokens: u32,
) -> Vec<Message>;
```

裁剪策略：

1. 只处理当前活跃链路
2. 从最新消息往前回收，按粗略 token 预算裁剪
3. `tool` 消息必须和触发它的 assistant 消息一起保留
4. 至少保留最后一轮 user 消息

---

## 带工具的对话循环

```rust
// src-tauri/src/services/chat.rs 内部
async fn run_tool_loop(
    &self,
    req: ChatRequest,
    provider: Arc<dyn LlmProvider>,
    tx: mpsc::Sender<StreamEvent>,
    conv_id: &str,
    app_handle: &AppHandle,
) -> Result<(String, Option<Vec<ToolCall>>, Option<TokenUsage>)> {
    let max_iterations: u8 = 5;
    let mut messages = req.messages.clone();
    let mut iteration = 0u8;
    let mut total_usage = TokenUsage::default();

    loop {
        if iteration >= max_iterations {
            return Err(AppError::ToolLoopExceeded { max: max_iterations });
        }
        iteration += 1;

        let response = provider.chat(&ChatRequest {
            messages: messages.clone(),
            stream: false,
            ..req.clone()
        }).await?;

        if let Some(total) = response.usage {
            total_usage.prompt_tokens += total.prompt_tokens;
            total_usage.completion_tokens += total.completion_tokens;
        }

        match response.finish_reason.as_deref() {
            Some("tool_calls") | Some("tool_use") => {
                let calls = response.tool_calls.unwrap_or_default();
                for call in &calls {
                    tx.send(StreamEvent::ToolCall { call: call.clone() }).await.ok();
                }
                messages.push(Message {
                    role: Role::Assistant,
                    content: MessageContent::Text(response.content),
                    tool_calls: Some(calls.clone()),
                    ..Default::default()
                });
                for call in &calls {
                    let result = self.tool_svc.execute(call).await
                        .unwrap_or_else(|e| format!("Tool error: {}", e));
                    messages.push(Message {
                        role: Role::Tool,
                        content: MessageContent::Text(result),
                        tool_call_id: Some(call.id.clone()),
                        ..Default::default()
                    });
                }
            }
            _ => {
                let final_req = ChatRequest { messages: messages.clone(), stream: true, ..req.clone() };
                provider.chat_stream(&final_req, tx).await?;
                return Ok((response.content, response.tool_calls, Some(total_usage)));
            }
        }
    }
}
```

---

## 生成约束

- `send_message` 返回前，user 和 assistant 占位消息都必须已经落库
- 不做分支对话，不做中间轮次流式展示
- 工具调用失败时，写回 tool message 文本，不直接中断整轮对话
- 取消生成时保留当前已收到的部分内容
