use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;

use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::db::models::{
    ConversationParticipantRow, SummaryGroupRow, WorkflowDefEdgeRow, WorkflowDefNodeRow,
    WorkflowDefRow, WorkflowRunNodeExecutionRow, WorkflowRunRow,
};
use crate::db::repos::{
    agents as agent_repo, conversations as conversation_repo, messages as message_repo,
    summaries as summary_repo, workflows as workflow_repo,
};
use crate::domain::content::{ContentType, ContentWriteInput, StoredContent};
use crate::domain::conversations::CreateConversationInput;
use crate::domain::messages::{
    BuildGenerationContextInput, BuiltGenerationContext, ContextPolicy, CreateMessageInput,
    GenerationContextItem, MessageRole, MessageVersionView, ViewerPolicy,
};
use crate::domain::native_capabilities::{
    CreateMcpEventInput, CreateRagRefInput, FinishToolInvocationInput, StartToolInvocationInput,
};
use crate::domain::variables::{
    CreateVariableDefInput, SetVariableValueInput, VariableScopeType, VariableValueType,
};
use crate::domain::workflows::{RunWorkflowInput, WorkflowRunResult};
use crate::providers::ProviderRegistry;
use crate::services::content;
use crate::services::content_store::ContentStore;
use crate::services::{
    context_builder, conversations, generation, mcp, messages, plugins, rag, tool_invocations,
    variables,
};
use crate::support::error::{AppError, Result};
use crate::support::time;

pub async fn run_workflow(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &RunWorkflowInput,
) -> Result<WorkflowRunResult> {
    run_workflow_boxed(db, store, providers, input).await
}

fn run_workflow_boxed<'a>(
    db: &'a SqlitePool,
    store: &'a ContentStore,
    providers: &'a ProviderRegistry,
    input: &'a RunWorkflowInput,
) -> Pin<Box<dyn Future<Output = Result<WorkflowRunResult>> + Send + 'a>> {
    Box::pin(async move {
        let workflow = workflow_repo::get_workflow_def(db, &input.workflow_def_id).await?;
        let nodes = workflow_repo::list_workflow_def_nodes(db, &input.workflow_def_id).await?;
        let edges = workflow_repo::list_workflow_def_edges(db, &input.workflow_def_id).await?;

        if nodes.is_empty() {
            return Err(AppError::Validation(
                "workflow definition must contain at least one node".to_string(),
            ));
        }

        validate_run_input(db, input).await?;
        let graph = WorkflowGraph::new(nodes, edges);
        validate_runnable_graph(&graph)?;

        let workspace_conversation_id =
            resolve_root_workspace_conversation(db, &workflow, input).await?;
        let entry_node = select_entry_node(&graph).ok_or_else(|| {
            AppError::Validation("workflow definition has no entry node".to_string())
        })?;
        let request_snapshot = create_request_snapshot_content(db, store, input).await?;

        let run = workflow_repo::create_workflow_run(
            db,
            &workflow_repo::CreateWorkflowRunRecord {
                workflow_def_id: &input.workflow_def_id,
                trigger_conversation_id: input.conversation_id.as_deref(),
                workspace_conversation_id: workspace_conversation_id.as_deref(),
                workspace_mode: if input.isolated_conversation_title.is_some() {
                    "isolated_conversation"
                } else {
                    "current_conversation"
                },
                trigger_message_version_id: input.trigger_message_version_id.as_deref(),
                entry_node_id: Some(&entry_node.id),
                status: "running",
                result_message_version_id: None,
                request_snapshot_content_id: Some(&request_snapshot.content_id),
                result_content_id: None,
                config_json: &input.config_json.to_string(),
                started_at: Some(time::now_ms()),
                finished_at: None,
            },
        )
        .await?;

        let entry_execution = workflow_repo::create_workflow_run_node_execution(
            db,
            &workflow_repo::CreateWorkflowRunNodeExecutionRecord {
                workflow_run_id: &run.id,
                workflow_def_node_id: &entry_node.id,
                parent_execution_id: None,
                incoming_edge_id: None,
                branch_key: None,
                loop_iteration: 0,
                retry_index: 0,
                status: "pending",
                generation_run_id: None,
                input_snapshot_content_id: Some(&request_snapshot.content_id),
                output_content_id: None,
                error_content_id: None,
                started_at: Some(time::now_ms()),
                finished_at: None,
                config_json: &json!({}).to_string(),
            },
        )
        .await?;

        let mut queue = VecDeque::from([entry_execution.id.clone()]);
        let mut final_run = None;

        while let Some(execution_id) = queue.pop_front() {
            let current_run = workflow_repo::get_workflow_run(db, &run.id).await?;
            if is_terminal_run_status(&current_run.status) {
                final_run = Some(current_run);
                break;
            }

            let execution =
                workflow_repo::get_workflow_run_node_execution(db, &execution_id).await?;
            if execution.status != "pending" {
                continue;
            }

            let node = graph.node(&execution.workflow_def_node_id)?.clone();
            match execute_node(
                db,
                store,
                providers,
                input,
                &workflow,
                &current_run,
                &execution,
                &node,
            )
            .await
            {
                Ok(outcome) => {
                    let finished_execution = workflow_repo::finish_workflow_run_node_execution(
                        db,
                        &execution.id,
                        &workflow_repo::FinishWorkflowRunNodeExecutionRecord {
                            status: &outcome.status,
                            generation_run_id: outcome.generation_run_id.as_deref(),
                            output_content_id: outcome
                                .output_content
                                .as_ref()
                                .map(|item| item.content_id.as_str()),
                            error_content_id: None,
                            finished_at: Some(time::now_ms()),
                            config_json: &outcome.execution_config.to_string(),
                        },
                    )
                    .await?;

                    if let Some(finalization) = outcome.finalization {
                        let run_config_json =
                            build_final_run_config(&current_run.config_json, &finalization)?;
                        let run_row = workflow_repo::finish_workflow_run(
                            db,
                            &current_run.id,
                            &workflow_repo::FinishWorkflowRunRecord {
                                status: &finalization.status,
                                result_message_version_id: finalization
                                    .result_message_version_id
                                    .as_deref(),
                                result_content_id: finalization.result_content_id.as_deref(),
                                finished_at: Some(time::now_ms()),
                                config_json: &run_config_json.to_string(),
                            },
                        )
                        .await?;
                        final_run = Some(run_row);
                        break;
                    }

                    let scheduled = match schedule_successors(
                        db,
                        store,
                        &graph,
                        &current_run,
                        &finished_execution,
                        &node,
                        &outcome,
                    )
                    .await
                    {
                        Ok(scheduled) => scheduled,
                        Err(err) => {
                            let error_content =
                                create_error_content(db, store, &err.to_string()).await?;
                            let run_config_json = build_failed_run_config(
                                &current_run.config_json,
                                &err,
                                &error_content,
                            )?;
                            let run_row = workflow_repo::finish_workflow_run(
                                db,
                                &current_run.id,
                                &workflow_repo::FinishWorkflowRunRecord {
                                    status: "failed",
                                    result_message_version_id: None,
                                    result_content_id: None,
                                    finished_at: Some(time::now_ms()),
                                    config_json: &run_config_json.to_string(),
                                },
                            )
                            .await?;
                            final_run = Some(run_row);
                            break;
                        }
                    };
                    queue.extend(scheduled);
                }
                Err(err) => {
                    let error_content = create_error_content(db, store, &err.to_string()).await?;
                    let _ = workflow_repo::finish_workflow_run_node_execution(
                        db,
                        &execution.id,
                        &workflow_repo::FinishWorkflowRunNodeExecutionRecord {
                            status: "failed",
                            generation_run_id: None,
                            output_content_id: None,
                            error_content_id: Some(&error_content.content_id),
                            finished_at: Some(time::now_ms()),
                            config_json: &json!({ "error": err.to_string() }).to_string(),
                        },
                    )
                    .await?;

                    let run_config_json =
                        build_failed_run_config(&current_run.config_json, &err, &error_content)?;
                    let run_row = workflow_repo::finish_workflow_run(
                        db,
                        &current_run.id,
                        &workflow_repo::FinishWorkflowRunRecord {
                            status: "failed",
                            result_message_version_id: None,
                            result_content_id: None,
                            finished_at: Some(time::now_ms()),
                            config_json: &run_config_json.to_string(),
                        },
                    )
                    .await?;
                    final_run = Some(run_row);
                    break;
                }
            }
        }

        let final_run = match final_run {
            Some(run) => run,
            None => {
                let current_run = workflow_repo::get_workflow_run(db, &run.id).await?;
                if is_terminal_run_status(&current_run.status) {
                    current_run
                } else {
                    let run_config_json = build_failed_run_config(
                        &current_run.config_json,
                        &AppError::Other(
                            "workflow finished without producing a terminal result".to_string(),
                        ),
                        &request_snapshot,
                    )?;
                    workflow_repo::finish_workflow_run(
                        db,
                        &current_run.id,
                        &workflow_repo::FinishWorkflowRunRecord {
                            status: "failed",
                            result_message_version_id: None,
                            result_content_id: None,
                            finished_at: Some(time::now_ms()),
                            config_json: &run_config_json.to_string(),
                        },
                    )
                    .await?
                }
            }
        };

        Ok(WorkflowRunResult {
            workflow_run_id: final_run.id,
            status: final_run.status,
            entry_node_id: final_run.entry_node_id,
            workspace_conversation_id: final_run.workspace_conversation_id,
            result_message_version_id: final_run.result_message_version_id,
        })
    })
}

struct WorkflowGraph {
    nodes_by_id: HashMap<String, WorkflowDefNodeRow>,
    edges_by_from: HashMap<String, Vec<WorkflowDefEdgeRow>>,
    incoming_by_to: HashMap<String, Vec<WorkflowDefEdgeRow>>,
}

impl WorkflowGraph {
    fn new(nodes: Vec<WorkflowDefNodeRow>, edges: Vec<WorkflowDefEdgeRow>) -> Self {
        let mut nodes_by_id = HashMap::with_capacity(nodes.len());
        for node in nodes {
            nodes_by_id.insert(node.id.clone(), node);
        }

        let mut edges_by_from = HashMap::<String, Vec<WorkflowDefEdgeRow>>::new();
        let mut incoming_by_to = HashMap::<String, Vec<WorkflowDefEdgeRow>>::new();
        for edge in edges {
            edges_by_from
                .entry(edge.from_node_id.clone())
                .or_default()
                .push(edge.clone());
            incoming_by_to
                .entry(edge.to_node_id.clone())
                .or_default()
                .push(edge);
        }

        for list in edges_by_from.values_mut() {
            list.sort_by(|lhs, rhs| lhs.priority.cmp(&rhs.priority).then(lhs.id.cmp(&rhs.id)));
        }
        for list in incoming_by_to.values_mut() {
            list.sort_by(|lhs, rhs| lhs.priority.cmp(&rhs.priority).then(lhs.id.cmp(&rhs.id)));
        }

        Self {
            nodes_by_id,
            edges_by_from,
            incoming_by_to,
        }
    }

    fn node(&self, id: &str) -> Result<&WorkflowDefNodeRow> {
        self.nodes_by_id.get(id).ok_or_else(|| AppError::NotFound {
            entity: "workflow_def_node",
            id: id.to_string(),
        })
    }

    fn outgoing(&self, node_id: &str) -> &[WorkflowDefEdgeRow] {
        self.edges_by_from
            .get(node_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn incoming(&self, node_id: &str) -> &[WorkflowDefEdgeRow] {
        self.incoming_by_to
            .get(node_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn nodes(&self) -> impl Iterator<Item = &WorkflowDefNodeRow> {
        self.nodes_by_id.values()
    }
}

#[derive(Clone)]
struct ExecutionInputMaterial {
    content: StoredContent,
    source_kind: String,
    source_message_node_id: Option<String>,
    source_message_version_id: Option<String>,
    source_tool_invocation_id: Option<String>,
    source_rag_ref_id: Option<String>,
    source_mcp_event_id: Option<String>,
    source_plugin_id: Option<String>,
}

struct ExecutionOutcome {
    status: String,
    output_content: Option<StoredContent>,
    generation_run_id: Option<String>,
    execution_config: Value,
    finalization: Option<WorkflowFinalization>,
}

struct WorkflowFinalization {
    status: String,
    result_message_version_id: Option<String>,
    result_content_id: Option<String>,
}

async fn validate_run_input(db: &SqlitePool, input: &RunWorkflowInput) -> Result<()> {
    if let Some(conversation_id) = input.conversation_id.as_deref() {
        let _ = conversation_repo::get_conversation(db, conversation_id).await?;
    }

    if let Some(trigger_message_version_id) = input.trigger_message_version_id.as_deref() {
        let version = message_repo::get_message_version(db, trigger_message_version_id).await?;
        let node = message_repo::get_message_node(db, &version.node_id).await?;
        if let Some(conversation_id) = input.conversation_id.as_deref() {
            if node.conversation_id != conversation_id {
                return Err(AppError::Validation(
                    "trigger_message_version_id does not belong to conversation_id".to_string(),
                ));
            }
        }
    }

    Ok(())
}

fn validate_runnable_graph(graph: &WorkflowGraph) -> Result<()> {
    let supported = HashSet::from([
        "input",
        "agent",
        "tool",
        "plugin",
        "rag",
        "mcp",
        "router",
        "merge",
        "loop",
        "writeback",
        "output",
        "subflow",
    ]);

    for node in graph.nodes() {
        if !supported.contains(node.node_type.as_str()) {
            return Err(AppError::Validation(format!(
                "workflow node type '{}' is not executable in v1",
                node.node_type
            )));
        }

        if node.node_type == "loop" {
            let config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
            let max_iterations = config
                .get("max_iterations")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            if max_iterations <= 0 {
                return Err(AppError::Validation(format!(
                    "loop node '{}' requires config_json.max_iterations > 0",
                    node.node_key
                )));
            }
        }

        if node.node_type == "subflow" {
            let config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
            if config
                .get("subflow_workflow_def_id")
                .and_then(Value::as_str)
                .is_none()
            {
                return Err(AppError::Validation(format!(
                    "subflow node '{}' requires config_json.subflow_workflow_def_id",
                    node.node_key
                )));
            }
        }
    }

    for edges in graph.edges_by_from.values() {
        for edge in edges {
            if edge.edge_type == "loop_back" {
                let source = graph.node(&edge.from_node_id)?;
                if source.node_type != "loop" {
                    return Err(AppError::Validation(format!(
                        "loop_back edge '{}' must originate from a loop node",
                        edge.id
                    )));
                }
            }
        }
    }

    Ok(())
}

async fn create_request_snapshot_content(
    db: &SqlitePool,
    store: &ContentStore,
    input: &RunWorkflowInput,
) -> Result<StoredContent> {
    content::create_content(
        db,
        store,
        &ContentWriteInput {
            content_type: ContentType::Json,
            mime_type: Some("application/json".to_string()),
            text_content: Some(
                json!({
                    "workflow_def_id": input.workflow_def_id,
                    "conversation_id": input.conversation_id,
                    "trigger_message_version_id": input.trigger_message_version_id,
                    "responder_participant_id": input.responder_participant_id,
                    "isolated_conversation_title": input.isolated_conversation_title,
                    "config_json": input.config_json,
                })
                .to_string(),
            ),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: None,
            config_json: json!({ "kind": "workflow_request_snapshot" }),
        },
    )
    .await
}

async fn resolve_root_workspace_conversation(
    db: &SqlitePool,
    workflow: &WorkflowDefRow,
    input: &RunWorkflowInput,
) -> Result<Option<String>> {
    if let Some(title) = input.isolated_conversation_title.as_deref() {
        let conversation = conversations::create_conversation(
            db,
            &CreateConversationInput {
                title: title.to_string(),
                description: Some(format!("Workflow workspace for {}", workflow.name)),
                conversation_mode: "workflow".to_string(),
                archived: false,
                pinned: false,
                participants: Vec::new(),
                config_json: json!({ "created_by": "workflow_run" }),
            },
        )
        .await?;
        return Ok(Some(conversation.summary.id));
    }

    Ok(input.conversation_id.clone())
}

fn select_entry_node<'a>(graph: &'a WorkflowGraph) -> Option<&'a WorkflowDefNodeRow> {
    let incoming = graph
        .incoming_by_to
        .values()
        .flatten()
        .filter(|edge| edge.enabled)
        .map(|edge| edge.to_node_id.as_str())
        .collect::<HashSet<_>>();

    graph
        .nodes()
        .filter(|node| !incoming.contains(node.id.as_str()))
        .min_by(|lhs, rhs| lhs.node_key.cmp(&rhs.node_key))
        .or_else(|| {
            graph
                .nodes()
                .min_by(|lhs, rhs| lhs.node_key.cmp(&rhs.node_key))
        })
}

async fn execute_node(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &RunWorkflowInput,
    workflow: &WorkflowDefRow,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    match node.node_type.as_str() {
        "input" => execute_input_node(db, store, input, run, execution).await,
        "agent" => execute_agent_node(db, store, providers, input, run, execution, node).await,
        "tool" => execute_tool_node(db, store, run, execution, node).await,
        "plugin" => execute_plugin_node(db, store, run, execution, node).await,
        "rag" => execute_rag_node(db, store, run, execution, node).await,
        "mcp" => execute_mcp_node(db, store, run, execution, node).await,
        "router" => execute_passthrough_node(db, store, execution, node).await,
        "merge" => execute_merge_node(db, store, run, execution).await,
        "loop" => execute_passthrough_node(db, store, execution, node).await,
        "writeback" => execute_writeback_node(db, store, input, run, execution, node).await,
        "output" => execute_output_node(db, store, input, run, execution, node).await,
        "subflow" => {
            execute_subflow_node(db, store, providers, input, workflow, execution, node).await
        }
        other => Err(AppError::Validation(format!(
            "workflow node type '{other}' is not executable in v1"
        ))),
    }
}

async fn execute_input_node(
    db: &SqlitePool,
    store: &ContentStore,
    input: &RunWorkflowInput,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
) -> Result<ExecutionOutcome> {
    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;

    let material =
        if let Some(trigger_message_version_id) = run.trigger_message_version_id.as_deref() {
            let view =
                messages::get_message_version_view(db, store, trigger_message_version_id).await?;
            execution_config["output_message_version_id"] = Value::String(view.version_id.clone());
            execution_config["source_message_node_id"] = Value::String(view.node_id.clone());
            execution_config["source_kind"] = Value::String("message_version".to_string());
            ExecutionInputMaterial {
                content: view.primary_content,
                source_kind: "message_version".to_string(),
                source_message_node_id: Some(view.node_id),
                source_message_version_id: Some(view.version_id),
                source_tool_invocation_id: None,
                source_rag_ref_id: None,
                source_mcp_event_id: None,
                source_plugin_id: None,
            }
        } else if let Some(content_id) = input
            .config_json
            .get("subflow_input_content_id")
            .and_then(Value::as_str)
        {
            ExecutionInputMaterial {
                content: content::get_content(db, store, content_id, true).await?,
                source_kind: "workflow_input".to_string(),
                source_message_node_id: None,
                source_message_version_id: None,
                source_tool_invocation_id: None,
                source_rag_ref_id: None,
                source_mcp_event_id: None,
                source_plugin_id: None,
            }
        } else {
            let content_id = execution
                .input_snapshot_content_id
                .as_deref()
                .or(run.request_snapshot_content_id.as_deref())
                .ok_or_else(|| {
                    AppError::Validation(
                    "input node requires request_snapshot_content_id or input_snapshot_content_id"
                        .to_string(),
                )
                })?;
            ExecutionInputMaterial {
                content: content::get_content(db, store, content_id, true).await?,
                source_kind: "workflow_input".to_string(),
                source_message_node_id: None,
                source_message_version_id: None,
                source_tool_invocation_id: None,
                source_rag_ref_id: None,
                source_mcp_event_id: None,
                source_plugin_id: None,
            }
        };

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(material.content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn execute_passthrough_node(
    db: &SqlitePool,
    store: &ContentStore,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_execution_inputs(db, store, execution).await?;
    let first = inputs.first().cloned().ok_or_else(|| {
        AppError::Validation(format!(
            "{} node requires at least one execution input",
            node.node_type
        ))
    })?;
    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    if let Some(message_version_id) = &first.source_message_version_id {
        execution_config["output_message_version_id"] = Value::String(message_version_id.clone());
    }
    if let Some(message_node_id) = &first.source_message_node_id {
        execution_config["source_message_node_id"] = Value::String(message_node_id.clone());
    }
    if let Some(tool_invocation_id) = &first.source_tool_invocation_id {
        execution_config["source_tool_invocation_id"] = Value::String(tool_invocation_id.clone());
    }
    if let Some(rag_ref_id) = &first.source_rag_ref_id {
        execution_config["source_rag_ref_id"] = Value::String(rag_ref_id.clone());
    }
    if let Some(mcp_event_id) = &first.source_mcp_event_id {
        execution_config["source_mcp_event_id"] = Value::String(mcp_event_id.clone());
    }
    if let Some(plugin_id) = &first.source_plugin_id {
        execution_config["source_plugin_id"] = Value::String(plugin_id.clone());
    }
    execution_config["source_kind"] = Value::String(first.source_kind.clone());

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(first.content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn execute_merge_node(
    db: &SqlitePool,
    store: &ContentStore,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_execution_inputs(db, store, execution).await?;
    if inputs.is_empty() {
        return Err(AppError::Validation(
            "merge node requires at least one completed upstream execution".to_string(),
        ));
    }

    let merged_text = inputs
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            format!(
                "[Input {}]\n{}",
                idx + 1,
                extract_textish_content(&item.content)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let merged_content = content::create_content(
        db,
        store,
        &ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: Some(merged_text),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: None,
            config_json: json!({
                "kind": "workflow_merge_output",
                "workflow_run_id": run.id,
                "execution_id": execution.id,
            }),
        },
    )
    .await?;

    let execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?
    .as_object()
    .cloned()
    .map(Value::Object)
    .unwrap_or_else(|| json!({}));
    let mut execution_config = execution_config;
    execution_config["source_kind"] = Value::String("merge_output".to_string());
    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(merged_content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn execute_agent_node(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &RunWorkflowInput,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_generation_inputs(db, store, execution).await?;
    if inputs.is_empty() {
        return Err(AppError::Validation(
            "agent node requires at least one execution input".to_string(),
        ));
    }

    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    let target_conversation_id =
        resolve_node_workspace_conversation(db, input, run, node, &mut execution_config)
            .await?
            .ok_or_else(|| {
                AppError::Validation("agent node requires a conversation workspace".to_string())
            })?;
    let node_config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;

    let participant = ensure_conversation_participant(
        db,
        &target_conversation_id,
        node.agent_id.as_deref(),
        input.responder_participant_id.as_deref(),
        Some("Workflow Agent"),
        "agent",
    )
    .await?;

    let trigger_message_version_id =
        if input.conversation_id.as_deref() == Some(target_conversation_id.as_str()) {
            input.trigger_message_version_id.clone()
        } else {
            None
        };

    let request_parameters_json = node_config.get("request_parameters_json").cloned();
    let mut built_context = context_builder::build_generation_context(
        db,
        store,
        &BuildGenerationContextInput {
            conversation_id: target_conversation_id.clone(),
            responder_participant_id: participant.id.clone(),
            trigger_message_version_id,
            override_api_channel_id: node.api_channel_id.clone(),
            override_api_channel_model_id: node.api_channel_model_id.clone(),
            request_parameters_json,
        },
    )
    .await?;
    append_workflow_inputs_to_context(db, store, &mut built_context, &inputs, run, execution)
        .await?;
    let create_hidden_message = !matches!(node.message_write_mode.as_str(), "append_visible")
        && !matches!(
            node.visible_output_mode.as_str(),
            "always" | "visible" | "intermediate"
        );

    let generated = generation::generate_reply_from_context(
        db,
        store,
        providers,
        &built_context,
        "workflow_agent",
        create_hidden_message,
        json!({
            "created_by": "workflow_agent",
            "workflow_run_id": run.id,
            "workflow_run_node_execution_id": execution.id,
            "workflow_def_node_id": node.id,
        }),
    )
    .await?;

    execution_config["workspace_conversation_id"] = Value::String(target_conversation_id);
    execution_config["output_message_version_id"] = Value::String(generated.version_id.clone());
    execution_config["source_message_node_id"] = Value::String(generated.node_id.clone());
    execution_config["source_kind"] = Value::String("message_version".to_string());

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(generated.primary_content.clone()),
        generation_run_id: generated.generation_run_id.clone(),
        execution_config,
        finalization: None,
    })
}

async fn execute_tool_node(
    db: &SqlitePool,
    store: &ContentStore,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_execution_inputs(db, store, execution).await?;
    let node_config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
    let request_input = build_request_content_from_inputs("tool", &inputs, &node_config)?;
    let started = tool_invocations::start_tool_invocation(
        db,
        store,
        &StartToolInvocationInput {
            generation_run_id: None,
            workflow_run_node_execution_id: Some(execution.id.clone()),
            message_version_id: inputs
                .first()
                .and_then(|item| item.source_message_version_id.clone()),
            tool_kind: node_config
                .get("tool_kind")
                .and_then(Value::as_str)
                .unwrap_or(if node.plugin_id.is_some() {
                    "plugin"
                } else {
                    "workflow"
                })
                .to_string(),
            tool_name: node_config
                .get("tool_name")
                .and_then(Value::as_str)
                .unwrap_or(&node.node_key)
                .to_string(),
            plugin_id: node.plugin_id.clone(),
            request_content: Some(request_input),
            config_json: json!({
                "workflow_run_id": run.id,
                "workflow_def_node_id": node.id,
            }),
        },
    )
    .await?;
    let response_input = build_node_output_content_input(
        "tool",
        node_config.get("response"),
        inputs.first().map(|item| &item.content),
    )?;
    let finished = tool_invocations::finish_tool_invocation(
        db,
        store,
        &started.id,
        &FinishToolInvocationInput {
            status: "succeeded".to_string(),
            response_content: Some(response_input),
            config_json: json!({
                "workflow_run_id": run.id,
                "workflow_def_node_id": node.id,
            }),
        },
    )
    .await?;
    let output_content = finished
        .response_content
        .clone()
        .or(finished.request_content.clone())
        .ok_or_else(|| {
            AppError::Validation(
                "tool node completed without request or response content".to_string(),
            )
        })?;

    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    execution_config["source_kind"] = Value::String("tool_invocation".to_string());
    execution_config["source_tool_invocation_id"] = Value::String(finished.id.clone());
    if let Some(plugin_id) = finished.plugin_id.clone() {
        execution_config["source_plugin_id"] = Value::String(plugin_id);
    }

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(output_content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn execute_plugin_node(
    db: &SqlitePool,
    store: &ContentStore,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let plugin_id = node
        .plugin_id
        .as_deref()
        .ok_or_else(|| AppError::Validation("plugin node requires plugin_id".to_string()))?;
    let plugin = plugins::get_plugin(db, plugin_id).await?;
    let inputs = load_execution_inputs(db, store, execution).await?;
    let node_config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
    let output_input = build_node_output_content_input(
        "plugin",
        node_config.get("response"),
        inputs.first().map(|item| &item.content),
    )?;
    let output_content = content::create_content(
        db,
        store,
        &ContentWriteInput {
            content_type: output_input.content_type,
            mime_type: output_input.mime_type,
            text_content: output_input.text_content,
            source_file_path: output_input.source_file_path,
            primary_storage_uri: output_input.primary_storage_uri,
            size_bytes_hint: output_input.size_bytes_hint,
            preview_text: output_input.preview_text,
            config_json: json!({
                "kind": "plugin_node_output",
                "plugin_id": plugin.id,
                "plugin_key": plugin.plugin_key,
                "workflow_run_id": run.id,
                "workflow_def_node_id": node.id,
            }),
        },
    )
    .await?;

    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    execution_config["source_plugin_id"] = Value::String(plugin.id);
    execution_config["source_kind"] = Value::String("plugin_content".to_string());

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(output_content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn execute_rag_node(
    db: &SqlitePool,
    store: &ContentStore,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_execution_inputs(db, store, execution).await?;
    let node_config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
    let excerpt_input = build_node_output_content_input(
        "rag",
        node_config.get("excerpt"),
        inputs.first().map(|item| &item.content),
    )?;
    let rag_ref = rag::record_rag_ref(
        db,
        store,
        &CreateRagRefInput {
            generation_run_id: None,
            workflow_run_node_execution_id: Some(execution.id.clone()),
            source_uri: node_config
                .get("source_uri")
                .and_then(Value::as_str)
                .map(str::to_string),
            document_title: node_config
                .get("document_title")
                .and_then(Value::as_str)
                .map(str::to_string),
            chunk_key: node_config
                .get("chunk_key")
                .and_then(Value::as_str)
                .map(str::to_string),
            score: node_config
                .get("score")
                .and_then(Value::as_f64)
                .map(|value| value as f32),
            excerpt_content: Some(excerpt_input),
            included_in_request: node_config
                .get("included_in_request")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            config_json: json!({
                "workflow_run_id": run.id,
                "workflow_def_node_id": node.id,
            }),
        },
    )
    .await?;
    let output_content = rag_ref.excerpt_content.clone().ok_or_else(|| {
        AppError::Validation("rag node did not produce excerpt content".to_string())
    })?;

    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    execution_config["source_kind"] = Value::String("rag_ref".to_string());
    execution_config["source_rag_ref_id"] = Value::String(rag_ref.id.clone());

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(output_content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn execute_mcp_node(
    db: &SqlitePool,
    store: &ContentStore,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_execution_inputs(db, store, execution).await?;
    let node_config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
    let payload_input = build_node_output_content_input(
        "mcp",
        node_config.get("payload"),
        inputs.first().map(|item| &item.content),
    )?;
    let event = mcp::record_mcp_event(
        db,
        store,
        &CreateMcpEventInput {
            generation_run_id: None,
            workflow_run_node_execution_id: Some(execution.id.clone()),
            server_name: node_config
                .get("server_name")
                .and_then(Value::as_str)
                .unwrap_or("workflow")
                .to_string(),
            event_kind: node_config
                .get("event_kind")
                .and_then(Value::as_str)
                .unwrap_or("invoke")
                .to_string(),
            method_name: node_config
                .get("method_name")
                .and_then(Value::as_str)
                .map(str::to_string),
            payload_content: Some(payload_input),
            status: node_config
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("succeeded")
                .to_string(),
            config_json: json!({
                "workflow_run_id": run.id,
                "workflow_def_node_id": node.id,
            }),
        },
    )
    .await?;
    let output_content = event.payload_content.clone().ok_or_else(|| {
        AppError::Validation("mcp node did not produce payload content".to_string())
    })?;

    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    execution_config["source_kind"] = Value::String("mcp_event".to_string());
    execution_config["source_mcp_event_id"] = Value::String(event.id.clone());

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(output_content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn append_workflow_inputs_to_context(
    db: &SqlitePool,
    store: &ContentStore,
    built_context: &mut BuiltGenerationContext,
    inputs: &[ExecutionInputMaterial],
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
) -> Result<()> {
    let mut known_message_versions = built_context
        .items
        .iter()
        .filter_map(|item| item.source_message_version_id.clone())
        .collect::<HashSet<_>>();
    let mut sequence_no = built_context
        .items
        .iter()
        .map(|item| item.sequence_no)
        .max()
        .unwrap_or(-1)
        + 1;

    for material in inputs {
        if let Some(message_version_id) = material.source_message_version_id.as_deref() {
            if known_message_versions.contains(message_version_id) {
                continue;
            }
        }

        let send_role = match material.source_message_version_id.as_deref() {
            Some(message_version_id) => {
                messages::get_message_version_view(db, store, message_version_id)
                    .await?
                    .role
            }
            None => MessageRole::System,
        };
        if let Some(message_version_id) = material.source_message_version_id.as_deref() {
            known_message_versions.insert(message_version_id.to_string());
        }

        built_context.items.push(GenerationContextItem {
            sequence_no,
            send_role,
            rendered_content: material.content.clone(),
            source_kind: material.source_kind.clone(),
            source_message_node_id: material.source_message_node_id.clone(),
            source_message_version_id: material.source_message_version_id.clone(),
            source_summary_version_id: None,
            source_preset_entry_id: None,
            source_lorebook_entry_id: None,
            source_user_profile_id: None,
            source_agent_id: None,
            source_agent_greeting_id: None,
            source_tool_invocation_id: material.source_tool_invocation_id.clone(),
            source_rag_ref_id: material.source_rag_ref_id.clone(),
            source_mcp_event_id: material.source_mcp_event_id.clone(),
            source_plugin_id: material.source_plugin_id.clone(),
            included_in_request: true,
            config_json: json!({
                "workflow_run_id": run.id,
                "workflow_run_node_execution_id": execution.id,
                "source_kind": material.source_kind,
            }),
        });
        sequence_no += 1;
    }

    Ok(())
}

async fn execute_writeback_node(
    db: &SqlitePool,
    store: &ContentStore,
    input: &RunWorkflowInput,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_execution_inputs(db, store, execution).await?;
    let first = inputs.first().cloned().ok_or_else(|| {
        AppError::Validation("writeback node requires at least one execution input".to_string())
    })?;
    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    let node_config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
    let target_conversation_id =
        resolve_node_workspace_conversation(db, input, run, node, &mut execution_config).await?;

    if matches!(
        node.message_write_mode.as_str(),
        "append_visible" | "append_hidden"
    ) {
        let conversation_id = target_conversation_id.clone().ok_or_else(|| {
            AppError::Validation("message writeback requires a conversation workspace".to_string())
        })?;
        let message_role = parse_optional_message_role(node_config.get("message_role"))?
            .unwrap_or(MessageRole::Assistant);
        let participant = ensure_conversation_participant(
            db,
            &conversation_id,
            node.agent_id.as_deref(),
            input.responder_participant_id.as_deref(),
            Some("Workflow"),
            participant_type_for_role(message_role),
        )
        .await?;
        let created = create_message_from_content(
            db,
            store,
            &conversation_id,
            &participant.id,
            message_role,
            stored_content_to_write_input(&first.content),
            ContextPolicy::Full,
            if node.message_write_mode == "append_hidden" {
                ViewerPolicy::Hidden
            } else {
                ViewerPolicy::Full
            },
            json!({
                "created_by": "workflow_writeback",
                "workflow_run_id": run.id,
                "workflow_run_node_execution_id": execution.id,
            }),
        )
        .await?;
        execution_config["output_message_version_id"] = Value::String(created.version_id.clone());
        execution_config["source_message_node_id"] = Value::String(created.node_id.clone());
        execution_config["source_kind"] = Value::String("message_version".to_string());

        let _ = workflow_repo::create_workflow_run_write(
            db,
            &workflow_repo::CreateWorkflowRunWriteRecord {
                workflow_run_id: &run.id,
                workflow_run_node_execution_id: Some(&execution.id),
                write_kind: if node.message_write_mode == "append_hidden" {
                    "message_hidden"
                } else {
                    "message_visible"
                },
                apply_mode: &node.message_write_mode,
                content_id: &created.primary_content.content_id,
                target_conversation_id: Some(&conversation_id),
                target_message_node_id: Some(&created.node_id),
                target_summary_group_id: None,
                target_lorebook_entry_id: None,
                target_preset_entry_id: None,
                target_agent_id: node.agent_id.as_deref(),
                target_user_profile_id: None,
                target_plugin_id: node.plugin_id.as_deref(),
                target_slot: Some("message"),
                visible_to_user: node.message_write_mode == "append_visible",
                config_json: &json!({}).to_string(),
            },
        )
        .await?;
    } else if let Some(message_version_id) = &first.source_message_version_id {
        execution_config["output_message_version_id"] = Value::String(message_version_id.clone());
        if let Some(message_node_id) = &first.source_message_node_id {
            execution_config["source_message_node_id"] = Value::String(message_node_id.clone());
        }
        execution_config["source_kind"] = Value::String(first.source_kind.clone());
    }

    if node.summary_write_mode != "none" {
        let conversation_id = target_conversation_id.clone().ok_or_else(|| {
            AppError::Validation("summary writeback requires a conversation workspace".to_string())
        })?;
        let summary_group = resolve_or_create_summary_group(
            db,
            &conversation_id,
            node_config
                .get("target_summary_group_id")
                .and_then(Value::as_str),
        )
        .await?;
        let next_version_index = summary_repo::list_summary_versions(db, &summary_group.id)
            .await?
            .into_iter()
            .map(|item| item.version_index)
            .max()
            .unwrap_or(0)
            + 1;

        let mut tx = db.begin().await?;
        let version = summary_repo::create_summary_version(
            &mut tx,
            &summary_repo::CreateSummaryVersionRecord {
                summary_group_id: &summary_group.id,
                version_index: next_version_index,
                is_active: false,
                content_id: &first.content.content_id,
                generator_type: "workflow",
                generator_preset_id: None,
                workflow_run_id: Some(&run.id),
                generation_run_id: None,
                config_json: &json!({
                    "workflow_run_node_execution_id": execution.id,
                })
                .to_string(),
                created_at: time::now_ms(),
            },
        )
        .await?;
        if matches!(
            node.summary_write_mode.as_str(),
            "replace_active" | "create_and_activate"
        ) {
            summary_repo::set_active_summary_version(&mut tx, &summary_group.id, &version.id)
                .await?;
        }
        let sources = build_summary_sources_for_inputs(&inputs);
        summary_repo::replace_summary_sources(&mut tx, &summary_group.id, &version.id, &sources)
            .await?;
        tx.commit().await?;

        let _ = workflow_repo::create_workflow_run_write(
            db,
            &workflow_repo::CreateWorkflowRunWriteRecord {
                workflow_run_id: &run.id,
                workflow_run_node_execution_id: Some(&execution.id),
                write_kind: "summary_version",
                apply_mode: &node.summary_write_mode,
                content_id: &first.content.content_id,
                target_conversation_id: Some(&conversation_id),
                target_message_node_id: None,
                target_summary_group_id: Some(&summary_group.id),
                target_lorebook_entry_id: None,
                target_preset_entry_id: None,
                target_agent_id: None,
                target_user_profile_id: None,
                target_plugin_id: None,
                target_slot: Some("summary"),
                visible_to_user: false,
                config_json: &json!({ "summary_version_id": version.id }).to_string(),
            },
        )
        .await?;
    }

    if let Some(variable_config) = node_config.get("variable") {
        let write = write_variable_value(
            db,
            store,
            run,
            node,
            target_conversation_id.as_deref(),
            variable_config,
            &first.content,
        )
        .await?;

        let _ = workflow_repo::create_workflow_run_write(
            db,
            &workflow_repo::CreateWorkflowRunWriteRecord {
                workflow_run_id: &run.id,
                workflow_run_node_execution_id: Some(&execution.id),
                write_kind: "variable_value",
                apply_mode: "upsert",
                content_id: &first.content.content_id,
                target_conversation_id: target_conversation_id.as_deref(),
                target_message_node_id: None,
                target_summary_group_id: None,
                target_lorebook_entry_id: None,
                target_preset_entry_id: None,
                target_agent_id: node.agent_id.as_deref(),
                target_user_profile_id: None,
                target_plugin_id: node.plugin_id.as_deref(),
                target_slot: Some(&write.var_key),
                visible_to_user: false,
                config_json: &json!({
                    "variable_def_id": write.variable_def_id,
                    "scope_type": write.scope_type,
                    "scope_id": write.scope_id,
                })
                .to_string(),
            },
        )
        .await?;
    }

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(first.content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn execute_output_node(
    db: &SqlitePool,
    store: &ContentStore,
    input: &RunWorkflowInput,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_execution_inputs(db, store, execution).await?;
    let first = inputs.first().cloned().ok_or_else(|| {
        AppError::Validation("output node requires at least one execution input".to_string())
    })?;
    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    let target_conversation_id =
        resolve_node_workspace_conversation(db, input, run, node, &mut execution_config).await?;

    let mut result_message_version_id = first.source_message_version_id.clone();
    let mut result_content_id = Some(first.content.content_id.clone());

    if let Some(source_version_id) = first.source_message_version_id.as_deref() {
        let view = messages::get_message_version_view(db, store, source_version_id).await?;
        if should_clone_output_message(target_conversation_id.as_deref(), &view) {
            if let Some(conversation_id) = target_conversation_id.as_deref() {
                let participant = ensure_conversation_participant(
                    db,
                    conversation_id,
                    node.agent_id.as_deref(),
                    input.responder_participant_id.as_deref(),
                    Some("Workflow"),
                    participant_type_for_role(view.role),
                )
                .await?;
                let cloned = clone_message_view_to_conversation(
                    db,
                    store,
                    &view,
                    conversation_id,
                    &participant.id,
                    ViewerPolicy::Full,
                )
                .await?;
                execution_config["output_message_version_id"] =
                    Value::String(cloned.version_id.clone());
                let _ = workflow_repo::create_workflow_run_write(
                    db,
                    &workflow_repo::CreateWorkflowRunWriteRecord {
                        workflow_run_id: &run.id,
                        workflow_run_node_execution_id: Some(&execution.id),
                        write_kind: "message_visible",
                        apply_mode: "append_visible",
                        content_id: &cloned.primary_content.content_id,
                        target_conversation_id: Some(conversation_id),
                        target_message_node_id: Some(&cloned.node_id),
                        target_summary_group_id: None,
                        target_lorebook_entry_id: None,
                        target_preset_entry_id: None,
                        target_agent_id: node.agent_id.as_deref(),
                        target_user_profile_id: None,
                        target_plugin_id: node.plugin_id.as_deref(),
                        target_slot: Some("result_message"),
                        visible_to_user: true,
                        config_json: &json!({}).to_string(),
                    },
                )
                .await?;
                result_message_version_id = Some(cloned.version_id.clone());
                result_content_id = Some(cloned.primary_content.content_id.clone());
            } else {
                result_message_version_id = None;
                result_content_id = Some(view.primary_content.content_id.clone());
            }
        } else {
            execution_config["output_message_version_id"] = Value::String(view.version_id.clone());
            result_message_version_id = Some(view.version_id.clone());
            result_content_id = Some(view.primary_content.content_id.clone());
        }
    } else if let Some(conversation_id) = target_conversation_id.as_deref() {
        let role = parse_optional_message_role(
            parse_json(&node.config_json, "workflow_def_nodes.config_json")?.get("message_role"),
        )?
        .unwrap_or(MessageRole::Assistant);
        let participant = ensure_conversation_participant(
            db,
            conversation_id,
            node.agent_id.as_deref(),
            input.responder_participant_id.as_deref(),
            Some("Workflow"),
            participant_type_for_role(role),
        )
        .await?;
        let created = create_message_from_content(
            db,
            store,
            conversation_id,
            &participant.id,
            role,
            stored_content_to_write_input(&first.content),
            ContextPolicy::Full,
            ViewerPolicy::Full,
            json!({
                "created_by": "workflow_output",
                "workflow_run_id": run.id,
                "workflow_run_node_execution_id": execution.id,
            }),
        )
        .await?;
        execution_config["output_message_version_id"] = Value::String(created.version_id.clone());
        let _ = workflow_repo::create_workflow_run_write(
            db,
            &workflow_repo::CreateWorkflowRunWriteRecord {
                workflow_run_id: &run.id,
                workflow_run_node_execution_id: Some(&execution.id),
                write_kind: "message_visible",
                apply_mode: "append_visible",
                content_id: &created.primary_content.content_id,
                target_conversation_id: Some(conversation_id),
                target_message_node_id: Some(&created.node_id),
                target_summary_group_id: None,
                target_lorebook_entry_id: None,
                target_preset_entry_id: None,
                target_agent_id: node.agent_id.as_deref(),
                target_user_profile_id: None,
                target_plugin_id: node.plugin_id.as_deref(),
                target_slot: Some("result_message"),
                visible_to_user: true,
                config_json: &json!({}).to_string(),
            },
        )
        .await?;
        result_message_version_id = Some(created.version_id.clone());
        result_content_id = Some(created.primary_content.content_id.clone());
    }

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(first.content),
        generation_run_id: None,
        execution_config,
        finalization: Some(WorkflowFinalization {
            status: "succeeded".to_string(),
            result_message_version_id,
            result_content_id,
        }),
    })
}

async fn execute_subflow_node(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &RunWorkflowInput,
    workflow: &WorkflowDefRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
) -> Result<ExecutionOutcome> {
    let inputs = load_execution_inputs(db, store, execution).await?;
    let first = inputs.first().cloned().ok_or_else(|| {
        AppError::Validation("subflow node requires at least one execution input".to_string())
    })?;
    let mut node_config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
    let subflow_workflow_def_id = node_config
        .get("subflow_workflow_def_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AppError::Validation(
                "subflow node requires config_json.subflow_workflow_def_id".to_string(),
            )
        })?
        .to_string();

    if subflow_workflow_def_id == workflow.id {
        return Err(AppError::Validation(
            "subflow node cannot recursively invoke the same workflow definition".to_string(),
        ));
    }

    node_config["subflow_input_content_id"] = Value::String(first.content.content_id.clone());
    node_config["parent_workflow_run_id"] = Value::String(execution.workflow_run_id.clone());
    node_config["parent_execution_id"] = Value::String(execution.id.clone());

    let child_input = RunWorkflowInput {
        workflow_def_id: subflow_workflow_def_id,
        conversation_id: if node.workspace_mode == "isolated_conversation" {
            None
        } else {
            input.conversation_id.clone()
        },
        trigger_message_version_id: None,
        responder_participant_id: input.responder_participant_id.clone(),
        isolated_conversation_title: if node.workspace_mode == "isolated_conversation" {
            Some(
                node_config
                    .get("isolated_conversation_title")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("{} / {}", workflow.name, node.node_key)),
            )
        } else {
            None
        },
        config_json: node_config,
    };

    let child_result = run_workflow_boxed(db, store, providers, &child_input).await?;
    let child_run = workflow_repo::get_workflow_run(db, &child_result.workflow_run_id).await?;

    let (output_content, output_message_version_id) = if let Some(message_version_id) =
        child_result.result_message_version_id.as_deref()
    {
        let view = messages::get_message_version_view(db, store, message_version_id).await?;
        (view.primary_content, Some(view.version_id))
    } else if let Some(content_id) = child_run.result_content_id.as_deref() {
        (
            content::get_content(db, store, content_id, true).await?,
            None,
        )
    } else {
        return Err(AppError::Validation(
            "subflow completed without result_message_version_id or result_content_id".to_string(),
        ));
    };

    let mut execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    execution_config["child_workflow_run_id"] = Value::String(child_run.id);
    if let Some(message_version_id) = &output_message_version_id {
        execution_config["output_message_version_id"] = Value::String(message_version_id.clone());
    }

    Ok(ExecutionOutcome {
        status: "succeeded".to_string(),
        output_content: Some(output_content),
        generation_run_id: None,
        execution_config,
        finalization: None,
    })
}

async fn schedule_successors(
    db: &SqlitePool,
    store: &ContentStore,
    graph: &WorkflowGraph,
    run: &WorkflowRunRow,
    execution: &WorkflowRunNodeExecutionRow,
    node: &WorkflowDefNodeRow,
    outcome: &ExecutionOutcome,
) -> Result<Vec<String>> {
    let outgoing = graph.outgoing(&node.id);
    if outgoing.is_empty() {
        return Ok(Vec::new());
    }

    let eligible = match node.node_type.as_str() {
        "router" => {
            let selected = outgoing
                .iter()
                .find(|edge| evaluate_edge_condition(edge, outcome))
                .cloned()
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "router node '{}' did not match any outgoing edge",
                        node.node_key
                    ))
                })?;
            vec![selected]
        }
        _ => outgoing
            .iter()
            .filter(|edge| evaluate_edge_condition(edge, outcome))
            .cloned()
            .collect::<Vec<_>>(),
    };

    let mut scheduled = Vec::new();
    for edge in eligible {
        let target = graph.node(&edge.to_node_id)?;
        let next_loop_iteration = next_loop_iteration(node, execution, &edge)?;
        if target.node_type == "merge" {
            if let Some(execution_id) =
                maybe_schedule_merge_execution(db, store, graph, run, &edge, next_loop_iteration)
                    .await?
            {
                scheduled.push(execution_id);
            }
            continue;
        }

        let execution_config = json!({
            "input_execution_ids": [execution.id],
        });
        let child = workflow_repo::create_workflow_run_node_execution(
            db,
            &workflow_repo::CreateWorkflowRunNodeExecutionRecord {
                workflow_run_id: &run.id,
                workflow_def_node_id: &target.id,
                parent_execution_id: Some(&execution.id),
                incoming_edge_id: Some(&edge.id),
                branch_key: edge.label.as_deref().or(Some(edge.edge_type.as_str())),
                loop_iteration: next_loop_iteration,
                retry_index: 0,
                status: "pending",
                generation_run_id: None,
                input_snapshot_content_id: outcome
                    .output_content
                    .as_ref()
                    .map(|item| item.content_id.as_str()),
                output_content_id: None,
                error_content_id: None,
                started_at: None,
                finished_at: None,
                config_json: &execution_config.to_string(),
            },
        )
        .await?;
        scheduled.push(child.id);
    }

    Ok(scheduled)
}

async fn maybe_schedule_merge_execution(
    db: &SqlitePool,
    store: &ContentStore,
    graph: &WorkflowGraph,
    run: &WorkflowRunRow,
    incoming_edge: &WorkflowDefEdgeRow,
    loop_iteration: i64,
) -> Result<Option<String>> {
    let target_node = graph.node(&incoming_edge.to_node_id)?;
    let executions = workflow_repo::list_workflow_run_node_executions(db, &run.id).await?;
    if executions.iter().any(|item| {
        item.workflow_def_node_id == target_node.id
            && item.loop_iteration == loop_iteration
            && matches!(item.status.as_str(), "pending" | "running" | "succeeded")
    }) {
        return Ok(None);
    }

    let mut input_execution_ids = Vec::new();
    for required_edge in graph
        .incoming(&target_node.id)
        .iter()
        .filter(|edge| edge.enabled)
    {
        let Some(candidate) = find_matching_execution_for_merge(
            db,
            store,
            &executions,
            required_edge,
            loop_iteration,
        )
        .await?
        else {
            return Ok(None);
        };
        input_execution_ids.push(candidate.id);
    }

    input_execution_ids.sort();
    input_execution_ids.dedup();
    let execution = workflow_repo::create_workflow_run_node_execution(
        db,
        &workflow_repo::CreateWorkflowRunNodeExecutionRecord {
            workflow_run_id: &run.id,
            workflow_def_node_id: &target_node.id,
            parent_execution_id: None,
            incoming_edge_id: Some(&incoming_edge.id),
            branch_key: Some("merge"),
            loop_iteration,
            retry_index: 0,
            status: "pending",
            generation_run_id: None,
            input_snapshot_content_id: None,
            output_content_id: None,
            error_content_id: None,
            started_at: None,
            finished_at: None,
            config_json: &json!({ "input_execution_ids": input_execution_ids }).to_string(),
        },
    )
    .await?;

    Ok(Some(execution.id))
}

async fn find_matching_execution_for_merge(
    db: &SqlitePool,
    store: &ContentStore,
    executions: &[WorkflowRunNodeExecutionRow],
    edge: &WorkflowDefEdgeRow,
    loop_iteration: i64,
) -> Result<Option<WorkflowRunNodeExecutionRow>> {
    let mut candidates = executions
        .iter()
        .filter(|execution| {
            execution.workflow_def_node_id == edge.from_node_id
                && execution.loop_iteration == loop_iteration
                && execution.status == "succeeded"
        })
        .cloned()
        .collect::<Vec<_>>();
    candidates.sort_by(|lhs, rhs| lhs.created_at.cmp(&rhs.created_at));

    for execution in candidates.into_iter().rev() {
        let outcome = load_execution_outcome_for_condition(db, store, &execution).await?;
        if evaluate_edge_condition(edge, &outcome) {
            return Ok(Some(execution));
        }
    }

    Ok(None)
}

async fn load_execution_outcome_for_condition(
    db: &SqlitePool,
    store: &ContentStore,
    execution: &WorkflowRunNodeExecutionRow,
) -> Result<ExecutionOutcome> {
    let execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    let output_content = match execution.output_content_id.as_deref() {
        Some(content_id) => Some(content::get_content(db, store, content_id, false).await?),
        None => None,
    };
    Ok(ExecutionOutcome {
        status: execution.status.clone(),
        output_content,
        generation_run_id: execution.generation_run_id.clone(),
        execution_config,
        finalization: None,
    })
}

fn next_loop_iteration(
    node: &WorkflowDefNodeRow,
    execution: &WorkflowRunNodeExecutionRow,
    edge: &WorkflowDefEdgeRow,
) -> Result<i64> {
    if edge.edge_type != "loop_back" {
        return Ok(execution.loop_iteration);
    }

    let config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
    let max_iterations = config
        .get("max_iterations")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let next = execution.loop_iteration + 1;
    if next > max_iterations {
        return Err(AppError::Validation(format!(
            "loop node '{}' exceeded max_iterations ({max_iterations})",
            node.node_key
        )));
    }
    Ok(next)
}

fn evaluate_edge_condition(edge: &WorkflowDefEdgeRow, outcome: &ExecutionOutcome) -> bool {
    if !edge.enabled {
        return false;
    }

    let Some(expr) = edge.condition_expr.as_deref() else {
        return true;
    };
    let expr = expr.trim();
    if expr.is_empty() {
        return true;
    }
    if expr.eq_ignore_ascii_case("true") {
        return true;
    }
    if expr.eq_ignore_ascii_case("false") {
        return false;
    }

    let haystack = outcome
        .output_content
        .as_ref()
        .map(extract_textish_content)
        .unwrap_or_default();

    if let Some(needle) = expr.strip_prefix("text_contains:") {
        return haystack.contains(needle);
    }
    if let Some(expected) = expr.strip_prefix("text_equals:") {
        return haystack == expected;
    }

    haystack == expr
}

async fn load_execution_inputs(
    db: &SqlitePool,
    store: &ContentStore,
    execution: &WorkflowRunNodeExecutionRow,
) -> Result<Vec<ExecutionInputMaterial>> {
    let execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    let mut source_execution_ids = execution_config
        .get("input_execution_ids")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if source_execution_ids.is_empty() {
        if let Some(parent_execution_id) = execution.parent_execution_id.as_deref() {
            source_execution_ids.push(parent_execution_id.to_string());
        }
    }

    let mut materials = Vec::new();
    if !source_execution_ids.is_empty() {
        for source_execution_id in source_execution_ids {
            let source_execution =
                workflow_repo::get_workflow_run_node_execution(db, &source_execution_id).await?;
            let source_config = parse_json(
                &source_execution.config_json,
                "workflow_run_node_executions.config_json",
            )?;
            let content_id = source_execution
                .output_content_id
                .as_deref()
                .or(source_execution.input_snapshot_content_id.as_deref())
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "workflow execution '{}' has no output content",
                        source_execution.id
                    ))
                })?;
            materials.push(ExecutionInputMaterial {
                content: content::get_content(db, store, content_id, true).await?,
                source_kind: source_config
                    .get("source_kind")
                    .and_then(Value::as_str)
                    .unwrap_or("workflow_input")
                    .to_string(),
                source_message_node_id: source_config
                    .get("source_message_node_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                source_message_version_id: source_config
                    .get("output_message_version_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                source_tool_invocation_id: source_config
                    .get("source_tool_invocation_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                source_rag_ref_id: source_config
                    .get("source_rag_ref_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                source_mcp_event_id: source_config
                    .get("source_mcp_event_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                source_plugin_id: source_config
                    .get("source_plugin_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            });
        }
        return Ok(materials);
    }

    if let Some(content_id) = execution.input_snapshot_content_id.as_deref() {
        materials.push(ExecutionInputMaterial {
            content: content::get_content(db, store, content_id, true).await?,
            source_kind: "workflow_input".to_string(),
            source_message_node_id: None,
            source_message_version_id: None,
            source_tool_invocation_id: None,
            source_rag_ref_id: None,
            source_mcp_event_id: None,
            source_plugin_id: None,
        });
    }

    Ok(materials)
}

async fn load_generation_inputs(
    db: &SqlitePool,
    store: &ContentStore,
    execution: &WorkflowRunNodeExecutionRow,
) -> Result<Vec<ExecutionInputMaterial>> {
    let execution_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    let mut source_execution_ids = execution_config
        .get("input_execution_ids")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if source_execution_ids.is_empty() {
        if let Some(parent_execution_id) = execution.parent_execution_id.as_deref() {
            source_execution_ids.push(parent_execution_id.to_string());
        }
    }

    if source_execution_ids.is_empty() {
        return load_execution_inputs(db, store, execution).await;
    }

    let mut visited_execution_ids = HashSet::new();
    let mut materials = Vec::new();
    for source_execution_id in source_execution_ids {
        collect_generation_materials(
            db,
            store,
            &source_execution_id,
            &mut visited_execution_ids,
            &mut materials,
        )
        .await?;
    }

    if materials.is_empty() {
        return load_execution_inputs(db, store, execution).await;
    }

    Ok(materials)
}

fn collect_generation_materials<'a>(
    db: &'a SqlitePool,
    store: &'a ContentStore,
    execution_id: &'a str,
    visited_execution_ids: &'a mut HashSet<String>,
    materials: &'a mut Vec<ExecutionInputMaterial>,
) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        if !visited_execution_ids.insert(execution_id.to_string()) {
            return Ok(());
        }

        let execution = workflow_repo::get_workflow_run_node_execution(db, execution_id).await?;
        let execution_config = parse_json(
            &execution.config_json,
            "workflow_run_node_executions.config_json",
        )?;
        let mut parent_ids = execution_config
            .get("input_execution_ids")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if parent_ids.is_empty() {
            if let Some(parent_execution_id) = execution.parent_execution_id.as_deref() {
                parent_ids.push(parent_execution_id.to_string());
            }
        }

        for parent_execution_id in parent_ids {
            collect_generation_materials(
                db,
                store,
                &parent_execution_id,
                visited_execution_ids,
                materials,
            )
            .await?;
        }

        if let Some(material) = load_material_from_execution(db, store, &execution).await? {
            push_unique_material(materials, material);
        }

        Ok(())
    })
}

async fn load_material_from_execution(
    db: &SqlitePool,
    store: &ContentStore,
    execution: &WorkflowRunNodeExecutionRow,
) -> Result<Option<ExecutionInputMaterial>> {
    let source_config = parse_json(
        &execution.config_json,
        "workflow_run_node_executions.config_json",
    )?;
    let content_id = match execution
        .output_content_id
        .as_deref()
        .or(execution.input_snapshot_content_id.as_deref())
    {
        Some(content_id) => content_id,
        None => return Ok(None),
    };

    Ok(Some(ExecutionInputMaterial {
        content: content::get_content(db, store, content_id, true).await?,
        source_kind: source_config
            .get("source_kind")
            .and_then(Value::as_str)
            .unwrap_or("workflow_input")
            .to_string(),
        source_message_node_id: source_config
            .get("source_message_node_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        source_message_version_id: source_config
            .get("output_message_version_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        source_tool_invocation_id: source_config
            .get("source_tool_invocation_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        source_rag_ref_id: source_config
            .get("source_rag_ref_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        source_mcp_event_id: source_config
            .get("source_mcp_event_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        source_plugin_id: source_config
            .get("source_plugin_id")
            .and_then(Value::as_str)
            .map(str::to_string),
    }))
}

fn push_unique_material(
    materials: &mut Vec<ExecutionInputMaterial>,
    material: ExecutionInputMaterial,
) {
    let already_present = materials.iter().any(|existing| {
        existing.content.content_id == material.content.content_id
            && existing.source_kind == material.source_kind
            && existing.source_message_version_id == material.source_message_version_id
            && existing.source_tool_invocation_id == material.source_tool_invocation_id
            && existing.source_rag_ref_id == material.source_rag_ref_id
            && existing.source_mcp_event_id == material.source_mcp_event_id
            && existing.source_plugin_id == material.source_plugin_id
    });
    if !already_present {
        materials.push(material);
    }
}

async fn resolve_node_workspace_conversation(
    db: &SqlitePool,
    input: &RunWorkflowInput,
    run: &WorkflowRunRow,
    node: &WorkflowDefNodeRow,
    execution_config: &mut Value,
) -> Result<Option<String>> {
    if let Some(workspace_conversation_id) = execution_config
        .get("workspace_conversation_id")
        .and_then(Value::as_str)
    {
        return Ok(Some(workspace_conversation_id.to_string()));
    }

    let conversation_id = match node.workspace_mode.as_str() {
        "isolated_conversation" => {
            let config = parse_json(&node.config_json, "workflow_def_nodes.config_json")?;
            let title = config
                .get("isolated_conversation_title")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("Workflow {} / {}", run.id, node.node_key));
            let conversation = conversations::create_conversation(
                db,
                &CreateConversationInput {
                    title,
                    description: Some(format!("Workflow node workspace for {}", node.node_key)),
                    conversation_mode: "workflow".to_string(),
                    archived: false,
                    pinned: false,
                    participants: Vec::new(),
                    config_json: json!({
                        "created_by": "workflow_node",
                        "workflow_run_id": run.id,
                        "workflow_def_node_id": node.id,
                    }),
                },
            )
            .await?;
            conversation.summary.id
        }
        _ => run
            .workspace_conversation_id
            .clone()
            .or_else(|| input.conversation_id.clone())
            .ok_or_else(|| {
                AppError::Validation(format!(
                    "workflow node '{}' requires a conversation context",
                    node.node_key
                ))
            })?,
    };

    execution_config["workspace_conversation_id"] = Value::String(conversation_id.clone());
    Ok(Some(conversation_id))
}

async fn ensure_conversation_participant(
    db: &SqlitePool,
    conversation_id: &str,
    preferred_agent_id: Option<&str>,
    preferred_participant_id: Option<&str>,
    default_display_name: Option<&str>,
    default_participant_type: &str,
) -> Result<ConversationParticipantRow> {
    let participants =
        conversation_repo::list_conversation_participants(db, conversation_id).await?;

    if let Some(agent_id) = preferred_agent_id {
        if let Some(existing) = participants
            .iter()
            .find(|item| item.enabled && item.agent_id.as_deref() == Some(agent_id))
            .cloned()
        {
            return Ok(existing);
        }

        let agent = agent_repo::get_agent(db, agent_id).await?;
        return conversation_repo::create_conversation_participant(
            db,
            conversation_id,
            &conversation_repo::ConversationParticipantRecord {
                agent_id: Some(agent_id),
                display_name: Some(agent.name.as_str()),
                participant_type: "agent",
                enabled: true,
                sort_order: participants.len() as i64,
                config_json: &json!({ "created_by": "workflow" }).to_string(),
            },
        )
        .await;
    }

    if let Some(participant_id) = preferred_participant_id {
        let participant =
            conversation_repo::get_conversation_participant(db, participant_id).await?;
        if participant.conversation_id == conversation_id {
            return Ok(participant);
        }

        return conversation_repo::create_conversation_participant(
            db,
            conversation_id,
            &conversation_repo::ConversationParticipantRecord {
                agent_id: participant.agent_id.as_deref(),
                display_name: participant.display_name.as_deref(),
                participant_type: &participant.participant_type,
                enabled: participant.enabled,
                sort_order: participants.len() as i64,
                config_json: &participant.config_json,
            },
        )
        .await;
    }

    if let Some(existing) = participants.iter().find(|item| item.enabled).cloned() {
        return Ok(existing);
    }

    conversation_repo::create_conversation_participant(
        db,
        conversation_id,
        &conversation_repo::ConversationParticipantRecord {
            agent_id: None,
            display_name: default_display_name,
            participant_type: default_participant_type,
            enabled: true,
            sort_order: 0,
            config_json: &json!({ "created_by": "workflow" }).to_string(),
        },
    )
    .await
}

async fn last_message_node_id(db: &SqlitePool, conversation_id: &str) -> Result<Option<String>> {
    Ok(message_repo::list_message_nodes(db, conversation_id)
        .await?
        .into_iter()
        .last()
        .map(|item| item.id))
}

async fn create_message_from_content(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
    author_participant_id: &str,
    role: MessageRole,
    primary_content: ContentWriteInput,
    context_policy: ContextPolicy,
    viewer_policy: ViewerPolicy,
    config_json: Value,
) -> Result<MessageVersionView> {
    let order_after_node_id = last_message_node_id(db, conversation_id).await?;
    let input = CreateMessageInput {
        conversation_id: conversation_id.to_string(),
        author_participant_id: author_participant_id.to_string(),
        role,
        reply_to_node_id: None,
        order_after_node_id,
        primary_content,
        context_policy,
        viewer_policy,
        config_json,
    };
    match role {
        MessageRole::System => messages::create_system_message(db, store, &input).await,
        MessageRole::User => messages::create_user_message(db, store, &input).await,
        MessageRole::Assistant | MessageRole::Tool => {
            messages::create_assistant_message(db, store, &input).await
        }
    }
}

async fn clone_message_view_to_conversation(
    db: &SqlitePool,
    store: &ContentStore,
    source: &MessageVersionView,
    conversation_id: &str,
    author_participant_id: &str,
    viewer_policy: ViewerPolicy,
) -> Result<MessageVersionView> {
    let created = create_message_from_content(
        db,
        store,
        conversation_id,
        author_participant_id,
        source.role,
        stored_content_to_write_input(&source.primary_content),
        source.context_policy,
        viewer_policy,
        json!({
            "created_by": "workflow_output_clone",
            "source_message_version_id": source.version_id,
        }),
    )
    .await?;

    for (index, item) in source.content_refs.iter().enumerate() {
        let _ = messages::append_attachment(
            db,
            store,
            &crate::domain::messages::AddAttachmentInput {
                message_version_id: created.version_id.clone(),
                plugin_id: item.plugin_id.clone(),
                ref_role: item.ref_role.clone(),
                sort_order: index as i64,
                content: stored_content_to_write_input(&item.content),
                config_json: json!({
                    "created_by": "workflow_output_clone",
                    "source_ref_id": item.ref_id,
                }),
            },
        )
        .await?;
    }

    let mut tx = db.begin().await?;
    message_repo::update_message_version_generation_metadata(
        &mut tx,
        &created.version_id,
        source.api_channel_id.as_deref(),
        source.api_channel_model_id.as_deref(),
        source.generation_run_id.as_deref(),
        source.prompt_tokens,
        source.completion_tokens,
        source.total_tokens,
        source.finish_reason.as_deref(),
    )
    .await?;
    tx.commit().await?;

    messages::get_message_version_view(db, store, &created.version_id).await
}

async fn resolve_or_create_summary_group(
    db: &SqlitePool,
    conversation_id: &str,
    target_summary_group_id: Option<&str>,
) -> Result<SummaryGroupRow> {
    if let Some(summary_group_id) = target_summary_group_id {
        return summary_repo::get_summary_group(db, summary_group_id).await;
    }

    if let Some(existing) = summary_repo::find_summary_group_by_scope(
        db,
        conversation_id,
        "conversation",
        None,
        None,
        None,
        None,
        "brief",
    )
    .await?
    {
        return Ok(existing);
    }

    summary_repo::create_summary_group(
        db,
        &summary_repo::CreateSummaryGroupRecord {
            conversation_id,
            scope_type: "conversation",
            scope_message_version_id: None,
            scope_start_node_id: None,
            scope_end_node_id: None,
            scope_summary_group_id: None,
            summary_kind: "brief",
            default_generator_preset_id: None,
            enabled: true,
        },
    )
    .await
}

fn build_summary_sources_for_inputs(
    inputs: &[ExecutionInputMaterial],
) -> Vec<summary_repo::SummarySourceRecord<'_>> {
    inputs
        .iter()
        .enumerate()
        .map(|(idx, item)| summary_repo::SummarySourceRecord {
            source_kind: if item.source_message_version_id.is_some() {
                "message_version"
            } else {
                "summary_version"
            },
            source_message_version_id: item.source_message_version_id.as_deref(),
            source_start_node_id: None,
            source_end_node_id: None,
            source_summary_version_id: None,
            sort_order: idx as i64,
        })
        .collect()
}

struct VariableWriteResult {
    variable_def_id: String,
    var_key: String,
    scope_type: String,
    scope_id: String,
}

async fn write_variable_value(
    db: &SqlitePool,
    store: &ContentStore,
    run: &WorkflowRunRow,
    node: &WorkflowDefNodeRow,
    target_conversation_id: Option<&str>,
    variable_config: &Value,
    content: &StoredContent,
) -> Result<VariableWriteResult> {
    let var_key = variable_config
        .get("var_key")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AppError::Validation("variable writeback requires variable.var_key".to_string())
        })?
        .to_string();
    let scope_type = variable_config
        .get("scope_type")
        .and_then(Value::as_str)
        .unwrap_or("workflow_run")
        .to_string();
    let scope_id = variable_config
        .get("scope_id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| match scope_type.as_str() {
            "workflow_run" => Some(run.id.clone()),
            "conversation" => target_conversation_id.map(str::to_string),
            "agent" => node.agent_id.clone(),
            "plugin_scope" => node.plugin_id.clone(),
            _ => None,
        })
        .ok_or_else(|| {
            AppError::Validation(format!(
                "variable writeback could not resolve scope_id for scope_type '{}'",
                scope_type
            ))
        })?;

    let value_type = variable_config
        .get("value_type")
        .and_then(Value::as_str)
        .map(VariableValueType::parse)
        .transpose()?
        .unwrap_or_else(|| infer_variable_value_type(content));
    let value_json = match value_type {
        VariableValueType::Json => content
            .text_content
            .clone()
            .or_else(|| content.preview_text.clone())
            .unwrap_or_else(|| "null".to_string()),
        VariableValueType::String => serde_json::to_string(
            &content
                .text_content
                .clone()
                .or_else(|| content.preview_text.clone())
                .unwrap_or_default(),
        )?,
        VariableValueType::Number => {
            let raw = content
                .text_content
                .clone()
                .or_else(|| content.preview_text.clone())
                .unwrap_or_default();
            let number = serde_json::from_str::<serde_json::Value>(&raw)
                .ok()
                .filter(serde_json::Value::is_number)
                .unwrap_or_else(|| json!(0));
            number.to_string()
        }
        VariableValueType::Boolean => {
            let raw = content
                .text_content
                .clone()
                .or_else(|| content.preview_text.clone())
                .unwrap_or_default();
            let lowered = raw.trim().to_ascii_lowercase();
            let boolean = matches!(lowered.as_str(), "1" | "true" | "yes" | "on");
            serde_json::to_string(&boolean)?
        }
        VariableValueType::ContentRef => "null".to_string(),
    };

    let variable_def = match variables::get_variable_def_by_key(db, &var_key).await? {
        Some(existing) => existing,
        None => {
            variables::create_variable_def(
                db,
                &CreateVariableDefInput {
                    var_key: var_key.clone(),
                    name: variable_config
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or(&var_key)
                        .to_string(),
                    value_type,
                    scope_type: VariableScopeType::parse(&scope_type)?,
                    namespace: variable_config
                        .get("namespace")
                        .and_then(Value::as_str)
                        .unwrap_or("workflow")
                        .to_string(),
                    is_user_editable: variable_config
                        .get("is_user_editable")
                        .and_then(Value::as_bool)
                        .unwrap_or(true),
                    is_plugin_editable: variable_config
                        .get("is_plugin_editable")
                        .and_then(Value::as_bool)
                        .unwrap_or(true),
                    ai_can_create: variable_config
                        .get("ai_can_create")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    ai_can_update: variable_config
                        .get("ai_can_update")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    ai_can_delete: variable_config
                        .get("ai_can_delete")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    ai_can_lock: variable_config
                        .get("ai_can_lock")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    ai_can_unlock_own_lock: variable_config
                        .get("ai_can_unlock_own_lock")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    ai_can_unlock_any_lock: variable_config
                        .get("ai_can_unlock_any_lock")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    default_json: serde_json::Value::Null,
                    config_json: json!({ "created_by": "workflow" }),
                },
            )
            .await?
        }
    };

    let _ = variables::set_value(
        db,
        store,
        &SetVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::parse(&scope_type)?,
            scope_id: scope_id.clone(),
            value_json: serde_json::from_str(&value_json)?,
            value_content: if matches!(value_type, VariableValueType::ContentRef) {
                Some(stored_content_to_write_input(content))
            } else {
                None
            },
            source_message_version_id: None,
            updated_by_kind: "workflow".to_string(),
            updated_by_ref_id: Some(run.id.clone()),
        },
    )
    .await?;

    Ok(VariableWriteResult {
        variable_def_id: variable_def.id,
        var_key,
        scope_type,
        scope_id,
    })
}

fn infer_variable_value_type(content: &StoredContent) -> VariableValueType {
    match content.content_type {
        ContentType::Json => VariableValueType::Json,
        ContentType::Text | ContentType::Markdown | ContentType::Html => VariableValueType::String,
        _ => VariableValueType::ContentRef,
    }
}

fn should_clone_output_message(
    target_conversation_id: Option<&str>,
    source: &MessageVersionView,
) -> bool {
    if matches!(source.viewer_policy, ViewerPolicy::Hidden) {
        return true;
    }
    match target_conversation_id {
        Some(conversation_id) => source.conversation_id != conversation_id,
        None => false,
    }
}

fn participant_type_for_role(role: MessageRole) -> &'static str {
    match role {
        MessageRole::Assistant => "agent",
        MessageRole::Tool | MessageRole::System => "system",
        MessageRole::User => "human",
    }
}

fn parse_optional_message_role(value: Option<&Value>) -> Result<Option<MessageRole>> {
    match value.and_then(Value::as_str) {
        Some(raw) => Ok(Some(MessageRole::parse(raw)?)),
        None => Ok(None),
    }
}

fn stored_content_to_write_input(content: &StoredContent) -> ContentWriteInput {
    ContentWriteInput {
        content_type: content.content_type,
        mime_type: content.mime_type.clone(),
        text_content: content
            .text_content
            .clone()
            .or_else(|| content.preview_text.clone()),
        source_file_path: None,
        primary_storage_uri: content.primary_storage_uri.clone(),
        size_bytes_hint: Some(content.size_bytes),
        preview_text: content.preview_text.clone(),
        config_json: content.config_json.clone(),
    }
}

fn build_request_content_from_inputs(
    kind: &str,
    inputs: &[ExecutionInputMaterial],
    node_config: &Value,
) -> Result<ContentWriteInput> {
    let merged_text = inputs
        .iter()
        .map(|item| extract_textish_content(&item.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    build_node_output_content_input(
        kind,
        node_config.get("request"),
        inputs.first().map(|item| &item.content),
    )
    .map(|mut item| {
        if kind == "tool" {
            item.content_type = ContentType::ToolRequest;
            if item.mime_type.is_none() {
                item.mime_type = Some("text/plain".to_string());
            }
        }
        if item.text_content.is_none() {
            item.text_content = Some(merged_text);
        }
        item
    })
}

fn build_node_output_content_input(
    kind: &str,
    config_value: Option<&Value>,
    fallback_content: Option<&StoredContent>,
) -> Result<ContentWriteInput> {
    if let Some(config) = config_value {
        if let Some(text) = config.get("text").and_then(Value::as_str) {
            return Ok(ContentWriteInput {
                content_type: infer_content_type_from_kind(kind),
                mime_type: Some(
                    default_mime_for_content_type(infer_content_type_from_kind(kind)).to_string(),
                ),
                text_content: Some(text.to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: Some(text.len() as u64),
                preview_text: Some(text.chars().take(1024).collect()),
                config_json: config.clone(),
            });
        }
        if config.get("json").is_some() {
            let payload = config.get("json").cloned().unwrap_or(Value::Null);
            return Ok(ContentWriteInput {
                content_type: ContentType::Json,
                mime_type: Some("application/json".to_string()),
                text_content: Some(payload.to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: config.clone(),
            });
        }
        if matches!(config.get("use_input").and_then(Value::as_bool), Some(true)) {
            if let Some(fallback_content) = fallback_content {
                return Ok(stored_content_to_write_input(fallback_content));
            }
        }
    }

    if let Some(fallback_content) = fallback_content {
        return Ok(stored_content_to_write_input(fallback_content));
    }

    Ok(ContentWriteInput {
        content_type: infer_content_type_from_kind(kind),
        mime_type: Some(
            default_mime_for_content_type(infer_content_type_from_kind(kind)).to_string(),
        ),
        text_content: Some(String::new()),
        source_file_path: None,
        primary_storage_uri: None,
        size_bytes_hint: Some(0),
        preview_text: None,
        config_json: json!({}),
    })
}

fn infer_content_type_from_kind(kind: &str) -> ContentType {
    match kind {
        "tool" => ContentType::ToolResponse,
        "rag" => ContentType::RagExcerpt,
        "mcp" => ContentType::McpPayload,
        "plugin" => ContentType::PluginPayload,
        _ => ContentType::Text,
    }
}

fn default_mime_for_content_type(content_type: ContentType) -> &'static str {
    match content_type {
        ContentType::Json => "application/json",
        _ => "text/plain",
    }
}

fn extract_textish_content(content: &StoredContent) -> String {
    content
        .text_content
        .clone()
        .or_else(|| content.preview_text.clone())
        .or_else(|| content.primary_storage_uri.clone())
        .unwrap_or_else(|| {
            format!(
                "<{}:{} bytes>",
                content.content_type.as_str(),
                content.size_bytes
            )
        })
}

async fn create_error_content(
    db: &SqlitePool,
    store: &ContentStore,
    text: &str,
) -> Result<StoredContent> {
    content::create_content(
        db,
        store,
        &ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: Some(text.to_string()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: None,
            config_json: json!({ "kind": "workflow_error" }),
        },
    )
    .await
}

fn build_final_run_config(existing: &str, finalization: &WorkflowFinalization) -> Result<Value> {
    let mut config = parse_json(existing, "workflow_runs.config_json")?;
    config["status"] = Value::String(finalization.status.clone());
    if let Some(message_version_id) = &finalization.result_message_version_id {
        config["result_message_version_id"] = Value::String(message_version_id.clone());
    }
    if let Some(content_id) = &finalization.result_content_id {
        config["result_content_id"] = Value::String(content_id.clone());
    }
    Ok(config)
}

fn build_failed_run_config(
    existing: &str,
    err: &AppError,
    content: &StoredContent,
) -> Result<Value> {
    let mut config = parse_json(existing, "workflow_runs.config_json")?;
    config["status"] = Value::String("failed".to_string());
    config["error"] = Value::String(err.to_string());
    config["error_content_id"] = Value::String(content.content_id.clone());
    Ok(config)
}

fn is_terminal_run_status(status: &str) -> bool {
    matches!(status, "succeeded" | "failed" | "cancelled")
}

fn parse_json(raw: &str, field: &'static str) -> Result<Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}
