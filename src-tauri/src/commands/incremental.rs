use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

use crate::domain::incremental::{IncrementalPatchEvent, IncrementalPatchOp};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub const INCREMENTAL_PATCH_EVENT: &str = "incremental_patch_event";

pub fn emit_upsert<R: Runtime, T: Serialize>(
    app: &AppHandle<R>,
    scope_kind: &str,
    scope_id: Option<&str>,
    resource_kind: &str,
    resource_id: Option<&str>,
    data: &T,
) -> Result<()> {
    emit_patch(
        app,
        scope_kind,
        scope_id,
        resource_kind,
        resource_id,
        IncrementalPatchOp::Upsert,
        serde_json::to_value(data)?,
    )
}

pub fn emit_replace<R: Runtime, T: Serialize>(
    app: &AppHandle<R>,
    scope_kind: &str,
    scope_id: Option<&str>,
    resource_kind: &str,
    data: &T,
) -> Result<()> {
    emit_patch(
        app,
        scope_kind,
        scope_id,
        resource_kind,
        None,
        IncrementalPatchOp::Replace,
        serde_json::to_value(data)?,
    )
}

pub fn emit_delete<R: Runtime>(
    app: &AppHandle<R>,
    scope_kind: &str,
    scope_id: Option<&str>,
    resource_kind: &str,
    resource_id: &str,
) -> Result<()> {
    emit_patch(
        app,
        scope_kind,
        scope_id,
        resource_kind,
        Some(resource_id),
        IncrementalPatchOp::Delete,
        serde_json::Value::Null,
    )
}

fn emit_patch<R: Runtime>(
    app: &AppHandle<R>,
    scope_kind: &str,
    scope_id: Option<&str>,
    resource_kind: &str,
    resource_id: Option<&str>,
    op: IncrementalPatchOp,
    data: serde_json::Value,
) -> Result<()> {
    let event = build_patch_event(scope_kind, scope_id, resource_kind, resource_id, op, data);

    app.emit(INCREMENTAL_PATCH_EVENT, event)
        .map_err(|err| AppError::Other(format!("failed to emit incremental patch event: {err}")))
}

pub fn build_patch_event(
    scope_kind: &str,
    scope_id: Option<&str>,
    resource_kind: &str,
    resource_id: Option<&str>,
    op: IncrementalPatchOp,
    data: serde_json::Value,
) -> IncrementalPatchEvent {
    IncrementalPatchEvent {
        patch_id: ids::new_id(),
        emitted_at: time::now_ms(),
        scope_kind: scope_kind.to_string(),
        scope_id: scope_id.map(str::to_string),
        resource_kind: resource_kind.to_string(),
        resource_id: resource_id.map(str::to_string),
        op,
        data,
    }
}
