use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use buyu_lib::domain::api_channels::{ApiChannel, ApiChannelModel};
use buyu_lib::domain::messages::{
    MessageRole, ProviderChatEvent, ProviderChatMessage, ProviderChatRequest, ProviderMessagePart,
    ProviderMessagePartKind,
};
use buyu_lib::providers::ProviderRegistry;
use serde_json::json;

struct MockServer {
    addr: SocketAddr,
    base_url: String,
    captured: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockServer {
    fn once(response_headers: &str, response_body: String) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind mock server");
        let addr = listener.local_addr().expect("failed to read mock addr");
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = Arc::clone(&captured);
        let response_headers = response_headers.to_string();

        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let request = read_request(&mut stream);
                captured_clone.lock().unwrap().push(request);

                let response = format!(
                    "HTTP/1.1 200 OK\r\n{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response_headers,
                    response_body.len(),
                    response_body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("failed to write response");
            }
        });

        Self {
            addr,
            base_url: format!("http://{}", addr),
            captured,
            handle: Some(handle),
        }
    }

    fn json(body: serde_json::Value) -> Self {
        Self::once("Content-Type: application/json", body.to_string())
    }

    fn sse(body: String) -> Self {
        Self::once("Content-Type: text/event-stream", body)
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
        if let Some(header_end) = find_header_end(&buffer) {
            let content_length = parse_content_length(&buffer[..header_end]);
            let target_len = header_end + 4 + content_length;
            while buffer.len() < target_len {
                let read = stream.read(&mut temp).expect("failed to read request body");
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&temp[..read]);
            }
            break;
        }
    }
    String::from_utf8_lossy(&buffer).to_string()
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(headers: &[u8]) -> usize {
    let text = String::from_utf8_lossy(headers);
    text.lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn sample_channel(channel_type: &str, base_url: String, api_key: &str) -> ApiChannel {
    ApiChannel {
        id: "ch_1".to_string(),
        name: channel_type.to_string(),
        channel_type: channel_type.to_string(),
        base_url,
        auth_type: "x-api-key".to_string(),
        api_key: Some(api_key.to_string()),
        models_endpoint: None,
        chat_endpoint: None,
        stream_endpoint: None,
        models_mode: "hybrid".to_string(),
        enabled: true,
        sort_order: 0,
        config_json: json!({}),
        created_at: 0,
        updated_at: 0,
    }
}

fn sample_model(channel_id: &str, model_id: &str) -> ApiChannelModel {
    ApiChannelModel {
        id: "model_1".to_string(),
        channel_id: channel_id.to_string(),
        model_id: model_id.to_string(),
        display_name: Some(model_id.to_string()),
        model_type: Some("chat".to_string()),
        context_window: None,
        max_output_tokens: Some(1024),
        capabilities_json: json!({}),
        pricing_json: json!({}),
        default_parameters_json: json!({}),
        sort_order: 0,
        config_json: json!({}),
    }
}

fn sample_request(channel: ApiChannel, model: ApiChannelModel) -> ProviderChatRequest {
    ProviderChatRequest {
        api_channel: channel,
        api_channel_model: model,
        request_parameters_json: json!({"temperature": 0.2}),
        messages: vec![
            ProviderChatMessage {
                role: MessageRole::System,
                name: None,
                parts: vec![ProviderMessagePart {
                    kind: ProviderMessagePartKind::Text,
                    text: Some("You are concise.".to_string()),
                    content: None,
                    metadata_json: json!({}),
                }],
                metadata_json: json!({}),
            },
            ProviderChatMessage {
                role: MessageRole::User,
                name: None,
                parts: vec![ProviderMessagePart {
                    kind: ProviderMessagePartKind::Text,
                    text: Some("Hello".to_string()),
                    content: None,
                    metadata_json: json!({}),
                }],
                metadata_json: json!({}),
            },
        ],
    }
}

#[tokio::test]
async fn providers_registry_includes_new_defaults() {
    let registry = ProviderRegistry::with_defaults();
    registry
        .get("openai_compatible")
        .expect("missing openai provider");
    registry
        .get("anthropic")
        .expect("missing anthropic provider");
    registry.get("gemini").expect("missing gemini provider");
}

#[tokio::test]
async fn anthropic_provider_lists_models_and_chats() {
    let registry = ProviderRegistry::with_defaults();
    let provider = registry
        .get("anthropic")
        .expect("missing anthropic provider");

    let models_server = MockServer::json(json!({
        "data": [
            { "id": "claude-sonnet-4-20250514", "display_name": "Claude Sonnet 4" }
        ]
    }));
    let models_channel = sample_channel("anthropic", models_server.base_url.clone(), "secret");
    let models = provider
        .list_models(&models_channel)
        .await
        .expect("failed to list anthropic models");
    assert_eq!(models[0].model_id, "claude-sonnet-4-20250514");

    let chat_server = MockServer::json(json!({
        "content": [{ "type": "text", "text": "hi from anthropic" }],
        "stop_reason": "end_turn",
        "usage": { "input_tokens": 11, "output_tokens": 7 }
    }));
    let chat_channel = sample_channel("anthropic", chat_server.base_url.clone(), "secret");
    let response = provider
        .chat(sample_request(
            chat_channel.clone(),
            sample_model(&chat_channel.id, "claude-sonnet-4-20250514"),
        ))
        .await
        .expect("failed to chat anthropic");
    assert_eq!(response.parts[0].text.as_deref(), Some("hi from anthropic"));

    let captured = chat_server.captured.lock().unwrap();
    let request_text = captured.first().expect("missing anthropic request");
    assert!(request_text.contains("POST /v1/messages"));
    assert!(request_text.contains("x-api-key: secret"));
    assert!(request_text
        .to_ascii_lowercase()
        .contains("anthropic-version: 2023-06-01"));
    assert!(request_text.contains("\"system\":\"You are concise.\""));
}

#[tokio::test]
async fn gemini_provider_lists_models_and_streams() {
    let registry = ProviderRegistry::with_defaults();
    let provider = registry.get("gemini").expect("missing gemini provider");

    let models_server = MockServer::json(json!({
        "models": [
            {
                "name": "models/gemini-2.5-flash",
                "displayName": "Gemini 2.5 Flash",
                "inputTokenLimit": 1048576,
                "outputTokenLimit": 8192,
                "supportedGenerationMethods": ["generateContent", "streamGenerateContent"]
            }
        ]
    }));
    let models_channel = sample_channel("gemini", models_server.base_url.clone(), "secret");
    let models = provider
        .list_models(&models_channel)
        .await
        .expect("failed to list gemini models");
    assert_eq!(models[0].model_id, "gemini-2.5-flash");

    let stream_server = MockServer::sse(
        [
            json!({
                "candidates": [{ "content": { "parts": [{ "text": "Hello " }] } }]
            }),
            json!({
                "candidates": [{ "content": { "parts": [{ "text": "Gemini" }] }, "finishReason": "STOP" }],
                "usageMetadata": { "promptTokenCount": 5, "candidatesTokenCount": 2, "totalTokenCount": 7 }
            }),
        ]
        .into_iter()
        .map(|chunk| format!("data: {}\n\n", chunk))
        .collect::<String>(),
    );
    let stream_channel = sample_channel("gemini", stream_server.base_url.clone(), "secret");
    let mut seen = Vec::new();
    let response = provider
        .chat_stream(
            sample_request(
                stream_channel.clone(),
                sample_model(&stream_channel.id, "gemini-2.5-flash"),
            ),
            &mut |event| {
                match event {
                    ProviderChatEvent::Delta { parts, .. } => seen.extend(
                        parts
                            .into_iter()
                            .filter_map(|part| part.text)
                            .collect::<Vec<_>>(),
                    ),
                    ProviderChatEvent::Finished { .. } => {}
                }
                Ok(())
            },
        )
        .await
        .expect("failed to stream gemini");

    assert_eq!(response.parts[0].text.as_deref(), Some("Hello Gemini"));
    assert_eq!(seen.concat(), "Hello Gemini");

    let captured = stream_server.captured.lock().unwrap();
    let request_text = captured.first().expect("missing gemini request");
    assert!(
        request_text.contains("POST /v1beta/models/gemini-2.5-flash:streamGenerateContent?alt=sse")
    );
    assert!(request_text
        .to_ascii_lowercase()
        .contains("x-goog-api-key: secret"));
    assert!(request_text.contains("\"generationConfig\":{\"temperature\":0.2}"));
}
