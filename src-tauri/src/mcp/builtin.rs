//! 内置工具定义 — fetch。

use std::error::Error as _;

use serde_json::{json, Value};

use super::tools::ToolExecutionResult;

/// 返回 fetch 工具的 OpenAI function calling schema。
pub fn fetch_tool_schema() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "fetch",
            "description": "Fetch content from a URL. Returns the response body as text (HTML tags stripped, max 32KB).",
            "parameters": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch"
                    },
                    "max_length": {
                        "type": "integer",
                        "description": "Max response length in characters (default 32000)"
                    }
                },
                "required": ["url"]
            }
        }
    })
}

/// 执行 fetch 工具调用。
pub async fn execute_fetch(http_client: &reqwest::Client, args: &Value) -> ToolExecutionResult {
    let url = match args.get("url").and_then(|v| v.as_str()) {
        Some(url) if !url.trim().is_empty() => url.trim().to_string(),
        _ => {
            return ToolExecutionResult {
                content: "error: missing or empty 'url' parameter".to_string(),
                is_error: true,
            };
        }
    };

    let max_length = args
        .get("max_length")
        .and_then(|v| v.as_u64())
        .unwrap_or(32_000) as usize;

    let response = match http_client
        .get(&url)
        .header("User-Agent", "BuYu/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            return ToolExecutionResult {
                content: format!(
                    "error: failed to fetch '{url}': {}",
                    format_reqwest_error(&error)
                ),
                is_error: true,
            };
        }
    };

    let status = response.status();
    if !status.is_success() {
        return ToolExecutionResult {
            content: format!("error: HTTP {status} for '{url}'"),
            is_error: true,
        };
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    // 限制 body 大小为 5MB。
    let body = match response.bytes().await {
        Ok(bytes) if bytes.len() <= 5 * 1024 * 1024 => String::from_utf8_lossy(&bytes).to_string(),
        Ok(bytes) => String::from_utf8_lossy(&bytes[..5 * 1024 * 1024]).to_string(),
        Err(error) => {
            return ToolExecutionResult {
                content: format!("error: failed to read response body: {error}"),
                is_error: true,
            };
        }
    };

    let text = if content_type.contains("html") {
        strip_html_tags(&body)
    } else {
        body
    };

    let truncated = if text.len() > max_length {
        let mut end = max_length;
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        format!("{}...\n[truncated at {} chars]", &text[..end], max_length)
    } else {
        text
    };

    ToolExecutionResult {
        content: truncated,
        is_error: false,
    }
}

/// 简易 HTML 标签去除：删除 script/style 块，然后去除所有标签。
fn strip_html_tags(html: &str) -> String {
    // 去除 <script>...</script> 和 <style>...</style> 块。
    let without_scripts = remove_tag_block(html, "script");
    let without_styles = remove_tag_block(&without_scripts, "style");

    // 去除剩余 HTML 标签。
    let mut result = String::with_capacity(without_styles.len());
    let mut inside_tag = false;
    for ch in without_styles.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(ch),
            _ => {}
        }
    }

    // 压缩连续空行。
    let mut collapsed = String::with_capacity(result.len());
    let mut blank_count = 0u32;
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                collapsed.push('\n');
            }
        } else {
            blank_count = 0;
            collapsed.push_str(trimmed);
            collapsed.push('\n');
        }
    }

    collapsed.trim().to_string()
}

fn format_reqwest_error(error: &reqwest::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();

    while let Some(cause) = source {
        let cause_text = cause.to_string();
        if !cause_text.is_empty() && !message.contains(&cause_text) {
            message.push_str(" | caused by: ");
            message.push_str(&cause_text);
        }
        source = cause.source();
    }

    if error.is_timeout() {
        message.push_str(" | category: timeout");
    } else if error.is_connect() {
        message.push_str(" | category: connect");
    } else if error.is_request() {
        message.push_str(" | category: request");
    } else if error.is_body() {
        message.push_str(" | category: body");
    } else if error.is_decode() {
        message.push_str(" | category: decode");
    }

    message
}

/// 移除指定标签对及其内部内容（大小写不敏感）。
fn remove_tag_block(input: &str, tag: &str) -> String {
    let lower = input.to_lowercase();
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut result = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(start) = lower[cursor..].find(&open) {
        let abs_start = cursor + start;
        result.push_str(&input[cursor..abs_start]);

        if let Some(end) = lower[abs_start..].find(&close) {
            cursor = abs_start + end + close.len();
        } else {
            // 没有对应的闭合标签，跳过 open 标签。
            cursor = abs_start + open.len();
        }
    }

    result.push_str(&input[cursor..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_removes_tags_and_scripts() {
        let html = r#"<html><head><script>alert('x')</script><style>body{}</style></head><body><h1>Hello</h1><p>World</p></body></html>"#;
        let text = strip_html_tags(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("body{}"));
        assert!(!text.contains("<"));
    }

    #[test]
    fn strip_html_handles_plain_text() {
        let plain = "no html here";
        assert_eq!(strip_html_tags(plain), "no html here");
    }
}
