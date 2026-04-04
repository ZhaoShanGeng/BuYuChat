//! 渠道管理相关的请求与响应模型。

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 返回给前端的持久化渠道资源。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromRow)]
pub struct Channel {
    /// 渠道 ID。
    pub id: String,
    /// 渠道名称。
    pub name: String,
    /// 渠道类型。
    pub channel_type: String,
    /// 渠道根地址。
    pub base_url: String,
    /// 鉴权使用的 API Key。
    pub api_key: Option<String>,
    /// 轮换使用的多个 API Key，JSON 数组字符串。
    pub api_keys: Option<String>,
    /// 对外请求使用的鉴权方式。
    pub auth_type: Option<String>,
    /// 模型列表与连通性测试使用的接口。
    pub models_endpoint: Option<String>,
    /// 非流式聊天接口。
    pub chat_endpoint: Option<String>,
    /// 流式聊天接口。
    pub stream_endpoint: Option<String>,
    /// 自定义 thinking 标签配置，JSON 数组字符串。
    pub thinking_tags: Option<String>,
    /// 渠道是否启用。
    pub enabled: bool,
    /// 创建时间（毫秒时间戳）。
    pub created_at: i64,
    /// 更新时间（毫秒时间戳）。
    pub updated_at: i64,
}

/// 创建渠道的输入载荷。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateChannelInput {
    /// 渠道名称。
    pub name: String,
    /// 渠道根地址。
    pub base_url: String,
    /// 渠道类型覆盖值。
    pub channel_type: Option<String>,
    /// API Key 覆盖值。
    pub api_key: Option<String>,
    /// 轮换 API Key 覆盖值，JSON 数组字符串。
    pub api_keys: Option<String>,
    /// 鉴权方式覆盖值。
    pub auth_type: Option<String>,
    /// 模型列表接口覆盖值。
    pub models_endpoint: Option<String>,
    /// 聊天接口覆盖值。
    pub chat_endpoint: Option<String>,
    /// 流式接口覆盖值。
    pub stream_endpoint: Option<String>,
    /// thinking 标签配置，JSON 数组字符串。
    pub thinking_tags: Option<String>,
    /// 启用状态覆盖值。
    pub enabled: Option<bool>,
}

/// 渠道部分更新的输入载荷。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UpdateChannelInput {
    /// 更新后的渠道名称。
    pub name: Option<String>,
    /// 更新后的渠道根地址。
    pub base_url: Option<String>,
    /// 更新后的渠道类型。
    pub channel_type: Option<String>,
    /// 更新后的 API Key。
    pub api_key: Option<String>,
    /// 更新后的轮换 API Key。
    pub api_keys: Option<String>,
    /// 更新后的鉴权方式。
    pub auth_type: Option<String>,
    /// 更新后的模型列表接口。
    pub models_endpoint: Option<String>,
    /// 更新后的聊天接口。
    pub chat_endpoint: Option<String>,
    /// 更新后的流式接口。
    pub stream_endpoint: Option<String>,
    /// 更新后的 thinking 标签配置。
    pub thinking_tags: Option<String>,
    /// 更新后的启用状态。
    pub enabled: Option<bool>,
}

/// 仓储层插入渠道时使用的内部模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewChannel {
    /// 渠道 ID。
    pub id: String,
    /// 渠道名称。
    pub name: String,
    /// 渠道类型。
    pub channel_type: String,
    /// 渠道根地址。
    pub base_url: String,
    /// 鉴权使用的 API Key。
    pub api_key: Option<String>,
    /// 轮换使用的多个 API Key，JSON 数组字符串。
    pub api_keys: Option<String>,
    /// 对外请求使用的鉴权方式。
    pub auth_type: Option<String>,
    /// 模型列表与连通性测试使用的接口。
    pub models_endpoint: Option<String>,
    /// 非流式聊天接口。
    pub chat_endpoint: Option<String>,
    /// 流式聊天接口。
    pub stream_endpoint: Option<String>,
    /// thinking 标签配置，JSON 数组字符串。
    pub thinking_tags: Option<String>,
    /// 渠道是否启用。
    pub enabled: bool,
    /// 创建时间（毫秒时间戳）。
    pub created_at: i64,
    /// 更新时间（毫秒时间戳）。
    pub updated_at: i64,
}

/// 仓储层部分更新时使用的内部模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPatch {
    /// 更新后的渠道名称。
    pub name: Option<String>,
    /// 更新后的渠道根地址。
    pub base_url: Option<String>,
    /// 更新后的渠道类型。
    pub channel_type: Option<String>,
    /// 更新后的 API Key。
    pub api_key: Option<String>,
    /// 更新后的轮换 API Key。
    pub api_keys: Option<String>,
    /// 更新后的鉴权方式。
    pub auth_type: Option<String>,
    /// 更新后的模型列表接口。
    pub models_endpoint: Option<String>,
    /// 更新后的聊天接口。
    pub chat_endpoint: Option<String>,
    /// 更新后的流式接口。
    pub stream_endpoint: Option<String>,
    /// 更新后的 thinking 标签配置。
    pub thinking_tags: Option<String>,
    /// 更新后的启用状态。
    pub enabled: Option<bool>,
    /// 更新后的时间戳。
    pub updated_at: i64,
}

/// `test_channel` 返回的连通性结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelTestResult {
    /// 连通性测试是否成功。
    pub success: bool,
    /// 可选的状态消息。
    pub message: Option<String>,
}

/// 归一化后的连通性探测请求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestChannelRequest {
    /// 最终探测 URL。
    pub url: String,
    /// 可选的鉴权请求头键值。
    pub auth_header: Option<(String, String)>,
}
