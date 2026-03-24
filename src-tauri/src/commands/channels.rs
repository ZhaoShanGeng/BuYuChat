use tauri::State;

use crate::{
    db::Db,
    errors::AppError,
    models::{Channel, ChannelTestResult, CreateChannelInput, UpdateChannelInput},
    services::channel_service,
};

pub fn list_channels_impl(
    db: &Db,
    include_disabled: Option<bool>,
) -> Result<Vec<Channel>, AppError> {
    channel_service::list(db, include_disabled.unwrap_or(true))
}

#[tauri::command]
pub fn list_channels(
    db: State<'_, Db>,
    include_disabled: Option<bool>,
) -> Result<Vec<Channel>, AppError> {
    list_channels_impl(&db, include_disabled)
}

pub fn get_channel_impl(db: &Db, id: String) -> Result<Channel, AppError> {
    channel_service::get(db, &id)
}

#[tauri::command]
pub fn get_channel(db: State<'_, Db>, id: String) -> Result<Channel, AppError> {
    get_channel_impl(&db, id)
}

pub fn create_channel_impl(db: &Db, input: CreateChannelInput) -> Result<Channel, AppError> {
    channel_service::create(db, input)
}

#[tauri::command]
pub fn create_channel(db: State<'_, Db>, input: CreateChannelInput) -> Result<Channel, AppError> {
    create_channel_impl(&db, input)
}

pub fn update_channel_impl(
    db: &Db,
    id: String,
    input: UpdateChannelInput,
) -> Result<Channel, AppError> {
    channel_service::update(db, &id, input)
}

#[tauri::command]
pub fn update_channel(
    db: State<'_, Db>,
    id: String,
    input: UpdateChannelInput,
) -> Result<Channel, AppError> {
    update_channel_impl(&db, id, input)
}

pub fn delete_channel_impl(db: &Db, id: String) -> Result<(), AppError> {
    channel_service::delete(db, &id)
}

#[tauri::command]
pub fn delete_channel(db: State<'_, Db>, id: String) -> Result<(), AppError> {
    delete_channel_impl(&db, id)
}

pub fn test_channel_impl(db: &Db, id: String) -> Result<ChannelTestResult, AppError> {
    channel_service::test_channel(db, &id)
}

#[tauri::command]
pub fn test_channel(db: State<'_, Db>, id: String) -> Result<ChannelTestResult, AppError> {
    test_channel_impl(&db, id)
}
