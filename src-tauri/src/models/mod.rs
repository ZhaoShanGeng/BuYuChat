//! 命令层、服务层与仓储层共享的领域模型。

pub mod channel;
pub mod model;

pub use channel::{
    Channel, ChannelPatch, ChannelTestResult, CreateChannelInput, NewChannel, TestChannelRequest,
    UpdateChannelInput,
};
pub use model::{
    ChannelModel, ChannelModelPatch, CreateModelInput, NewChannelModel, RemoteModelInfo,
    UpdateModelInput,
};
