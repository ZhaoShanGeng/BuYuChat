use serde::{Deserialize, Serialize};

use crate::domain::common::Id;
use crate::domain::content::StoredContent;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryScopeType {
    Message,
    Node,
    NodeRange,
    Conversation,
    Summary,
}

impl SummaryScopeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Node => "node",
            Self::NodeRange => "node_range",
            Self::Conversation => "conversation",
            Self::Summary => "summary",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "message" => Ok(Self::Message),
            "node" => Ok(Self::Node),
            "node_range" => Ok(Self::NodeRange),
            "conversation" => Ok(Self::Conversation),
            "summary" => Ok(Self::Summary),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported summary scope type '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryKind {
    Brief,
    Compression,
    Note,
    Title,
    Custom,
}

impl SummaryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Brief => "brief",
            Self::Compression => "compression",
            Self::Note => "note",
            Self::Title => "title",
            Self::Custom => "custom",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "brief" => Ok(Self::Brief),
            "compression" => Ok(Self::Compression),
            "note" => Ok(Self::Note),
            "title" => Ok(Self::Title),
            "custom" => Ok(Self::Custom),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported summary kind '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryUsageScope {
    Viewer,
    Request,
    Both,
}

impl SummaryUsageScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Request => "request",
            Self::Both => "both",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "viewer" => Ok(Self::Viewer),
            "request" => Ok(Self::Request),
            "both" => Ok(Self::Both),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported summary usage scope '{value}'"
            ))),
        }
    }

    pub fn matches_request(self) -> bool {
        matches!(self, Self::Request | Self::Both)
    }

    pub fn matches_viewer(self) -> bool {
        matches!(self, Self::Viewer | Self::Both)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryTargetKind {
    MessageVersion,
    NodeRange,
    Conversation,
}

impl SummaryTargetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MessageVersion => "message_version",
            Self::NodeRange => "node_range",
            Self::Conversation => "conversation",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "message_version" => Ok(Self::MessageVersion),
            "node_range" => Ok(Self::NodeRange),
            "conversation" => Ok(Self::Conversation),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported summary target kind '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryActivationMode {
    Manual,
    Auto,
}

impl SummaryActivationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Auto => "auto",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "manual" => Ok(Self::Manual),
            "auto" => Ok(Self::Auto),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported summary activation mode '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryVersion {
    pub id: Id,
    pub summary_group_id: Id,
    pub version_index: i64,
    pub is_active: bool,
    pub content: StoredContent,
    pub generator_type: String,
    pub generator_preset_id: Option<Id>,
    pub workflow_run_id: Option<Id>,
    pub generation_run_id: Option<Id>,
    pub config_json: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryGroup {
    pub id: Id,
    pub conversation_id: Id,
    pub scope_type: SummaryScopeType,
    pub scope_message_version_id: Option<Id>,
    pub scope_start_node_id: Option<Id>,
    pub scope_end_node_id: Option<Id>,
    pub scope_summary_group_id: Option<Id>,
    pub summary_kind: SummaryKind,
    pub default_generator_preset_id: Option<Id>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub active_version: Option<SummaryVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarySource {
    pub id: Id,
    pub summary_group_id: Id,
    pub summary_version_id: Id,
    pub source_kind: String,
    pub source_message_version_id: Option<Id>,
    pub source_start_node_id: Option<Id>,
    pub source_end_node_id: Option<Id>,
    pub source_summary_version_id: Option<Id>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryUsage {
    pub id: Id,
    pub summary_group_id: Id,
    pub summary_version_id: Option<Id>,
    pub usage_scope: SummaryUsageScope,
    pub target_kind: SummaryTargetKind,
    pub target_message_version_id: Option<Id>,
    pub target_start_node_id: Option<Id>,
    pub target_end_node_id: Option<Id>,
    pub conversation_id: Option<Id>,
    pub activation_mode: SummaryActivationMode,
    pub replace_from_node_id: Option<Id>,
    pub replace_after_message_count: Option<i64>,
    pub replace_after_total_bytes: Option<i64>,
    pub enabled: bool,
    pub priority: i64,
    pub config_json: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSummaryUsageInput {
    pub usage_id: Option<Id>,
    pub summary_group_id: Id,
    pub summary_version_id: Option<Id>,
    pub usage_scope: SummaryUsageScope,
    pub target_kind: SummaryTargetKind,
    pub target_message_version_id: Option<Id>,
    pub target_start_node_id: Option<Id>,
    pub target_end_node_id: Option<Id>,
    pub conversation_id: Option<Id>,
    pub activation_mode: SummaryActivationMode,
    pub replace_from_node_id: Option<Id>,
    pub replace_after_message_count: Option<i64>,
    pub replace_after_total_bytes: Option<i64>,
    pub enabled: bool,
    pub priority: i64,
    pub config_json: serde_json::Value,
}
