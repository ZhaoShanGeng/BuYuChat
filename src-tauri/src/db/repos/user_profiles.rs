use sqlx::SqlitePool;

use crate::db::models::UserProfileRow;
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub async fn list_user_profiles(db: &SqlitePool) -> Result<Vec<UserProfileRow>> {
    sqlx::query_as::<_, UserProfileRow>(
        r#"
        SELECT *
        FROM user_profiles
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_user_profile(db: &SqlitePool, id: &str) -> Result<UserProfileRow> {
    sqlx::query_as::<_, UserProfileRow>("SELECT * FROM user_profiles WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "user_profile",
            id: id.to_string(),
        })
}

pub async fn create_user_profile(
    db: &SqlitePool,
    name: &str,
    title: Option<&str>,
    description_content_id: Option<&str>,
    avatar_uri: Option<&str>,
    injection_position: &str,
    injection_depth: Option<i64>,
    injection_role: Option<&str>,
    enabled: bool,
    sort_order: i64,
    config_json: &str,
) -> Result<UserProfileRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO user_profiles (
            id, name, title, description_content_id, avatar_uri, injection_position,
            injection_depth, injection_role, enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(title)
    .bind(description_content_id)
    .bind(avatar_uri)
    .bind(injection_position)
    .bind(injection_depth)
    .bind(injection_role)
    .bind(enabled)
    .bind(sort_order)
    .bind(config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_user_profile(db, &id).await
}

pub async fn update_user_profile(
    db: &SqlitePool,
    id: &str,
    name: &str,
    title: Option<&str>,
    description_content_id: Option<&str>,
    avatar_uri: Option<&str>,
    injection_position: &str,
    injection_depth: Option<i64>,
    injection_role: Option<&str>,
    enabled: bool,
    sort_order: i64,
    config_json: &str,
) -> Result<UserProfileRow> {
    let affected = sqlx::query(
        r#"
        UPDATE user_profiles
        SET name = ?, title = ?, description_content_id = ?, avatar_uri = ?, injection_position = ?,
            injection_depth = ?, injection_role = ?, enabled = ?, sort_order = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(title)
    .bind(description_content_id)
    .bind(avatar_uri)
    .bind(injection_position)
    .bind(injection_depth)
    .bind(injection_role)
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
            entity: "user_profile",
            id: id.to_string(),
        });
    }

    get_user_profile(db, id).await
}

pub async fn delete_user_profile(db: &SqlitePool, id: &str) -> Result<()> {
    let _ = get_user_profile(db, id).await?;

    let mut tx = db.begin().await?;

    sqlx::query("DELETE FROM agent_user_profile_bindings WHERE user_profile_id = ?")
        .bind(id)
        .execute(tx.as_mut())
        .await?;

    sqlx::query("DELETE FROM conversation_user_profile_bindings WHERE user_profile_id = ?")
        .bind(id)
        .execute(tx.as_mut())
        .await?;

    sqlx::query(
        r#"
        UPDATE generation_runs
        SET user_profile_id = NULL,
            user_profile_source_scope = NULL
        WHERE user_profile_id = ?
        "#,
    )
    .bind(id)
    .execute(tx.as_mut())
    .await?;

    sqlx::query(
        "UPDATE generation_run_context_items SET source_user_profile_id = NULL WHERE source_user_profile_id = ?",
    )
    .bind(id)
    .execute(tx.as_mut())
    .await?;

    sqlx::query("UPDATE workflow_def_nodes SET user_profile_id = NULL WHERE user_profile_id = ?")
        .bind(id)
        .execute(tx.as_mut())
        .await?;

    sqlx::query(
        "UPDATE workflow_run_writes SET target_user_profile_id = NULL WHERE target_user_profile_id = ?",
    )
    .bind(id)
    .execute(tx.as_mut())
    .await?;

    let affected = sqlx::query("DELETE FROM user_profiles WHERE id = ?")
        .bind(id)
        .execute(tx.as_mut())
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "user_profile",
            id: id.to_string(),
        });
    }

    tx.commit().await?;

    Ok(())
}
