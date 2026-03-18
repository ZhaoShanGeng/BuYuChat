# 增强模块：多模型对比 / TTS / 翻译 / 自动命名

> 路径：`src-tauri/src/services/`
> 这些功能属于展示增强，不应阻塞主线对话能力。

---

## MultiChatOrchestrator（多模型对比）

```rust
// src-tauri/src/services/multi_chat.rs
pub struct MultiChatOrchestrator {
    providers: Arc<ProviderRegistry>,
    db: SqlitePool,
}

impl MultiChatOrchestrator {
    /// 并发向多个模型发送同一问题
    pub async fn start(
        &self,
        conv_id: &str,
        content: &str,
        model_ids: Vec<String>,   // ["openai:gpt-4o", "claude:claude-3-5-sonnet"]
        app_handle: &AppHandle,
    ) -> Result<String>;

    /// 将某个模型的结果采纳为正式历史
    pub async fn adopt_response(
        &self,
        conv_id: &str,
        model_id: &str,
        content: &str,
    ) -> Result<MessageRow>;
}
```

实现约束：

- 多模型对比不修改原主对话历史，除非用户点击“采纳”
- 每个模型的流式事件独立推送：`chat:stream:{conv_id}:{model_id}`
- 任一模型失败，不影响其他模型

---

## VoiceEngine（TTS）

```rust
// src-tauri/src/services/voice.rs
pub struct VoiceEngine {
    current_handle: Mutex<Option<TtsHandle>>,
}

pub enum TtsHandle {
    Native,
    Api,
}

impl VoiceEngine {
    pub async fn speak(&self, text: &str, voice: Option<&str>, rate: Option<f32>) -> Result<()>;
    pub async fn stop(&self) -> Result<()>;
    pub fn is_speaking(&self) -> bool;
}
```

实现优先级：

1. 先做系统原生 TTS
2. 再做 API TTS

不要一开始就同时做两套复杂实现。

---

## TranslateService（翻译）

```rust
// src-tauri/src/services/translate.rs
pub struct TranslateService {
    providers: Arc<ProviderRegistry>,
}

impl TranslateService {
    pub async fn translate(
        &self,
        text: &str,
        src_lang: &str,
        tgt_lang: &str,
        provider: Option<&str>,
        model: Option<&str>,
    ) -> Result<String>;
}
```

固定 system prompt：

```text
你是专业翻译。请将以下文本从{src_lang}翻译为{tgt_lang}。只输出译文，不解释，不添加任何多余内容。
```

---

## NamingService（自动命名）

```rust
// src-tauri/src/services/naming.rs
pub struct NamingService {
    providers: Arc<ProviderRegistry>,
    db: SqlitePool,
}

impl NamingService {
    pub async fn auto_name(
        &self,
        conv_id: &str,
        app_handle: &AppHandle,
    ) -> ();
}
```

实现约束：

1. 只在首轮 user + assistant 生成后触发
2. 非流式调用
3. 出错只记日志，不中断主对话流程
4. 完成后发事件：`chat:title_updated`

---

## 本文件刻意不包含

- 记忆系统
- STT 语音输入
- 统计面板

这些不属于当前标准毕业设计版本。
