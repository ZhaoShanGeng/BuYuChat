use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};
use crate::domain::content::{ContentWriteInput, StoredContent};
use crate::support::error::{AppError, Result};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableValueType {
    String,
    Number,
    Boolean,
    Json,
    ContentRef,
}

impl VariableValueType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Json => "json",
            Self::ContentRef => "content_ref",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "string" => Ok(Self::String),
            "number" => Ok(Self::Number),
            "boolean" => Ok(Self::Boolean),
            "json" => Ok(Self::Json),
            "content_ref" => Ok(Self::ContentRef),
            _ => Err(AppError::Validation(format!(
                "unsupported variable value type '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableScopeType {
    Conversation,
    Node,
    MessageVersion,
    Agent,
    WorkflowRun,
    PluginScope,
}

impl VariableScopeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::Node => "node",
            Self::MessageVersion => "message_version",
            Self::Agent => "agent",
            Self::WorkflowRun => "workflow_run",
            Self::PluginScope => "plugin_scope",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "conversation" => Ok(Self::Conversation),
            "node" => Ok(Self::Node),
            "message_version" => Ok(Self::MessageVersion),
            "agent" => Ok(Self::Agent),
            "workflow_run" => Ok(Self::WorkflowRun),
            "plugin_scope" => Ok(Self::PluginScope),
            _ => Err(AppError::Validation(format!(
                "unsupported variable scope type '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDef {
    pub id: Id,
    pub var_key: String,
    pub name: String,
    pub value_type: VariableValueType,
    pub scope_type: VariableScopeType,
    pub namespace: String,
    pub is_user_editable: bool,
    pub is_plugin_editable: bool,
    pub ai_can_create: bool,
    pub ai_can_update: bool,
    pub ai_can_delete: bool,
    pub ai_can_lock: bool,
    pub ai_can_unlock_own_lock: bool,
    pub ai_can_unlock_any_lock: bool,
    pub default_json: serde_json::Value,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableValue {
    pub id: Id,
    pub variable_def_id: Id,
    pub scope_type: VariableScopeType,
    pub scope_id: Id,
    pub value_json: serde_json::Value,
    pub value_content: Option<StoredContent>,
    pub source_message_version_id: Option<Id>,
    pub updated_by_kind: String,
    pub updated_by_ref_id: Option<Id>,
    pub event_no: i64,
    pub is_deleted: bool,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableEventKind {
    Set,
    Delete,
    Restore,
}

impl VariableEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Delete => "delete",
            Self::Restore => "restore",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "set" => Ok(Self::Set),
            "delete" => Ok(Self::Delete),
            "restore" => Ok(Self::Restore),
            _ => Err(AppError::Validation(format!(
                "unsupported variable event kind '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableEvent {
    pub id: Id,
    pub variable_value_id: Id,
    pub event_no: i64,
    pub event_kind: VariableEventKind,
    pub value_json: serde_json::Value,
    pub value_content: Option<StoredContent>,
    pub source_message_version_id: Option<Id>,
    pub updated_by_kind: String,
    pub updated_by_ref_id: Option<Id>,
    pub created_at: TimestampMs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableLockKind {
    Update,
    Delete,
    All,
}

impl VariableLockKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Update => "update",
            Self::Delete => "delete",
            Self::All => "all",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            "all" => Ok(Self::All),
            _ => Err(AppError::Validation(format!(
                "unsupported variable lock kind '{value}'"
            ))),
        }
    }

    pub fn blocks(self, action: VariableActionKind) -> bool {
        matches!(self, Self::All)
            || matches!((self, action), (Self::Update, VariableActionKind::Update))
            || matches!((self, action), (Self::Delete, VariableActionKind::Delete))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableUnlockPolicy {
    Owner,
    UserOnly,
    Nobody,
}

impl VariableUnlockPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::UserOnly => "user_only",
            Self::Nobody => "nobody",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "owner" => Ok(Self::Owner),
            "user_only" => Ok(Self::UserOnly),
            "nobody" => Ok(Self::Nobody),
            _ => Err(AppError::Validation(format!(
                "unsupported variable unlock policy '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableLock {
    pub id: Id,
    pub variable_value_id: Id,
    pub lock_kind: VariableLockKind,
    pub owner_kind: String,
    pub owner_ref_id: Option<Id>,
    pub unlock_policy: VariableUnlockPolicy,
    pub active: bool,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableActionKind {
    Create,
    Update,
    Delete,
    Lock,
    Unlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVariableDefInput {
    pub var_key: String,
    pub name: String,
    pub value_type: VariableValueType,
    pub scope_type: VariableScopeType,
    pub namespace: String,
    pub is_user_editable: bool,
    pub is_plugin_editable: bool,
    pub ai_can_create: bool,
    pub ai_can_update: bool,
    pub ai_can_delete: bool,
    pub ai_can_lock: bool,
    pub ai_can_unlock_own_lock: bool,
    pub ai_can_unlock_any_lock: bool,
    pub default_json: serde_json::Value,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVariableDefInput {
    pub name: String,
    pub namespace: String,
    pub is_user_editable: bool,
    pub is_plugin_editable: bool,
    pub ai_can_create: bool,
    pub ai_can_update: bool,
    pub ai_can_delete: bool,
    pub ai_can_lock: bool,
    pub ai_can_unlock_own_lock: bool,
    pub ai_can_unlock_any_lock: bool,
    pub default_json: serde_json::Value,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetVariableValueInput {
    pub variable_def_id: Id,
    pub scope_type: VariableScopeType,
    pub scope_id: Id,
    pub value_json: serde_json::Value,
    pub value_content: Option<ContentWriteInput>,
    pub source_message_version_id: Option<Id>,
    pub updated_by_kind: String,
    pub updated_by_ref_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteVariableValueInput {
    pub variable_def_id: Id,
    pub scope_type: VariableScopeType,
    pub scope_id: Id,
    pub source_message_version_id: Option<Id>,
    pub updated_by_kind: String,
    pub updated_by_ref_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVariableLockInput {
    pub variable_def_id: Id,
    pub scope_type: VariableScopeType,
    pub scope_id: Id,
    pub lock_kind: VariableLockKind,
    pub owner_kind: String,
    pub owner_ref_id: Option<Id>,
    pub unlock_policy: VariableUnlockPolicy,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseVariableLockInput {
    pub variable_lock_id: Id,
    pub released_by_kind: String,
    pub released_by_ref_id: Option<Id>,
}
