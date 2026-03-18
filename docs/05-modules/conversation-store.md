# ConversationStore — 对话与消息 CRUD

> 路径：`src-tauri/src/db/`，分为 `conversation.rs`、`message.rs` 两个子模块。
> 所有函数不封装到结构体。
> 下文展示逻辑接口；需要事务的写操作，实际代码额外提供 `*_tx` 版本。

---

## db::conversation

```rust
// src-tauri/src/db/conversation.rs

/// 创建新对话，返回插入的 ConversationRow
pub async fn create(
    db: &SqlitePool,
    model_id: &str,
    provider: &str,
    assistant_id: Option<&str>,
) -> Result<ConversationRow>;
// 实现：id=uuid, title="新对话", created_at=updated_at=now()s, 其余字段用默认值

/// 按 ID 查询对话；找不到返回 AppError::NotFound
pub async fn get(db: &SqlitePool, id: &str) -> Result<ConversationRow>;

/// 对话列表（分页，置顶优先，按 updated_at 倒序）
/// ORDER BY pinned DESC, updated_at DESC LIMIT per_page OFFSET (page-1)*per_page
pub async fn list(
    db: &SqlitePool,
    page: u32,
    per_page: u32,
) -> Result<PageResponse<ConversationRow>>;

/// 更新对话标题（手动修改或 AI 命名完成后调用）
pub async fn update_title(db: &SqlitePool, id: &str, title: &str) -> Result<()>;

/// 更新对话的模型和 Provider（用于对话中切换模型）
pub async fn update_model(
    db: &SqlitePool,
    id: &str,
    model_id: &str,
    provider: &str,
) -> Result<()>;

/// 更新对话的 system_prompt（用户在对话中直接覆盖提示词）
pub async fn update_system_prompt(
    db: &SqlitePool,
    id: &str,
    system_prompt: Option<&str>,
) -> Result<()>;

/// 切换置顶状态（toggle）
pub async fn toggle_pin(db: &SqlitePool, id: &str) -> Result<bool>;  // 返回新状态

/// 更新 updated_at 为 now()（每次消息落库后调用，推动对话在列表置顶）
pub async fn touch(db: &SqlitePool, id: &str) -> Result<()>;

/// 删除对话（CASCADE：自动删除关联 messages）
pub async fn delete(db: &SqlitePool, id: &str) -> Result<()>;

/// 清空对话的所有消息（保留对话本身）
pub async fn clear_messages(db: &SqlitePool, id: &str) -> Result<()>;

/// 统计总对话数（用于分页 total 字段）
pub async fn count(db: &SqlitePool) -> Result<u32>;
```

---

## db::message

```rust
// src-tauri/src/db/message.rs

/// 插入一条消息
pub async fn insert(db: &SqlitePool, row: MessageRow) -> Result<()>;
// 注意：调用方负责生成 id、version_group_id，设置 is_active=true

/// 获取对话中所有活跃版本消息（is_active=true），按 created_at ASC
pub async fn list_active(
    db: &SqlitePool,
    conv_id: &str,
) -> Result<Vec<MessageRow>>;

/// 获取对话中所有消息（含非活跃版本，用于版本切换 UI）
pub async fn list_all(
    db: &SqlitePool,
    conv_id: &str,
) -> Result<Vec<MessageRow>>;

/// 获取同一 version_group_id 的所有版本（用于版本导航）
pub async fn list_versions(
    db: &SqlitePool,
    version_group_id: &str,
) -> Result<Vec<MessageRow>>;
// 返回按 version_index ASC 排序的列表

/// 切换活跃版本
/// 1. UPDATE messages SET is_active=0 WHERE version_group_id=?
/// 2. UPDATE messages SET is_active=1 WHERE version_group_id=? AND version_index=?
pub async fn set_active_version(
    db: &SqlitePool,
    version_group_id: &str,
    version_index: i64,
) -> Result<()>;

/// 查当前 version_group_id 的最大 version_index（用于重新生成时确定新序号）
pub async fn max_version_index(
    db: &SqlitePool,
    version_group_id: &str,
) -> Result<i64>;

/// 查找最后一条活跃 assistant 消息（用于重新生成）
pub async fn find_last_active_assistant(
    db: &SqlitePool,
    conv_id: &str,
) -> Result<MessageRow>;

/// 删除某个时间点及之后的所有消息（用于编辑用户消息）
pub async fn delete_from_created_at(
    db: &SqlitePool,
    conv_id: &str,
    cutoff_created_at: i64,
) -> Result<()>;

/// 按 ID 获取单条消息
pub async fn get(db: &SqlitePool, id: &str) -> Result<MessageRow>;

/// 更新 assistant 占位消息为最终结果
pub async fn update_assistant_result(
    db: &SqlitePool,
    id: &str,
    content: &str,
    tool_calls_json: Option<&str>,
    citations_json: Option<&str>,
    tokens_used: Option<i64>,
) -> Result<()>;

/// 统计对话中活跃消息数
pub async fn count_active(db: &SqlitePool, conv_id: &str) -> Result<u32>;

/// 导出对话（返回当前活跃链路）
pub async fn export_active(
    db: &SqlitePool,
    conv_id: &str,
) -> Result<Vec<MessageRow>>;
```

---

## ChatService 还需要的辅助查询

```rust
/// 返回某条消息之前的当前活跃链路
pub async fn list_active_before(
    db: &SqlitePool,
    conv_id: &str,
    before_created_at: i64,
) -> Result<Vec<MessageRow>>;

/// 返回最后一条活跃消息的 id，用于新消息 parent_message_id
pub async fn find_last_active_message_id(
    db: &SqlitePool,
    conv_id: &str,
) -> Result<Option<String>>;
```

---

## 事务使用规范

> 以下操作必须在事务中执行，避免部分成功：

| 操作 | 涉及步骤 |
|------|---------|
| **编辑用户消息** | delete_from_created_at + insert(新用户消息) |
| **重新生成** | set_active_version(全设 false) + insert(新 assistant 消息) |
| **版本切换** | set_active_version 本身已是两条 UPDATE，用事务包裹 |
| **删除对话** | 依赖 CASCADE，不需要额外事务 |

```rust
// 事务示例（编辑用户消息）
let mut tx = db.begin().await?;
db::message::delete_from_created_at(&mut *tx, conv_id, cutoff).await?;
db::message::insert(&mut *tx, new_user_msg).await?;
tx.commit().await?;
```
