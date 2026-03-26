//! 会话领域模型与输入输出载荷定义。
//!
//! 这个文件覆盖会话管理模块的核心数据结构，主要面向以下场景：
//! 1. `Conversation` / `ConversationSummary` 作为查询结果，分别用于详情页与列表页。
//! 2. `CreateConversationInput` / `UpdateConversationInput` 作为 API 输入模型，对应创建和补丁更新。
//! 3. `NewConversation` / `ConversationPatch` 作为 repo 层内部模型，承接 service 校验后的写入数据。
//!
//! 设计要点：
//! - MVP 阶段会话直接内嵌 `agent_id`、`channel_id`、`channel_model_id`，不引入中间表。
//! - 列表接口只返回 `ConversationSummary`，避免把完整详情结构用于侧边栏等轻量场景。
//! - IPC 层显式携带 `*_set` 标记，避免 `null` 在 JSON/Tauri 序列化里与“字段缺失”
//!   混淆；repo 层仍使用 `Option<Option<String>>` 表达“保留原值 / 设为具体值 / 显式清空”。

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 会话详情资源。
///
/// 该结构对应 `GET /conversations/{id}` 的完整返回值。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromRow)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub agent_id: Option<String>,
    pub channel_id: Option<String>,
    pub channel_model_id: Option<String>,
    pub archived: bool,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 会话列表项。
///
/// 该结构用于 `GET /conversations`，只保留列表渲染和排序所需字段。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromRow)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub agent_id: Option<String>,
    pub channel_id: Option<String>,
    pub channel_model_id: Option<String>,
    pub archived: bool,
    pub pinned: bool,
    pub updated_at: i64,
}

/// 创建会话的输入载荷。
///
/// 该结构对应 `POST /conversations` 的请求体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CreateConversationInput {
    pub title: Option<String>,
    pub agent_id: Option<String>,
    pub channel_id: Option<String>,
    pub channel_model_id: Option<String>,
}

/// 更新会话的输入载荷。
///
/// 该结构对应 `PATCH /conversations/{id}` 的请求体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UpdateConversationInput {
    pub title: Option<String>,
    #[serde(default)]
    pub agent_id_set: bool,
    pub agent_id: Option<String>,
    #[serde(default)]
    pub channel_id_set: bool,
    pub channel_id: Option<String>,
    #[serde(default)]
    pub channel_model_id_set: bool,
    pub channel_model_id: Option<String>,
    pub archived: Option<bool>,
    pub pinned: Option<bool>,
}

/// 仓储层插入会话时使用的内部模型。
///
/// service 层会在校验绑定关系、补齐默认标题和时间戳之后构造该结构。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewConversation {
    pub id: String,
    pub title: String,
    pub agent_id: Option<String>,
    pub channel_id: Option<String>,
    pub channel_model_id: Option<String>,
    pub archived: bool,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 仓储层更新会话时使用的补丁模型。
///
/// 该结构在 `UpdateConversationInput` 的基础上补充了写库时必需的 `updated_at`。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConversationPatch {
    pub title: Option<String>,
    pub agent_id: Option<Option<String>>,
    pub channel_id: Option<Option<String>>,
    pub channel_model_id: Option<Option<String>>,
    pub archived: Option<bool>,
    pub pinned: Option<bool>,
    pub updated_at: i64,
}
