use sqlx::SqlitePool;

use crate::db::models::UserProfileRow;
use crate::db::repos::user_profiles as repo;
use crate::domain::content::{ContentType, ContentWriteInput, StoredContent};
use crate::domain::messages::MessageRole;
use crate::domain::user_profiles::{
    CreateUserProfileInput, UpdateUserProfileInput, UserProfileDetail, UserProfileSummary,
};
use crate::services::content as content_service;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn list_user_profiles(db: &SqlitePool) -> Result<Vec<UserProfileSummary>> {
    repo::list_user_profiles(db)
        .await?
        .into_iter()
        .map(map_user_profile_summary)
        .collect()
}

pub async fn get_user_profile(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
) -> Result<UserProfileDetail> {
    let row = repo::get_user_profile(db, id).await?;
    map_user_profile_detail(db, store, row).await
}

pub async fn create_user_profile(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateUserProfileInput,
) -> Result<UserProfileDetail> {
    let description_content_id =
        store_optional_text_content(db, store, input.description_content.as_ref())
            .await?
            .map(|content| content.content_id);

    let row = repo::create_user_profile(
        db,
        &input.name,
        input.title.as_deref(),
        description_content_id.as_deref(),
        input.avatar_uri.as_deref(),
        &input.injection_position,
        input.injection_depth,
        input.injection_role.map(MessageRole::as_str),
        input.enabled,
        input.sort_order,
        &input.config_json.to_string(),
    )
    .await?;

    map_user_profile_detail(db, store, row).await
}

pub async fn update_user_profile(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
    input: &UpdateUserProfileInput,
) -> Result<UserProfileDetail> {
    let description_content_id =
        store_optional_text_content(db, store, input.description_content.as_ref())
            .await?
            .map(|content| content.content_id);

    let row = repo::update_user_profile(
        db,
        id,
        &input.name,
        input.title.as_deref(),
        description_content_id.as_deref(),
        input.avatar_uri.as_deref(),
        &input.injection_position,
        input.injection_depth,
        input.injection_role.map(MessageRole::as_str),
        input.enabled,
        input.sort_order,
        &input.config_json.to_string(),
    )
    .await?;

    map_user_profile_detail(db, store, row).await
}

pub async fn delete_user_profile(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_user_profile(db, id).await
}

async fn map_user_profile_detail(
    db: &SqlitePool,
    store: &ContentStore,
    row: UserProfileRow,
) -> Result<UserProfileDetail> {
    let summary = map_user_profile_summary(row.clone())?;
    let description_content =
        load_optional_content(db, store, row.description_content_id.as_deref(), true).await?;

    Ok(UserProfileDetail {
        summary,
        description_content,
        injection_position: row.injection_position,
        injection_depth: row.injection_depth,
        injection_role: row
            .injection_role
            .as_deref()
            .map(MessageRole::parse)
            .transpose()?,
        config_json: parse_json(&row.config_json, "user_profiles.config_json")?,
    })
}

fn map_user_profile_summary(row: UserProfileRow) -> Result<UserProfileSummary> {
    Ok(UserProfileSummary {
        id: row.id,
        name: row.name,
        title: row.title,
        avatar_uri: row.avatar_uri,
        enabled: row.enabled,
        sort_order: row.sort_order,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn store_optional_text_content(
    db: &SqlitePool,
    store: &ContentStore,
    input: Option<&ContentWriteInput>,
) -> Result<Option<StoredContent>> {
    let Some(input) = input else {
        return Ok(None);
    };

    ensure_textual_content(&input.content_type)?;
    content_service::create_content(db, store, input)
        .await
        .map(Some)
}

async fn load_optional_content(
    db: &SqlitePool,
    store: &ContentStore,
    content_id: Option<&str>,
    include_body: bool,
) -> Result<Option<StoredContent>> {
    let Some(content_id) = content_id else {
        return Ok(None);
    };

    content_service::get_content(db, store, content_id, include_body)
        .await
        .map(Some)
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}

fn ensure_textual_content(content_type: &ContentType) -> Result<()> {
    match content_type {
        ContentType::Text | ContentType::Markdown | ContentType::Html | ContentType::Json => Ok(()),
        _ => Err(AppError::Validation(
            "user profile content must be textual".to_string(),
        )),
    }
}
