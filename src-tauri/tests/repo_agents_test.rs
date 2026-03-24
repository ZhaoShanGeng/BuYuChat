//! Agent 仓储层的 SQLite 集成测试。

mod helpers;

use buyu_lib::{
    models::{AgentPatch, NewAgent},
    repo::agent_repo::{AgentRepo, SqlxAgentRepo},
    utils::ids::new_uuid_v7,
};
use sqlx::Row;

/// 构造仓储层测试使用的 Agent 样本。
fn sample_new_agent(id: String, name: &str, created_at: i64) -> NewAgent {
    NewAgent {
        id,
        name: name.to_string(),
        system_prompt: Some("你是测试助手".to_string()),
        avatar_uri: None,
        enabled: true,
        created_at,
        updated_at: created_at,
    }
}

/// 插入 Agent 后应能按 ID 正常读取。
#[tokio::test]
async fn test_insert_agent_persists_and_can_be_loaded() {
    let state = helpers::test_state().await;
    let repo = SqlxAgentRepo::new(state.db.clone());

    let created = repo
        .insert(&sample_new_agent(new_uuid_v7(), "助手", 100))
        .await
        .unwrap();
    let loaded = repo.get(&created.id).await.unwrap().unwrap();

    assert_eq!(loaded.id, created.id);
    assert_eq!(loaded.name, "助手");
    assert_eq!(loaded.system_prompt.as_deref(), Some("你是测试助手"));
}

/// Agent 列表应按 created_at 倒序排列，并支持过滤禁用项。
#[tokio::test]
async fn test_list_agents_orders_by_created_at_and_filters_disabled() {
    let state = helpers::test_state().await;
    let repo = SqlxAgentRepo::new(state.db.clone());

    repo.insert(&sample_new_agent(new_uuid_v7(), "旧助手", 100))
        .await
        .unwrap();
    let disabled = repo
        .insert(&sample_new_agent(new_uuid_v7(), "新助手", 200))
        .await
        .unwrap();
    repo.update(
        &disabled.id,
        &AgentPatch {
            enabled: Some(false),
            updated_at: 300,
            ..AgentPatch::default()
        },
    )
    .await
    .unwrap();

    let all_agents = repo.list(true).await.unwrap();
    let enabled_agents = repo.list(false).await.unwrap();

    assert_eq!(all_agents[0].name, "新助手");
    assert_eq!(enabled_agents.len(), 1);
    assert_eq!(enabled_agents[0].name, "旧助手");
}

/// 删除 Agent 后，会话上的 agent_id 应被置空。
#[tokio::test]
async fn test_delete_agent_sets_conversation_binding_to_null() {
    let state = helpers::test_state().await;
    let repo = SqlxAgentRepo::new(state.db.clone());
    let created = repo
        .insert(&sample_new_agent(new_uuid_v7(), "助手", 100))
        .await
        .unwrap();
    let conversation_id = new_uuid_v7();

    sqlx::query(
        r#"
        INSERT INTO conversations (id, title, agent_id, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(&conversation_id)
    .bind("测试会话")
    .bind(&created.id)
    .bind(100_i64)
    .bind(100_i64)
    .execute(&state.db)
    .await
    .unwrap();

    repo.delete(&created.id).await.unwrap();

    let row = sqlx::query("SELECT agent_id FROM conversations WHERE id = ?1")
        .bind(&conversation_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    let agent_id: Option<String> = row.try_get("agent_id").unwrap();

    assert_eq!(agent_id, None);
}
