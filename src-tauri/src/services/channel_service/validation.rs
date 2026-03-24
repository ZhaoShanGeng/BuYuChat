//! 渠道服务的校验与默认值解析辅助函数。

use crate::{
    channel_types::{config_for, ChannelTypeConfig},
    error::AppError,
};

/// 校验渠道名称在去除空白后不能为空。
pub fn validate_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::validation(
            "NAME_EMPTY",
            "channel name cannot be empty",
        ));
    }
    Ok(())
}

/// 校验根地址必须以 http:// 或 https:// 开头。
pub fn validate_base_url(base_url: &str) -> Result<(), AppError> {
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err(AppError::validation(
            "INVALID_URL",
            "base_url must start with http:// or https://",
        ));
    }
    Ok(())
}

/// 解析渠道类型对应的默认接口与鉴权配置。
pub fn resolve_config(channel_type: Option<&str>) -> Result<ChannelTypeConfig, AppError> {
    config_for(channel_type.unwrap_or("openai_compatible"))
}
