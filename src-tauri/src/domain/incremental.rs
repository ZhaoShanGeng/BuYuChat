use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncrementalPatchOp {
    Upsert,
    Delete,
    Replace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalPatchEvent {
    pub patch_id: Id,
    pub emitted_at: TimestampMs,
    pub scope_kind: String,
    pub scope_id: Option<Id>,
    pub resource_kind: String,
    pub resource_id: Option<Id>,
    pub op: IncrementalPatchOp,
    pub data: serde_json::Value,
}
