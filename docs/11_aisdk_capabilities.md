# aisdk v0.5.2 能力参考

> 信息来源以 docs.rs 为准：
> - https://docs.rs/aisdk/0.5.2/aisdk/
> - https://docs.rs/aisdk-macros/0.3.0/aisdk_macros/

## 1. 核心架构

- `LanguageModel` trait
  - `generate_text()` / `stream_text()`
  - docs.rs: https://docs.rs/aisdk/0.5.2/aisdk/core/language_model/trait.LanguageModel.html
- `LanguageModelRequest`
  - type-state builder，要求先设置 `model`，再进入 prompt/messages/options
  - docs.rs: https://docs.rs/aisdk/0.5.2/aisdk/core/struct.LanguageModelRequest.html
- Provider 抽象
  - BuYu 当前使用 `OpenAICompatible<DynamicModel>`，本质是 `/v1/chat/completions` 兼容层
  - docs.rs: https://docs.rs/aisdk/0.5.2/aisdk/providers/struct.OpenAICompatible.html

## 2. 请求选项 `LanguageModelOptions`

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/language_model/struct.LanguageModelOptions.html

| 字段 | 类型 | 说明 | BuYu 当前使用 |
|------|------|------|--------------|
| `system` | `Option<String>` | 系统提示词 | ✅ 通过消息列表首条 `system` 实现 |
| `schema` | `Option<Schema>` | 结构化输出 JSON Schema | ❌ |
| `seed` | `Option<u32>` | 随机种子 | ❌ |
| `temperature` | `Option<u32>` | 温度，SDK 会缩放到 `0.0-1.0` | ❌ |
| `top_p` | `Option<u32>` | nucleus sampling，SDK 会缩放到 `0.0-1.0` | ❌ |
| `top_k` | `Option<u32>` | top-k 采样 | ❌ |
| `max_retries` | `Option<u32>` | 最大重试次数 | ❌ |
| `max_output_tokens` | `Option<u32>` | 最大输出 token 数 | ✅ |
| `stop_sequences` | `Option<Vec<String>>` | 停止序列 | ❌ |
| `presence_penalty` | `Option<f32>` | 存在惩罚 | ❌ |
| `frequency_penalty` | `Option<f32>` | 频率惩罚 | ❌ |
| `stop_when` | `Option<StopWhenHook>` | 条件停止 hook | ❌ |
| `on_step_start` | `Option<OnStepStartHook>` | step 开始 hook | ❌ |
| `on_step_finish` | `Option<OnStepFinishHook>` | step 完成 hook | ❌ |
| `reasoning_effort` | `Option<ReasoningEffort>` | 思维链强度 | ❌ |

## 3. 流式 chunk 类型 `LanguageModelStreamChunkType`

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/language_model/enum.LanguageModelStreamChunkType.html

| 变体 | 说明 | BuYu 当前使用 |
|------|------|--------------|
| `Start` | 流开始 | ❌ |
| `Text(String)` | 文本增量 | ✅ |
| `Reasoning(String)` | reasoning/thinking 增量 | ❌ |
| `ToolCall(String)` | 工具调用参数增量 | ❌ |
| `End(AssistantMessage)` | 流正常结束 | ❌ |
| `Failed(String)` | 流失败 | ✅ |
| `Incomplete(String)` | 上游返回不完整 | ✅ |
| `NotSupported(String)` | provider 不支持的事件 | ❌ |

## 4. 响应内容类型 `LanguageModelResponseContentType`

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/language_model/enum.LanguageModelResponseContentType.html

- `Text(String)`：普通文本
- `ToolCall(ToolCallInfo)`：工具调用
- `Reasoning { content, extensions }`：思维链内容，包含 provider 扩展数据
- `NotSupported(String)`：不支持的能力

BuYu 当前只消费了 `Text(String)`。

## 5. 思维链支持

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/language_model/enum.ReasoningEffort.html

- `ReasoningEffort::Low`
- `ReasoningEffort::Medium`
- `ReasoningEffort::High`
- `ReasoningSupport` capability trait
  - https://docs.rs/aisdk/0.5.2/aisdk/core/capabilities/trait.ReasoningSupport.html
- 流式输出：
  - `LanguageModelStreamChunkType::Reasoning(String)`
- 非流式输出：
  - `LanguageModelResponseContentType::Reasoning { content, extensions }`

补充：
- 在 `aisdk` 的 OpenAI provider 转换逻辑中，`Low` 会映射为 OpenAI `minimal`。
- `extensions` 用于携带 provider 私有元数据，BuYu 当前不消费。

## 6. 工具调用

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/language_model/struct.Tool.html

- `Tool`
  - 字段包括 `name`、`description`、`input_schema`
- `Tool::builder()`
- `ToolCallInfo`
  - docs.rs: https://docs.rs/aisdk/0.5.2/aisdk/core/tools/struct.ToolCallInfo.html
- `ToolResultInfo`
  - docs.rs: https://docs.rs/aisdk/0.5.2/aisdk/core/tools/struct.ToolResultInfo.html
- `ToolList`
- `ToolCallSupport`
  - https://docs.rs/aisdk/0.5.2/aisdk/core/capabilities/trait.ToolCallSupport.html

BuYu 当前未接入工具调用 UI 或执行链路。

## 7. `#[tool]` 宏

来源：
https://docs.rs/aisdk-macros/0.3.0/aisdk_macros/attr.tool.html

`aisdk-macros v0.3.0` 提供 `#[tool]` attribute macro，可从普通 Rust 函数推导 Tool：

- 函数名 -> tool name
- 文档注释 -> description
- 参数 -> input schema

要求：

- 参数类型需支持 schema 推导
- 生成的工具最终仍走 `Tool` 结构

## 8. Capability Traits

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/capabilities/index.html

| Trait | 含义 |
|-------|------|
| `TextInputSupport` | 文本输入 |
| `TextOutputSupport` | 文本输出 |
| `AudioInputSupport` | 音频输入 |
| `AudioOutputSupport` | 音频输出 |
| `ImageInputSupport` | 图片输入 |
| `ImageOutputSupport` | 图片输出 |
| `VideoInputSupport` | 视频输入 |
| `VideoOutputSupport` | 视频输出 |
| `ToolCallSupport` | 工具调用 |
| `StructuredOutputSupport` | 结构化输出 |
| `ReasoningSupport` | 思维链/推理 |
| `ModelName` | 静态模型名 trait |

## 9. Token 使用统计 `Usage`

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/language_model/struct.Usage.html

- `input_tokens`
- `output_tokens`
- `reasoning_tokens`
- `cached_tokens`

支持 `Add` 聚合，适合多 step/tool call 汇总。

## 10. 停止原因 `StopReason`

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/language_model/enum.StopReason.html

- `Finish`
- `Provider(String)`
- `Hook`
- `Error(Error)`
- `Other(String)`

BuYu 当前统一映射为对外 `finish_reason` 字符串。

## 11. Vercel UI 集成

来源：
https://docs.rs/aisdk/0.5.2/aisdk/integrations/vercel_aisdk_ui/index.html

- `VercelUIStream`
- `VercelUIStreamOptions`
- `StreamTextResponse::to_vercel_ui_stream(...)`

这部分更适合 Web 服务端，不是 BuYu 当前桌面端主路径。

## 12. 嵌入模型

来源：
https://docs.rs/aisdk/0.5.2/aisdk/core/embedding_model/index.html

- `EmbeddingModel` trait
- `EmbeddingModelOptions`
- 对应 provider embedding 实现

BuYu 当前未使用嵌入能力。

## 13. 与 BuYu 相关的关键结论

### 13.1 已可直接接入的能力

- `Reasoning(String)` 流式 chunk
- `reasoning_effort`
- `Usage.reasoning_tokens`
- `ToolCallSupport`

这些不需要替换 `aisdk` 版本，当前 `0.5.2` 已具备。

### 13.2 当前 `OpenAICompatible` 路径的真实限制

- `OpenAICompatible` 使用的是 `/v1/chat/completions` 兼容层
- `aisdk` 对这条路径的 `Message::User` 仍是纯字符串
- `OpenAICompatible` 的现成消息转换不支持把用户消息编码成“文本 + 图片数组 content”

结论：

- reasoning 接入可以继续走现有 AISDK
- 图片输入需要在 BuYu adapter 中对“带图请求”单独构造 chat-completions body

## 14. BuYu 扩展计划勾选

- [x] `max_output_tokens`
- [ ] `temperature`
- [ ] `top_p`
- [ ] `top_k`
- [ ] `reasoning_effort`
- [ ] `Reasoning` chunk 展示
- [ ] `thinking_tags` 自定义标签解析
- [ ] `ToolCall`
- [ ] `ImageInput`
- [ ] `presence_penalty`
- [ ] `frequency_penalty`
- [ ] `stop_sequences`
