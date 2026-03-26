//! AI 生成任务执行引擎。
//!
//! 这个模块只负责“已经拿到 prompt 之后”的后台生成生命周期，不再处理会话绑定、
//! reroll 语义或消息楼层创建。这些前置业务决策由 `message_service` 完成。
//!
//! 引擎职责包括：
//! - 为每个 generating version 建立并管理 `CancellationToken`
//! - 受 `Semaphore(5)` 控制的并发生成排队
//! - 流式模式下按 `2048 bytes / 2 秒` 刷盘到 `message_contents`
//! - 统一写入终态：`committed / failed / cancelled`
//! - 在 AI 返回空文本时执行自动回滚，并发送 `empty_rollback` 事件
//!
//! 当前实现继续使用 `AiAdapter`，即 `aisdk + aisdk-macros` 体系下的统一 AI 接入层。

use std::time::Duration;

use aisdk::core::{
    LanguageModelStreamChunkType,
    language_model::ReasoningEffort,
};
use tauri::ipc::Channel;
use tokio::time::{self, MissedTickBehavior};
use tokio_util::sync::CancellationToken;

use crate::{
    ai::adapter::{AiAdapter, AiChannelConfig, AiChatCompletion},
    error::AppError,
    models::{GenerationEvent, MessageVersionPatch, NewMessageContent, PromptMessage},
    repo::message_repo,
    state::AppState,
    utils::ids::new_uuid_v7,
};

/// 后台生成任务的静态输入快照。
pub struct GenerationRequest {
    pub conversation_id: String,
    pub assistant_node_id: String,
    pub assistant_version_id: String,
    pub prompt_messages: Vec<PromptMessage>,
    pub config: AiChannelConfig,
    pub max_output_tokens: Option<i64>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub thinking_tags: Vec<String>,
    pub stream: bool,
    pub event_channel: Option<Channel<GenerationEvent>>,
}

/// 启动后台生成任务。
///
/// 调用方只需要保证 generating version 已经写入数据库；后续状态流转由引擎接管。
pub fn spawn_generation(state: &AppState, request: GenerationRequest) {
    let token = CancellationToken::new();
    state
        .cancellation_tokens
        .insert(request.assistant_version_id.clone(), token.clone());

    let state = state.clone();
    tauri::async_runtime::spawn(async move {
        run_generation(state, request, token).await;
    });
}

/// 取消指定版本的后台生成任务。
///
/// 该操作对不存在版本、已终结版本都保持幂等。
pub async fn cancel_generation(state: &AppState, version_id: &str) -> Result<(), AppError> {
    if let Some(token) = state.cancellation_tokens.get(version_id) {
        token.cancel();
        return Ok(());
    }

    let Some(version) = message_repo::get_version_meta(&state.db, version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load version: {error}")))? else {
        return Ok(());
    };

    if version.status != "generating" {
        return Ok(());
    }

    let timestamp = current_timestamp_ms();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;
    let _ = message_repo::update_version_tx(
        &mut tx,
        version_id,
        &MessageVersionPatch {
            status: Some("cancelled".to_string()),
            ..MessageVersionPatch::default()
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to mark version cancelled: {error}")))?;
    if let Some(node) = message_repo::get_node_record_by_version(&state.db, version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load version node: {error}")))? {
        message_repo::touch_conversation_updated_at_tx(&mut tx, &node.conversation_id, timestamp)
            .await
            .map_err(|error| {
                AppError::internal(format!("failed to touch conversation timestamp: {error}"))
            })?;
    }
    tx.commit()
        .await
        .map_err(|error| AppError::internal(format!("failed to commit cancellation: {error}")))?;

    Ok(())
}

/// 执行单个后台生成任务的完整生命周期。
async fn run_generation(state: AppState, request: GenerationRequest, token: CancellationToken) {
    let permit = tokio::select! {
        _ = token.cancelled() => {
            let _ = mark_cancelled(&state, &request).await;
            state.cancellation_tokens.remove(&request.assistant_version_id);
            return;
        }
        permit = state.generation_semaphore.clone().acquire_owned() => {
            match permit {
                Ok(permit) => permit,
                Err(error) => {
                    let _ = mark_failed(
                        &state,
                        &request,
                        AppError::internal(format!("failed to acquire generation permit: {error}")),
                    ).await;
                    state.cancellation_tokens.remove(&request.assistant_version_id);
                    return;
                }
            }
        }
    };

    let result = if request.stream {
        run_stream_generation(&state, &request, &token).await
    } else {
        run_non_stream_generation(&state, &request, &token).await
    };

    drop(permit);
    state.cancellation_tokens.remove(&request.assistant_version_id);

    if let Err(error) = result {
        let _ = mark_failed(&state, &request, error).await;
    }
}

/// 执行非流式生成。
async fn run_non_stream_generation(
    state: &AppState,
    request: &GenerationRequest,
    token: &CancellationToken,
) -> Result<(), AppError> {
    let adapter = AiAdapter;
    let completion = tokio::select! {
        _ = token.cancelled() => {
            mark_cancelled(state, request).await?;
            return Ok(());
        }
        completion = adapter.generate_chat(
            &request.config,
            &request.prompt_messages,
            request.max_output_tokens,
            request.reasoning_effort,
        ) => completion?,
    };

    finalize_completion(state, request, completion, None, None).await
}

/// 执行流式生成，并按文档约定刷盘内容块。
async fn run_stream_generation(
    state: &AppState,
    request: &GenerationRequest,
    token: &CancellationToken,
) -> Result<(), AppError> {
    let adapter = AiAdapter;
    let mut handle = adapter
        .stream_chat(
            &request.config,
            &request.prompt_messages,
            request.max_output_tokens,
            request.reasoning_effort,
        )
        .await?;

    let mut pending_buffer = String::new();
    let mut pending_reasoning_buffer = String::new();
    let mut has_persisted_content = false;
    let mut has_persisted_reasoning = false;
    let mut thinking_detector = (!request.thinking_tags.is_empty())
        .then(|| ThinkingTagDetector::new(request.thinking_tags.clone()));
    let mut interval = time::interval(Duration::from_secs(2));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    interval.tick().await;

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                let _ = flush_pending_chunks(state, request, &mut pending_buffer).await?;
                flush_pending_thinking(state, request, &mut pending_reasoning_buffer).await?;
                mark_cancelled(state, request).await?;
                return Ok(());
            }
            _ = interval.tick() => {
                has_persisted_content |= flush_pending_chunks(state, request, &mut pending_buffer).await?;
                has_persisted_reasoning |= flush_pending_thinking(state, request, &mut pending_reasoning_buffer).await?;
            }
            maybe_chunk = handle.next_chunk() => {
                match maybe_chunk {
                    Some(LanguageModelStreamChunkType::Text(delta)) => {
                        let segments = if let Some(detector) = &mut thinking_detector {
                            detector.feed(&delta)
                        } else {
                            vec![TextSegment::Normal(delta)]
                        };

                        for segment in segments {
                            match segment {
                                TextSegment::Normal(normal) => {
                                    if request.stream && !normal.is_empty() {
                                        emit_event(
                                            &request.event_channel,
                                            GenerationEvent::Chunk {
                                                conversation_id: request.conversation_id.clone(),
                                                node_id: request.assistant_node_id.clone(),
                                                version_id: request.assistant_version_id.clone(),
                                                delta: normal.clone(),
                                                reasoning_delta: None,
                                            },
                                        );
                                    }
                                    pending_buffer.push_str(&normal);
                                    if pending_buffer.len() >= 2_048 {
                                        has_persisted_content |= flush_pending_chunks(state, request, &mut pending_buffer).await?;
                                    }
                                }
                                TextSegment::Thinking(thinking) => {
                                    if request.stream && !thinking.is_empty() {
                                        emit_event(
                                            &request.event_channel,
                                            GenerationEvent::Chunk {
                                                conversation_id: request.conversation_id.clone(),
                                                node_id: request.assistant_node_id.clone(),
                                                version_id: request.assistant_version_id.clone(),
                                                delta: String::new(),
                                                reasoning_delta: Some(thinking.clone()),
                                            },
                                        );
                                    }
                                    pending_reasoning_buffer.push_str(&thinking);
                                    if pending_reasoning_buffer.len() >= 2_048 {
                                        has_persisted_reasoning |=
                                            flush_pending_thinking(state, request, &mut pending_reasoning_buffer).await?;
                                    }
                                }
                            }
                        }
                    }
                    Some(LanguageModelStreamChunkType::Reasoning(delta)) => {
                        if request.stream && !delta.is_empty() {
                            emit_event(
                                &request.event_channel,
                                GenerationEvent::Chunk {
                                    conversation_id: request.conversation_id.clone(),
                                    node_id: request.assistant_node_id.clone(),
                                    version_id: request.assistant_version_id.clone(),
                                    delta: String::new(),
                                    reasoning_delta: Some(delta.clone()),
                                },
                            );
                        }
                        pending_reasoning_buffer.push_str(&delta);
                        if pending_reasoning_buffer.len() >= 2_048 {
                            has_persisted_reasoning |=
                                flush_pending_thinking(state, request, &mut pending_reasoning_buffer).await?;
                        }
                    }
                    Some(LanguageModelStreamChunkType::Failed(error))
                    | Some(LanguageModelStreamChunkType::Incomplete(error)) => {
                        return Err(AppError::ai_request_failed(error));
                    }
                    Some(_) => {}
                    None => {
                        break;
                    }
                }
            }
        }
    }

    if let Some(detector) = &mut thinking_detector {
        for segment in detector.finish() {
            match segment {
                TextSegment::Normal(normal) => pending_buffer.push_str(&normal),
                TextSegment::Thinking(thinking) => pending_reasoning_buffer.push_str(&thinking),
            }
        }
    }

    has_persisted_content |= flush_pending_chunks(state, request, &mut pending_buffer).await?;
    has_persisted_reasoning |= flush_pending_thinking(state, request, &mut pending_reasoning_buffer).await?;

    let completion = handle.finish().await?;

    finalize_completion(
        state,
        request,
        completion,
        Some(if has_persisted_content { 1 } else { 0 }),
        Some(if has_persisted_reasoning { 1 } else { 0 }),
    )
    .await
}

/// 完成生成后的统一收尾逻辑。
async fn finalize_completion(
    state: &AppState,
    request: &GenerationRequest,
    completion: AiChatCompletion,
    existing_content_bytes: Option<usize>,
    existing_reasoning_bytes: Option<usize>,
) -> Result<(), AppError> {
    let (visible_text, extracted_thinking) = split_thinking_from_text(
        &completion.text,
        &request.thinking_tags,
    );

    // 流式兼容层在部分服务商上可能只返回 delta，不在最终消息里回填完整正文。
    // 只要前面已经有内容刷入 `message_contents`，这里就不能再按“空消息”回滚。
    if visible_text.trim().is_empty() && existing_content_bytes.unwrap_or(0) == 0 {
        rollback_empty_version(state, request).await?;
        return Ok(());
    }

    if existing_content_bytes.unwrap_or(0) == 0 && !visible_text.is_empty() {
        let chunk_index = message_repo::next_chunk_index(&state.db, &request.assistant_version_id)
            .await
            .map_err(|error| AppError::internal(format!("failed to load next chunk index: {error}")))?;
        message_repo::append_content_chunk(
            &state.db,
            &NewMessageContent {
                id: new_uuid_v7(),
                version_id: request.assistant_version_id.clone(),
                chunk_index,
                content_type: "text/plain".to_string(),
                body: visible_text,
                created_at: current_timestamp_ms(),
            },
        )
        .await
        .map_err(|error| AppError::internal(format!("failed to persist completion content: {error}")))?;
    }

    if existing_reasoning_bytes.unwrap_or(0) == 0 && !extracted_thinking.is_empty() {
        let chunk_index = message_repo::next_chunk_index(&state.db, &request.assistant_version_id)
            .await
            .map_err(|error| AppError::internal(format!("failed to load next chunk index: {error}")))?;
        message_repo::append_content_chunk(
            &state.db,
            &NewMessageContent {
                id: new_uuid_v7(),
                version_id: request.assistant_version_id.clone(),
                chunk_index,
                content_type: "text/thinking".to_string(),
                body: extracted_thinking,
                created_at: current_timestamp_ms(),
            },
        )
        .await
        .map_err(|error| AppError::internal(format!("failed to persist thinking content: {error}")))?;
    }

    let timestamp = current_timestamp_ms();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;
    let updated = message_repo::update_version_tx(
        &mut tx,
        &request.assistant_version_id,
        &MessageVersionPatch {
            status: Some("committed".to_string()),
            prompt_tokens: Some(Some(completion.prompt_tokens)),
            completion_tokens: Some(Some(completion.completion_tokens)),
            finish_reason: Some(Some(completion.finish_reason.clone())),
            model_name: Some(Some(completion.model.clone())),
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to finalize version: {error}")))?;

    if !updated {
        tx.rollback()
            .await
            .map_err(|error| AppError::internal(format!("failed to rollback missing version: {error}")))?;
        return Ok(());
    }

    message_repo::touch_conversation_updated_at_tx(&mut tx, &request.conversation_id, timestamp)
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to touch conversation timestamp: {error}"))
        })?;
    tx.commit()
        .await
        .map_err(|error| AppError::internal(format!("failed to commit completion: {error}")))?;

    emit_event(
        &request.event_channel,
        GenerationEvent::Completed {
            conversation_id: request.conversation_id.clone(),
            node_id: request.assistant_node_id.clone(),
            version_id: request.assistant_version_id.clone(),
            prompt_tokens: completion.prompt_tokens,
            completion_tokens: completion.completion_tokens,
            finish_reason: completion.finish_reason,
            model: completion.model,
        },
    );

    Ok(())
}

/// 终止任务并把版本标记为失败。
async fn mark_failed(
    state: &AppState,
    request: &GenerationRequest,
    error: AppError,
) -> Result<(), AppError> {
    let timestamp = current_timestamp_ms();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|db_error| AppError::internal(format!("failed to open transaction: {db_error}")))?;
    let updated = message_repo::update_version_tx(
        &mut tx,
        &request.assistant_version_id,
        &MessageVersionPatch {
            status: Some("failed".to_string()),
            finish_reason: Some(Some("error".to_string())),
            ..MessageVersionPatch::default()
        },
    )
    .await
    .map_err(|db_error| AppError::internal(format!("failed to mark version failed: {db_error}")))?;

    if !updated {
        tx.rollback()
            .await
            .map_err(|db_error| AppError::internal(format!("failed to rollback missing version: {db_error}")))?;
        return Ok(());
    }

    message_repo::touch_conversation_updated_at_tx(&mut tx, &request.conversation_id, timestamp)
        .await
        .map_err(|db_error| {
            AppError::internal(format!("failed to touch conversation timestamp: {db_error}"))
        })?;
    tx.commit()
        .await
        .map_err(|db_error| AppError::internal(format!("failed to commit failed status: {db_error}")))?;

    emit_event(
        &request.event_channel,
        GenerationEvent::Failed {
            conversation_id: request.conversation_id.clone(),
            node_id: request.assistant_node_id.clone(),
            version_id: request.assistant_version_id.clone(),
            error: error.message,
        },
    );

    Ok(())
}

/// 终止任务并把版本标记为取消。
async fn mark_cancelled(state: &AppState, request: &GenerationRequest) -> Result<(), AppError> {
    let timestamp = current_timestamp_ms();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;
    let updated = message_repo::update_version_tx(
        &mut tx,
        &request.assistant_version_id,
        &MessageVersionPatch {
            status: Some("cancelled".to_string()),
            finish_reason: Some(Some("cancelled".to_string())),
            ..MessageVersionPatch::default()
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to mark version cancelled: {error}")))?;

    if !updated {
        tx.rollback()
            .await
            .map_err(|error| AppError::internal(format!("failed to rollback missing version: {error}")))?;
        return Ok(());
    }

    message_repo::touch_conversation_updated_at_tx(&mut tx, &request.conversation_id, timestamp)
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to touch conversation timestamp: {error}"))
        })?;
    tx.commit()
        .await
        .map_err(|error| AppError::internal(format!("failed to commit cancelled status: {error}")))?;

    emit_event(
        &request.event_channel,
        GenerationEvent::Cancelled {
            conversation_id: request.conversation_id.clone(),
            node_id: request.assistant_node_id.clone(),
            version_id: request.assistant_version_id.clone(),
        },
    );

    Ok(())
}

/// 处理 AI 返回空文本时的自动回滚。
async fn rollback_empty_version(
    state: &AppState,
    request: &GenerationRequest,
) -> Result<(), AppError> {
    let node = message_repo::get_node_record(&state.db, &request.conversation_id, &request.assistant_node_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load node: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("node '{}' not found", request.assistant_node_id)))?;
    let versions = message_repo::list_versions_for_node(&state.db, &request.assistant_node_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load node versions: {error}")))?;
    let fallback = versions
        .iter()
        .rfind(|version| version.id != request.assistant_version_id)
        .cloned();

    let timestamp = current_timestamp_ms();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;

    if fallback.is_none() {
        let _ = message_repo::delete_node_tx(&mut tx, &request.assistant_node_id)
            .await
            .map_err(|error| AppError::internal(format!("failed to delete empty node: {error}")))?;
        message_repo::touch_conversation_updated_at_tx(&mut tx, &request.conversation_id, timestamp)
            .await
            .map_err(|error| {
                AppError::internal(format!("failed to touch conversation timestamp: {error}"))
            })?;
        tx.commit()
            .await
            .map_err(|error| AppError::internal(format!("failed to commit empty rollback: {error}")))?;

        emit_event(
            &request.event_channel,
            GenerationEvent::EmptyRollback {
                conversation_id: request.conversation_id.clone(),
                node_id: request.assistant_node_id.clone(),
                node_deleted: true,
                fallback_version_id: None,
            },
        );
        return Ok(());
    }

    let _ = message_repo::delete_version_tx(&mut tx, &request.assistant_version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to delete empty version: {error}")))?;

    let fallback_version_id = if node.active_version_id.as_deref()
        == Some(request.assistant_version_id.as_str())
    {
        let fallback_version_id = fallback.as_ref().map(|version| version.id.clone());
        message_repo::set_node_active_version_tx(
            &mut tx,
            &request.assistant_node_id,
            fallback_version_id.as_deref(),
        )
        .await
        .map_err(|error| AppError::internal(format!("failed to restore fallback version: {error}")))?;
        fallback_version_id
    } else {
        node.active_version_id.clone()
    };

    message_repo::touch_conversation_updated_at_tx(&mut tx, &request.conversation_id, timestamp)
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to touch conversation timestamp: {error}"))
        })?;
    tx.commit()
        .await
        .map_err(|error| AppError::internal(format!("failed to commit empty rollback: {error}")))?;

    emit_event(
        &request.event_channel,
        GenerationEvent::EmptyRollback {
            conversation_id: request.conversation_id.clone(),
            node_id: request.assistant_node_id.clone(),
            node_deleted: false,
            fallback_version_id,
        },
    );

    Ok(())
}

/// 把当前缓冲区追加为新的内容块。
async fn flush_pending_chunks(
    state: &AppState,
    request: &GenerationRequest,
    pending_buffer: &mut String,
) -> Result<bool, AppError> {
    flush_pending_buffer(state, request, pending_buffer, "text/plain").await
}

async fn flush_pending_thinking(
    state: &AppState,
    request: &GenerationRequest,
    pending_buffer: &mut String,
) -> Result<bool, AppError> {
    flush_pending_buffer(state, request, pending_buffer, "text/thinking").await
}

async fn flush_pending_buffer(
    state: &AppState,
    request: &GenerationRequest,
    pending_buffer: &mut String,
    content_type: &str,
) -> Result<bool, AppError> {
    if pending_buffer.is_empty() {
        return Ok(false);
    }

    let body = pending_buffer.clone();
    let chunk_index = message_repo::next_chunk_index(&state.db, &request.assistant_version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load next chunk index: {error}")))?;

    message_repo::append_content_chunk(
        &state.db,
        &NewMessageContent {
            id: new_uuid_v7(),
            version_id: request.assistant_version_id.clone(),
            chunk_index,
            content_type: content_type.to_string(),
            body,
            created_at: current_timestamp_ms(),
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to append content chunk: {error}")))?;

    pending_buffer.clear();
    Ok(true)
}

/// 向前端事件通道发送生成事件。
fn emit_event(channel: &Option<Channel<GenerationEvent>>, event: GenerationEvent) {
    if let Some(channel) = channel {
        let _ = channel.send(event);
    }
}

/// 统一生成当前毫秒时间戳。
fn current_timestamp_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[derive(Debug, Clone)]
enum TextSegment {
    Normal(String),
    Thinking(String),
}

#[derive(Debug, Clone)]
struct ThinkingTagDetector {
    tags: Vec<String>,
    active_tag: Option<String>,
    carry: String,
    max_open_tag_len: usize,
}

impl ThinkingTagDetector {
    fn new(tags: Vec<String>) -> Self {
        let max_open_tag_len = tags
            .iter()
            .map(|tag| tag.len() + 2)
            .max()
            .unwrap_or(0);

        Self {
            tags,
            active_tag: None,
            carry: String::new(),
            max_open_tag_len,
        }
    }

    fn feed(&mut self, text: &str) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let mut remaining = format!("{}{}", self.carry, text);
        self.carry.clear();

        loop {
            if let Some(tag) = &self.active_tag {
                let closing_tag = format!("</{tag}>");
                if let Some(index) = remaining.find(&closing_tag) {
                    let content = remaining[..index].to_string();
                    if !content.is_empty() {
                        segments.push(TextSegment::Thinking(content));
                    }
                    remaining = remaining[index + closing_tag.len()..].to_string();
                    self.active_tag = None;
                    continue;
                }

                let keep = remaining.len().min(closing_tag.len().saturating_sub(1));
                let emit_len = remaining.len().saturating_sub(keep);
                if emit_len > 0 {
                    segments.push(TextSegment::Thinking(remaining[..emit_len].to_string()));
                }
                self.carry = remaining[emit_len..].to_string();
                break;
            }

            if let Some((index, tag)) = self.find_next_opening(&remaining) {
                let before = remaining[..index].to_string();
                if !before.is_empty() {
                    segments.push(TextSegment::Normal(before));
                }
                remaining = remaining[index + tag.len() + 2..].to_string();
                self.active_tag = Some(tag);
                continue;
            }

            let keep = remaining.len().min(self.max_open_tag_len.saturating_sub(1));
            let emit_len = remaining.len().saturating_sub(keep);
            if emit_len > 0 {
                segments.push(TextSegment::Normal(remaining[..emit_len].to_string()));
            }
            self.carry = remaining[emit_len..].to_string();
            break;
        }

        segments
    }

    fn finish(&mut self) -> Vec<TextSegment> {
        if self.carry.is_empty() {
            return Vec::new();
        }

        let tail = std::mem::take(&mut self.carry);
        if self.active_tag.is_some() {
            vec![TextSegment::Thinking(tail)]
        } else {
            vec![TextSegment::Normal(tail)]
        }
    }

    fn find_next_opening(&self, text: &str) -> Option<(usize, String)> {
        self.tags
            .iter()
            .filter_map(|tag| text.find(&format!("<{tag}>")).map(|index| (index, tag.clone())))
            .min_by_key(|(index, _)| *index)
    }
}

fn split_thinking_from_text(content: &str, tags: &[String]) -> (String, String) {
    if tags.is_empty() {
        return (content.to_string(), String::new());
    }

    let mut detector = ThinkingTagDetector::new(tags.to_vec());
    let mut body = String::new();
    let mut thinking = String::new();

    for segment in detector.feed(content).into_iter().chain(detector.finish()) {
        match segment {
            TextSegment::Normal(text) => body.push_str(&text),
            TextSegment::Thinking(text) => thinking.push_str(&text),
        }
    }

    (body, thinking)
}
