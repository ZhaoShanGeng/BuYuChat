use uuid::Uuid;

use crate::db::conversation;
use crate::providers::ProviderRegistry;
use crate::services::{keyring::KeyringService, provider::ProviderService};
use crate::types::{ChatRequest, Message, MessageContent, ModelParams, Role};

#[tokio::test]
async fn backend_provider_smoke_test() {
    let (base_url, api_key, model) = match (
        std::env::var("OMNICHAT_TEST_BASE_URL"),
        std::env::var("OMNICHAT_TEST_API_KEY"),
        std::env::var("OMNICHAT_TEST_MODEL"),
    ) {
        (Ok(base_url), Ok(api_key), Ok(model)) => (base_url, api_key, model),
        _ => {
            eprintln!("skipping backend_provider_smoke_test: missing OMNICHAT_TEST_* env");
            return;
        }
    };

    let db_path = std::env::temp_dir().join(format!("omnichat-smoke-{}.db", Uuid::now_v7()));
    let db = crate::db::init_pool(&db_path)
        .await
        .expect("db init failed");

    let providers = ProviderRegistry::new_shared();
    let keyring = KeyringService::new(format!("omnichat-smoke-{}", Uuid::now_v7()));
    let provider_service = ProviderService::new(db.clone(), providers.clone(), keyring.clone());

    provider_service
        .save_config("openai", Some(&api_key), Some(&base_url))
        .await
        .expect("save provider config failed");

    provider_service
        .test_connection("openai")
        .await
        .expect("provider health check failed");

    let models = provider_service
        .list_models("openai")
        .await
        .expect("list models failed");

    assert!(
        !models.is_empty(),
        "provider returned an empty model list from backend service"
    );

    let conv = conversation::create(&db, &model, "openai", None)
        .await
        .expect("create conversation failed");

    let provider = providers
        .get("openai")
        .await
        .expect("provider not registered");
    let response = provider
        .chat(&ChatRequest {
            model: model.clone(),
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text("Reply with exactly: ok".to_string()),
                tool_calls: None,
                tool_call_id: None,
                tool_result: None,
            }],
            system_prompt: None,
            params: ModelParams::default(),
            tools: None,
            stream: false,
        })
        .await
        .expect("chat request failed");

    assert_eq!(conv.model_id, model);
    assert_eq!(conv.provider, "openai");
    assert!(
        !response.content.trim().is_empty(),
        "chat response content should not be empty"
    );

    println!("conversation_id={}", conv.id);
    println!("models_available={}", models.len());
    println!("reply={}", response.content.trim());

    let _ = keyring.delete("provider:openai");
    let _ = std::fs::remove_file(db_path);
}
