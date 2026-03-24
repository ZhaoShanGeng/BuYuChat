//! 模型管理相关的请求与响应模型。

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 返回给前端的持久化模型资源。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromRow)]
pub struct ChannelModel {
    /// 模型记录 ID。
    pub id: String,
    /// 所属渠道 ID。
    pub channel_id: String,
    /// 实际调用时使用的模型标识。
    pub model_id: String,
    /// 用户可读的展示名称。
    pub display_name: Option<String>,
    /// 上下文窗口大小。
    pub context_window: Option<i64>,
    /// 最大输出 token 数。
    pub max_output_tokens: Option<i64>,
}

/// 创建模型时使用的输入载荷。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateModelInput {
    /// 实际调用时使用的模型标识。
    pub model_id: String,
    /// 用户可读的展示名称。
    pub display_name: Option<String>,
    /// 上下文窗口大小。
    pub context_window: Option<i64>,
    /// 最大输出 token 数。
    pub max_output_tokens: Option<i64>,
}

/// 更新模型时使用的输入载荷。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UpdateModelInput {
    /// 更新后的展示名称；`Some(None)` 表示显式清空。
    pub display_name: Option<Option<String>>,
    /// 更新后的上下文窗口；`Some(None)` 表示显式清空。
    pub context_window: Option<Option<i64>>,
    /// 更新后的最大输出 token；`Some(None)` 表示显式清空。
    pub max_output_tokens: Option<Option<i64>>,
}

/// 仓储层插入模型时使用的内部模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewChannelModel {
    /// 模型记录 ID。
    pub id: String,
    /// 所属渠道 ID。
    pub channel_id: String,
    /// 实际调用时使用的模型标识。
    pub model_id: String,
    /// 用户可读的展示名称。
    pub display_name: Option<String>,
    /// 上下文窗口大小。
    pub context_window: Option<i64>,
    /// 最大输出 token 数。
    pub max_output_tokens: Option<i64>,
}

/// 仓储层更新模型时使用的内部补丁模型。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelModelPatch {
    /// 更新后的展示名称；`Some(None)` 表示显式清空。
    pub display_name: Option<Option<String>>,
    /// 更新后的上下文窗口；`Some(None)` 表示显式清空。
    pub context_window: Option<Option<i64>>,
    /// 更新后的最大输出 token；`Some(None)` 表示显式清空。
    pub max_output_tokens: Option<Option<i64>>,
}

/// 从远程渠道读取到的模型元信息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteModelInfo {
    /// 远程返回的模型标识。
    pub model_id: String,
    /// 远程返回的展示名称。
    pub display_name: Option<String>,
    /// 远程返回的上下文窗口大小。
    pub context_window: Option<i64>,
}
