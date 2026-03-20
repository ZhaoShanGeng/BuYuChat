use std::collections::HashSet;

use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{LorebookEntryKeyRow, LorebookEntryRow, LorebookRow};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct InsertLorebookEntry<'a> {
    pub lorebook_id: &'a str,
    pub title: Option<&'a str>,
    pub primary_content_id: &'a str,
    pub activation_strategy: &'a str,
    pub keyword_logic: &'a str,
    pub insertion_position: &'a str,
    pub insertion_order: i64,
    pub insertion_depth: Option<i64>,
    pub insertion_role: Option<&'a str>,
    pub outlet_name: Option<&'a str>,
    pub entry_scope: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct UpdateLorebookEntry<'a> {
    pub title: Option<&'a str>,
    pub primary_content_id: &'a str,
    pub activation_strategy: &'a str,
    pub keyword_logic: &'a str,
    pub insertion_position: &'a str,
    pub insertion_order: i64,
    pub insertion_depth: Option<i64>,
    pub insertion_role: Option<&'a str>,
    pub outlet_name: Option<&'a str>,
    pub entry_scope: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub async fn list_lorebooks(db: &SqlitePool) -> Result<Vec<LorebookRow>> {
    sqlx::query_as::<_, LorebookRow>(
        r#"
        SELECT *
        FROM lorebooks
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_lorebook(db: &SqlitePool, id: &str) -> Result<LorebookRow> {
    sqlx::query_as::<_, LorebookRow>("SELECT * FROM lorebooks WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "lorebook",
            id: id.to_string(),
        })
}

pub async fn create_lorebook(
    db: &SqlitePool,
    name: &str,
    description: Option<&str>,
    scan_depth: i64,
    token_budget: Option<i64>,
    insertion_strategy: &str,
    enabled: bool,
    sort_order: i64,
    config_json: &str,
) -> Result<LorebookRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO lorebooks (
            id, name, description, scan_depth, token_budget, insertion_strategy,
            enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .bind(scan_depth)
    .bind(token_budget)
    .bind(insertion_strategy)
    .bind(enabled)
    .bind(sort_order)
    .bind(config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_lorebook(db, &id).await
}

pub async fn update_lorebook(
    db: &SqlitePool,
    id: &str,
    name: &str,
    description: Option<&str>,
    scan_depth: i64,
    token_budget: Option<i64>,
    insertion_strategy: &str,
    enabled: bool,
    sort_order: i64,
    config_json: &str,
) -> Result<LorebookRow> {
    let affected = sqlx::query(
        r#"
        UPDATE lorebooks
        SET name = ?, description = ?, scan_depth = ?, token_budget = ?, insertion_strategy = ?,
            enabled = ?, sort_order = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(scan_depth)
    .bind(token_budget)
    .bind(insertion_strategy)
    .bind(enabled)
    .bind(sort_order)
    .bind(config_json)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "lorebook",
            id: id.to_string(),
        });
    }

    get_lorebook(db, id).await
}

pub async fn delete_lorebook(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM lorebooks WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "lorebook",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_lorebook_entries(
    db: &SqlitePool,
    lorebook_id: &str,
) -> Result<Vec<LorebookEntryRow>> {
    sqlx::query_as::<_, LorebookEntryRow>(
        r#"
        SELECT *
        FROM lorebook_entries
        WHERE lorebook_id = ?
        ORDER BY insertion_order ASC, sort_order ASC, created_at ASC
        "#,
    )
    .bind(lorebook_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_lorebook_entry(db: &SqlitePool, id: &str) -> Result<LorebookEntryRow> {
    sqlx::query_as::<_, LorebookEntryRow>("SELECT * FROM lorebook_entries WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "lorebook_entry",
            id: id.to_string(),
        })
}

pub async fn create_lorebook_entry(
    db: &SqlitePool,
    input: &InsertLorebookEntry<'_>,
) -> Result<LorebookEntryRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO lorebook_entries (
            id, lorebook_id, title, primary_content_id, activation_strategy, keyword_logic,
            insertion_position, insertion_order, insertion_depth, insertion_role, outlet_name,
            entry_scope, enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.lorebook_id)
    .bind(input.title)
    .bind(input.primary_content_id)
    .bind(input.activation_strategy)
    .bind(input.keyword_logic)
    .bind(input.insertion_position)
    .bind(input.insertion_order)
    .bind(input.insertion_depth)
    .bind(input.insertion_role)
    .bind(input.outlet_name)
    .bind(input.entry_scope)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_lorebook_entry(db, &id).await
}

pub async fn update_lorebook_entry(
    db: &SqlitePool,
    id: &str,
    input: &UpdateLorebookEntry<'_>,
) -> Result<LorebookEntryRow> {
    let affected = sqlx::query(
        r#"
        UPDATE lorebook_entries
        SET title = ?, primary_content_id = ?, activation_strategy = ?, keyword_logic = ?,
            insertion_position = ?, insertion_order = ?, insertion_depth = ?, insertion_role = ?,
            outlet_name = ?, entry_scope = ?, enabled = ?, sort_order = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.title)
    .bind(input.primary_content_id)
    .bind(input.activation_strategy)
    .bind(input.keyword_logic)
    .bind(input.insertion_position)
    .bind(input.insertion_order)
    .bind(input.insertion_depth)
    .bind(input.insertion_role)
    .bind(input.outlet_name)
    .bind(input.entry_scope)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "lorebook_entry",
            id: id.to_string(),
        });
    }

    get_lorebook_entry(db, id).await
}

pub async fn delete_lorebook_entry(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM lorebook_entries WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "lorebook_entry",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_lorebook_entry_keys_by_lorebook(
    db: &SqlitePool,
    lorebook_id: &str,
) -> Result<Vec<LorebookEntryKeyRow>> {
    sqlx::query_as::<_, LorebookEntryKeyRow>(
        r#"
        SELECT k.*
        FROM lorebook_entry_keys k
        INNER JOIN lorebook_entries e ON e.id = k.entry_id
        WHERE e.lorebook_id = ?
        ORDER BY k.entry_id ASC, k.sort_order ASC
        "#,
    )
    .bind(lorebook_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn replace_lorebook_entry_keys(
    tx: &mut Transaction<'_, Sqlite>,
    entry_id: &str,
    keys: &[String],
) -> Result<()> {
    let mut unique_keys = HashSet::new();
    let mut normalized = Vec::new();

    for key in keys {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            continue;
        }
        if unique_keys.insert(trimmed.to_string()) {
            normalized.push(trimmed.to_string());
        }
    }

    sqlx::query("DELETE FROM lorebook_entry_keys WHERE entry_id = ?")
        .bind(entry_id)
        .execute(tx.as_mut())
        .await?;

    for (index, pattern_text) in normalized.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO lorebook_entry_keys (
                id, entry_id, key_type, match_type, pattern_text,
                case_sensitive, enabled, sort_order, config_json
            ) VALUES (?, ?, 'primary', 'plain', ?, 0, 1, ?, '{}')
            "#,
        )
        .bind(ids::new_id())
        .bind(entry_id)
        .bind(pattern_text)
        .bind(index as i64)
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}
