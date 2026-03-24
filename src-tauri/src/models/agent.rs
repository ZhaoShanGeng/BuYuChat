//! Agent 领域模型与输入输出载荷定义。
//!
//! 这个文件承担三类职责：
//! 1. 定义 `agents` 表在后端内存中的映射结构，供 repo/service/command 共用。
//! 2. 定义 Agent 相关 API 的输入模型，覆盖创建与更新两个写操作。
//! 3. 定义仓储层内部使用的插入/补丁模型，避免 command 直接依赖数据库写入细节。
//!
//! 语义约束：
//! - `Agent` 是完整持久化资源，对应数据库中的一行记录。
//! - `CreateAgentInput` 面向外部调用，允许 `system_prompt` 为空，不提供内置默认提示词。
//! - `UpdateAgentInput` 使用补丁语义；其中 `system_prompt: Option<Option<String>>`
//!   用来区分“未提供该字段”和“显式清空该字段”。
//! - `NewAgent` / `AgentPatch` 只用于 repo 层，不直接暴露给前端。

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Agent 持久化资源。
///
/// 该结构与 `agents` 表基本一一对应，表示一个可被会话绑定的 Agent 定义。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromRow)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub system_prompt: Option<String>,
    pub avatar_uri: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 创建 Agent 的输入载荷。
///
/// 该结构对应 `POST /agents` 的请求体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateAgentInput {
    pub name: String,
    pub system_prompt: Option<String>,
}

/// 更新 Agent 的输入载荷。
///
/// 该结构对应 `PATCH /agents/{id}` 的请求体，所有字段均为可选补丁。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UpdateAgentInput {
    pub name: Option<String>,
    pub system_prompt: Option<Option<String>>,
    pub enabled: Option<bool>,
}

/// 仓储层插入 Agent 时使用的内部模型。
///
/// service 层会在完成校验、默认值填充和时间戳生成后构造这个结构交给 repo。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAgent {
    pub id: String,
    pub name: String,
    pub system_prompt: Option<String>,
    pub avatar_uri: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 仓储层更新 Agent 时使用的补丁模型。
///
/// 该结构与 `UpdateAgentInput` 类似，但追加了 repo 写库所需的 `updated_at`。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentPatch {
    pub name: Option<String>,
    pub system_prompt: Option<Option<String>>,
    pub enabled: Option<bool>,
    pub updated_at: i64,
}
