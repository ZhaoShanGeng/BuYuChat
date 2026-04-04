//! 内置工具注册表。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::builtin;

/// 工具执行结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub content: String,
    pub is_error: bool,
}

/// 内置工具的描述信息（供前端渲染工具面板）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinToolInfo {
    pub name: String,
    pub description: String,
}

/// 内置工具注册表，管理所有可用的内置工具定义与执行器。
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    tools: Vec<RegisteredTool>,
}

#[derive(Debug, Clone)]
struct RegisteredTool {
    name: String,
    description: String,
    schema: Value,
}

impl ToolRegistry {
    /// 创建包含所有内置工具的注册表。
    pub fn new() -> Self {
        Self {
            tools: vec![RegisteredTool {
                name: "fetch".to_string(),
                description: "获取网页内容（HTML 转纯文本，最大 32KB）".to_string(),
                schema: builtin::fetch_tool_schema(),
            }],
        }
    }

    /// 返回所有已注册工具的信息列表。
    pub fn list_tools(&self) -> Vec<BuiltinToolInfo> {
        self.tools
            .iter()
            .map(|tool| BuiltinToolInfo {
                name: tool.name.clone(),
                description: tool.description.clone(),
            })
            .collect()
    }

    /// 按名称列表过滤，返回对应的 OpenAI function calling schemas。
    pub fn schemas_for(&self, names: &[String]) -> Vec<Value> {
        self.tools
            .iter()
            .filter(|tool| names.iter().any(|name| name == &tool.name))
            .map(|tool| tool.schema.clone())
            .collect()
    }

    /// 返回所有工具的 schemas。
    pub fn all_schemas(&self) -> Vec<Value> {
        self.tools.iter().map(|tool| tool.schema.clone()).collect()
    }

    /// 按名称执行工具，返回执行结果。
    pub async fn execute(
        &self,
        http_client: &reqwest::Client,
        name: &str,
        args: &Value,
    ) -> ToolExecutionResult {
        match name {
            "fetch" => builtin::execute_fetch(http_client, args).await,
            _ => ToolExecutionResult {
                content: format!("error: unknown tool '{name}'"),
                is_error: true,
            },
        }
    }

    /// 返回所有内置工具名称。
    pub fn all_tool_names(&self) -> Vec<String> {
        self.tools.iter().map(|tool| tool.name.clone()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
