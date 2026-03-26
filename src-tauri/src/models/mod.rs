//! 命令层、服务层与仓储层共享的领域模型。

pub mod agent;
pub mod channel;
pub mod conversation;
pub mod message;
pub mod model;

pub use agent::{Agent, AgentPatch, CreateAgentInput, NewAgent, UpdateAgentInput};
pub use channel::{
    Channel, ChannelPatch, ChannelTestResult, CreateChannelInput, NewChannel, TestChannelRequest,
    UpdateChannelInput,
};
pub use conversation::{
    Conversation, ConversationPatch, ConversationSummary, CreateConversationInput, NewConversation,
    UpdateConversationInput,
};
pub use message::{
    DeleteVersionResult, DryRunResult, EditMessageInput, EditMessageResult, GenerationEvent,
    ImageAttachment, MessageNode, MessageNodeRecord, MessageVersion, MessageVersionPatch,
    NewMessageContent, NewMessageNode, NewMessageVersion, PromptMessage, RerollInput, RerollResult,
    SendMessageInput, SendMessageResponse, SendMessageResult, VersionContent, VersionMeta,
};
pub use model::{
    ChannelModel, ChannelModelPatch, CreateModelInput, NewChannelModel, RemoteModelInfo,
    UpdateModelInput,
};
