use std::{
    env, fs,
    net::{SocketAddr, TcpListener, TcpStream},
    io::{Read, Write},
    path::PathBuf,
    thread,
};

use buyu_lib::db::pool;
use buyu_lib::domain::api_channels::{CreateApiChannelInput, UpsertApiChannelModelInput};
use buyu_lib::providers::ProviderRegistry;
use buyu_lib::services::api_channels;
use serde_json::json;

struct TestEnv {
    _root: PathBuf,
    db: sqlx::SqlitePool,
    providers: ProviderRegistry,
}

impl TestEnv {
    async fn new() -> Self {
        let root = env::temp_dir()
            .join("buyu-tests")
            .join(uuid::Uuid::now_v7().to_string());
        fs::create_dir_all(&root).expect("failed to create test root");

        let db_path = root.join("test.db");
        let db = pool::init_pool(&db_path).await.expect("failed to init db");

        Self {
            _root: root,
            db,
            providers: ProviderRegistry::with_defaults(),
        }
    }
}

struct MockServer {
    addr: SocketAddr,
    base_url: String,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockServer {
    fn json(body: serde_json::Value) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind mock server");
        let addr = listener.local_addr().expect("failed to read mock addr");
        let body = body.to_string();

        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _request = read_request(&mut stream);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("failed to write response");
            }
        });

        Self {
            addr,
            base_url: format!("http://{}", addr),
            handle: Some(handle),
        }
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 4096];
    loop {
        let read = stream.read(&mut temp).expect("failed to read request");
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    String::from_utf8_lossy(&buffer).to_string()
}

#[tokio::test]
async fn refresh_channel_models_upserts_remote_models_without_removing_local_ones() {
    let env = TestEnv::new().await;
    let server = MockServer::json(json!({
        "data": [
            {
                "id": "gpt-5.1",
                "name": "GPT-5.1",
                "type": "chat",
                "context_window": 200000,
                "max_output_tokens": 8192
            },
            {
                "id": "gpt-4.1-mini",
                "name": "GPT-4.1 mini",
                "type": "chat",
                "context_window": 128000,
                "max_output_tokens": 4096
            }
        ]
    }));

    let channel = api_channels::create_channel(
        &env.db,
        &CreateApiChannelInput {
            name: "OpenAI".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: server.base_url.clone(),
            auth_type: "bearer".to_string(),
            api_key: Some("secret".to_string()),
            models_endpoint: Some("/models".to_string()),
            chat_endpoint: Some("/chat/completions".to_string()),
            stream_endpoint: Some("/chat/completions".to_string()),
            models_mode: "hybrid".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create channel");

    let _local_model = api_channels::upsert_channel_model(
        &env.db,
        &UpsertApiChannelModelInput {
            channel_id: channel.id.clone(),
            model_id: "custom-local-model".to_string(),
            display_name: Some("Custom Local Model".to_string()),
            model_type: Some("chat".to_string()),
            context_window: Some(32000),
            max_output_tokens: Some(2048),
            capabilities_json: json!({}),
            pricing_json: json!({}),
            default_parameters_json: json!({}),
            sort_order: 99,
            config_json: json!({ "manual": true }),
        },
    )
    .await
    .expect("failed to insert local model");

    let refreshed =
        api_channels::refresh_channel_models(&env.db, &env.providers, &channel.id)
            .await
            .expect("failed to refresh remote channel models");

    assert!(
        refreshed.iter().any(|model| model.model_id == "gpt-5.1"),
        "remote model gpt-5.1 should exist after refresh"
    );
    assert!(
        refreshed.iter().any(|model| model.model_id == "gpt-4.1-mini"),
        "remote model gpt-4.1-mini should exist after refresh"
    );
    assert!(
        refreshed
            .iter()
            .any(|model| model.model_id == "custom-local-model"),
        "manual local model should be preserved during refresh"
    );
}
