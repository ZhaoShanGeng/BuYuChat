//! 消息楼层、版本与内容块的仓储访问实现。
//!
//! 该文件直接映射消息系统的三层数据库结构：
//! - `message_nodes`：会话中的楼层位置，记录角色、排序键与当前 active version。
//! - `message_versions`：同一楼层下的多版本元数据，承载状态机与 token 统计。
//! - `message_contents`：版本正文的分块存储，支持流式追加与按需拼接。
//!
//! 对外主要提供两类能力：
//! 1. 查询聚合：读取消息列表、按需加载版本内容、构建 AI prompt 上下文。
//! 2. 事务辅助：供 service 层在单事务里创建楼层、版本、chunk、active 指针与更新时间。
//!
//! 这里刻意不做业务错误码翻译，repo 只返回数据库层面的成功/失败结果。

use std::collections::HashMap;

use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool, Transaction};

use crate::models::{
    ImageAttachment, MessageNode, MessageNodeRecord, MessageVersion, MessageVersionPatch,
    NewMessageContent, NewMessageNode, NewMessageVersion, PromptMessage, VersionContent,
    VersionMeta,
};

/// 消息聚合查询使用的中间行结构。
#[derive(Debug, Clone, sqlx::FromRow)]
struct MessageNodeVersionRow {
    node_id: String,
    conversation_id: String,
    author_agent_id: Option<String>,
    role: String,
    order_key: String,
    active_version_id: Option<String>,
    node_created_at: i64,
    version_id: Option<String>,
    status: Option<String>,
    model_name: Option<String>,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    finish_reason: Option<String>,
    version_created_at: Option<i64>,
}

/// 版本正文拼接时使用的中间行结构。
#[derive(Debug, Clone, sqlx::FromRow)]
struct ContentRow {
    version_id: String,
    content_type: String,
    body: String,
}

#[derive(Debug, Clone, Default)]
struct AggregatedVersionContent {
    text: String,
    thinking: String,
    images: Vec<ImageAttachment>,
}

/// 判断指定会话是否存在。
pub async fn conversation_exists(pool: &SqlitePool, conversation_id: &str) -> Result<bool, String> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM conversations WHERE id = ?1")
        .bind(conversation_id)
        .fetch_one(pool)
        .await
        .map(|count| count > 0)
        .map_err(|error| error.to_string())
}

/// 按会话读取消息楼层列表。
///
/// 该查询返回所有 node 及其全部 version 元数据，但只有 active version 会拼接正文。
/// `before_order_key` 与 `limit` 目前主要用于预留分页签名，即使 MVP 只做基础查询，
/// 这里也按文档要求保留了过滤能力。
pub async fn list_messages(
    pool: &SqlitePool,
    conversation_id: &str,
    before_order_key: Option<&str>,
    limit: Option<i64>,
    from_latest: bool,
) -> Result<Option<Vec<MessageNode>>, String> {
    if !conversation_exists(pool, conversation_id).await? {
        return Ok(None);
    }

    let descending_page = before_order_key.is_some() || from_latest;

    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        WITH selected_nodes AS (
            SELECT *
            FROM (
                SELECT
                    id,
                    conversation_id,
                    author_agent_id,
                    role,
                    order_key,
                    active_version_id,
                    created_at
                FROM message_nodes
                WHERE conversation_id =
        "#,
    );
    query.push_bind(conversation_id);

    if let Some(before_order_key) = before_order_key {
        query.push(" AND order_key < ");
        query.push_bind(before_order_key);
    }

    query.push(" ORDER BY order_key ");
    query.push(if descending_page { "DESC" } else { "ASC" });
    query.push(" LIMIT ");
    query.push_bind(limit.unwrap_or(200));
    query.push(
        r#"
            ) paged_nodes
            ORDER BY order_key ASC
        )
        SELECT
            n.id AS node_id,
            n.conversation_id,
            n.author_agent_id,
            n.role,
            n.order_key,
            n.active_version_id,
            n.created_at AS node_created_at,
            v.id AS version_id,
            v.status,
            v.model_name,
            v.prompt_tokens,
            v.completion_tokens,
            v.finish_reason,
            v.created_at AS version_created_at
        FROM selected_nodes n
        LEFT JOIN message_versions v ON v.node_id = n.id
        ORDER BY n.order_key ASC, v.created_at ASC, v.id ASC
        "#,
    );

    let rows = query
        .build_query_as::<MessageNodeVersionRow>()
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?;

    let active_version_ids = rows
        .iter()
        .filter_map(|row| row.active_version_id.clone())
        .collect::<Vec<_>>();
    let active_contents = load_contents_map(pool, &active_version_ids).await?;

    let mut nodes = Vec::new();
    let mut indices = HashMap::new();

    for row in rows {
        let index = if let Some(index) = indices.get(&row.node_id).copied() {
            index
        } else {
            let index = nodes.len();
            nodes.push(MessageNode {
                id: row.node_id.clone(),
                conversation_id: row.conversation_id.clone(),
                author_agent_id: row.author_agent_id.clone(),
                role: row.role.clone(),
                order_key: row.order_key.clone(),
                active_version_id: row.active_version_id.clone(),
                versions: Vec::new(),
                created_at: row.node_created_at,
            });
            indices.insert(row.node_id.clone(), index);
            index
        };

        if let Some(version_id) = row.version_id {
            let is_active = nodes[index].active_version_id.as_deref() == Some(&version_id);
            nodes[index].versions.push(MessageVersion {
                id: version_id.clone(),
                node_id: row.node_id.clone(),
                content: if is_active {
                    active_contents
                        .get(&version_id)
                        .and_then(|content| (!content.text.is_empty()).then(|| content.text.clone()))
                } else {
                    None
                },
                thinking_content: if is_active {
                    active_contents.get(&version_id).and_then(|content| {
                        (!content.thinking.is_empty()).then(|| content.thinking.clone())
                    })
                } else {
                    None
                },
                images: if is_active {
                    active_contents
                        .get(&version_id)
                        .map(|content| content.images.clone())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                },
                status: row.status.unwrap_or_else(|| "committed".to_string()),
                model_name: row.model_name,
                prompt_tokens: row.prompt_tokens,
                completion_tokens: row.completion_tokens,
                finish_reason: row.finish_reason,
                created_at: row.version_created_at.unwrap_or(row.node_created_at),
            });
        }
    }

    Ok(Some(nodes))
}

/// 拼接指定版本的所有内容块，返回完整正文。
pub async fn get_version_content(
    pool: &SqlitePool,
    version_id: &str,
) -> Result<Option<VersionContent>, String> {
    let version_exists =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM message_versions WHERE id = ?1")
            .bind(version_id)
            .fetch_one(pool)
            .await
            .map_err(|error| error.to_string())?
            > 0;

    if !version_exists {
        return Ok(None);
    }

    let rows = sqlx::query_as::<_, ContentRow>(
        r#"
        SELECT version_id, content_type, body
        FROM message_contents
        WHERE version_id = ?1
          AND content_type = 'text/plain'
        ORDER BY chunk_index ASC
        "#,
    )
    .bind(version_id)
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?;

    let content = rows.iter().map(|row| row.body.as_str()).collect::<String>();

    Ok(Some(VersionContent {
        version_id: version_id.to_string(),
        content,
        content_type: "text/plain".to_string(),
    }))
}

/// 读取指定版本的全部图片附件。
pub async fn get_version_images(
    pool: &SqlitePool,
    version_id: &str,
) -> Result<Vec<ImageAttachment>, String> {
    let rows = sqlx::query_as::<_, ContentRow>(
        r#"
        SELECT version_id, content_type, body
        FROM message_contents
        WHERE version_id = ?1
          AND content_type = 'image/base64'
        ORDER BY chunk_index ASC
        "#,
    )
    .bind(version_id)
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?;

    rows.into_iter()
        .map(|row| serde_json::from_str::<ImageAttachment>(&row.body).map_err(|error| error.to_string()))
        .collect()
}

/// 按会话与楼层 ID 读取单个楼层记录。
pub async fn get_node_record(
    pool: &SqlitePool,
    conversation_id: &str,
    node_id: &str,
) -> Result<Option<MessageNodeRecord>, String> {
    sqlx::query_as::<_, MessageNodeRecord>(
        "SELECT * FROM message_nodes WHERE conversation_id = ?1 AND id = ?2",
    )
    .bind(conversation_id)
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())
}

/// 按版本 ID 读取版本元数据。
pub async fn get_version_meta(
    pool: &SqlitePool,
    version_id: &str,
) -> Result<Option<VersionMeta>, String> {
    sqlx::query_as::<_, VersionMeta>("SELECT * FROM message_versions WHERE id = ?1")
        .bind(version_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())
}

/// 通过版本 ID 反查所属楼层记录。
pub async fn get_node_record_by_version(
    pool: &SqlitePool,
    version_id: &str,
) -> Result<Option<MessageNodeRecord>, String> {
    sqlx::query_as::<_, MessageNodeRecord>(
        r#"
        SELECT n.*
        FROM message_nodes n
        JOIN message_versions v ON v.node_id = n.id
        WHERE v.id = ?1
        "#,
    )
    .bind(version_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())
}

/// 读取指定楼层下的全部版本元数据。
pub async fn list_versions_for_node(
    pool: &SqlitePool,
    node_id: &str,
) -> Result<Vec<VersionMeta>, String> {
    sqlx::query_as::<_, VersionMeta>(
        "SELECT * FROM message_versions WHERE node_id = ?1 ORDER BY created_at ASC, id ASC",
    )
    .bind(node_id)
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())
}

/// 读取会话中的最后一个楼层。
pub async fn get_last_node(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Option<MessageNodeRecord>, String> {
    sqlx::query_as::<_, MessageNodeRecord>(
        r#"
        SELECT *
        FROM message_nodes
        WHERE conversation_id = ?1
        ORDER BY order_key DESC
        LIMIT 1
        "#,
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())
}

/// 读取指定楼层在会话中的紧邻后继楼层。
pub async fn get_next_node(
    pool: &SqlitePool,
    conversation_id: &str,
    order_key: &str,
) -> Result<Option<MessageNodeRecord>, String> {
    sqlx::query_as::<_, MessageNodeRecord>(
        r#"
        SELECT *
        FROM message_nodes
        WHERE conversation_id = ?1
          AND order_key > ?2
        ORDER BY order_key ASC
        LIMIT 1
        "#,
    )
    .bind(conversation_id)
    .bind(order_key)
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())
}

/// 读取会话中的全部楼层记录。
pub async fn list_node_records(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<MessageNodeRecord>, String> {
    sqlx::query_as::<_, MessageNodeRecord>(
        r#"
        SELECT *
        FROM message_nodes
        WHERE conversation_id = ?1
        ORDER BY order_key ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())
}

/// 读取某个楼层当前 active version 的完整正文。
pub async fn get_active_version_content_for_node(
    pool: &SqlitePool,
    node: &MessageNodeRecord,
) -> Result<Option<VersionContent>, String> {
    match node.active_version_id.as_deref() {
        Some(version_id) => get_version_content(pool, version_id).await,
        None => Ok(None),
    }
}

/// 构建 AI 请求所需的 prompt 消息。
///
/// 查询只读取 active version 的 chunk，并允许通过 `before_order_key` 截断上下文，
/// 用于实现 send_message 与 reroll 的不同上下文规则。
pub async fn build_prompt_messages(
    pool: &SqlitePool,
    conversation_id: &str,
    before_order_key: Option<&str>,
    excluded_node_id: Option<&str>,
) -> Result<Vec<PromptMessage>, String> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT n.id AS node_id, n.role, c.body
        , c.content_type
        FROM message_nodes n
        JOIN message_contents c ON c.version_id = n.active_version_id
        WHERE n.conversation_id = 
        "#,
    );
    query.push_bind(conversation_id);

    if let Some(before_order_key) = before_order_key {
        query.push(" AND n.order_key < ");
        query.push_bind(before_order_key);
    }

    if let Some(excluded_node_id) = excluded_node_id {
        query.push(" AND n.id != ");
        query.push_bind(excluded_node_id);
    }

    query.push(" ORDER BY n.order_key ASC, c.chunk_index ASC");

    let rows = query
        .build()
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?;

    let mut prompts = Vec::new();
    let mut current_node_id = String::new();
    let mut current_role = String::new();
    let mut current_body = String::new();
    let mut current_images: Vec<ImageAttachment> = Vec::new();

    for row in rows {
        let node_id = row
            .try_get::<String, _>("node_id")
            .map_err(|error| error.to_string())?;
        let role = row
            .try_get::<String, _>("role")
            .map_err(|error| error.to_string())?;
        let body = row
            .try_get::<String, _>("body")
            .map_err(|error| error.to_string())?;
        let content_type = row
            .try_get::<String, _>("content_type")
            .map_err(|error| error.to_string())?;

        if current_node_id.is_empty() {
            current_node_id = node_id.clone();
            current_role = role.clone();
        }

        if current_node_id != node_id {
            prompts.push(PromptMessage {
                role: current_role.clone(),
                content: current_body.clone(),
                images: current_images.clone(),
            });
            current_node_id = node_id.clone();
            current_role = role.clone();
            current_body.clear();
            current_images.clear();
        }

        match content_type.as_str() {
            "text/plain" => current_body.push_str(&body),
            "image/base64" => current_images
                .push(serde_json::from_str::<ImageAttachment>(&body).map_err(|error| error.to_string())?),
            "text/thinking" => {}
            _ => {}
        }
    }

    if !current_body.is_empty() || !current_images.is_empty() {
        prompts.push(PromptMessage {
            role: current_role,
            content: current_body,
            images: current_images,
        });
    }

    Ok(prompts)
}

/// 在事务中插入楼层记录。
pub async fn insert_node_tx(
    tx: &mut Transaction<'_, Sqlite>,
    new_node: &NewMessageNode,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO message_nodes (
            id, conversation_id, author_agent_id, role, order_key, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(&new_node.id)
    .bind(&new_node.conversation_id)
    .bind(&new_node.author_agent_id)
    .bind(&new_node.role)
    .bind(&new_node.order_key)
    .bind(new_node.created_at)
    .execute(&mut **tx)
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

/// 在事务中插入版本记录。
pub async fn insert_version_tx(
    tx: &mut Transaction<'_, Sqlite>,
    new_version: &NewMessageVersion,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO message_versions (
            id, node_id, status, model_name, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(&new_version.id)
    .bind(&new_version.node_id)
    .bind(&new_version.status)
    .bind(&new_version.model_name)
    .bind(new_version.created_at)
    .execute(&mut **tx)
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

/// 在事务中插入一个内容块。
pub async fn insert_content_tx(
    tx: &mut Transaction<'_, Sqlite>,
    new_content: &NewMessageContent,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO message_contents (
            id, version_id, chunk_index, content_type, body, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(&new_content.id)
    .bind(&new_content.version_id)
    .bind(new_content.chunk_index)
    .bind(&new_content.content_type)
    .bind(&new_content.body)
    .bind(new_content.created_at)
    .execute(&mut **tx)
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

/// 在事务中更新楼层的 active version。
pub async fn set_node_active_version_tx(
    tx: &mut Transaction<'_, Sqlite>,
    node_id: &str,
    active_version_id: Option<&str>,
) -> Result<(), String> {
    sqlx::query("UPDATE message_nodes SET active_version_id = ?1 WHERE id = ?2")
        .bind(active_version_id)
        .bind(node_id)
        .execute(&mut **tx)
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

/// 在事务中更新楼层的顺序键。
pub async fn update_node_order_key_tx(
    tx: &mut Transaction<'_, Sqlite>,
    node_id: &str,
    order_key: &str,
) -> Result<(), String> {
    sqlx::query("UPDATE message_nodes SET order_key = ?1 WHERE id = ?2")
        .bind(order_key)
        .bind(node_id)
        .execute(&mut **tx)
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

/// 在事务中更新版本元数据。
pub async fn update_version_tx(
    tx: &mut Transaction<'_, Sqlite>,
    version_id: &str,
    patch: &MessageVersionPatch,
) -> Result<bool, String> {
    let prompt_tokens_provided = patch.prompt_tokens.is_some();
    let completion_tokens_provided = patch.completion_tokens.is_some();
    let finish_reason_provided = patch.finish_reason.is_some();
    let model_name_provided = patch.model_name.is_some();

    sqlx::query(
        r#"
        UPDATE message_versions
        SET
            status = COALESCE(?1, status),
            prompt_tokens = CASE WHEN ?2 THEN ?3 ELSE prompt_tokens END,
            completion_tokens = CASE WHEN ?4 THEN ?5 ELSE completion_tokens END,
            finish_reason = CASE WHEN ?6 THEN ?7 ELSE finish_reason END,
            model_name = CASE WHEN ?8 THEN ?9 ELSE model_name END
        WHERE id = ?10
        "#,
    )
    .bind(&patch.status)
    .bind(prompt_tokens_provided)
    .bind(patch.prompt_tokens.flatten())
    .bind(completion_tokens_provided)
    .bind(patch.completion_tokens.flatten())
    .bind(finish_reason_provided)
    .bind(patch.finish_reason.clone().flatten())
    .bind(model_name_provided)
    .bind(patch.model_name.clone().flatten())
    .bind(version_id)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected() > 0)
    .map_err(|error| error.to_string())
}

/// 在事务中删除版本。
pub async fn delete_version_tx(
    tx: &mut Transaction<'_, Sqlite>,
    version_id: &str,
) -> Result<bool, String> {
    sqlx::query("DELETE FROM message_versions WHERE id = ?1")
        .bind(version_id)
        .execute(&mut **tx)
        .await
        .map(|result| result.rows_affected() > 0)
        .map_err(|error| error.to_string())
}

/// 在事务中删除楼层。
pub async fn delete_node_tx(
    tx: &mut Transaction<'_, Sqlite>,
    node_id: &str,
) -> Result<bool, String> {
    sqlx::query("DELETE FROM message_nodes WHERE id = ?1")
        .bind(node_id)
        .execute(&mut **tx)
        .await
        .map(|result| result.rows_affected() > 0)
        .map_err(|error| error.to_string())
}

/// 在事务中更新会话的 `updated_at`。
pub async fn touch_conversation_updated_at_tx(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    updated_at: i64,
) -> Result<(), String> {
    sqlx::query("UPDATE conversations SET updated_at = ?1 WHERE id = ?2")
        .bind(updated_at)
        .bind(conversation_id)
        .execute(&mut **tx)
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

/// 在连接池上追加新的内容块。
pub async fn append_content_chunk(
    pool: &SqlitePool,
    new_content: &NewMessageContent,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    insert_content_tx(&mut tx, new_content).await?;
    tx.commit().await.map_err(|error| error.to_string())
}

/// 读取版本的下一个 chunk 序号。
pub async fn next_chunk_index(pool: &SqlitePool, version_id: &str) -> Result<i64, String> {
    sqlx::query_scalar::<_, Option<i64>>(
        "SELECT MAX(chunk_index) FROM message_contents WHERE version_id = ?1",
    )
    .bind(version_id)
    .fetch_one(pool)
    .await
    .map(|value| value.unwrap_or(-1) + 1)
    .map_err(|error| error.to_string())
}

/// 批量拼接多个 active version 的完整正文。
async fn load_contents_map(
    pool: &SqlitePool,
    version_ids: &[String],
) -> Result<HashMap<String, AggregatedVersionContent>, String> {
    if version_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT version_id, content_type, body FROM message_contents WHERE version_id IN (",
    );
    let mut separated = query.separated(", ");
    for version_id in version_ids {
        separated.push_bind(version_id);
    }
    separated.push_unseparated(") ORDER BY version_id ASC, chunk_index ASC");

    let rows = query
        .build_query_as::<ContentRow>()
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?;

    let mut contents: HashMap<String, AggregatedVersionContent> = HashMap::new();
    for row in rows {
        let entry = contents.entry(row.version_id).or_default();
        match row.content_type.as_str() {
            "text/plain" => entry.text.push_str(&row.body),
            "text/thinking" => entry.thinking.push_str(&row.body),
            "image/base64" => {
                let attachment =
                    serde_json::from_str::<ImageAttachment>(&row.body).map_err(|error| error.to_string())?;
                entry.images.push(attachment);
            }
            _ => {}
        }
    }

    Ok(contents)
}
