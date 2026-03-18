# 助手系统 & 提示词模块

> 路径：`src-tauri/src/services/assistant.rs`、`src-tauri/src/services/prompt.rs`

---

## AssistantService

```rust
// src-tauri/src/services/assistant.rs
pub struct AssistantService {
    db: SqlitePool,
}
```

### 公开方法

```rust
impl AssistantService {
    /// 列出所有助手（内置在前，自定义按 created_at ASC）
    /// SQL: ORDER BY builtin DESC, created_at ASC
    pub async fn list(&self) -> Result<Vec<AssistantRow>>;

    /// 按 ID 获取助手
    pub async fn get(&self, id: &str) -> Result<AssistantRow>;

    /// 创建自定义助手
    /// - id = uuid, builtin = false
    /// - tools_json / knowledge_base_ids 传 [] 时存 "[]"，不存 NULL
    pub async fn create(&self, req: CreateAssistantReq) -> Result<AssistantRow>;

    /// 更新助手（只允许更新非内置助手，builtin=true 时返回错误）
    pub async fn update(&self, id: &str, req: UpdateAssistantReq) -> Result<AssistantRow>;

    /// 删除助手（builtin=true 时返回 AppError::Other("不能删除内置助手")）
    pub async fn delete(&self, id: &str) -> Result<()>;

    /// 复制助手（用于用户从内置助手派生）
    /// - 复制所有字段，新 id，builtin = false
    /// - name 加上 " (副本)" 后缀
    pub async fn duplicate(&self, id: &str) -> Result<AssistantRow>;
}
```

### 辅助类型

```rust
#[derive(Debug, Deserialize)]
pub struct CreateAssistantReq {
    pub name: String,
    pub icon: String,              // emoji
    pub category: String,
    pub system_prompt: String,
    pub model_id: Option<String>,
    pub provider: Option<String>,
    pub tools: Vec<String>,        // 工具 ID 列表
    pub knowledge_base_ids: Vec<String>,
    pub params: Option<ModelParams>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssistantReq {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub system_prompt: Option<String>,
    pub model_id: Option<String>,
    pub provider: Option<String>,
    pub tools: Option<Vec<String>>,
    pub knowledge_base_ids: Option<Vec<String>>,
    pub params: Option<ModelParams>,
}
```

---

## PromptService

```rust
// src-tauri/src/services/prompt.rs
pub struct PromptService {
    db: SqlitePool,
    assistant_svc: Arc<AssistantService>,
}
```

### 公开方法

```rust
impl PromptService {
    /// 根据对话配置合成最终 System Prompt（供 ChatService 调用）
    ///
    /// 合成顺序：
    /// 1. 取 base prompt：
    ///    a. 优先使用 conversation.system_prompt（用户在对话中覆盖的）
    ///    b. 若为 NULL 且 conv.assistant_id 非 NULL，用 assistant.system_prompt
    ///    c. 若都为 NULL，返回 None（不注入 system 字段）
    /// 2. 对 base prompt 做变量替换（见内置变量表）
    /// 3. 若有 rag_context（由 ChatService 传入），追加到末尾：
    ///    ```
    ///    参考资料：
    ///    [1] {snippet}
    ///    来源：{doc_name} p.{page}
    ///
    ///    回答时：
    ///    - 只在确实使用资料的句子后面添加 [n]
    ///    - 不要编造不存在的引用编号
    ///    ```
    pub async fn compose_system(
        &self,
        conv: &ConversationRow,
        rag_context: Option<Vec<Citation>>,
    ) -> Result<Option<String>>;

    /// 对单个 prompt 字符串执行变量替换
    /// 支持变量见"内置变量表"
    /// 未知变量原样保留（不报错）
    pub fn render_variables(&self, template: &str, ctx: &RenderContext) -> String;

}
```

### 内置变量表

| 变量 | 说明 | 示例值 |
|------|------|--------|
| `{{date}}` | 当前日期（ISO 8601） | `2026-03-18` |
| `{{time}}` | 当前时间（HH:MM:SS） | `15:20:00` |
| `{{datetime}}` | 日期+时间 | `2026-03-18 15:20:00` |
| `{{weekday}}` | 英文星期 | `Wednesday` |
| `{{model_name}}` | 当前使用的模型 ID | `gpt-4o` |
| `{{provider}}` | Provider 名称 | `openai` |
| `{{os}}` | 操作系统 | `Windows 11` |
| `{{language}}` | 系统语言 | `zh-CN` |

### RenderContext

```rust
pub struct RenderContext {
    pub model_name: String,
    pub provider: String,
    pub os: String,
    pub language: String,
    // 日期时间在 render_variables 中通过 chrono::Local::now() 实时取
}
```

---

## ParamService

```rust
// src-tauri/src/services/param.rs
pub struct ParamService {
    db: SqlitePool,
}

impl ParamService {
    /// 列出所有参数预设（内置 + 自定义）
    pub async fn list_presets(&self, provider: Option<&str>) -> Result<Vec<ParamPresetRow>>;

    /// 创建自定义预设
    pub async fn create_preset(&self, name: &str, params: &ModelParams, provider: Option<&str>) -> Result<ParamPresetRow>;

    /// 删除预设
    pub async fn delete_preset(&self, id: &str) -> Result<()>;
}
```

**内置预设（由 migrations 初始数据写入）：**

```sql
-- migrations/002_seed.sql
INSERT OR IGNORE INTO param_presets (id, name, provider, params_json) VALUES
  ('preset-precise', '精确',  NULL, '{"temperature":0.1,"top_p":0.9}'),
  ('preset-balanced','平衡',  NULL, '{"temperature":0.7,"top_p":0.95}'),
  ('preset-creative','创意',  NULL, '{"temperature":1.2,"top_p":1.0}');
```

---

## 当前版本刻意不做

- 助手 Prompt 历史
- 全局 Prompt 库
- Prompt 版本回滚界面
