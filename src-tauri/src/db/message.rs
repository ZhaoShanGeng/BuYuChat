use std::collections::HashMap;

use chrono::Utc;
use sqlx::{QueryBuilder, SqlitePool};

use crate::db::models::{MessageRow, TurnRow, TurnVersionRow};
use crate::error::{AppError, Result};

pub async fn backfill_turns_from_legacy_messages(db: &SqlitePool) -> Result<()> {
    let turn_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversation_turns")
        .fetch_one(db)
        .await?;
    if turn_count > 0 {
        return Ok(());
    }

    let legacy_rows = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT *
        FROM messages
        ORDER BY conversation_id ASC, created_at ASC, version_index ASC
        "#,
    )
    .fetch_all(db)
    .await?;

    if legacy_rows.is_empty() {
        return Ok(());
    }

    let mut by_group: HashMap<String, Vec<MessageRow>> = HashMap::new();
    let mut message_to_group: HashMap<String, String> = HashMap::new();

    for row in legacy_rows {
        message_to_group.insert(row.id.clone(), row.version_group_id.clone());
        by_group
            .entry(row.version_group_id.clone())
            .or_default()
            .push(row);
    }

    let mut tx = db.begin().await?;
    let mut pending_parent_links: Vec<(String, Option<String>)> = Vec::new();

    for (turn_id, rows) in &by_group {
        let representative = rows
            .iter()
            .find(|row| row.is_active)
            .or_else(|| {
                rows.iter()
                    .max_by_key(|row| (row.version_index, row.created_at))
            })
            .ok_or_else(|| AppError::Other(format!("legacy version group {turn_id} is empty")))?;
        let parent_turn_id = representative
            .parent_message_id
            .as_ref()
            .and_then(|message_id| message_to_group.get(message_id))
            .cloned();
        let created_at = rows.iter().map(|row| row.created_at).min().unwrap_or(0);
        let updated_at = rows
            .iter()
            .map(|row| row.created_at)
            .max()
            .unwrap_or(created_at);
        let active_version_id = rows
            .iter()
            .find(|row| row.is_active)
            .map(|row| row.id.clone());
        pending_parent_links.push((
            turn_id.clone(),
            parent_turn_id.filter(|parent_id| parent_id != turn_id),
        ));

        sqlx::query(
            r#"
            INSERT INTO conversation_turns (
                id, conversation_id, parent_turn_id, role, active_version_id,
                deleted_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
            "#,
        )
        .bind(turn_id)
        .bind(&representative.conversation_id)
        .bind(Option::<String>::None)
        .bind(&representative.role)
        .bind(active_version_id)
        .bind(created_at)
        .bind(updated_at)
        .execute(&mut *tx)
        .await?;

        for row in rows {
            insert_turn_version_tx(
                &mut tx,
                &TurnVersionRow {
                    id: row.id.clone(),
                    turn_id: turn_id.clone(),
                    version_index: row.version_index,
                    content: row.content.clone(),
                    content_parts: row.content_parts.clone(),
                    tool_calls: row.tool_calls.clone(),
                    tool_call_id: row.tool_call_id.clone(),
                    citations_json: row.citations_json.clone(),
                    tokens_used: row.tokens_used,
                    provider: row.provider.clone(),
                    model_id: row.model_id.clone(),
                    created_at: row.created_at,
                },
            )
            .await?;
        }
    }

    for (turn_id, parent_turn_id) in pending_parent_links {
        sqlx::query("UPDATE conversation_turns SET parent_turn_id = ? WHERE id = ?")
            .bind(parent_turn_id)
            .bind(turn_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn insert(db: &SqlitePool, row: &MessageRow) -> Result<()> {
    let mut tx = db.begin().await?;
    ensure_turn_exists_tx(&mut tx, row).await?;
    insert_turn_version_tx(
        &mut tx,
        &TurnVersionRow {
            id: row.id.clone(),
            turn_id: row.version_group_id.clone(),
            version_index: row.version_index,
            content: row.content.clone(),
            content_parts: row.content_parts.clone(),
            tool_calls: row.tool_calls.clone(),
            tool_call_id: row.tool_call_id.clone(),
            citations_json: row.citations_json.clone(),
            tokens_used: row.tokens_used,
            provider: row.provider.clone(),
            model_id: row.model_id.clone(),
            created_at: row.created_at,
        },
    )
    .await?;

    if row.is_active {
        sqlx::query(
            "UPDATE conversation_turns SET active_version_id = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&row.id)
        .bind(Utc::now().timestamp_millis())
        .bind(&row.version_group_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list_active(db: &SqlitePool, conv_id: &str) -> Result<Vec<MessageRow>> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        WITH RECURSIVE active_turns(id, parent_turn_id, depth) AS (
            SELECT root.id, root.parent_turn_id, 0
            FROM (
                SELECT id, parent_turn_id
                FROM conversation_turns
                WHERE conversation_id = ?
                  AND parent_turn_id IS NULL
                  AND deleted_at IS NULL
                ORDER BY created_at ASC
                LIMIT 1
            ) AS root
            UNION ALL
            SELECT child.id, child.parent_turn_id, active_turns.depth + 1
            FROM active_turns
            INNER JOIN conversation_turns child ON child.id = (
                SELECT id
                FROM conversation_turns
                WHERE conversation_id = ?
                  AND parent_turn_id = active_turns.id
                  AND deleted_at IS NULL
                ORDER BY created_at ASC
                LIMIT 1
            )
        )
        SELECT
            v.id AS id,
            t.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            t.id AS version_group_id,
            v.version_index AS version_index,
            CASE WHEN t.active_version_id = v.id THEN 1 ELSE 0 END AS is_active,
            t.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM active_turns
        INNER JOIN conversation_turns t ON t.id = active_turns.id
        INNER JOIN turn_versions v ON v.id = t.active_version_id
        LEFT JOIN conversation_turns parent_turn ON parent_turn.id = t.parent_turn_id
        ORDER BY active_turns.depth ASC, t.created_at ASC
        "#,
    )
    .bind(conv_id)
    .bind(conv_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn list_all(db: &SqlitePool, conv_id: &str) -> Result<Vec<MessageRow>> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT
            v.id AS id,
            t.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            t.id AS version_group_id,
            v.version_index AS version_index,
            CASE WHEN t.active_version_id = v.id THEN 1 ELSE 0 END AS is_active,
            t.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM conversation_turns t
        INNER JOIN turn_versions v ON v.turn_id = t.id
        LEFT JOIN conversation_turns parent_turn ON parent_turn.id = t.parent_turn_id
        WHERE t.conversation_id = ?
          AND t.deleted_at IS NULL
        ORDER BY t.created_at ASC, v.version_index ASC, v.created_at ASC
        "#,
    )
    .bind(conv_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn list_versions(db: &SqlitePool, version_group_id: &str) -> Result<Vec<MessageRow>> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT
            v.id AS id,
            t.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            t.id AS version_group_id,
            v.version_index AS version_index,
            CASE WHEN t.active_version_id = v.id THEN 1 ELSE 0 END AS is_active,
            t.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM conversation_turns t
        INNER JOIN turn_versions v ON v.turn_id = t.id
        LEFT JOIN conversation_turns parent_turn ON parent_turn.id = t.parent_turn_id
        WHERE t.id = ?
          AND t.deleted_at IS NULL
        ORDER BY v.version_index ASC, v.created_at ASC
        "#,
    )
    .bind(version_group_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn list_versions_for_groups(
    db: &SqlitePool,
    version_group_ids: &[String],
) -> Result<Vec<MessageRow>> {
    if version_group_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::new(
        r#"
        SELECT
            v.id AS id,
            t.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            t.id AS version_group_id,
            v.version_index AS version_index,
            CASE WHEN t.active_version_id = v.id THEN 1 ELSE 0 END AS is_active,
            t.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM conversation_turns t
        INNER JOIN turn_versions v ON v.turn_id = t.id
        LEFT JOIN conversation_turns parent_turn ON parent_turn.id = t.parent_turn_id
        WHERE t.deleted_at IS NULL
          AND t.id IN (
        "#,
    );
    {
        let mut separated = query.separated(", ");
        for group_id in version_group_ids {
            separated.push_bind(group_id);
        }
    }
    query.push(") ORDER BY t.created_at ASC, v.version_index ASC, v.created_at ASC");

    query
        .build_query_as::<MessageRow>()
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

pub async fn find_last_active_message_id(db: &SqlitePool, conv_id: &str) -> Result<Option<String>> {
    sqlx::query_scalar::<_, String>(
        r#"
        WITH RECURSIVE active_turns(id, parent_turn_id, depth) AS (
            SELECT root.id, root.parent_turn_id, 0
            FROM (
                SELECT id, parent_turn_id
                FROM conversation_turns
                WHERE conversation_id = ?
                  AND parent_turn_id IS NULL
                  AND deleted_at IS NULL
                ORDER BY created_at ASC
                LIMIT 1
            ) AS root
            UNION ALL
            SELECT child.id, child.parent_turn_id, active_turns.depth + 1
            FROM active_turns
            INNER JOIN conversation_turns child ON child.id = (
                SELECT id
                FROM conversation_turns
                WHERE conversation_id = ?
                  AND parent_turn_id = active_turns.id
                  AND deleted_at IS NULL
                ORDER BY created_at ASC
                LIMIT 1
            )
        )
        SELECT t.active_version_id
        FROM active_turns
        INNER JOIN conversation_turns t ON t.id = active_turns.id
        ORDER BY active_turns.depth DESC, t.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(conv_id)
    .bind(conv_id)
    .fetch_optional(db)
    .await
    .map_err(Into::into)
}

pub async fn find_last_active_assistant(db: &SqlitePool, conv_id: &str) -> Result<MessageRow> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        WITH RECURSIVE active_turns(id, parent_turn_id, depth) AS (
            SELECT root.id, root.parent_turn_id, 0
            FROM (
                SELECT id, parent_turn_id
                FROM conversation_turns
                WHERE conversation_id = ?
                  AND parent_turn_id IS NULL
                  AND deleted_at IS NULL
                ORDER BY created_at ASC
                LIMIT 1
            ) AS root
            UNION ALL
            SELECT child.id, child.parent_turn_id, active_turns.depth + 1
            FROM active_turns
            INNER JOIN conversation_turns child ON child.id = (
                SELECT id
                FROM conversation_turns
                WHERE conversation_id = ?
                  AND parent_turn_id = active_turns.id
                  AND deleted_at IS NULL
                ORDER BY created_at ASC
                LIMIT 1
            )
        )
        SELECT
            v.id AS id,
            t.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            t.id AS version_group_id,
            v.version_index AS version_index,
            1 AS is_active,
            t.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM active_turns
        INNER JOIN conversation_turns t ON t.id = active_turns.id
        INNER JOIN turn_versions v ON v.id = t.active_version_id
        LEFT JOIN conversation_turns parent_turn ON parent_turn.id = t.parent_turn_id
        WHERE t.role = 'assistant'
        ORDER BY active_turns.depth DESC, t.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(conv_id)
    .bind(conv_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "message",
        id: format!("last_active_assistant:{conv_id}"),
    })
}

pub async fn get(db: &SqlitePool, id: &str) -> Result<MessageRow> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT
            v.id AS id,
            t.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            t.id AS version_group_id,
            v.version_index AS version_index,
            CASE WHEN t.active_version_id = v.id THEN 1 ELSE 0 END AS is_active,
            t.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM turn_versions v
        INNER JOIN conversation_turns t ON t.id = v.turn_id
        LEFT JOIN conversation_turns parent_turn ON parent_turn.id = t.parent_turn_id
        WHERE v.id = ?
          AND t.deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "message",
        id: id.to_string(),
    })
}

pub async fn max_version_index(db: &SqlitePool, version_group_id: &str) -> Result<i64> {
    let max = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT MAX(version_index) FROM turn_versions WHERE turn_id = ?",
    )
    .bind(version_group_id)
    .fetch_one(db)
    .await?
    .unwrap_or(0);
    Ok(max)
}

pub async fn get_version(
    db: &SqlitePool,
    version_group_id: &str,
    version_index: i64,
) -> Result<MessageRow> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT
            v.id AS id,
            t.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            t.id AS version_group_id,
            v.version_index AS version_index,
            CASE WHEN t.active_version_id = v.id THEN 1 ELSE 0 END AS is_active,
            t.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM turn_versions v
        INNER JOIN conversation_turns t ON t.id = v.turn_id
        LEFT JOIN conversation_turns parent_turn ON parent_turn.id = t.parent_turn_id
        WHERE v.turn_id = ?
          AND v.version_index = ?
          AND t.deleted_at IS NULL
        "#,
    )
    .bind(version_group_id)
    .bind(version_index)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "message_version",
        id: format!("{version_group_id}:{version_index}"),
    })
}

pub async fn set_active_message_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    version_group_id: &str,
    message_id: &str,
) -> Result<()> {
    let affected = sqlx::query(
        "UPDATE conversation_turns SET active_version_id = ?, updated_at = ? WHERE id = ?",
    )
    .bind(message_id)
    .bind(Utc::now().timestamp_millis())
    .bind(version_group_id)
    .execute(&mut **tx)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "message_turn",
            id: version_group_id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_path_to_message(db: &SqlitePool, message_id: &str) -> Result<Vec<MessageRow>> {
    let rows = sqlx::query_as::<_, MessageRow>(
        r#"
        WITH RECURSIVE ancestry(turn_id, parent_turn_id, depth) AS (
            SELECT t.id, t.parent_turn_id, 0
            FROM conversation_turns t
            INNER JOIN turn_versions target ON target.turn_id = t.id
            WHERE target.id = ?
              AND t.deleted_at IS NULL
            UNION ALL
            SELECT parent.id, parent.parent_turn_id, ancestry.depth + 1
            FROM conversation_turns parent
            INNER JOIN ancestry ON ancestry.parent_turn_id = parent.id
            WHERE parent.deleted_at IS NULL
        )
        SELECT
            COALESCE(target.id, active.id) AS id,
            t.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            t.id AS version_group_id,
            COALESCE(target.version_index, active.version_index) AS version_index,
            CASE
                WHEN COALESCE(target.id, active.id) = t.active_version_id THEN 1
                ELSE 0
            END AS is_active,
            t.role AS role,
            COALESCE(target.content, active.content) AS content,
            COALESCE(target.content_parts, active.content_parts) AS content_parts,
            COALESCE(target.tool_calls, active.tool_calls) AS tool_calls,
            COALESCE(target.tool_call_id, active.tool_call_id) AS tool_call_id,
            COALESCE(target.citations_json, active.citations_json) AS citations_json,
            COALESCE(target.tokens_used, active.tokens_used) AS tokens_used,
            COALESCE(target.provider, active.provider) AS provider,
            COALESCE(target.model_id, active.model_id) AS model_id,
            COALESCE(target.created_at, active.created_at) AS created_at
        FROM ancestry
        INNER JOIN conversation_turns t ON t.id = ancestry.turn_id
        LEFT JOIN turn_versions active ON active.id = t.active_version_id
        LEFT JOIN turn_versions target ON target.id = ? AND target.turn_id = t.id
        LEFT JOIN conversation_turns parent_turn ON parent_turn.id = t.parent_turn_id
        ORDER BY ancestry.depth DESC, t.created_at ASC
        "#,
    )
    .bind(message_id)
    .bind(message_id)
    .fetch_all(db)
    .await?;

    if rows.is_empty() {
        return Err(AppError::NotFound {
            entity: "message",
            id: message_id.to_string(),
        });
    }

    Ok(rows)
}

pub async fn delete_subtree(db: &SqlitePool, root_message_id: &str) -> Result<()> {
    let mut tx = db.begin().await?;
    delete_subtree_tx(&mut tx, root_message_id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete_subtree_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    root_message_id: &str,
) -> Result<()> {
    let root_turn_id = version_to_turn_id_tx(tx, root_message_id).await?;
    delete_turn_subtree_by_turn_id_tx(tx, &root_turn_id).await
}

pub async fn delete_descendants_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    root_message_id: &str,
) -> Result<()> {
    let root_turn_id = version_to_turn_id_tx(tx, root_message_id).await?;
    sqlx::query(
        r#"
        WITH RECURSIVE subtree(id) AS (
            SELECT id
            FROM conversation_turns
            WHERE parent_turn_id = ?
              AND deleted_at IS NULL
            UNION ALL
            SELECT child.id
            FROM conversation_turns child
            INNER JOIN subtree s ON child.parent_turn_id = s.id
            WHERE child.deleted_at IS NULL
        )
        DELETE FROM conversation_turns
        WHERE id IN (SELECT id FROM subtree)
        "#,
    )
    .bind(&root_turn_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn delete_message_only(db: &SqlitePool, message_id: &str) -> Result<()> {
    let target = get(db, message_id).await?;
    let turn = get_turn(db, &target.version_group_id).await?;
    let mut tx = db.begin().await?;

    let version_count = count_turn_versions_tx(&mut tx, &turn.id).await?;
    if version_count > 1 {
        if turn.active_version_id.as_deref() == Some(message_id) {
            let replacement_id = sqlx::query_scalar::<_, String>(
                r#"
                SELECT id
                FROM turn_versions
                WHERE turn_id = ?
                  AND id <> ?
                ORDER BY version_index DESC, created_at DESC
                LIMIT 1
                "#,
            )
            .bind(&turn.id)
            .bind(message_id)
            .fetch_optional(&mut *tx)
            .await?;

            let Some(replacement_id) = replacement_id else {
                return Err(AppError::Other(
                    "active version replacement was not found".to_string(),
                ));
            };

            sqlx::query(
                "UPDATE conversation_turns SET active_version_id = ?, updated_at = ? WHERE id = ?",
            )
            .bind(replacement_id)
            .bind(Utc::now().timestamp_millis())
            .bind(&turn.id)
            .execute(&mut *tx)
            .await?;
        }

        let affected = sqlx::query("DELETE FROM turn_versions WHERE id = ?")
            .bind(message_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound {
                entity: "message",
                id: message_id.to_string(),
            });
        }
    } else {
        sqlx::query(
            r#"
            UPDATE conversation_turns
            SET parent_turn_id = ?
            WHERE parent_turn_id = ?
            "#,
        )
        .bind(&turn.parent_turn_id)
        .bind(&turn.id)
        .execute(&mut *tx)
        .await?;

        let affected = sqlx::query("DELETE FROM conversation_turns WHERE id = ?")
            .bind(&turn.id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound {
                entity: "message_turn",
                id: turn.id,
            });
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn update_assistant_result(
    db: &SqlitePool,
    id: &str,
    content: &str,
    tool_calls_json: Option<&str>,
    citations_json: Option<&str>,
    tokens_used: Option<i64>,
) -> Result<()> {
    let affected = sqlx::query(
        r#"
        UPDATE turn_versions
        SET content = ?, tool_calls = ?, citations_json = ?, tokens_used = ?
        WHERE id = ?
        "#,
    )
    .bind(content)
    .bind(tool_calls_json)
    .bind(citations_json)
    .bind(tokens_used)
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "message",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn find_children(db: &SqlitePool, parent_message_id: &str) -> Result<Vec<MessageRow>> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT
            v.id AS id,
            child_turn.conversation_id AS conversation_id,
            child_parent.active_version_id AS parent_message_id,
            child_turn.id AS version_group_id,
            v.version_index AS version_index,
            CASE WHEN child_turn.active_version_id = v.id THEN 1 ELSE 0 END AS is_active,
            child_turn.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM turn_versions parent_version
        INNER JOIN conversation_turns parent_turn ON parent_turn.id = parent_version.turn_id
        INNER JOIN conversation_turns child_turn ON child_turn.parent_turn_id = parent_turn.id
        INNER JOIN turn_versions v ON v.turn_id = child_turn.id
        LEFT JOIN conversation_turns child_parent ON child_parent.id = child_turn.parent_turn_id
        WHERE parent_version.id = ?
          AND child_turn.deleted_at IS NULL
        ORDER BY child_turn.created_at ASC, v.version_index ASC, v.created_at ASC
        "#,
    )
    .bind(parent_message_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn find_active_child(
    db: &SqlitePool,
    parent_message_id: &str,
) -> Result<Option<MessageRow>> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT
            v.id AS id,
            child_turn.conversation_id AS conversation_id,
            parent_turn.active_version_id AS parent_message_id,
            child_turn.id AS version_group_id,
            v.version_index AS version_index,
            1 AS is_active,
            child_turn.role AS role,
            v.content AS content,
            v.content_parts AS content_parts,
            v.tool_calls AS tool_calls,
            v.tool_call_id AS tool_call_id,
            v.citations_json AS citations_json,
            v.tokens_used AS tokens_used,
            v.provider AS provider,
            v.model_id AS model_id,
            v.created_at AS created_at
        FROM turn_versions parent_version
        INNER JOIN conversation_turns parent_turn ON parent_turn.id = parent_version.turn_id
        INNER JOIN conversation_turns child_turn ON child_turn.parent_turn_id = parent_turn.id
        INNER JOIN turn_versions v ON v.id = child_turn.active_version_id
        WHERE parent_version.id = ?
          AND child_turn.deleted_at IS NULL
        ORDER BY child_turn.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(parent_message_id)
    .fetch_optional(db)
    .await
    .map_err(Into::into)
}

pub async fn reparent_active_child_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    from_parent_id: &str,
    to_parent_id: &str,
) -> Result<()> {
    let from_turn_id = version_to_turn_id_tx(tx, from_parent_id).await?;
    let to_turn_id = version_to_turn_id_tx(tx, to_parent_id).await?;

    let child_turn_id = sqlx::query_scalar::<_, String>(
        r#"
        SELECT id
        FROM conversation_turns
        WHERE parent_turn_id = ?
          AND deleted_at IS NULL
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(&from_turn_id)
    .fetch_optional(&mut **tx)
    .await?;

    let Some(child_turn_id) = child_turn_id else {
        return Ok(());
    };

    sqlx::query("UPDATE conversation_turns SET parent_turn_id = ?, updated_at = ? WHERE id = ?")
        .bind(&to_turn_id)
        .bind(Utc::now().timestamp_millis())
        .bind(child_turn_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn ensure_turn_exists_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &MessageRow,
) -> Result<()> {
    let exists = sqlx::query_scalar::<_, String>("SELECT id FROM conversation_turns WHERE id = ?")
        .bind(&row.version_group_id)
        .fetch_optional(&mut **tx)
        .await?;

    if exists.is_some() {
        return Ok(());
    }

    let parent_turn_id = match row.parent_message_id.as_deref() {
        Some(parent_message_id) => Some(version_to_turn_id_tx(tx, parent_message_id).await?),
        None => None,
    };

    sqlx::query(
        r#"
        INSERT INTO conversation_turns (
            id, conversation_id, parent_turn_id, role, active_version_id,
            deleted_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
        "#,
    )
    .bind(&row.version_group_id)
    .bind(&row.conversation_id)
    .bind(parent_turn_id)
    .bind(&row.role)
    .bind(if row.is_active {
        Some(row.id.as_str())
    } else {
        None
    })
    .bind(row.created_at)
    .bind(row.created_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_turn_version_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &TurnVersionRow,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO turn_versions (
            id, turn_id, version_index, content, content_parts, tool_calls,
            tool_call_id, citations_json, tokens_used, provider, model_id, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&row.id)
    .bind(&row.turn_id)
    .bind(row.version_index)
    .bind(&row.content)
    .bind(&row.content_parts)
    .bind(&row.tool_calls)
    .bind(&row.tool_call_id)
    .bind(&row.citations_json)
    .bind(row.tokens_used)
    .bind(&row.provider)
    .bind(&row.model_id)
    .bind(row.created_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn version_to_turn_id_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    version_id: &str,
) -> Result<String> {
    sqlx::query_scalar::<_, String>("SELECT turn_id FROM turn_versions WHERE id = ?")
        .bind(version_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "message",
            id: version_id.to_string(),
        })
}

async fn count_turn_versions_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    turn_id: &str,
) -> Result<i64> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM turn_versions WHERE turn_id = ?")
        .bind(turn_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(Into::into)
}

async fn delete_turn_subtree_by_turn_id_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    turn_id: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        WITH RECURSIVE subtree(id) AS (
            SELECT id
            FROM conversation_turns
            WHERE id = ?
              AND deleted_at IS NULL
            UNION ALL
            SELECT child.id
            FROM conversation_turns child
            INNER JOIN subtree s ON child.parent_turn_id = s.id
            WHERE child.deleted_at IS NULL
        )
        DELETE FROM conversation_turns
        WHERE id IN (SELECT id FROM subtree)
        "#,
    )
    .bind(turn_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn get_turn(db: &SqlitePool, turn_id: &str) -> Result<TurnRow> {
    sqlx::query_as::<_, TurnRow>(
        r#"
        SELECT *
        FROM conversation_turns
        WHERE id = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(turn_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "message_turn",
        id: turn_id.to_string(),
    })
}
