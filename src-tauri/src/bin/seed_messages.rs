use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

use buyu_lib::{
    state::AppState,
    utils::{
        ids::new_uuid_v7,
        order_key::{build_order_key, ASSISTANT_POSITION_TAG, USER_POSITION_TAG},
    },
};
use chrono::Utc;

const DEFAULT_MODEL_NAME: &str = "perf-seed";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeedStyle {
    Plain,
    Markdown,
    Mixed,
}

impl SeedStyle {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "plain" => Some(Self::Plain),
            "markdown" => Some(Self::Markdown),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Config {
    count: usize,
    conversation_id: Option<String>,
    title: Option<String>,
    database_url: String,
    style: SeedStyle,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("seed_messages failed: {error}");
        process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let config = parse_args(env::args().skip(1).collect())?;
    let state = AppState::initialize_with_url(&config.database_url).await?;

    let existing_conversation = if let Some(conversation_id) = config.conversation_id.as_deref() {
        sqlx::query_scalar::<_, Option<String>>("SELECT id FROM conversations WHERE id = ?1")
            .bind(conversation_id)
            .fetch_one(&state.db)
            .await?
            .is_some()
    } else {
        false
    };

    let conversation_id = config.conversation_id.clone().unwrap_or_else(new_uuid_v7);

    let now_ms = Utc::now().timestamp_millis();
    let latest_created_at = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT MAX(created_at) FROM message_nodes WHERE conversation_id = ?1",
    )
    .bind(&conversation_id)
    .fetch_one(&state.db)
    .await?;
    let start_timestamp = latest_created_at
        .map(|value| value + 1)
        .unwrap_or(now_ms)
        .max(now_ms);

    let title = config
        .title
        .clone()
        .unwrap_or_else(|| format!("性能测试 {} 条消息", config.count));

    let mut tx = state.db.begin().await?;

    if !existing_conversation {
        sqlx::query(
            r#"
            INSERT INTO conversations (id, title, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(&conversation_id)
        .bind(&title)
        .bind(start_timestamp)
        .bind(start_timestamp)
        .execute(&mut *tx)
        .await?;
    }

    for index in 0..config.count {
        let created_at = start_timestamp + index as i64;
        let is_user = index % 2 == 0;
        let role = if is_user { "user" } else { "assistant" };
        let position_tag = if is_user {
            USER_POSITION_TAG
        } else {
            ASSISTANT_POSITION_TAG
        };
        let node_id = new_uuid_v7();
        let version_id = new_uuid_v7();
        let order_key = build_order_key(created_at, position_tag)
            .map_err(|error| anyhow::anyhow!("failed to build order key: {error}"))?;
        let body = build_body(index, role, config.style);
        let prompt_tokens = if is_user {
            None
        } else {
            Some(40 + (index % 17) as i64)
        };
        let completion_tokens = if is_user {
            None
        } else {
            Some(((body.chars().count() as i64) / 4).max(24))
        };
        let received_at = if is_user { None } else { Some(created_at + 40) };
        let completed_at = if is_user {
            None
        } else {
            Some(created_at + 180)
        };
        let finish_reason = if is_user { None } else { Some("stop") };

        sqlx::query(
            r#"
            INSERT INTO message_nodes (
                id, conversation_id, role, order_key, active_version_id, created_at
            ) VALUES (?1, ?2, ?3, ?4, NULL, ?5)
            "#,
        )
        .bind(&node_id)
        .bind(&conversation_id)
        .bind(role)
        .bind(&order_key)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO message_versions (
                id,
                node_id,
                status,
                model_name,
                prompt_tokens,
                completion_tokens,
                finish_reason,
                received_at,
                completed_at,
                created_at
            ) VALUES (?1, ?2, 'committed', ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&version_id)
        .bind(&node_id)
        .bind(if is_user {
            None
        } else {
            Some(DEFAULT_MODEL_NAME)
        })
        .bind(prompt_tokens)
        .bind(completion_tokens)
        .bind(finish_reason)
        .bind(received_at)
        .bind(completed_at)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO message_contents (
                id, version_id, chunk_index, content_type, body, created_at
            ) VALUES (?1, ?2, 0, 'text/plain', ?3, ?4)
            "#,
        )
        .bind(new_uuid_v7())
        .bind(&version_id)
        .bind(&body)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE message_nodes SET active_version_id = ?1 WHERE id = ?2")
            .bind(&version_id)
            .bind(&node_id)
            .execute(&mut *tx)
            .await?;

        if (index + 1) % 1000 == 0 || index + 1 == config.count {
            println!("seeded {}/{} messages", index + 1, config.count);
        }
    }

    sqlx::query("UPDATE conversations SET updated_at = ?1 WHERE id = ?2")
        .bind(start_timestamp + config.count as i64)
        .bind(&conversation_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    println!("conversation_id={conversation_id}");
    println!("database_url={}", config.database_url);
    println!("count={}", config.count);
    if existing_conversation {
        println!("mode=append");
    } else {
        println!("mode=create");
    }

    Ok(())
}

fn parse_args(args: Vec<String>) -> anyhow::Result<Config> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        process::exit(0);
    }

    let mut count = None;
    let mut conversation_id = None;
    let mut title = None;
    let mut db = None;
    let mut style = SeedStyle::Mixed;

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--count" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("missing value for --count"))?;
                count = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| anyhow::anyhow!("--count must be a positive integer"))?,
                );
            }
            "--conversation-id" => {
                index += 1;
                conversation_id = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("missing value for --conversation-id"))?
                        .clone(),
                );
            }
            "--title" => {
                index += 1;
                title = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("missing value for --title"))?
                        .clone(),
                );
            }
            "--db" => {
                index += 1;
                db = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("missing value for --db"))?
                        .clone(),
                );
            }
            "--style" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("missing value for --style"))?;
                style = SeedStyle::parse(value).ok_or_else(|| {
                    anyhow::anyhow!("--style must be one of: plain, markdown, mixed")
                })?;
            }
            unknown => {
                return Err(anyhow::anyhow!("unknown argument: {unknown}"));
            }
        }
        index += 1;
    }

    let count = count.ok_or_else(|| anyhow::anyhow!("--count is required"))?;
    if count == 0 {
        return Err(anyhow::anyhow!("--count must be greater than 0"));
    }

    Ok(Config {
        count,
        conversation_id,
        title,
        database_url: db
            .map(|value| normalize_db_arg(&value))
            .transpose()?
            .unwrap_or_else(default_database_url),
        style,
    })
}

fn default_database_url() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("buyu.db");
    ensure_parent_dir(&path).expect("failed to create default database parent directory");
    path_to_sqlite_url(&path)
}

fn normalize_db_arg(value: &str) -> anyhow::Result<String> {
    if value.starts_with("sqlite:") {
        return Ok(value.to_string());
    }

    let path = PathBuf::from(value);
    ensure_parent_dir(&path)?;
    Ok(path_to_sqlite_url(&path))
}

fn path_to_sqlite_url(path: &Path) -> String {
    let normalized = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let display = normalized.to_string_lossy().replace('\\', "/");
    format!("sqlite:///{display}")
}

fn ensure_parent_dir(path: &Path) -> anyhow::Result<()> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };

    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

fn build_body(index: usize, role: &str, style: SeedStyle) -> String {
    match style {
        SeedStyle::Plain => build_plain_body(index, role),
        SeedStyle::Markdown => build_markdown_body(index, role),
        SeedStyle::Mixed => {
            if role == "user" {
                build_plain_body(index, role)
            } else {
                build_markdown_body(index, role)
            }
        }
    }
}

fn build_plain_body(index: usize, role: &str) -> String {
    if role == "user" {
        format!(
            "性能测试消息 #{index}: 这是第 {} 条用户输入，用于验证长列表滚动和富文本渲染性能。",
            index + 1
        )
    } else {
        format!(
            "性能测试回复 #{index}: 这是第 {} 条助手输出，包含一段中等长度文本用于观察列表渲染、复制按钮和工具栏行为。",
            index + 1
        )
    }
}

fn build_markdown_body(index: usize, role: &str) -> String {
    if role == "user" {
        return format!(
            "### 用户问题 #{index}\n\n请解释以下公式在聊天记录中的渲染表现：\\(a^2 + b^2 = c^2\\)。\n\n- 目标：压测消息列表\n- 编号：{}\n- 标签：performance, markdown, math",
            index + 1
        );
    }

    format!(
        "### 助手回复 #{index}\n\n这里是一段用于性能测试的富文本内容。\n\n1. 行内公式：\\(e^{{i\\pi}} + 1 = 0\\)\n2. 分式：\\(\\frac{{\\sqrt{{a^2+b^2}}}}{{1+\\frac{{1}}{{x}}}}\\)\n3. 代码块：\n\n```ts\nconst caseId = {index};\nconst mode = \"perf\";\n```\n\n> 这条消息用于观察长列表滚动、代码高亮、KaTeX 和工具栏交互在高消息量下的表现。"
    )
}

fn print_usage() {
    println!(
        "Usage: cargo run --manifest-path src-tauri/Cargo.toml --bin seed_messages -- --count <N> [--conversation-id <ID>] [--title <TEXT>] [--db <PATH_OR_SQLITE_URL>] [--style plain|markdown|mixed]"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_count() {
        let config = parse_args(vec!["--count".into(), "1000".into()]).unwrap();
        assert_eq!(config.count, 1000);
        assert_eq!(config.style, SeedStyle::Mixed);
    }

    #[test]
    fn normalizes_plain_path_to_sqlite_url() {
        let value = normalize_db_arg("tmp/test.db").unwrap();
        assert!(value.starts_with("sqlite:///"));
        assert!(value.ends_with("/tmp/test.db"));
    }
}
