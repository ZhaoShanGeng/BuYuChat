//! 模型服务入口与测试模块绑定。

mod crud;
mod remote_fetch;
mod validation;

pub use crud::{
    create, create_with, delete, delete_with, list, list_with, update, update_with,
};
pub use remote_fetch::{fetch_remote_models, fetch_remote_models_with};

#[cfg(test)]
mod tests;
