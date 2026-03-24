//! 模型服务的校验与归一化辅助函数。

use crate::error::AppError;

/// 校验模型标识在去除空白后不能为空。
pub fn validate_model_id(model_id: &str) -> Result<(), AppError> {
    if model_id.trim().is_empty() {
        return Err(AppError::validation(
            "VALIDATION_ERROR",
            "model_id cannot be empty",
        ));
    }

    Ok(())
}

/// 将可选文本归一化为“裁剪空白后为空则置空”。
pub fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}
