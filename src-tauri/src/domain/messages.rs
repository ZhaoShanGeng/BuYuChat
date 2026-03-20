use serde::{Deserialize, Serialize};

use crate::domain::agents::AgentDetail;
use crate::domain::api_channels::{ApiChannel, ApiChannelModel};
use crate::domain::common::Id;
use crate::domain::content::{ContentWriteInput, StoredContent};
use crate::domain::lorebooks::{LorebookDetail, MatchedLorebookEntry};
use crate::domain::presets::{PresetDetail, PresetEntryDetail};
use crate::domain::user_profiles::UserProfileDetail;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "system" => Ok(Self::System),
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "tool" => Ok(Self::Tool),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported message role '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPolicy {
    Full,
    Summary,
    Exclude,
    Auto,
}

impl ContextPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Summary => "summary",
            Self::Exclude => "exclude",
            Self::Auto => "auto",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "full" => Ok(Self::Full),
            "summary" => Ok(Self::Summary),
            "exclude" => Ok(Self::Exclude),
            "auto" => Ok(Self::Auto),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported context policy '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerPolicy {
    Full,
    Summary,
    Placeholder,
    Hidden,
    Auto,
}

impl ViewerPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Summary => "summary",
            Self::Placeholder => "placeholder",
            Self::Hidden => "hidden",
            Self::Auto => "auto",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "full" => Ok(Self::Full),
            "summary" => Ok(Self::Summary),
            "placeholder" => Ok(Self::Placeholder),
            "hidden" => Ok(Self::Hidden),
            "auto" => Ok(Self::Auto),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported viewer policy '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentRefView {
    pub ref_id: Id,
    pub ref_role: String,
    pub plugin_id: Option<Id>,
    pub sort_order: i64,
    pub content: StoredContent,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageVersionView {
    pub node_id: Id,
    pub version_id: Id,
    pub conversation_id: Id,
    pub author_participant_id: Id,
    pub role: MessageRole,
    pub reply_to_node_id: Option<Id>,
    pub order_key: String,
    pub version_index: i64,
    pub is_active: bool,
    pub primary_content: StoredContent,
    pub content_refs: Vec<MessageContentRefView>,
    pub context_policy: ContextPolicy,
    pub viewer_policy: ViewerPolicy,
    pub api_channel_id: Option<Id>,
    pub api_channel_model_id: Option<Id>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub finish_reason: Option<String>,
    pub generation_run_id: Option<Id>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageInput {
    pub conversation_id: Id,
    pub author_participant_id: Id,
    pub role: MessageRole,
    pub reply_to_node_id: Option<Id>,
    pub order_after_node_id: Option<Id>,
    pub primary_content: ContentWriteInput,
    pub context_policy: ContextPolicy,
    pub viewer_policy: ViewerPolicy,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageVersionInput {
    pub node_id: Id,
    pub base_version_id: Id,
    pub primary_content: ContentWriteInput,
    pub context_policy: ContextPolicy,
    pub viewer_policy: ViewerPolicy,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAttachmentInput {
    pub message_version_id: Id,
    pub plugin_id: Option<Id>,
    pub ref_role: String,
    pub sort_order: i64,
    pub content: ContentWriteInput,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderMessagePartKind {
    Text,
    ImageRef,
    AudioRef,
    VideoRef,
    FileRef,
    JsonPayload,
    ToolRequest,
    ToolResponse,
    RagExcerpt,
    McpPayload,
    PluginPayload,
    ReasoningTrace,
    ProviderSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMessagePart {
    pub kind: ProviderMessagePartKind,
    pub text: Option<String>,
    pub content: Option<StoredContent>,
    pub metadata_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderChatMessage {
    pub role: MessageRole,
    pub name: Option<String>,
    pub parts: Vec<ProviderMessagePart>,
    pub metadata_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderChatRequest {
    pub api_channel: ApiChannel,
    pub api_channel_model: ApiChannelModel,
    pub request_parameters_json: serde_json::Value,
    pub messages: Vec<ProviderChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderChatResponse {
    pub parts: Vec<ProviderMessagePart>,
    pub finish_reason: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub raw_response_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub supports_streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderChatEvent {
    Delta {
        parts: Vec<ProviderMessagePart>,
        raw_event_json: Option<serde_json::Value>,
    },
    Finished {
        finish_reason: Option<String>,
        prompt_tokens: Option<i64>,
        completion_tokens: Option<i64>,
        total_tokens: Option<i64>,
        raw_event_json: Option<serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMaterial {
    pub sequence_no: i64,
    pub source_kind: String,
    pub message_version: Option<MessageVersionView>,
    pub summary_version_id: Option<Id>,
    pub summary_content: Option<StoredContent>,
    pub preset_entry: Option<PresetEntryDetail>,
    pub lorebook_entry: Option<MatchedLorebookEntry>,
    pub user_profile: Option<UserProfileDetail>,
    pub agent: Option<AgentDetail>,
    pub preset: Option<PresetDetail>,
    pub lorebook: Option<LorebookDetail>,
    pub plugin_content: Option<MessageContentRefView>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationContextItem {
    pub sequence_no: i64,
    pub send_role: MessageRole,
    pub rendered_content: StoredContent,
    pub source_kind: String,
    pub source_message_node_id: Option<Id>,
    pub source_message_version_id: Option<Id>,
    pub source_summary_version_id: Option<Id>,
    pub source_preset_entry_id: Option<Id>,
    pub source_lorebook_entry_id: Option<Id>,
    pub source_user_profile_id: Option<Id>,
    pub source_agent_id: Option<Id>,
    pub source_agent_greeting_id: Option<Id>,
    pub source_tool_invocation_id: Option<Id>,
    pub source_rag_ref_id: Option<Id>,
    pub source_mcp_event_id: Option<Id>,
    pub source_plugin_id: Option<Id>,
    pub included_in_request: bool,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltGenerationContext {
    pub conversation_id: Id,
    pub responder_participant_id: Id,
    pub trigger_node_id: Option<Id>,
    pub trigger_message_version_id: Option<Id>,
    pub api_channel: ApiChannel,
    pub api_channel_model: ApiChannelModel,
    pub request_parameters_json: serde_json::Value,
    pub preset_id: Option<Id>,
    pub preset_source_scope: Option<String>,
    pub lorebook_id: Option<Id>,
    pub lorebook_source_scope: Option<String>,
    pub user_profile_id: Option<Id>,
    pub user_profile_source_scope: Option<String>,
    pub api_channel_source_scope: Option<String>,
    pub api_channel_model_source_scope: Option<String>,
    pub items: Vec<GenerationContextItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildGenerationContextInput {
    pub conversation_id: Id,
    pub responder_participant_id: Id,
    pub trigger_message_version_id: Option<Id>,
    pub override_api_channel_id: Option<Id>,
    pub override_api_channel_model_id: Option<Id>,
    pub request_parameters_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReplyInput {
    pub conversation_id: Id,
    pub responder_participant_id: Id,
    pub trigger_message_version_id: Option<Id>,
    pub override_api_channel_id: Option<Id>,
    pub override_api_channel_model_id: Option<Id>,
    pub request_parameters_json: Option<serde_json::Value>,
    pub create_hidden_message: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReplyStreamInput {
    pub stream_id: Id,
    pub request: GenerateReplyInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateReplyInput {
    pub conversation_id: Id,
    pub responder_participant_id: Id,
    pub trigger_message_version_id: Id,
    pub override_api_channel_id: Option<Id>,
    pub override_api_channel_model_id: Option<Id>,
    pub request_parameters_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateReplyStreamInput {
    pub stream_id: Id,
    pub request: RegenerateReplyInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistGenerationSuccessInput {
    pub generation_run_id: Id,
    pub conversation_id: Id,
    pub responder_participant_id: Id,
    pub reply_to_node_id: Option<Id>,
    pub assistant_response: ProviderChatResponse,
    pub context_policy: ContextPolicy,
    pub viewer_policy: ViewerPolicy,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistGenerationFailureInput {
    pub generation_run_id: Id,
    pub error_text: String,
    pub response_payload_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStreamEventKind {
    Started,
    Delta,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStreamEvent {
    pub stream_id: Id,
    pub kind: GenerationStreamEventKind,
    pub delta_text: Option<String>,
    pub accumulated_text: Option<String>,
    pub message_version_id: Option<Id>,
    pub finish_reason: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub error_text: Option<String>,
}
