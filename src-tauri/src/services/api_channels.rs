use sqlx::SqlitePool;

use crate::db::models::{ApiChannelModelRow, ApiChannelRow};
use crate::db::repos::api_channels as repo;
use crate::domain::api_channels::{
    ApiChannel, ApiChannelModel, ApiChannelTestResponse, CreateApiChannelInput, UpdateApiChannelInput,
    UpsertApiChannelModelInput,
};
use crate::domain::messages::{
    MessageRole, ProviderChatMessage, ProviderChatRequest, ProviderMessagePart,
    ProviderMessagePartKind,
};
use crate::providers::ProviderRegistry;
use crate::support::error::{AppError, Result};

pub async fn list_channels(db: &SqlitePool) -> Result<Vec<ApiChannel>> {
    repo::list_channels(db)
        .await?
        .into_iter()
        .map(map_channel_row)
        .collect()
}

pub async fn get_channel(db: &SqlitePool, id: &str) -> Result<ApiChannel> {
    map_channel_row(repo::get_channel(db, id).await?)
}

pub async fn create_channel(db: &SqlitePool, input: &CreateApiChannelInput) -> Result<ApiChannel> {
    map_channel_row(repo::create_channel(db, input).await?)
}

pub async fn update_channel(
    db: &SqlitePool,
    id: &str,
    input: &UpdateApiChannelInput,
) -> Result<ApiChannel> {
    map_channel_row(repo::update_channel(db, id, input).await?)
}

pub async fn delete_channel(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_channel(db, id).await
}

pub async fn list_channel_models(
    db: &SqlitePool,
    channel_id: &str,
) -> Result<Vec<ApiChannelModel>> {
    repo::list_channel_models(db, channel_id)
        .await?
        .into_iter()
        .map(map_channel_model_row)
        .collect()
}

pub async fn upsert_channel_model(
    db: &SqlitePool,
    input: &UpsertApiChannelModelInput,
) -> Result<ApiChannelModel> {
    map_channel_model_row(repo::upsert_channel_model(db, input).await?)
}

pub async fn delete_channel_model(db: &SqlitePool, channel_id: &str, model_id: &str) -> Result<()> {
    repo::delete_channel_model(db, channel_id, model_id).await
}

pub async fn fetch_remote_channel_models(
    db: &SqlitePool,
    providers: &ProviderRegistry,
    channel_id: &str,
) -> Result<Vec<ApiChannelModel>> {
    let channel = get_channel(db, channel_id).await?;
    let provider = providers.get(&channel.channel_type)?;
    provider.list_models(&channel).await
}

pub async fn refresh_channel_models(
    db: &SqlitePool,
    providers: &ProviderRegistry,
    channel_id: &str,
) -> Result<Vec<ApiChannelModel>> {
    let channel = get_channel(db, channel_id).await?;
    let remote_models = fetch_remote_channel_models(db, providers, channel_id).await?;

    for (index, model) in remote_models.into_iter().enumerate() {
        let input = UpsertApiChannelModelInput {
            channel_id: channel.id.clone(),
            model_id: model.model_id,
            display_name: model.display_name,
            model_type: model.model_type,
            context_window: model.context_window,
            max_output_tokens: model.max_output_tokens,
            capabilities_json: model.capabilities_json,
            pricing_json: model.pricing_json,
            default_parameters_json: model.default_parameters_json,
            sort_order: if model.sort_order != 0 {
                model.sort_order
            } else {
                index as i64
            },
            config_json: model.config_json,
        };
        let _ = repo::upsert_channel_model(db, &input).await?;
    }

    list_channel_models(db, channel_id).await
}

pub async fn test_channel_message(
    db: &SqlitePool,
    providers: &ProviderRegistry,
    channel_id: &str,
    model_id: &str,
) -> Result<ApiChannelTestResponse> {
    let channel = get_channel(db, channel_id).await?;
    let provider = providers.get(&channel.channel_type)?;

    let model = if let Some(model) = list_channel_models(db, channel_id)
        .await?
        .into_iter()
        .find(|model| model.model_id == model_id)
    {
        model
    } else if let Some(model) = fetch_remote_channel_models(db, providers, channel_id)
        .await?
        .into_iter()
        .find(|model| model.model_id == model_id)
    {
        model
    } else {
        return Err(AppError::Validation(
            format!("未找到要测试的模型: {model_id}"),
        ));
    };

    let request = ProviderChatRequest {
        api_channel: channel,
        api_channel_model: model.clone(),
        request_parameters_json: serde_json::json!({}),
        messages: vec![ProviderChatMessage {
            role: MessageRole::User,
            name: None,
            parts: vec![ProviderMessagePart {
                kind: ProviderMessagePartKind::Text,
                text: Some("hi".to_string()),
                content: None,
                metadata_json: serde_json::json!({}),
            }],
            metadata_json: serde_json::json!({}),
        }],
    };

    let response = provider.chat(request).await?;
    let response_text = response
        .parts
        .iter()
        .filter_map(|part| part.text.as_deref())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(ApiChannelTestResponse {
        model_id: model.model_id,
        response_text: if response_text.is_empty() {
            "连接成功，但接口未返回可显示文本".to_string()
        } else {
            response_text
        },
    })
}

fn map_channel_row(row: ApiChannelRow) -> Result<ApiChannel> {
    Ok(ApiChannel {
        id: row.id,
        name: row.name,
        channel_type: row.channel_type,
        base_url: row.base_url,
        auth_type: row.auth_type,
        api_key: row.api_key,
        models_endpoint: row.models_endpoint,
        chat_endpoint: row.chat_endpoint,
        stream_endpoint: row.stream_endpoint,
        models_mode: row.models_mode,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "api_channels.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub(crate) fn map_channel_model_row(row: ApiChannelModelRow) -> Result<ApiChannelModel> {
    Ok(ApiChannelModel {
        id: row.id,
        channel_id: row.channel_id,
        model_id: row.model_id,
        display_name: row.display_name,
        model_type: row.model_type,
        context_window: row.context_window,
        max_output_tokens: row.max_output_tokens,
        capabilities_json: parse_json(
            &row.capabilities_json,
            "api_channel_models.capabilities_json",
        )?,
        pricing_json: parse_json(&row.pricing_json, "api_channel_models.pricing_json")?,
        default_parameters_json: parse_json(
            &row.default_parameters_json,
            "api_channel_models.default_parameters_json",
        )?,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "api_channel_models.config_json")?,
    })
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}
