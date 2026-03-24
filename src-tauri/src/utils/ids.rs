//! 资源 ID 生成辅助函数。

use uuid::Uuid;

/// 生成适合持久化资源使用的 UUID v7 字符串。
pub fn new_uuid_v7() -> String {
    Uuid::now_v7().to_string()
}
