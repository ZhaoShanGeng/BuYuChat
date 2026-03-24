//! 消息楼层顺序键生成工具。

use crate::{error::AppError, utils::ids::new_uuid_v7};

/// order_key 中使用的 user 楼层位置标记。
pub const USER_POSITION_TAG: u8 = 0;
/// order_key 中使用的 assistant 楼层位置标记。
pub const ASSISTANT_POSITION_TAG: u8 = 1;

/// 按文档格式生成可排序的消息顺序键。
pub fn build_order_key(timestamp_ms: i64, position_tag: u8) -> Result<String, AppError> {
    if position_tag > 9 {
        return Err(AppError::internal("position_tag must be between 0 and 9"));
    }

    let suffix = new_uuid_v7()
        .chars()
        .filter(|ch| *ch != '-')
        .take(8)
        .collect::<String>();

    Ok(format!("{timestamp_ms:016}-{position_tag}-{suffix}"))
}
