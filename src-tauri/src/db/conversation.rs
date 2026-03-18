use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::ConversationRow;
use crate::error::{AppError, Result};
use crate::types::PageResponse;

pub async fn create(
    db: &SqlitePool,
    model_id: &str,
    provider: &str,
    assistant_id: Option<&str>,
) -> Result<ConversationRow> {
    create_with_fields(db, "新对话", model_id, provider, assistant_id, None).await
}

pub async fn create_with_fields(
    db: &SqlitePool,
    title: &str,
    model_id: &str,
    provider: &str,
    assistant_id: Option<&str>,
    system_prompt: Option<&str>,
) -> Result<ConversationRow> {
    let id = Uuid::now_v7().to_string();
    let now = Utc::now().timestamp_millis();

    sqlx::query(
        r#"
        INSERT INTO conversations (
            id, title, model_id, provider, assistant_id, system_prompt, pinned, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(title)
    .bind(model_id)
    .bind(provider)
    .bind(assistant_id)
    .bind(system_prompt)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get(db, &id).await
}

pub async fn get(db: &SqlitePool, id: &str) -> Result<ConversationRow> {
    sqlx::query_as::<_, ConversationRow>("SELECT * FROM conversations WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        })
}

pub async fn list(
    db: &SqlitePool,
    page: u32,
    per_page: u32,
) -> Result<PageResponse<ConversationRow>> {
    let page = page.max(1);
    let per_page = per_page.max(1);
    let offset = ((page - 1) * per_page) as i64;

    let items = sqlx::query_as::<_, ConversationRow>(
        r#"
        SELECT * FROM conversations
        ORDER BY pinned DESC, updated_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(db)
    .await?;

    let total = count(db).await?;

    Ok(PageResponse {
        items,
        total,
        page,
        per_page,
    })
}

pub async fn count(db: &SqlitePool) -> Result<u32> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversations")
        .fetch_one(db)
        .await?;
    Ok(total as u32)
}

pub async fn update_title(db: &SqlitePool, id: &str, title: &str) -> Result<()> {
    let affected = sqlx::query("UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?")
        .bind(title)
        .bind(Utc::now().timestamp_millis())
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn update_model(db: &SqlitePool, id: &str, model_id: &str, provider: &str) -> Result<()> {
    let affected = sqlx::query(
        "UPDATE conversations SET model_id = ?, provider = ?, updated_at = ? WHERE id = ?",
    )
    .bind(model_id)
    .bind(provider)
    .bind(Utc::now().timestamp_millis())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn update_system_prompt(
    db: &SqlitePool,
    id: &str,
    system_prompt: Option<&str>,
) -> Result<()> {
    let affected =
        sqlx::query("UPDATE conversations SET system_prompt = ?, updated_at = ? WHERE id = ?")
            .bind(system_prompt)
            .bind(Utc::now().timestamp_millis())
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn touch(db: &SqlitePool, id: &str) -> Result<()> {
    let now = Utc::now().timestamp_millis();
    let affected = sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn delete(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM conversations WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn clear_messages(db: &SqlitePool, id: &str) -> Result<()> {
    get(db, id).await?;
    sqlx::query("DELETE FROM conversation_turns WHERE conversation_id = ?")
        .bind(id)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM messages WHERE conversation_id = ?")
        .bind(id)
        .execute(db)
        .await?;
    touch(db, id).await?;
    Ok(())
}
