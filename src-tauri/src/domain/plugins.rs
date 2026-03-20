use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDef {
    pub id: Id,
    pub name: String,
    pub plugin_key: String,
    pub version: String,
    pub runtime_kind: String,
    pub entrypoint: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub capabilities_json: serde_json::Value,
    pub permissions_json: serde_json::Value,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

impl PluginDef {
    pub fn has_capability(&self, capability: &str) -> bool {
        json_has_capability(&self.capabilities_json, capability)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePluginInput {
    pub name: String,
    pub plugin_key: String,
    pub version: String,
    pub runtime_kind: String,
    pub entrypoint: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub capabilities_json: serde_json::Value,
    pub permissions_json: serde_json::Value,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePluginInput {
    pub name: String,
    pub version: String,
    pub runtime_kind: String,
    pub entrypoint: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub capabilities_json: serde_json::Value,
    pub permissions_json: serde_json::Value,
    pub config_json: serde_json::Value,
}

pub(crate) fn json_has_capability(value: &serde_json::Value, capability: &str) -> bool {
    match value {
        serde_json::Value::String(item) => item == capability,
        serde_json::Value::Array(items) => items
            .iter()
            .any(|item| json_has_capability(item, capability)),
        serde_json::Value::Object(map) => map.get(capability).is_some_and(|item| match item {
            serde_json::Value::Bool(flag) => *flag,
            serde_json::Value::Null => false,
            _ => true,
        }),
        _ => false,
    }
}
