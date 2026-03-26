//! 消息楼层、消息版本、生成事件与相关输入输出模型。
//!
//! 这是后端 MVP 中信息密度最高的模型文件，集中描述了消息系统的三层结构：
//! 1. 楼层层：`MessageNode` / `MessageNodeRecord`，对应 `message_nodes` 表，表示会话中的一个位置实体。
//! 2. 版本层：`MessageVersion` / `VersionMeta`，对应 `message_versions` 表，表示同一楼层的多个版本。
//! 3. 内容层：`VersionContent` / `NewMessageContent`，对应 `message_contents` 表，负责分块存储实际文本。
//!
//! 同时，这个文件还定义了消息模块对外暴露的命令输入输出：
//! - 发送消息：`SendMessageInput`、`SendMessageResult`、`SendMessageResponse`
//! - Reroll：`RerollInput`、`RerollResult`
//! - 删除版本：`DeleteVersionResult`
//! - dry run：`DryRunResult`、`PromptMessage`
//! - 流式事件：`GenerationEvent`
//!
//! 设计约束：
//! - `MessageNode` / `MessageVersion` 面向前端返回，只有 active version 会内嵌 `content`。
//! - `VersionContent` 用于版本切换时按需加载完整文本。
//! - `SendMessageResponse` 是 `send_message` 的联合返回值，用于承载“正常发送”与“dry_run”两类结果。
//! - `NewMessageNode` / `NewMessageVersion` / `NewMessageContent` / `MessageVersionPatch`
//!   是 repo 层的内部写入模型，避免把数据库写入语义泄漏到命令层。

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 返回给前端的消息楼层资源。
///
/// 该结构对应 `GET /conversations/{id}/messages` 中的单个楼层对象。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageNode {
    pub id: String,
    pub conversation_id: String,
    pub author_agent_id: Option<String>,
    pub role: String,
    pub order_key: String,
    pub active_version_id: Option<String>,
    pub versions: Vec<MessageVersion>,
    pub created_at: i64,
}

/// 返回给前端的消息版本元数据。
///
/// 该结构描述某个楼层下的单个版本；非 active 版本的 `content` 为空。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageVersion {
    pub id: String,
    pub node_id: String,
    pub content: Option<String>,
    #[serde(default)]
    pub thinking_content: Option<String>,
    #[serde(default)]
    pub images: Vec<ImageAttachment>,
    pub status: String,
    pub model_name: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub finish_reason: Option<String>,
    pub created_at: i64,
}

/// 按需加载单个版本内容时返回的结果。
///
/// 该结构对应 `GET /versions/{versionId}/content` 的返回值。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionContent {
    pub version_id: String,
    pub content: String,
    pub content_type: String,
}

/// 发送消息的输入载荷。
///
/// 该结构对应 `POST /conversations/{id}/send` 的请求体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SendMessageInput {
    pub content: String,
    pub images: Option<Vec<ImageAttachment>>,
    pub stream: Option<bool>,
    pub dry_run: Option<bool>,
}

/// 发送消息成功后的即时返回值。
///
/// 后端在真正开始生成前，会先创建 user/assistant 两个楼层与对应版本，然后立即返回这些 ID。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SendMessageResult {
    pub user_node_id: String,
    pub user_version_id: String,
    pub assistant_node_id: String,
    pub assistant_version_id: String,
}

/// `send_message` 命令的联合返回值。
///
/// 该结构统一承载“dry_run 调试结果”和“正式发送后的 ID 返回值”。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum SendMessageResponse {
    DryRun(DryRunResult),
    Started(SendMessageResult),
}

/// dry run 的 prompt 调试结果。
///
/// 该结构用于把最终拼装出的 prompt 与目标模型直接返回给前端调试。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DryRunResult {
    pub messages: Vec<PromptMessage>,
    pub total_tokens_estimate: i64,
    pub model: String,
}

/// Reroll 的输入载荷。
///
/// 该结构对应 `POST /conversations/{id}/nodes/{nodeId}/reroll` 的请求体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RerollInput {
    pub stream: Option<bool>,
}

/// 编辑消息的输入载荷。
///
/// 该结构用于在原 node 下创建一个新的 committed version，并可选触发重新发送。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EditMessageInput {
    pub content: String,
    pub resend: Option<bool>,
    pub stream: Option<bool>,
}

/// Reroll 的即时返回值。
///
/// assistant reroll 与 user reroll 都复用这个结果结构。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RerollResult {
    pub new_user_version_id: Option<String>,
    pub assistant_node_id: String,
    pub assistant_version_id: String,
}

/// 编辑消息的即时返回值。
///
/// 若 `resend = false`，只返回新的已提交版本 ID。
/// 若 `resend = true`，则额外返回用于生成的新 assistant node/version。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditMessageResult {
    pub edited_version_id: String,
    pub assistant_node_id: Option<String>,
    pub assistant_version_id: Option<String>,
}

/// 删除版本的结果。
///
/// 该结构用于告诉前端删除后是否连楼层一起删除，以及 active version 是否发生切换。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteVersionResult {
    pub node_deleted: bool,
    pub new_active_version_id: Option<String>,
}

/// dry run 与 AI 调用共用的 prompt 消息。
///
/// 这是后端自己的轻量 prompt 结构，用于避免在 service 层直接依赖 AISDK 的消息类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub images: Vec<ImageAttachment>,
}

/// 通过 Tauri Channel 推送到前端的生成事件。
///
/// 该枚举统一描述流式文本增量、终态通知与空内容回滚事件。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GenerationEvent {
    Chunk {
        conversation_id: String,
        node_id: String,
        version_id: String,
        delta: String,
        reasoning_delta: Option<String>,
    },
    Completed {
        conversation_id: String,
        node_id: String,
        version_id: String,
        prompt_tokens: i64,
        completion_tokens: i64,
        finish_reason: String,
        model: String,
    },
    Failed {
        conversation_id: String,
        node_id: String,
        version_id: String,
        error: String,
    },
    Cancelled {
        conversation_id: String,
        node_id: String,
        version_id: String,
    },
    EmptyRollback {
        conversation_id: String,
        node_id: String,
        node_deleted: bool,
        fallback_version_id: Option<String>,
    },
}

/// 消息中的图片附件。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageAttachment {
    pub base64: String,
    pub mime_type: String,
}

/// `message_nodes` 表对应的持久化楼层记录。
///
/// 这个结构主要在 repo/service 内部使用，用于事务写入和精确查询。
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct MessageNodeRecord {
    pub id: String,
    pub conversation_id: String,
    pub author_agent_id: Option<String>,
    pub role: String,
    pub order_key: String,
    pub active_version_id: Option<String>,
    pub created_at: i64,
}

/// `message_versions` 表对应的版本元数据。
///
/// 这个结构主要在 repo/service 内部使用，用于状态流转和版本切换逻辑。
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct VersionMeta {
    pub id: String,
    pub node_id: String,
    pub status: String,
    pub model_name: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub finish_reason: Option<String>,
    pub created_at: i64,
}

/// 仓储层插入楼层时使用的内部模型。
///
/// 该结构只关心写入 `message_nodes` 所必需的数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMessageNode {
    pub id: String,
    pub conversation_id: String,
    pub author_agent_id: Option<String>,
    pub role: String,
    pub order_key: String,
    pub created_at: i64,
}

/// 仓储层插入版本时使用的内部模型。
///
/// service 层会在创建 user/assistant 版本或 reroll 版本时构造该结构。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMessageVersion {
    pub id: String,
    pub node_id: String,
    pub status: String,
    pub model_name: Option<String>,
    pub created_at: i64,
}

/// 仓储层插入内容块时使用的内部模型。
///
/// 该结构用于把完整文本拆分后的单个 chunk 追加到 `message_contents`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMessageContent {
    pub id: String,
    pub version_id: String,
    pub chunk_index: i64,
    pub content_type: String,
    pub body: String,
    pub created_at: i64,
}

/// 仓储层更新版本终态时使用的补丁模型。
///
/// 该结构封装了生成完成、失败、取消等终态写库时需要更新的元数据。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessageVersionPatch {
    pub status: Option<String>,
    pub prompt_tokens: Option<Option<i64>>,
    pub completion_tokens: Option<Option<i64>>,
    pub finish_reason: Option<Option<String>>,
    pub model_name: Option<Option<String>>,
}
