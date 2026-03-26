//! 消息查询、版本切换与 AI 生成业务编排。
//!
//! 该模块是消息子系统的 service 入口，负责把文档中的业务规则翻译成一组稳定函数：
//! - 查询侧：消息列表、按需加载版本正文
//! - 版本管理：切换 active version、删除版本
//! - 生成控制：`send_message`、`reroll`、`cancel_generation`
//!
//! 重要设计点：
//! - prompt 上下文始终读取“当下 active version + 当下 system_prompt”，不做快照缓存。
//! - 真正的 AI 调用在 `generation_engine` 里异步执行，service 只负责创建记录与调度任务。
//! - 所有跨表写操作都放在同一事务内完成，保证 node/version/content/updated_at 一致。

use tauri::ipc::Channel as EventChannel;

use crate::{
    ai::adapter::AiChannelConfig,
    error::AppError,
    models::{
        Agent, ChannelModel, DeleteVersionResult, DryRunResult, EditMessageInput,
        EditMessageResult, GenerationEvent, MessageNode, MessageNodeRecord, NewMessageContent,
        NewMessageNode, NewMessageVersion, PromptMessage, RerollInput, RerollResult,
        SendMessageInput, SendMessageResponse, SendMessageResult, VersionContent,
    },
    repo::{
        agent_repo::{AgentRepo, SqlxAgentRepo},
        channel_repo::{ChannelRepo, SqlxChannelRepo},
        conversation_repo::{ConversationRepo, SqlxConversationRepo},
        message_repo,
        model_repo::{ModelRepo, SqlxModelRepo},
    },
    services::generation_engine::{self, GenerationRequest},
    state::AppState,
    utils::{
        ids::new_uuid_v7,
        order_key::{ASSISTANT_POSITION_TAG, USER_POSITION_TAG, build_order_key},
    },
};

/// 发送消息前需要解析出的会话绑定上下文。
struct GenerationContext {
    agent: Agent,
    model: ChannelModel,
    config: AiChannelConfig,
}

/// 使用连接池查询消息列表。
pub async fn list_messages(
    state: &AppState,
    conversation_id: &str,
    before_order_key: Option<String>,
    limit: Option<i64>,
    from_latest: bool,
) -> Result<Vec<MessageNode>, AppError> {
    message_repo::list_messages(
        &state.db,
        conversation_id,
        before_order_key.as_deref(),
        limit,
        from_latest,
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to list messages: {error}")))?
    .ok_or_else(|| AppError::not_found(format!("conversation '{conversation_id}' not found")))
}

/// 按需加载某个版本的完整正文。
pub async fn get_version_content(
    state: &AppState,
    version_id: &str,
) -> Result<VersionContent, AppError> {
    message_repo::get_version_content(&state.db, version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load version content: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("version '{version_id}' not found")))
}

/// 切换楼层的 active version。
pub async fn set_active_version(
    state: &AppState,
    conversation_id: &str,
    node_id: &str,
    version_id: &str,
) -> Result<(), AppError> {
    let node = message_repo::get_node_record(&state.db, conversation_id, node_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load node: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("node '{node_id}' not found")))?;
    let version = message_repo::get_version_meta(&state.db, version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load version: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("version '{version_id}' not found")))?;

    if version.node_id != node.id {
        return Err(AppError::version_not_in_node());
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;
    message_repo::set_node_active_version_tx(&mut tx, node_id, Some(version_id))
        .await
        .map_err(|error| AppError::internal(format!("failed to switch active version: {error}")))?;
    tx.commit()
        .await
        .map_err(|error| AppError::internal(format!("failed to commit active version change: {error}")))?;

    Ok(())
}

/// 删除某个版本，并在必要时切换 fallback version 或删除整个 node。
pub async fn delete_version(
    state: &AppState,
    conversation_id: &str,
    node_id: &str,
    version_id: &str,
) -> Result<DeleteVersionResult, AppError> {
    let node = message_repo::get_node_record(&state.db, conversation_id, node_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load node: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("node '{node_id}' not found")))?;
    let version = message_repo::get_version_meta(&state.db, version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load version: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("version '{version_id}' not found")))?;

    if version.node_id != node.id {
        return Err(AppError::version_not_in_node());
    }

    if version.status == "generating" {
        generation_engine::cancel_generation(state, version_id).await?;
    }

    let versions = message_repo::list_versions_for_node(&state.db, node_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load node versions: {error}")))?;
    let timestamp = current_timestamp_ms();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;

    if versions.len() == 1 {
        let _ = message_repo::delete_node_tx(&mut tx, node_id)
            .await
            .map_err(|error| AppError::internal(format!("failed to delete node: {error}")))?;
        message_repo::touch_conversation_updated_at_tx(&mut tx, conversation_id, timestamp)
            .await
            .map_err(|error| {
                AppError::internal(format!("failed to touch conversation timestamp: {error}"))
            })?;
        tx.commit()
            .await
            .map_err(|error| AppError::internal(format!("failed to commit node deletion: {error}")))?;
        state.cancellation_tokens.remove(version_id);

        return Ok(DeleteVersionResult {
            node_deleted: true,
            new_active_version_id: None,
        });
    }

    let fallback_version_id = if node.active_version_id.as_deref() == Some(version_id) {
        versions
            .iter()
            .rfind(|item| item.id != version_id)
            .map(|item| item.id.clone())
    } else {
        node.active_version_id.clone()
    };

    let _ = message_repo::delete_version_tx(&mut tx, version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to delete version: {error}")))?;
    if node.active_version_id.as_deref() == Some(version_id) {
        message_repo::set_node_active_version_tx(&mut tx, node_id, fallback_version_id.as_deref())
            .await
            .map_err(|error| {
                AppError::internal(format!("failed to restore fallback version: {error}"))
            })?;
    }
    message_repo::touch_conversation_updated_at_tx(&mut tx, conversation_id, timestamp)
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to touch conversation timestamp: {error}"))
        })?;
    tx.commit()
        .await
        .map_err(|error| AppError::internal(format!("failed to commit version deletion: {error}")))?;
    state.cancellation_tokens.remove(version_id);

    Ok(DeleteVersionResult {
        node_deleted: false,
        new_active_version_id: fallback_version_id,
    })
}

/// 发送用户消息并创建对应的 assistant generating version。
pub async fn send_message(
    state: &AppState,
    conversation_id: &str,
    input: SendMessageInput,
    event_channel: Option<EventChannel<GenerationEvent>>,
) -> Result<SendMessageResponse, AppError> {
    let content = normalize_message_content(&input.content)?;
    let context = resolve_generation_context(state, conversation_id).await?;
    let prompt_messages = build_prompt_with_system(
        &context.agent,
        message_repo::build_prompt_messages(&state.db, conversation_id, None, None)
            .await
            .map_err(|error| AppError::internal(format!("failed to build prompt history: {error}")))?,
    );

    let mut final_prompt_messages = prompt_messages;
    final_prompt_messages.push(PromptMessage {
        role: "user".to_string(),
        content: content.clone(),
    });

    if input.dry_run.unwrap_or(false) {
        return Ok(SendMessageResponse::DryRun(DryRunResult {
            total_tokens_estimate: estimate_tokens(&final_prompt_messages),
            messages: final_prompt_messages,
            model: context.model.model_id.clone(),
        }));
    }

    let stream = input.stream.unwrap_or(true);
    let created = create_send_message_records(state, conversation_id, &context, &content).await?;

    generation_engine::spawn_generation(
        state,
        GenerationRequest {
            conversation_id: conversation_id.to_string(),
            assistant_node_id: created.assistant_node_id.clone(),
            assistant_version_id: created.assistant_version_id.clone(),
            prompt_messages: final_prompt_messages,
            config: context.config,
            max_output_tokens: context.model.max_output_tokens,
            stream,
            event_channel,
        },
    );

    Ok(SendMessageResponse::Started(created))
}

/// 对指定楼层执行 reroll。
pub async fn reroll(
    state: &AppState,
    conversation_id: &str,
    node_id: &str,
    input: RerollInput,
    event_channel: Option<EventChannel<GenerationEvent>>,
) -> Result<RerollResult, AppError> {
    let context = resolve_generation_context(state, conversation_id).await?;
    let node = message_repo::get_node_record(&state.db, conversation_id, node_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load node: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("node '{node_id}' not found")))?;
    let stream = input.stream.unwrap_or(true);

    match node.role.as_str() {
        "assistant" => {
            let prompt_messages = build_prompt_with_system(
                &context.agent,
                message_repo::build_prompt_messages(
                    &state.db,
                    conversation_id,
                    Some(&node.order_key),
                    None,
                )
                .await
                .map_err(|error| {
                    AppError::internal(format!("failed to build assistant reroll prompt: {error}"))
                })?,
            );
            let result =
                create_assistant_reroll_records(state, conversation_id, &context, &node).await?;

            generation_engine::spawn_generation(
                state,
                GenerationRequest {
                    conversation_id: conversation_id.to_string(),
                    assistant_node_id: result.assistant_node_id.clone(),
                    assistant_version_id: result.assistant_version_id.clone(),
                    prompt_messages,
                    config: context.config,
                    max_output_tokens: context.model.max_output_tokens,
                    stream,
                    event_channel,
                },
            );

            Ok(result)
        }
        "user" => {
            let active_content = message_repo::get_active_version_content_for_node(&state.db, &node)
                .await
                .map_err(|error| {
                    AppError::internal(format!("failed to load active user version content: {error}"))
                })?
                .ok_or_else(|| {
                    AppError::internal("active user version content missing".to_string())
                })?;

            let mut prompt_messages = build_prompt_with_system(
                &context.agent,
                message_repo::build_prompt_messages(
                    &state.db,
                    conversation_id,
                    Some(&node.order_key),
                    None,
                )
                .await
                .map_err(|error| {
                    AppError::internal(format!("failed to build user reroll prompt: {error}"))
                })?,
            );
            prompt_messages.push(PromptMessage {
                role: "user".to_string(),
                content: active_content.content.clone(),
            });

            let result =
                create_user_followup_records(state, conversation_id, &context, &node).await?;

            generation_engine::spawn_generation(
                state,
                GenerationRequest {
                    conversation_id: conversation_id.to_string(),
                    assistant_node_id: result.assistant_node_id.clone(),
                    assistant_version_id: result.assistant_version_id.clone(),
                    prompt_messages,
                    config: context.config,
                    max_output_tokens: context.model.max_output_tokens,
                    stream,
                    event_channel,
                },
            );

            Ok(result)
        }
        _ => Err(AppError::internal(format!(
            "unsupported node role '{}'",
            node.role
        ))),
    }
}

/// 编辑当前楼层的 active version，并可选地基于新版本重新发送。
pub async fn edit_message(
    state: &AppState,
    conversation_id: &str,
    node_id: &str,
    input: EditMessageInput,
    event_channel: Option<EventChannel<GenerationEvent>>,
) -> Result<EditMessageResult, AppError> {
    let node = message_repo::get_node_record(&state.db, conversation_id, node_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load node: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("node '{node_id}' not found")))?;
    let active_version_id = node
        .active_version_id
        .as_deref()
        .ok_or_else(|| AppError::internal("node has no active version".to_string()))?;
    let active_version = message_repo::get_version_meta(&state.db, active_version_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load active version: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("version '{active_version_id}' not found")))?;
    if active_version.status == "generating" {
        return Err(AppError::validation(
            "MESSAGE_STILL_GENERATING",
            "cannot edit a generating version",
        ));
    }

    let active_content = message_repo::get_active_version_content_for_node(&state.db, &node)
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to load active version content: {error}"))
        })?
        .ok_or_else(|| AppError::internal("active version content missing".to_string()))?;
    let content = normalize_message_content(&input.content)?;
    let edited_version_id = create_committed_version_for_node(
        state,
        conversation_id,
        &node,
        &content,
        &active_content.content_type,
    )
    .await?;

    if !input.resend.unwrap_or(false) {
        return Ok(EditMessageResult {
            edited_version_id,
            assistant_node_id: None,
            assistant_version_id: None,
        });
    }

    let context = resolve_generation_context(state, conversation_id).await?;
    let stream = input.stream.unwrap_or(true);

    match node.role.as_str() {
        "assistant" => {
            let prompt_messages = build_prompt_with_system(
                &context.agent,
                message_repo::build_prompt_messages(
                    &state.db,
                    conversation_id,
                    Some(&node.order_key),
                    None,
                )
                .await
                .map_err(|error| {
                    AppError::internal(format!("failed to build assistant resend prompt: {error}"))
                })?,
            );
            let reroll_result =
                create_assistant_reroll_records(state, conversation_id, &context, &node).await?;

            generation_engine::spawn_generation(
                state,
                GenerationRequest {
                    conversation_id: conversation_id.to_string(),
                    assistant_node_id: reroll_result.assistant_node_id.clone(),
                    assistant_version_id: reroll_result.assistant_version_id.clone(),
                    prompt_messages,
                    config: context.config,
                    max_output_tokens: context.model.max_output_tokens,
                    stream,
                    event_channel,
                },
            );

            Ok(EditMessageResult {
                edited_version_id,
                assistant_node_id: Some(reroll_result.assistant_node_id),
                assistant_version_id: Some(reroll_result.assistant_version_id),
            })
        }
        "user" => {
            let mut prompt_messages = build_prompt_with_system(
                &context.agent,
                message_repo::build_prompt_messages(
                    &state.db,
                    conversation_id,
                    Some(&node.order_key),
                    None,
                )
                .await
                .map_err(|error| {
                    AppError::internal(format!("failed to build user resend prompt: {error}"))
                })?,
            );
            prompt_messages.push(PromptMessage {
                role: "user".to_string(),
                content: content.clone(),
            });

            let reroll_result =
                create_user_followup_records(state, conversation_id, &context, &node).await?;

            generation_engine::spawn_generation(
                state,
                GenerationRequest {
                    conversation_id: conversation_id.to_string(),
                    assistant_node_id: reroll_result.assistant_node_id.clone(),
                    assistant_version_id: reroll_result.assistant_version_id.clone(),
                    prompt_messages,
                    config: context.config,
                    max_output_tokens: context.model.max_output_tokens,
                    stream,
                    event_channel,
                },
            );

            Ok(EditMessageResult {
                edited_version_id,
                assistant_node_id: Some(reroll_result.assistant_node_id),
                assistant_version_id: Some(reroll_result.assistant_version_id),
            })
        }
        _ => Err(AppError::internal(format!(
            "unsupported node role '{}'",
            node.role
        ))),
    }
}

/// 取消指定 generating version 的后台任务。
pub async fn cancel_generation(state: &AppState, version_id: &str) -> Result<(), AppError> {
    generation_engine::cancel_generation(state, version_id).await
}

/// 解析发送消息所需的会话绑定上下文。
async fn resolve_generation_context(
    state: &AppState,
    conversation_id: &str,
) -> Result<GenerationContext, AppError> {
    let conversation_repo = SqlxConversationRepo::new(state.db.clone());
    let agent_repo = SqlxAgentRepo::new(state.db.clone());
    let channel_repo = SqlxChannelRepo::new(state.db.clone());
    let model_repo = SqlxModelRepo::new(state.db.clone());

    let conversation = conversation_repo
        .get(conversation_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load conversation: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("conversation '{conversation_id}' not found")))?;
    let agent_id = conversation.agent_id.as_deref().ok_or_else(AppError::no_agent)?;
    let channel_id = conversation.channel_id.as_deref().ok_or_else(AppError::no_channel)?;
    let model_id = conversation
        .channel_model_id
        .as_deref()
        .ok_or_else(AppError::no_model)?;

    let agent = agent_repo
        .get(agent_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load agent: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("agent '{agent_id}' not found")))?;
    if !agent.enabled {
        return Err(AppError::agent_disabled());
    }

    let channel = channel_repo
        .get(channel_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load channel: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("channel '{channel_id}' not found")))?;
    if !channel.enabled {
        return Err(AppError::channel_disabled());
    }

    let model = model_repo
        .get_by_channel_and_id(channel_id, model_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load model: {error}")))?
        .ok_or_else(AppError::no_model)?;
    let config = AiChannelConfig::try_from(&channel)?.with_model_name(model.model_id.clone());

    Ok(GenerationContext {
        agent,
        model,
        config,
    })
}

/// 构建包含 system_prompt 的最终 prompt 列表。
fn build_prompt_with_system(agent: &Agent, mut messages: Vec<PromptMessage>) -> Vec<PromptMessage> {
    if let Some(system_prompt) = agent
        .system_prompt
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        messages.insert(
            0,
            PromptMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
        );
    }

    messages
}

/// 在 send_message 路径下创建 user node / assistant node 及其版本。
async fn create_send_message_records(
    state: &AppState,
    conversation_id: &str,
    context: &GenerationContext,
    content: &str,
) -> Result<SendMessageResult, AppError> {
    for _ in 0..3 {
        let timestamp = current_timestamp_ms();
        let user_node_id = new_uuid_v7();
        let user_version_id = new_uuid_v7();
        let assistant_node_id = new_uuid_v7();
        let assistant_version_id = new_uuid_v7();
        let user_order_key = build_order_key(timestamp, USER_POSITION_TAG)?;
        let assistant_order_key = build_order_key(timestamp, ASSISTANT_POSITION_TAG)?;

        let mut tx = state
            .db
            .begin()
            .await
            .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;

        let result = async {
            message_repo::insert_node_tx(
                &mut tx,
                &NewMessageNode {
                    id: user_node_id.clone(),
                    conversation_id: conversation_id.to_string(),
                    author_agent_id: None,
                    role: "user".to_string(),
                    order_key: user_order_key,
                    created_at: timestamp,
                },
            )
            .await?;
            message_repo::insert_version_tx(
                &mut tx,
                &NewMessageVersion {
                    id: user_version_id.clone(),
                    node_id: user_node_id.clone(),
                    status: "committed".to_string(),
                    model_name: None,
                    created_at: timestamp,
                },
            )
            .await?;
            message_repo::insert_content_tx(
                &mut tx,
                &NewMessageContent {
                    id: new_uuid_v7(),
                    version_id: user_version_id.clone(),
                    chunk_index: 0,
                    content_type: "text/plain".to_string(),
                    body: content.to_string(),
                    created_at: timestamp,
                },
            )
            .await?;
            message_repo::set_node_active_version_tx(&mut tx, &user_node_id, Some(&user_version_id))
                .await?;

            message_repo::insert_node_tx(
                &mut tx,
                &NewMessageNode {
                    id: assistant_node_id.clone(),
                    conversation_id: conversation_id.to_string(),
                    author_agent_id: Some(context.agent.id.clone()),
                    role: "assistant".to_string(),
                    order_key: assistant_order_key,
                    created_at: timestamp,
                },
            )
            .await?;
            message_repo::insert_version_tx(
                &mut tx,
                &NewMessageVersion {
                    id: assistant_version_id.clone(),
                    node_id: assistant_node_id.clone(),
                    status: "generating".to_string(),
                    model_name: Some(context.model.model_id.clone()),
                    created_at: timestamp,
                },
            )
            .await?;
            message_repo::set_node_active_version_tx(
                &mut tx,
                &assistant_node_id,
                Some(&assistant_version_id),
            )
            .await?;
            message_repo::touch_conversation_updated_at_tx(&mut tx, conversation_id, timestamp)
                .await?;
            Ok::<(), String>(())
        }
        .await;

        match result {
            Ok(()) => {
                tx.commit().await.map_err(|error| {
                    AppError::internal(format!("failed to commit send message transaction: {error}"))
                })?;
                return Ok(SendMessageResult {
                    user_node_id,
                    user_version_id,
                    assistant_node_id,
                    assistant_version_id,
                });
            }
            Err(error) if is_order_key_conflict(&error) => {
                tx.rollback().await.map_err(|rollback_error| {
                    AppError::internal(format!(
                        "failed to rollback send message order key retry: {rollback_error}"
                    ))
                })?;
            }
            Err(error) => {
                tx.rollback().await.map_err(|rollback_error| {
                    AppError::internal(format!(
                        "failed to rollback send message transaction: {rollback_error}"
                    ))
                })?;
                return Err(AppError::internal(format!(
                    "failed to create message records: {error}"
                )));
            }
        }
    }

    Err(AppError::internal(
        "failed to allocate unique order_key after 3 attempts",
    ))
}

/// 在 assistant reroll 路径下创建新版本并切换 active_version。
async fn create_assistant_reroll_records(
    state: &AppState,
    conversation_id: &str,
    context: &GenerationContext,
    node: &MessageNodeRecord,
) -> Result<RerollResult, AppError> {
    let timestamp = current_timestamp_ms();
    let assistant_version_id = new_uuid_v7();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;

    message_repo::insert_version_tx(
        &mut tx,
        &NewMessageVersion {
            id: assistant_version_id.clone(),
            node_id: node.id.clone(),
            status: "generating".to_string(),
            model_name: Some(context.model.model_id.clone()),
            created_at: timestamp,
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to create reroll version: {error}")))?;
    message_repo::set_node_active_version_tx(&mut tx, &node.id, Some(&assistant_version_id))
        .await
        .map_err(|error| AppError::internal(format!("failed to switch reroll version: {error}")))?;
    message_repo::touch_conversation_updated_at_tx(&mut tx, conversation_id, timestamp)
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to touch conversation timestamp: {error}"))
        })?;
    tx.commit()
        .await
        .map_err(|error| AppError::internal(format!("failed to commit assistant reroll: {error}")))?;

    Ok(RerollResult {
        new_user_version_id: None,
        assistant_node_id: node.id.clone(),
        assistant_version_id,
    })
}

/// 在当前楼层下创建一个 committed version 并切换 active version。
async fn create_committed_version_for_node(
    state: &AppState,
    conversation_id: &str,
    node: &MessageNodeRecord,
    content: &str,
    content_type: &str,
) -> Result<String, AppError> {
    let timestamp = current_timestamp_ms();
    let edited_version_id = new_uuid_v7();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;

    message_repo::insert_version_tx(
        &mut tx,
        &NewMessageVersion {
            id: edited_version_id.clone(),
            node_id: node.id.clone(),
            status: "committed".to_string(),
            model_name: None,
            created_at: timestamp,
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to create edited version: {error}")))?;
    message_repo::insert_content_tx(
        &mut tx,
        &NewMessageContent {
            id: new_uuid_v7(),
            version_id: edited_version_id.clone(),
            chunk_index: 0,
            content_type: content_type.to_string(),
            body: content.to_string(),
            created_at: timestamp,
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to persist edited content: {error}")))?;
    message_repo::set_node_active_version_tx(&mut tx, &node.id, Some(&edited_version_id))
        .await
        .map_err(|error| AppError::internal(format!("failed to activate edited version: {error}")))?;
    message_repo::touch_conversation_updated_at_tx(&mut tx, conversation_id, timestamp)
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to touch conversation timestamp: {error}"))
        })?;
    tx.commit()
        .await
        .map_err(|error| AppError::internal(format!("failed to commit edited version: {error}")))?;

    Ok(edited_version_id)
}

/// 为 user node 生成回复：优先复用紧邻 assistant node，否则插入新的 assistant node。
async fn create_user_followup_records(
    state: &AppState,
    conversation_id: &str,
    context: &GenerationContext,
    user_node: &MessageNodeRecord,
) -> Result<RerollResult, AppError> {
    let next_node = message_repo::get_next_node(&state.db, conversation_id, &user_node.order_key)
        .await
        .map_err(|error| AppError::internal(format!("failed to load next node: {error}")))?;

    if let Some(next_node) = next_node {
        if next_node.role == "assistant" {
            return create_assistant_reroll_records(state, conversation_id, context, &next_node)
                .await;
        }
    }

    create_inserted_assistant_records(state, conversation_id, context, user_node).await
}

/// 在指定 user node 后插入一个新的 assistant node，并重新分配该会话的顺序键。
async fn create_inserted_assistant_records(
    state: &AppState,
    conversation_id: &str,
    context: &GenerationContext,
    user_node: &MessageNodeRecord,
) -> Result<RerollResult, AppError> {
    let existing_nodes = message_repo::list_node_records(&state.db, conversation_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to list conversation nodes: {error}")))?;
    let insert_index = existing_nodes
        .iter()
        .position(|node| node.id == user_node.id)
        .ok_or_else(|| AppError::not_found(format!("node '{}' not found", user_node.id)))?
        + 1;
    let timestamp = current_timestamp_ms();
    let assistant_node_id = new_uuid_v7();
    let assistant_version_id = new_uuid_v7();
    let temporary_order_key = format!("tmp-{timestamp:016}-{assistant_node_id}");
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|error| AppError::internal(format!("failed to open transaction: {error}")))?;

    message_repo::insert_node_tx(
        &mut tx,
        &NewMessageNode {
            id: assistant_node_id.clone(),
            conversation_id: conversation_id.to_string(),
            author_agent_id: Some(context.agent.id.clone()),
            role: "assistant".to_string(),
            order_key: temporary_order_key,
            created_at: timestamp,
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to insert assistant node: {error}")))?;
    message_repo::insert_version_tx(
        &mut tx,
        &NewMessageVersion {
            id: assistant_version_id.clone(),
            node_id: assistant_node_id.clone(),
            status: "generating".to_string(),
            model_name: Some(context.model.model_id.clone()),
            created_at: timestamp,
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to create assistant version: {error}")))?;
    message_repo::set_node_active_version_tx(&mut tx, &assistant_node_id, Some(&assistant_version_id))
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to activate assistant version: {error}"))
        })?;

    reindex_conversation_nodes_tx(
        &mut tx,
        &existing_nodes,
        insert_index,
        &assistant_node_id,
        "assistant",
    )
    .await?;

    message_repo::touch_conversation_updated_at_tx(&mut tx, conversation_id, timestamp)
        .await
        .map_err(|error| {
            AppError::internal(format!("failed to touch conversation timestamp: {error}"))
        })?;
    tx.commit().await.map_err(|error| {
        AppError::internal(format!("failed to commit inserted assistant node: {error}"))
    })?;

    Ok(RerollResult {
        new_user_version_id: None,
        assistant_node_id,
        assistant_version_id,
    })
}

/// 为包含插入节点的新顺序重建整段会话的 order_key。
async fn reindex_conversation_nodes_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    existing_nodes: &[MessageNodeRecord],
    insert_index: usize,
    inserted_node_id: &str,
    inserted_role: &str,
) -> Result<(), AppError> {
    let temp_prefix = format!("tmp-reindex-{}-", current_timestamp_ms());
    let mut ordered_nodes = existing_nodes
        .iter()
        .map(|node| (node.id.clone(), node.role.clone()))
        .collect::<Vec<_>>();
    ordered_nodes.insert(
        insert_index,
        (inserted_node_id.to_string(), inserted_role.to_string()),
    );

    for (index, (node_id, _)) in ordered_nodes.iter().enumerate() {
        let temporary_key = format!("{temp_prefix}{index:04}-{node_id}");
        message_repo::update_node_order_key_tx(tx, node_id, &temporary_key)
            .await
            .map_err(|error| {
                AppError::internal(format!("failed to assign temporary order key: {error}"))
            })?;
    }

    let base_timestamp = current_timestamp_ms();
    for (index, (node_id, role)) in ordered_nodes.iter().enumerate() {
        let position_tag = if role == "assistant" {
            ASSISTANT_POSITION_TAG
        } else {
            USER_POSITION_TAG
        };
        let order_key = build_order_key(base_timestamp + index as i64, position_tag)?;
        message_repo::update_node_order_key_tx(tx, node_id, &order_key)
            .await
            .map_err(|error| {
                AppError::internal(format!("failed to assign final order key: {error}"))
            })?;
    }

    Ok(())
}

/// 规范化发送消息时的用户正文。
fn normalize_message_content(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(
            "VALIDATION_ERROR",
            "content must not be empty",
        ));
    }

    Ok(trimmed.to_string())
}

/// 用简单估算值返回 dry_run 所需的 token 统计。
fn estimate_tokens(messages: &[PromptMessage]) -> i64 {
    let total_bytes = messages
        .iter()
        .map(|message| message.role.len() + message.content.len())
        .sum::<usize>();
    ((total_bytes as i64) + 3) / 4
}

/// 判断数据库错误是否为 order_key 唯一约束冲突。
fn is_order_key_conflict(error: &str) -> bool {
    error.contains("message_nodes.conversation_id")
        && error.contains("message_nodes.order_key")
        && error.contains("UNIQUE")
}

/// 返回当前毫秒时间戳。
fn current_timestamp_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
