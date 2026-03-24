//! Rust 集成测试共用的测试辅助函数。

use buyu_lib::state::AppState;

/// 创建一份全新的测试应用状态。
pub async fn test_state() -> AppState {
    AppState::initialize_with_url("sqlite::memory:")
        .await
        .expect("初始化测试状态失败")
}
