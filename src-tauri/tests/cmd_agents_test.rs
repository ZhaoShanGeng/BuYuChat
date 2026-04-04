//! Agent 命令层的集成测试。

mod helpers;

use buyu_lib::{
    commands::agents::{
        create_agent_impl, delete_agent_impl, get_agent_impl, list_agents_impl, update_agent_impl,
    },
    error::AppError,
    models::{CreateAgentInput, UpdateAgentInput},
};

/// Agent 命令应支持完整的 CRUD 流程。
#[tokio::test]
async fn test_agent_commands_cover_crud_flow() {
    let state = helpers::test_state().await;

    let created = create_agent_impl(
        &state,
        CreateAgentInput {
            name: "助手".to_string(),
            system_prompt: Some("你是一个有帮助的助手".to_string()),
        },
    )
    .await
    .unwrap();

    let listed = list_agents_impl(&state, None).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);

    let loaded = get_agent_impl(&state, created.id.clone()).await.unwrap();
    assert_eq!(loaded.name, "助手");

    let updated = update_agent_impl(
        &state,
        created.id.clone(),
        UpdateAgentInput {
            name: Some("新助手".to_string()),
            system_prompt: Some(Some("新的系统提示词".to_string())),
            enabled: Some(false),
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "新助手");
    assert!(!updated.enabled);

    delete_agent_impl(&state, created.id.clone()).await.unwrap();

    let error = get_agent_impl(&state, created.id.clone())
        .await
        .unwrap_err();
    assert_eq!(
        error,
        AppError::not_found(format!("agent '{}' not found", created.id))
    );
}

/// 创建空名称 Agent 时应返回结构化校验错误。
#[tokio::test]
async fn test_create_agent_rejects_empty_name() {
    let state = helpers::test_state().await;

    let error = create_agent_impl(
        &state,
        CreateAgentInput {
            name: "   ".to_string(),
            system_prompt: None,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(
        error,
        AppError::validation("VALIDATION_ERROR", "name must not be empty")
    );
}
