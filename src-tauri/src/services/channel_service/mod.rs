//! 渠道服务入口与测试模块绑定。

mod connectivity;
mod crud;
mod validation;

pub use connectivity::{test_channel, test_with};
pub use crud::{
    create, create_with, delete, delete_with, get, get_with, list, list_with, update, update_with,
    Clock, SystemClock,
};

#[cfg(test)]
mod tests;
