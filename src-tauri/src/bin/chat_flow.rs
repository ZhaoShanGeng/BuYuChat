use std::sync::Arc;

use omnichat_lib::db::{conversation, message};
use omnichat_lib::providers::{openai::OpenAiProvider, ProviderRegistry};
use omnichat_lib::services::{chat::ChatService, versioning::VersioningService};

#[tokio::main]
async fn main() {
    omnichat_lib::services::logging::init_logging();
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args = parse_args(std::env::args().skip(1).collect())?;

    let db_path =
        std::env::temp_dir().join(format!("omnichat-chat-flow-{}.db", uuid::Uuid::now_v7()));
    let db = omnichat_lib::db::init_pool(&db_path)
        .await
        .map_err(|err| err.to_string())?;

    let providers = ProviderRegistry::new_shared();
    providers
        .register(Arc::new(OpenAiProvider::new(
            Some(args.api_key.clone()),
            Some(args.base_url.clone()),
        )))
        .await;

    let conv = conversation::create(&db, &args.model, "openai", None)
        .await
        .map_err(|err| err.to_string())?;
    println!("conversation_id={}", conv.id);

    let chat_service = ChatService::new(db.clone(), providers.clone());
    let versioning_service = VersioningService::new(db.clone());

    let (user_msg_id, assistant_msg_id) = chat_service
        .send_message_no_emit(&conv.id, args.prompt.clone(), None)
        .await
        .map_err(|err| err.to_string())?;
    println!("send_message user_msg_id={user_msg_id} assistant_msg_id={assistant_msg_id}");

    let assistant_row = message::get(&db, &assistant_msg_id)
        .await
        .map_err(|err| err.to_string())?;
    println!(
        "assistant_v1={}",
        assistant_row.content.clone().unwrap_or_default().trim()
    );

    let regenerated_id = chat_service
        .regenerate_no_emit(&conv.id)
        .await
        .map_err(|err| err.to_string())?;
    println!("regenerate assistant_msg_id={regenerated_id}");

    let versions = message::list_versions(&db, &assistant_row.version_group_id)
        .await
        .map_err(|err| err.to_string())?;
    println!("assistant_versions={}", versions.len());

    let switched = versioning_service
        .switch_version(&assistant_row.version_group_id, 1)
        .await
        .map_err(|err| err.to_string())?;
    println!(
        "switched_to_v1={}",
        switched.content.clone().unwrap_or_default().trim()
    );

    let edited_user_msg_id = chat_service
        .save_message_edit(&conv.id, &user_msg_id, args.edit_prompt.clone())
        .await
        .map_err(|err| err.to_string())?;
    println!("save_message_edit user_msg_id={edited_user_msg_id}");

    let edited_assistant_msg_id = chat_service
        .regenerate_no_emit(&conv.id)
        .await
        .map_err(|err| err.to_string())?;
    println!("regenerate_after_edit assistant_msg_id={edited_assistant_msg_id}");

    let final_messages = message::list_active(&db, &conv.id)
        .await
        .map_err(|err| err.to_string())?;
    println!("final_active_messages={}", final_messages.len());
    for row in final_messages {
        println!(
            "{} [{}] {}",
            row.id,
            row.role,
            row.content.unwrap_or_default().replace('\n', "\\n")
        );
    }

    let _ = std::fs::remove_file(db_path);
    Ok(())
}

struct CliArgs {
    base_url: String,
    api_key: String,
    model: String,
    prompt: String,
    edit_prompt: String,
}

fn parse_args(args: Vec<String>) -> Result<CliArgs, String> {
    let mut base_url = None;
    let mut api_key = None;
    let mut model = None;
    let mut prompt = Some("Reply with exactly: ok".to_string());
    let mut edit_prompt = Some("Reply with exactly: edited".to_string());

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--base-url" => {
                i += 1;
                base_url = Some(value_at(&args, i, "--base-url")?.to_string());
            }
            "--api-key" => {
                i += 1;
                api_key = Some(value_at(&args, i, "--api-key")?.to_string());
            }
            "--model" => {
                i += 1;
                model = Some(value_at(&args, i, "--model")?.to_string());
            }
            "--prompt" => {
                i += 1;
                prompt = Some(value_at(&args, i, "--prompt")?.to_string());
            }
            "--edit-prompt" => {
                i += 1;
                edit_prompt = Some(value_at(&args, i, "--edit-prompt")?.to_string());
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }

    Ok(CliArgs {
        base_url: base_url.ok_or_else(|| "missing --base-url".to_string())?,
        api_key: api_key.ok_or_else(|| "missing --api-key".to_string())?,
        model: model.ok_or_else(|| "missing --model".to_string())?,
        prompt: prompt.unwrap(),
        edit_prompt: edit_prompt.unwrap(),
    })
}

fn value_at<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn print_help() {
    println!("Usage:");
    println!(
        "  cargo run --bin chat_flow -- --base-url <url> --api-key <key> --model <model> [--prompt <text>] [--edit-prompt <text>]"
    );
}
