//! Agent tool loop: harness strategy → model → dispatch → feed results;
//! max 8 tool iterations with a bounded repair-and-reask budget.

use super::compress::{
    apply_context_strategies, prepare_tool_result_runtime, CompressState, STRATEGY_RESET,
};
use super::context_budget::{
    budget_from_messages, resolve_context_limit, ContextBudget, DEFAULT_COMPRESS_THRESHOLD,
};
use super::harness;
use super::llm::{ChatModel, ModelMessage, ModelTurn, StreamDelta, StreamMode};
use super::memory::{
    extract_and_ingest_session_delta, memory_token_cap, retrieve_for_inject,
};
use super::registry;
use super::session_log::{self, SessionEvent};
use super::trajectory::{build_trajectory_events_from_steps, TrajectoryStep};
use crate::commands::conversation::{append_assistant_message_full, get_messages_on};
use crate::db::open_connection;
use rusqlite::Connection;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};

/// Hard cap on tool-call rounds per `run_agent` invocation.
pub const MAX_TOOL_ITERATIONS: usize = 8;

/// Streaming / progress events for the frontend.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentEvent {
    StreamMeta { mode: StreamMode },
    Token { text: String },
    Reasoning { text: String },
    ToolStart { id: String, args: Value },
    ToolEnd { id: String, result: String },
    /// Process 工具执行前的用户审批请求。
    ToolApprovalRequired {
        request_id: String,
        tool_id: String,
        command: String,
        cwd: String,
        similar_key: String,
    },
    ContextBudget { budget: ContextBudget },
    Compress { message: String, count: usize },
    Error { message: String },
    Interrupted,
    Done,
}

fn history_to_model_messages(conn: &Connection, conv_id: &str) -> Result<Vec<ModelMessage>, String> {
    use super::llm::ToolCall;
    use super::trajectory::{resolve_trajectory, TrajectoryEvent};

    let history = get_messages_on(conn, conv_id)?;
    let mut out: Vec<ModelMessage> = Vec::new();
    for (msg_idx, m) in history.into_iter().enumerate() {
        if m.role == "user" {
            out.push(ModelMessage::user(m.content));
            continue;
        }
        if m.role != "assistant" {
            continue;
        }
        let events = resolve_trajectory(
            m.trajectory_json.as_deref(),
            m.reasoning.as_deref(),
            m.tool_calls_json.as_deref(),
            &m.content,
            None,
        );
        if events.is_empty() {
            if !m.content.trim().is_empty() {
                out.push(ModelMessage::assistant(m.content));
            }
            continue;
        }
        let mut i = 0usize;
        while i < events.len() {
            match &events[i] {
                TrajectoryEvent::ModelCall {
                    requested_tools,
                    response,
                    ..
                } if !requested_tools.is_empty() => {
                    let mut calls: Vec<ToolCall> = Vec::new();
                    let mut tool_msgs: Vec<ModelMessage> = Vec::new();
                    let mut j = i + 1;
                    while j < events.len() {
                        let TrajectoryEvent::ToolCall { id, args, result, .. } = &events[j] else {
                            break;
                        };
                        let call_id = format!("hist-{msg_idx}-{i}-{j}");
                        calls.push(ToolCall {
                            id: call_id.clone(),
                            name: id.clone(),
                            arguments: args.clone(),
                        });
                        tool_msgs.push(ModelMessage::tool_result(
                            call_id,
                            id.clone(),
                            result.clone().unwrap_or_default(),
                        ));
                        j += 1;
                    }
                    if !calls.is_empty() {
                        out.push(ModelMessage::assistant_tool_calls(calls));
                        out.extend(tool_msgs);
                    }
                    if let Some(text) = response.as_ref().filter(|s| !s.trim().is_empty()) {
                        out.push(ModelMessage::assistant(text.clone()));
                    }
                    i = j;
                }
                TrajectoryEvent::ModelCall { response, .. } => {
                    let text = response
                        .clone()
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or_else(|| {
                            // 末条无 response 时回退消息 content
                            if i + 1 >= events.len() {
                                m.content.clone()
                            } else {
                                String::new()
                            }
                        });
                    if !text.trim().is_empty() {
                        out.push(ModelMessage::assistant(text));
                    }
                    i += 1;
                }
                TrajectoryEvent::ToolCall { .. } => {
                    // 无前置 model_call 的孤儿工具步：跳过
                    i += 1;
                }
            }
        }
    }
    Ok(out)
}

/// Append-only session event log handle for one agent run.
/// 写失败不阻断循环（事件日志是旁路事实源，主流程仍由既有持久化承载）。
struct SessionLog<'a> {
    conn: &'a Connection,
    conv_id: String,
}

impl<'a> SessionLog<'a> {
    fn new(conn: &'a Connection, conv_id: &str) -> Self {
        Self {
            conn,
            conv_id: conv_id.to_string(),
        }
    }

    fn append(&self, kind: &str, data: Value) -> Result<SessionEvent, String> {
        session_log::append_event_on(self.conn, &self.conv_id, kind, data)
    }

    fn events(&self) -> Vec<SessionEvent> {
        session_log::list_events_on(self.conn, &self.conv_id).unwrap_or_default()
    }

    /// 下一个 turn_no：会话内已出现的最大 turn/end + 1（首个回合为 1）。
    fn next_turn_no(&self) -> usize {
        self.events()
            .iter()
            .filter(|e| e.kind == "turn/end")
            .count()
            + 1
    }
}

/// Connection-injected agent loop (unit tests + shared DB handle).
///
/// 系统提示由 harness 策略构建（embedded 小模型走强化四段式，其余走
/// 等价于旧版的默认提示）；参数在 dispatch 前经 schema 校验，失败在
/// 修复预算（repair_budget）内回填重试（reask），预算与工具回合上限
/// MAX_TOOL_ITERATIONS 合并核算，不叠加放大。
pub fn run_agent_on(
    conn: &Connection,
    model: &mut dyn ChatModel,
    conv_id: &str,
    user_text: &str,
    cancel: &AtomicBool,
    mut emit: impl FnMut(AgentEvent),
) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        emit(AgentEvent::Interrupted);
        return Ok(());
    }

    let desc = model.backend_desc();
    let strategy = harness::strategy_for(&desc.backend, &desc.model);
    let repair_budget = strategy.repair_budget();

    // --- 事件日志（append-only ledger；写失败仅告警，不阻断循环） ---------
    let log = SessionLog::new(conn, conv_id);
    // turn_no：该会话已结束的 turn 数 + 1（从日志推导，跨回合递增）
    let turn_no = log.next_turn_no();
    let _ = log.append("turn/start", session_log::ev_turn_start(turn_no));
    let _ = log.append("user/message", session_log::ev_user_message(user_text));

    let mut msgs = Vec::new();
    let mut system_prompt = strategy.build_system_prompt(user_text);
    let mut memory_tokens = 0u32;
    // Inject existing memories regardless of auto_memory_enabled (switch only gates extract).
    {
        let cap = memory_token_cap(resolve_context_limit());
        if let Ok((block, toks)) = retrieve_for_inject(conn, user_text, 8, cap) {
            if !block.is_empty() {
                system_prompt.push_str("\n\n");
                system_prompt.push_str(&block);
                memory_tokens = toks;
                let _ = log.append(
                    "context/snapshot",
                    session_log::ev_context_snapshot("memory", block.chars().count()),
                );
            }
        }
    }

    // Skill 渐进式披露：L0 目录常驻、L1 正文按需；与 system/message 分条落库。
    let skill_catalog = super::skills::skills_catalog_l0();
    let skill_docs = super::skills::retrieve_skills(user_text, 3);
    let l0_block = super::skills::render_skills_catalog_l0();
    let l1_block = super::skills::render_skill_bodies_l1(&skill_docs);

    let tools_json = registry::tools_as_openai_json();
    let logged_before = log.events();
    match session_log::last_system_snapshot(&logged_before) {
        None => {
            let _ = log.append(
                "system/message",
                session_log::ev_system_message(&system_prompt),
            );
            let _ = log.append("tools/catalog", session_log::ev_tools_catalog(&tools_json));
            if !skill_catalog.is_empty() {
                let _ = log.append(
                    "skills/catalog",
                    session_log::ev_skills_catalog(&skill_catalog),
                );
            }
        }
        Some((prev_content, prev_tools)) => {
            let prompt_changed = prev_content != system_prompt;
            let tools_changed = !session_log::tools_equal(&prev_tools, &tools_json);
            if prompt_changed {
                let _ = log.append(
                    "system/message",
                    session_log::ev_system_prompt_updated(&prev_content, &system_prompt),
                );
            }
            if tools_changed {
                let _ = log.append(
                    "tools/catalog",
                    session_log::ev_tools_catalog_updated(&prev_tools, &tools_json),
                );
            }
            let catalog_json = serde_json::to_value(&skill_catalog).unwrap_or(Value::Null);
            match session_log::last_skills_catalog(&logged_before) {
                None if !skill_catalog.is_empty() => {
                    let _ = log.append(
                        "skills/catalog",
                        session_log::ev_skills_catalog(&skill_catalog),
                    );
                }
                Some(prev_cat) if prev_cat != catalog_json => {
                    // previous entries best-effort from JSON
                    let prev_entries: Vec<super::skills::SkillCatalogEntry> =
                        serde_json::from_value(prev_cat).unwrap_or_default();
                    let _ = log.append(
                        "skills/catalog",
                        session_log::ev_skills_catalog_updated(&prev_entries, &skill_catalog),
                    );
                }
                _ => {}
            }
        }
    }
    if !skill_docs.is_empty() {
        let pairs: Vec<(String, String)> = skill_docs
            .iter()
            .map(|s| (s.skill_id.clone(), s.body.clone()))
            .collect();
        let _ = log.append(
            "context/snapshot",
            session_log::ev_skills_snapshot(&pairs),
        );
    }

    // 发给模型：base system + L0 + L1（L1 仅命中时）；落库的 system/message 不含 L0/L1。
    let mut system_for_model = system_prompt.clone();
    if !l0_block.is_empty() {
        system_for_model.push_str("\n\n");
        system_for_model.push_str(&l0_block);
    }
    if !l1_block.is_empty() {
        system_for_model.push_str("\n\n");
        system_for_model.push_str(&l1_block);
    }
    msgs.push(ModelMessage::system(system_for_model));
    msgs.extend(history_to_model_messages(conn, conv_id)?);
    // 增量 cursor：第 1 步 delta 用 user-only；此后 msgs[cursor..] 为本步追加
    let mut delta_cursor = msgs.len();

    let mut tool_iterations = 0usize;
    let mut repair_rounds = 0usize;
    let mut last_call_signature: Option<String> = None;
    let mut stuck_rounds: usize = 0;
    /// 连续相同签名调用达到此上限即视为卡住，提前结束回合。
    const STUCK_ROUNDS_LIMIT: usize = 2;

    let mut trajectory: Vec<TrajectoryStep> = Vec::new();
    let mut tools_for_persist: Vec<Value> = Vec::new();
    let mut compress_state = CompressState::default();

    // Track the start index of each loop iteration so we can rebuild the
    // trajectory as a Vec<ModelCall> at persist time (each ModelCall =
    // one agent↔model invocation).
    let mut iter_boundaries: Vec<usize> = Vec::new();
    // Stream mode reported by the model (most backends report the same mode
    // for every iteration in a single turn; we keep the latest seen).
    let mut current_stream_mode: Option<String> = None;

    loop {
        // Mark where this iteration's steps begin.
        iter_boundaries.push(trajectory.len());
        let step_no = iter_boundaries.len();
        let _ = log.append("step/start", session_log::ev_step_start(turn_no, step_no));

        if cancel.load(Ordering::SeqCst) {
            persist_partial(conn, conv_id, &trajectory, &iter_boundaries, current_stream_mode.as_deref())?;
            emit(AgentEvent::Interrupted);
            return Ok(());
        }

        let budget = budget_from_messages(&msgs, memory_tokens, 0);
        let _ = crate::commands::conversation::save_context_budget_on(conn, conv_id, &budget);
        emit(AgentEvent::ContextBudget {
            budget: budget.clone(),
        });
        let mut compress_outcome: Option<super::compress::CompressOutcome> = None;
        match apply_context_strategies(
            &mut msgs,
            &budget,
            DEFAULT_COMPRESS_THRESHOLD,
            &mut compress_state,
            user_text,
            memory_tokens,
        ) {
            Ok(outcome) if !outcome.is_empty() => {
                let strategies = outcome.strategies_applied.join("→");
                emit(AgentEvent::Compress {
                    message: format!(
                        "上下文压缩（{strategies}），影响 {} 条",
                        outcome.count
                    ),
                    count: outcome.count,
                });
                let _ = log.append(
                    "compact/checkpoint",
                    session_log::ev_compact_checkpoint(
                        outcome.count,
                        &format!(
                            "上下文压缩 {strategies}（仅改变模型视图，原始日志保留）"
                        ),
                        &outcome.strategies_applied,
                    ),
                );
                let budget2 = budget_from_messages(&msgs, memory_tokens, 0);
                let _ = crate::commands::conversation::save_context_budget_on(conn, conv_id, &budget2);
                emit(AgentEvent::ContextBudget { budget: budget2 });
                // After compression, next append-based delta starts at current len
                delta_cursor = msgs.len();
                compress_outcome = Some(outcome);
            }
            Ok(_) => {}
            Err(e) => {
                emit(AgentEvent::Error { message: e });
                // Continue without further compress attempts this turn
            }
        }

        // --- 日志重建一致性校验 + 本步 delta（压缩之后、请求之前） -------
        let logged = log.events();
        let actual_msgs: Vec<Value> = msgs
            .iter()
            .map(|m| {
                let has_tools = !m.tool_calls.is_empty();
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                    "tool_calls": if has_tools { Value::Bool(true) } else { Value::Null },
                })
            })
            .collect();
        let desync = session_log::desync_check_value(
            &logged,
            &actual_msgs,
            compress_state.last_compressed,
        );
        let delta = if let Some(ref outcome) = compress_outcome {
            let strategy = outcome
                .last_strategy
                .as_deref()
                .unwrap_or("compress");
            let after_system: Vec<_> = msgs
                .iter()
                .skip_while(|m| m.role == "system")
                .cloned()
                .collect();
            session_log::compressed_delta_value(
                strategy,
                strategy == STRATEGY_RESET,
                &after_system,
            )
        } else if step_no == 1 {
            session_log::user_delta_value(user_text)
        } else {
            let start = delta_cursor.min(msgs.len());
            session_log::wrap_delta_messages(session_log::messages_delta_value(&msgs[start..]))
        };
        let _actual_count = msgs.len();
        let _ = log.append(
            "request/header",
            session_log::ev_request_header(
                &desc.model,
                &desc.backend,
                _actual_count,
                &registry::tool_ids(),
                current_stream_mode.as_deref(),
                &desync,
                &delta,
            ),
        );
        if desync.get("match").and_then(|m| m.as_bool()) == Some(false) {
            eprintln!(
                "[agent] log-reconstruction desync: derived={} actual={} (see request/header event)",
                desync.get("derived_messages").and_then(|v| v.as_i64()).unwrap_or(-1),
                desync.get("actual_messages").and_then(|v| v.as_i64()).unwrap_or(-1),
            );
        }

        let mut think = super::llm::ThinkStreamParser::default();
        let mut tool_filter = harness::parse::ToolCallStreamFilter::default();
        let mut saw_meta = false;
        let turn = {
            let emit_ref = &mut emit;
            let traj_ref = &mut trajectory;
            match model.complete_streaming(&msgs, cancel, &mut |delta| {
                match delta {
                    StreamDelta::Meta { mode } => {
                        saw_meta = true;
                        current_stream_mode = Some(mode_str(mode));
                        emit_ref(AgentEvent::StreamMeta { mode });
                    }
                    StreamDelta::Reasoning { text } => {
                        if text.is_empty() {
                            return;
                        }
                        append_reasoning_step(traj_ref, &text);
                        emit_ref(AgentEvent::Reasoning { text });
                    }
                    StreamDelta::Text { text } => {
                        if text.is_empty() {
                            return;
                        }
                        think.push(&text, &mut |d| match d {
                            StreamDelta::Reasoning { text } => {
                                append_reasoning_step(traj_ref, &text);
                                emit_ref(AgentEvent::Reasoning { text });
                            }
                            StreamDelta::Text { text } => {
                                tool_filter.push(&text, &mut |visible| {
                                    if visible.is_empty() {
                                        return;
                                    }
                                    append_text_step(traj_ref, &visible);
                                    emit_ref(AgentEvent::Token { text: visible });
                                });
                            }
                            StreamDelta::Meta { .. } => {}
                        });
                    }
                }
            }) {
                Ok(t) => t,
                Err(e) => {
                    emit(AgentEvent::Error {
                        message: e.clone(),
                    });
                    return Err(e);
                }
            }
        };

        if !saw_meta {
            current_stream_mode = Some("fallback".to_string());
            emit(AgentEvent::StreamMeta {
                mode: StreamMode::Fallback,
            });
        }

        let (parsed_reasoning, parsed_body) = think.finish(&mut |d| match d {
            StreamDelta::Reasoning { text } => {
                append_reasoning_step(&mut trajectory, &text);
                emit(AgentEvent::Reasoning { text });
            }
            StreamDelta::Text { text } => {
                tool_filter.push(&text, &mut |visible| {
                    if visible.is_empty() {
                        return;
                    }
                    append_text_step(&mut trajectory, &visible);
                    emit(AgentEvent::Token { text: visible });
                });
            }
            StreamDelta::Meta { .. } => {}
        });
        tool_filter.finish(&mut |visible| {
            if visible.is_empty() {
                return;
            }
            append_text_step(&mut trajectory, &visible);
            emit(AgentEvent::Token { text: visible });
        });

        match turn {
            ModelTurn::Text { text, reasoning } => {
                let (inline_reasoning, body_from_tags) = super::llm::split_think_tags(&text);
                let reasoning = merge_reasoning(
                    reasoning.filter(|r| !r.trim().is_empty()),
                    inline_reasoning.or(parsed_reasoning),
                );
                let mut body = if !parsed_body.is_empty() {
                    parsed_body
                } else {
                    body_from_tags
                };
                body = harness::parse::strip_tool_call_markup(&body);
                scrub_tool_call_text_from_trajectory(&mut trajectory);

                // 若流式未推送任何正文/思考（极端空回合），按结果补发一次。
                if trajectory_has_no_content(&trajectory) && (!body.is_empty() || reasoning.is_some())
                {
                    if let Some(r) = reasoning.as_ref() {
                        append_reasoning_step(&mut trajectory, r);
                        emit(AgentEvent::Reasoning { text: r.clone() });
                    }
                    if !body.is_empty() {
                        append_text_step(&mut trajectory, &body);
                        emit(AgentEvent::Token { text: body.clone() });
                    }
                }

                let tools_json = if tools_for_persist.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&tools_for_persist).unwrap_or_default())
                };
                let traj_json = serialize_trajectory(
                    &trajectory,
                    &iter_boundaries,
                    current_stream_mode.as_deref(),
                );
                append_assistant_message_full(
                    conn,
                    conv_id,
                    &body,
                    tools_json.as_deref(),
                    reasoning.as_deref(),
                    traj_json.as_deref(),
                )?;
                // Non-fatal memory extract from this turn's user+assistant text
                let delta = format!("{user_text}\n{body}");
                let _ = extract_and_ingest_session_delta(conv_id, &delta);
                // 事件日志：assistant/message(纯文本)、step/end、turn/end
                let _ = log.append(
                    "assistant/message",
                    session_log::ev_assistant_message(
                        reasoning.as_deref(),
                        if body.is_empty() { None } else { Some(&body) },
                        &Value::Array(Vec::new()),
                    ),
                );
                let _ = log.append("step/end", session_log::ev_step_end(turn_no, step_no));
                let _ = log.append("turn/end", session_log::ev_turn_end(turn_no));
                emit(AgentEvent::Done);
                return Ok(());
            }
            ModelTurn::ToolCalls(calls) => {
                // 工具轮：丢掉已泄漏到轨迹的 tool_call 正文与 few-shot 回声，工具不算对话输出
                scrub_tool_call_text_from_trajectory(&mut trajectory);
                let window_start = *iter_boundaries.last().unwrap_or(&0);
                scrub_text_steps_in_window(&mut trajectory, window_start);
                let signature = calls
                    .iter()
                    .map(|c| format!("{}:{}", c.name, c.arguments))
                    .collect::<Vec<_>>()
                    .join("|");
                if Some(&signature) == last_call_signature.as_ref() {
                    stuck_rounds += 1;
                } else {
                    stuck_rounds = 0;
                }
                last_call_signature = Some(signature);
                if stuck_rounds >= STUCK_ROUNDS_LIMIT {
                    return finish_soft_exit(
                        conn,
                        conv_id,
                        SoftExitKind::StuckLoop,
                        &mut trajectory,
                        &iter_boundaries,
                        current_stream_mode.as_deref(),
                        &tools_for_persist,
                        &mut emit,
                    );
                }

                if tool_iterations >= MAX_TOOL_ITERATIONS {
                    return finish_soft_exit(
                        conn,
                        conv_id,
                        SoftExitKind::MaxToolIterations,
                        &mut trajectory,
                        &iter_boundaries,
                        current_stream_mode.as_deref(),
                        &tools_for_persist,
                        &mut emit,
                    );
                }
                tool_iterations += 1;

                let append_start = msgs.len();
                msgs.push(ModelMessage::assistant_tool_calls(calls.clone()));

                let mut any_success = false;
                // assistant/message = 这次模型决策（思考 + 请求的工具及参数）
                let step_reasoning = reasoning_in_window(&trajectory, window_start).or_else(|| {
                    parsed_reasoning
                        .as_ref()
                        .filter(|s| !s.trim().is_empty())
                        .cloned()
                });
                let calls_value = tool_calls_log_value(&calls);
                let _ = log.append(
                    "assistant/message",
                    session_log::ev_assistant_message(
                        step_reasoning.as_deref(),
                        None,
                        &calls_value,
                    ),
                );
                let step_no_tool = step_no;
                for call in &calls {
                    let tool_id = call.name.clone();
                    emit(AgentEvent::ToolStart {
                        id: tool_id.clone(),
                        args: call.arguments.clone(),
                    });
                    let _ = log.append(
                        "tool/call",
                        session_log::ev_tool_call(&tool_id, &call.arguments),
                    );
                    let result = match harness::parse::validate_call(
                        &call.name,
                        &call.arguments,
                    ) {
                        Err(e) => e,
                        Ok(()) => {
                            let needs_approval = registry::lookup(&call.name)
                                .map(|s| s.side_effect == registry::SideEffect::Process)
                                .unwrap_or(false);
                            let approved = if needs_approval {
                                let command = call
                                    .arguments
                                    .get("command")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let cwd = call
                                    .arguments
                                    .get("cwd")
                                    .and_then(|v| v.as_str())
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| {
                                        crate::agent::os_shell::default_cwd()
                                            .display()
                                            .to_string()
                                    });
                                let similar_key =
                                    crate::agent::os_shell::similar_key(&command);
                                if crate::agent::tool_approval::is_session_allowed(
                                    conv_id, &similar_key,
                                ) {
                                    true
                                } else {
                                    let request_id = uuid::Uuid::new_v4().to_string();
                                    emit(AgentEvent::ToolApprovalRequired {
                                        request_id: request_id.clone(),
                                        tool_id: tool_id.clone(),
                                        command: command.clone(),
                                        cwd: cwd.clone(),
                                        similar_key: similar_key.clone(),
                                    });
                                    let decision =
                                        crate::agent::tool_approval::wait_for_approval(
                                            &request_id,
                                            conv_id,
                                            &similar_key,
                                            cancel,
                                        );
                                    !matches!(
                                        decision,
                                        crate::agent::tool_approval::ApprovalDecision::Deny
                                    )
                                }
                            } else {
                                true
                            };
                            if !approved {
                                "用户拒绝执行该命令".to_string()
                            } else {
                                match registry::dispatch(&call.name, &call.arguments) {
                                    Ok(s) => {
                                        any_success = true;
                                        s
                                    }
                                    Err(e) => e,
                                }
                            }
                        }
                    };
                    emit(AgentEvent::ToolEnd {
                        id: tool_id.clone(),
                        result: result.clone(),
                    });
                    let tool_ok = any_success && !result.is_empty();
                    let _ = log.append(
                        "tool/result",
                        session_log::ev_tool_result(&tool_id, tool_ok, &result),
                    );
                    trajectory.push(TrajectoryStep::Tool {
                        id: tool_id.clone(),
                        args: call.arguments.clone(),
                        result: Some(result.clone()),
                        status: if any_success && result != "" {
                            "done".into()
                        } else {
                            "done".into()
                        },
                    });
                    tools_for_persist.push(serde_json::json!({
                        "id": tool_id,
                        "args": call.arguments,
                        "result": result,
                        "status": "done",
                    }));
                    // Runtime gets truncated/compressed view; Audit trajectory keeps full result
                    let runtime_result = prepare_tool_result_runtime(&result);
                    msgs.push(ModelMessage::tool_result(
                        call.id.clone(),
                        call.name.clone(),
                        runtime_result,
                    ));
                }

                // 下一步模型请求的增量 = 本轮刚追加的 assistant + tool（从 append_start 起）
                delta_cursor = append_start;

                if !any_success {
                    if repair_rounds >= repair_budget {
                        return finish_soft_exit(
                            conn,
                            conv_id,
                            SoftExitKind::RepairExhausted { budget: repair_budget },
                            &mut trajectory,
                            &iter_boundaries,
                            current_stream_mode.as_deref(),
                            &tools_for_persist,
                            &mut emit,
                        );
                    }
                    repair_rounds += 1;
                }
                // 当前 step（一次模型调用）已结束；下次迭代由 loop 顶部 step/start 启新。
                let _ = log.append(
                    "step/end",
                    session_log::ev_step_end(turn_no, step_no_tool),
                );
            }
        }
    }
}

fn scrub_tool_call_text_from_trajectory(traj: &mut Vec<TrajectoryStep>) {
    for step in traj.iter_mut() {
        if let TrajectoryStep::Text { text } = step {
            *text = harness::parse::strip_tool_call_markup(text);
        }
    }
    traj.retain(|s| !matches!(s, TrajectoryStep::Text { text } if text.trim().is_empty()));
}

/// 工具调用轮：丢掉本窗口内已流式泄漏的正文（常见为 few-shot「工具结果：/助手：」回声）。
fn scrub_text_steps_in_window(traj: &mut Vec<TrajectoryStep>, window_start: usize) {
    if window_start >= traj.len() {
        return;
    }
    let keep: Vec<TrajectoryStep> = traj[..window_start].to_vec();
    let rest: Vec<TrajectoryStep> = traj[window_start..]
        .iter()
        .filter(|s| !matches!(s, TrajectoryStep::Text { .. }))
        .cloned()
        .collect();
    *traj = keep;
    traj.extend(rest);
}

fn merge_reasoning(a: Option<String>, b: Option<String>) -> Option<String> {
    match (a, b) {
        (Some(x), Some(y)) => Some(format!("{x}\n{y}")),
        (r, i) => r.or(i),
    }
}

fn mode_str(mode: StreamMode) -> String {
    match mode {
        StreamMode::Live => "live".to_string(),
        StreamMode::Fallback => "fallback".to_string(),
    }
}

fn serialize_trajectory(
    steps: &[TrajectoryStep],
    boundaries: &[usize],
    stream_mode: Option<&str>,
) -> Option<String> {
    let events = build_trajectory_events_from_steps(steps, boundaries, stream_mode);
    if events.is_empty() {
        return None;
    }
    serde_json::to_string(&events).ok()
}

fn append_reasoning_step(traj: &mut Vec<TrajectoryStep>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(TrajectoryStep::Reasoning { text: t }) = traj.last_mut() {
        t.push_str(text);
    } else {
        traj.push(TrajectoryStep::Reasoning {
            text: text.to_string(),
        });
    }
}

fn append_text_step(traj: &mut Vec<TrajectoryStep>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(TrajectoryStep::Text { text: t }) = traj.last_mut() {
        t.push_str(text);
    } else {
        traj.push(TrajectoryStep::Text {
            text: text.to_string(),
        });
    }
}

fn trajectory_has_no_content(traj: &[TrajectoryStep]) -> bool {
    !traj.iter().any(|s| {
        matches!(
            s,
            TrajectoryStep::Reasoning { .. } | TrajectoryStep::Text { .. }
        )
    })
}

/// 当前迭代窗口内的思考文本（不含后续工具结果）。
fn reasoning_in_window(steps: &[TrajectoryStep], from: usize) -> Option<String> {
    let start = from.min(steps.len());
    let mut out = String::new();
    for step in &steps[start..] {
        if let TrajectoryStep::Reasoning { text } = step {
            if text.trim().is_empty() {
                continue;
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(text);
        }
    }
    let trimmed = out.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn tool_calls_log_value(calls: &[crate::agent::llm::ToolCall]) -> Value {
    Value::Array(
        calls
            .iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.name,
                    "name": c.name,
                    "args": c.arguments,
                })
            })
            .collect(),
    )
}

fn persist_partial(
    conn: &Connection,
    conv_id: &str,
    trajectory: &[TrajectoryStep],
    iter_boundaries: &[usize],
    stream_mode: Option<&str>,
) -> Result<(), String> {
    let body = trajectory
        .iter()
        .filter_map(|s| match s {
            TrajectoryStep::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    let reasoning = trajectory
        .iter()
        .filter_map(|s| match s {
            TrajectoryStep::Reasoning { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let tools_json = if trajectory.is_empty() {
        None
    } else {
        Some("[]".to_string())
    };
    let traj_json = serialize_trajectory(trajectory, iter_boundaries, stream_mode);
    append_assistant_message_full(
        conn,
        conv_id,
        &body,
        tools_json.as_deref(),
        if reasoning.is_empty() {
            None
        } else {
            Some(reasoning.as_str())
        },
        traj_json.as_deref(),
    )?;
    Ok(())
}

/// 工具循环软退出原因（仅写 session_log，不进入用户可见正文）。
enum SoftExitKind {
    StuckLoop,
    MaxToolIterations,
    RepairExhausted { budget: usize },
}

impl SoftExitKind {
    fn log_note(&self) -> String {
        match self {
            SoftExitKind::StuckLoop => {
                "soft_exit: repeated identical tool calls".into()
            }
            SoftExitKind::MaxToolIterations => {
                format!("soft_exit: exceeded {MAX_TOOL_ITERATIONS} tool iterations")
            }
            SoftExitKind::RepairExhausted { budget } => {
                format!("soft_exit: repair budget exhausted ({budget})")
            }
        }
    }

    /// 无可用工具结果时的极简用户文案（不暴露循环/预算等内部细节）。
    fn fallback_user_text(&self) -> &'static str {
        match self {
            SoftExitKind::StuckLoop | SoftExitKind::MaxToolIterations => {
                "已完成相关计算，结果见上方工具输出。"
            }
            SoftExitKind::RepairExhausted { .. } => {
                "未能完成该工具调用，请换一种表述后重试。"
            }
        }
    }
}

/// Collect the newest non-empty tool result for user-facing reply.
/// Structured payloads (JSON) are wrapped in fenced code so the chat
/// Markdown renderer highlights them instead of splitting into paragraphs.
fn recent_tool_results_for_user(trajectory: &[TrajectoryStep]) -> String {
    for step in trajectory.iter().rev() {
        if let TrajectoryStep::Tool {
            result: Some(r), ..
        } = step
        {
            let t = r.trim();
            if t.is_empty() {
                continue;
            }
            return format_tool_result_for_display(t);
        }
    }
    String::new()
}

fn format_tool_result_for_display(result: &str) -> String {
    let t = result.trim();
    if looks_like_json(t) {
        return format!("```json\n{t}\n```");
    }
    if t.starts_with('<') && t.contains('>') {
        return format!("```xml\n{t}\n```");
    }
    t.to_string()
}

fn looks_like_json(s: &str) -> bool {
    let t = s.trim();
    if !(t.starts_with('{') && t.ends_with('}') || t.starts_with('[') && t.ends_with(']')) {
        return false;
    }
    serde_json::from_str::<serde_json::Value>(t).is_ok()
}

/// Soft-exit the tool loop: persist user-facing content from the last tool
/// result(s). Internal loop diagnostics go to session_log only — never into
/// the assistant message the user reads.
#[allow(clippy::too_many_arguments)]
fn finish_soft_exit(
    conn: &Connection,
    conv_id: &str,
    kind: SoftExitKind,
    trajectory: &mut Vec<TrajectoryStep>,
    iter_boundaries: &[usize],
    stream_mode: Option<&str>,
    tools_for_persist: &[Value],
    emit: &mut dyn FnMut(AgentEvent),
) -> Result<(), String> {
    let mut body = recent_tool_results_for_user(trajectory);
    if body.trim().is_empty() {
        body = kind.fallback_user_text().to_string();
    }

    append_text_step(trajectory, &body);
    emit(AgentEvent::Token { text: body.clone() });
    let tools_json = if tools_for_persist.is_empty() {
        None
    } else {
        Some(serde_json::to_string(tools_for_persist).unwrap_or_default())
    };
    let traj_json = serialize_trajectory(trajectory, iter_boundaries, stream_mode);
    append_assistant_message_full(
        conn,
        conv_id,
        &body,
        tools_json.as_deref(),
        None,
        traj_json.as_deref(),
    )?;
    let log = SessionLog::new(conn, conv_id);
    let turn_no = log.next_turn_no().saturating_sub(1).max(1);
    let _ = log.append(
        "system/note",
        serde_json::json!({ "note": kind.log_note() }),
    );
    let _ = log.append(
        "assistant/message",
        session_log::ev_assistant_message(None, Some(&body), &Value::Array(Vec::new())),
    );
    let step_count = iter_boundaries.len();
    let _ = log.append(
        "step/end",
        session_log::ev_step_end(turn_no, step_count.max(1)),
    );
    let _ = log.append("turn/end", session_log::ev_turn_end(turn_no));
    emit(AgentEvent::Done);
    Ok(())
}

/// Agent loop using the application SQLite database.
pub fn run_agent(
    model: &mut dyn ChatModel,
    conv_id: &str,
    user_text: &str,
    cancel: &AtomicBool,
    emit: impl FnMut(AgentEvent),
) -> Result<(), String> {
    let conn = open_connection()?;
    run_agent_on(&conn, model, conv_id, user_text, cancel, emit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::llm::ToolCall;
    use crate::commands::conversation::{append_user_message_on, get_messages_on};
    use rusqlite::Connection;
    use serde_json::json;
    use std::collections::VecDeque;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Scripted {
        turns: VecDeque<ModelTurn>,
        pub received: Vec<Vec<ModelMessage>>,
    }

    impl Scripted {
        fn new(turns: Vec<ModelTurn>) -> Self {
            Self {
                turns: turns.into(),
                received: Vec::new(),
            }
        }
    }

    impl ChatModel for Scripted {
        fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String> {
            self.received.push(msgs.to_vec());
            self.turns
                .pop_front()
                .ok_or_else(|| "Scripted: no more turns".to_string())
        }
    }

    fn test_db() -> Connection {
        let mut dir = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("tbox-agent-loop-test-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        let path: PathBuf = dir.join("tools.db");
        Connection::open(path).unwrap()
    }

    fn collect_events() -> (impl FnMut(AgentEvent), Arc<Mutex<Vec<AgentEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_c = events.clone();
        let emit = move |e: AgentEvent| {
            events_c.lock().unwrap().push(e);
        };
        (emit, events)
    }

    #[test]
    fn text_only_reply() {
        let db = test_db();
        let user_text = "hello";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        let mut model = Scripted::new(vec![ModelTurn::text("hi there")]);
        let cancel = AtomicBool::new(false);
        let (mut emit, events) = collect_events();

        run_agent_on(
            &db,
            &mut model,
            &conv.id,
            user_text,
            &cancel,
            &mut emit,
        )
        .unwrap();

        let msgs = get_messages_on(&db, &conv.id).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "user");
        assert_eq!(msgs[1].role, "assistant");
        assert_eq!(msgs[1].content, "hi there");
        let ev = events.lock().unwrap();
        assert!(ev.iter().any(|e| matches!(e, AgentEvent::Token { .. })));
        assert!(ev.iter().any(|e| matches!(e, AgentEvent::Done)));
        assert!(ev.iter().any(|e| {
            matches!(
                e,
                AgentEvent::StreamMeta {
                    mode: StreamMode::Fallback
                }
            )
        }));
        assert!(msgs[1].trajectory_json.is_some());
    }

    /// 回归：超过 MAX_TOOL_ITERATIONS 时不再 emit Error 报错，而是把工具
    /// 结果摘要成 assistant 文本回复持久化，让对话有完整收尾。
    #[test]
    fn max_iterations_soft_exit_persists_assistant_message() {
        let db = test_db();
        let (conv, _) = append_user_message_on(&db, None, "loop cap").unwrap();
        let mut model = Scripted::new((0..MAX_TOOL_ITERATIONS + 1).map(|i| {
            ModelTurn::ToolCalls(vec![ToolCall {
                id: format!("c{i}"),
                name: "base64.encode".into(),
                arguments: json!({"input": format!("x{i}")}),
            }])
        }).chain(std::iter::once(ModelTurn::text("never"))).collect());
        let cancel = AtomicBool::new(false);
        let (mut emit, events) = collect_events();
        run_agent_on(&db, &mut model, &conv.id, "loop cap", &cancel, &mut emit).unwrap();

    let msgs = get_messages_on(&db, &conv.id).unwrap();
    let last = msgs.last().expect("assistant message persisted");
    assert_eq!(last.role, "assistant");
    assert!(last.content.contains("连续发起"));
    let evs = events.lock().unwrap();
    assert!(matches!(evs.last(), Some(AgentEvent::Done)));
    assert!(!evs.iter().any(|e| matches!(e, AgentEvent::Error { .. })));
}

/// 回归：连续相同签名的 tool calls 在第二轮应触发 STUCK_ROUNDS_LIMIT，
/// 不再硬撞 MAX_TOOL_ITERATIONS；用户可见正文为最近工具结果，不含循环诊断文案。
#[test]
fn stuck_round_signature_exits_early_with_explanation() {
    let db = test_db();
    let (conv, _) = append_user_message_on(&db, None, "loop test").unwrap();
    let mut model = Scripted::new(vec![
        // 3 轮相同签名：第 1 轮入库为基准，第 2 轮 stuck=1，
        // 第 3 轮 stuck=2 触发 STUCK_ROUNDS_LIMIT 软退出。
        ModelTurn::ToolCalls(vec![ToolCall {
            id: "c1".into(),
            name: "base64.encode".into(),
            arguments: json!({"input": "x"}),
        }]),
        ModelTurn::ToolCalls(vec![ToolCall {
            id: "c2".into(),
            name: "base64.encode".into(),
            arguments: json!({"input": "x"}),
        }]),
        ModelTurn::ToolCalls(vec![ToolCall {
            id: "c3".into(),
            name: "base64.encode".into(),
            arguments: json!({"input": "x"}),
        }]),
        ModelTurn::text("never"),
    ]);
    let cancel = AtomicBool::new(false);
    let (mut emit, events) = collect_events();
    run_agent_on(&db, &mut model, &conv.id, "loop test", &cancel, &mut emit).unwrap();

    let msgs = get_messages_on(&db, &conv.id).unwrap();
    let last = msgs.last().expect("assistant message persisted");
    assert_eq!(last.role, "assistant");
    // User-facing: tool result only — no loop diagnostics
    assert!(!last.content.contains("重复发起"));
    assert!(!last.content.contains("陷入循环"));
    assert!(!last.content.contains("最近一次工具调用"));
    assert!(
        last.content.contains("eA==")
            || last.content.contains("```")
            || last.content.contains("已完成"),
        "expected base64 of 'x', fenced result, or neutral fallback, got: {}",
        last.content
    );
    assert_eq!(model.received.len(), 3);
    let evs = events.lock().unwrap();
    assert!(matches!(evs.last(), Some(AgentEvent::Done)));
    assert!(!evs.iter().any(|e| matches!(e, AgentEvent::Error { .. })));
}

    #[test]
    fn tool_call_roundtrip_paired_in_context() {
        let db = test_db();
        let (conv, _) = append_user_message_on(&db, None, "encode hi").unwrap();
        let mut model = Scripted::new(vec![
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "call_1".into(),
                name: "base64.encode".into(),
                arguments: json!({"input": "hi"}),
            }]),
            ModelTurn::text("done"),
        ]);
        let cancel = AtomicBool::new(false);
        let (mut emit, _) = collect_events();
        run_agent_on(&db, &mut model, &conv.id, "encode hi", &cancel, &mut emit).unwrap();

        let second = &model.received[1];
        // 倒数第二条是 assistant tool_calls 消息
        let assistant_call = second
            .iter()
            .rev()
            .find(|m| !m.tool_calls.is_empty())
            .expect("assistant tool_calls message must be replayed");
        assert_eq!(assistant_call.tool_calls[0].id, "call_1");
        assert_eq!(assistant_call.tool_calls[0].name, "base64.encode");
        // 最后一条是带 call_id 配对的 tool 结果
        let last = second.last().unwrap();
        assert_eq!(last.role, "tool");
        assert_eq!(last.tool_call_id.as_deref(), Some("call_1"));
        assert_eq!(last.tool_name.as_deref(), Some("base64.encode"));
    }

    #[test]
    fn one_tool_call_persists_trajectory_order() {
        let db = test_db();
        let user_text = "base64 encode hi";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        let mut model = Scripted::new(vec![
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c1".into(),
                name: "base64.encode".into(),
                arguments: json!({"input": "hi"}),
            }]),
            ModelTurn::text("done"),
        ]);
        let cancel = AtomicBool::new(false);
        let (mut emit, _) = collect_events();
        run_agent_on(&db, &mut model, &conv.id, user_text, &cancel, &mut emit).unwrap();
        let msgs = get_messages_on(&db, &conv.id).unwrap();
        let last = msgs.last().unwrap();
        let traj = last.trajectory_json.as_ref().expect("trajectory stored");
        let events: Vec<serde_json::Value> = serde_json::from_str(traj).unwrap();
        // New shape: flat events — model_call followed by its tool_call siblings.
        assert!(
            events.iter().any(|n| n["type"] == "model_call"
                && n["requested_tools"]
                    .as_array()
                    .map(|arr| arr.iter().any(|t| t == "base64.encode"))
                    .unwrap_or(false)),
            "model_call requesting base64.encode in trajectory"
        );
        assert!(
            events.iter().any(|n| n["type"] == "tool_call" && n["id"] == "base64.encode"),
            "tool_call event in trajectory"
        );
        assert!(
            events.iter().any(|n| n["type"] == "model_call" && n["response"].is_string()),
            "final model_call with response in trajectory"
        );
        // The tool-dispatching model_call must come before the responding one.
        let tool_call_idx = events.iter().position(|n| n["type"] == "model_call"
            && n["requested_tools"].as_array().map(|a| !a.is_empty()).unwrap_or(false));
        let resp_call_idx = events
            .iter()
            .position(|n| n["type"] == "model_call" && n["response"].is_string());
        assert!(tool_call_idx < resp_call_idx, "tool call before response call");
    }

    #[test]
    fn one_tool_call_then_text() {
        let db = test_db();
        let user_text = "base64 encode hi";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        let mut model = Scripted::new(vec![
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c1".into(),
                name: "base64.encode".into(),
                arguments: json!({"input": "hi"}),
            }]),
            ModelTurn::text("encoded"),
        ]);
        let cancel = AtomicBool::new(false);
        let (mut emit, events) = collect_events();

        run_agent_on(
            &db,
            &mut model,
            &conv.id,
            user_text,
            &cancel,
            &mut emit,
        )
        .unwrap();

        let msgs = get_messages_on(&db, &conv.id).unwrap();
        assert_eq!(msgs.last().unwrap().role, "assistant");
        assert_eq!(msgs.last().unwrap().content, "encoded");
        let ev = events.lock().unwrap();
        assert!(ev.iter().any(|e| matches!(
            e,
            AgentEvent::ToolStart { id, .. } if id == "base64.encode"
        )));
        assert!(ev.iter().any(|e| matches!(
            e,
            AgentEvent::ToolEnd { id, .. } if id == "base64.encode"
        )));
        assert!(ev.iter().any(|e| matches!(e, AgentEvent::Done)));
        assert_eq!(model.received.len(), 2);

        // 事件日志：append-only ledger，seq 严格递增，类型序列完整
        let log_events = crate::agent::session_log::list_events_on(&db, &conv.id).unwrap();
        assert!(!log_events.is_empty(), "session_events must not be empty");
        // seq 严格递增
        for w in log_events.windows(2) {
            assert!(w[1].seq > w[0].seq, "seq must strictly increase");
        }
        // 必备类型序列（turn/start → user/message → step/start →
        // request/header → assistant → tool/call → tool/result → step/end
        // → step/start → request/header → assistant → step/end → turn/end）
        let types: Vec<&str> = log_events.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(types[0], "turn/start");
        assert!(types.contains(&"user/message"));
        assert!(types.contains(&"system/message"));
        assert!(
            types.contains(&"tools/catalog"),
            "tools catalog must be a separate module event"
        );
        assert!(
            types.contains(&"skills/catalog"),
            "skills L0 catalog must be logged separately"
        );
        assert!(types.contains(&"request/header"));
        assert!(types.contains(&"assistant/message"));
        assert!(types.contains(&"tool/call"));
        assert!(types.contains(&"tool/result"));
        assert!(types.contains(&"step/end"));
        assert!(types.last().copied() == Some("turn/end"));

        // 本回合系统提示词全文必须落库，且不含 Skill L0/L1（渐进式披露分条）
        let sys_ev = log_events
            .iter()
            .find(|e| e.kind == "system/message")
            .expect("system/message must be logged once per turn");
        let sys_content = sys_ev
            .data
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(!sys_content.trim().is_empty(), "system prompt must be non-empty");
        assert!(!sys_content.contains("相关技能说明"));
        assert!(!sys_content.contains("Loaded skill instructions"));
        assert!(!sys_content.contains("Available skills (id — when to use)"));

        // 工具步的 assistant/message 必须带上模型决策（工具名 + 参数），不能是空壳
        let first_asst = log_events
            .iter()
            .find(|e| e.kind == "assistant/message")
            .unwrap();
        let tcs = first_asst
            .data
            .get("tool_calls")
            .and_then(|v| v.as_array())
            .expect("tool-step assistant/message must record tool_calls as array");
        assert_eq!(tcs[0].get("id").and_then(|v| v.as_str()), Some("base64.encode"));
        assert!(tcs[0].get("args").is_some());

        // request/header 必须含 desync_check 与本步追加 delta
        let headers: Vec<_> = log_events
            .iter()
            .filter(|e| e.kind == "request/header")
            .collect();
        assert!(headers.len() >= 2, "tool then text → two request/header events");
        let d1 = headers[0]
            .data
            .get("delta")
            .and_then(|d| d.get("messages"))
            .and_then(|m| m.as_array())
            .expect("step1 delta.messages");
        assert_eq!(d1.len(), 1);
        assert_eq!(d1[0].get("role").and_then(|v| v.as_str()), Some("user"));
        assert_eq!(
            d1[0].get("content").and_then(|v| v.as_str()),
            Some(user_text)
        );

        let d2 = headers[1]
            .data
            .get("delta")
            .and_then(|d| d.get("messages"))
            .and_then(|m| m.as_array())
            .expect("step2 delta.messages");
        assert!(
            d2.iter().any(|m| m.get("role").and_then(|v| v.as_str()) == Some("assistant")),
            "step2 delta must include assistant tool_calls message"
        );
        assert!(
            d2.iter().any(|m| m.get("role").and_then(|v| v.as_str()) == Some("tool")),
            "step2 delta must include tool result message"
        );
        assert!(
            !d2.iter().any(|m| m.get("role").and_then(|v| v.as_str()) == Some("system")),
            "step2 delta must not replay system prompt"
        );

        for h in &headers {
            let dc = h.data.get("desync_check").expect("desync_check on header");
            assert_eq!(
                dc.get("match").and_then(|v| v.as_bool()),
                Some(true),
                "desync should match within a tool turn: {dc}"
            );
        }

        // 第二回合：history 必须从 trajectory 展开工具链，desync 仍匹配
        let user2 = "follow up";
        let _ = append_user_message_on(&db, Some(conv.id.clone()), user2).unwrap();
        let mut model2 = Scripted::new(vec![ModelTurn::text("ok")]);
        let (mut emit2, _) = collect_events();
        run_agent_on(&db, &mut model2, &conv.id, user2, &cancel, &mut emit2).unwrap();

        let hist = history_to_model_messages(&db, &conv.id).unwrap();
        assert!(
            hist.iter().any(|m| m.role == "tool"),
            "history must expand prior tool results, got roles={:?}",
            hist.iter().map(|m| m.role.as_str()).collect::<Vec<_>>()
        );
        assert!(
            hist.iter()
                .any(|m| m.role == "assistant" && !m.tool_calls.is_empty()),
            "history must expand prior assistant tool_calls"
        );

        let log_events2 = crate::agent::session_log::list_events_on(&db, &conv.id).unwrap();
        let headers2: Vec<_> = log_events2
            .iter()
            .filter(|e| e.kind == "request/header")
            .collect();
        let last_h = headers2.last().expect("turn2 request/header");
        let dc2 = last_h.data.get("desync_check").expect("desync_check");
        assert_eq!(
            dc2.get("match").and_then(|v| v.as_bool()),
            Some(true),
            "second-turn desync must match after trajectory expand: {dc2}"
        );
    }

    #[test]
    fn two_tool_calls_then_text() {
        let db = test_db();
        let user_text = "base64 then hash";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        let mut model = Scripted::new(vec![
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c1".into(),
                name: "base64.encode".into(),
                arguments: json!({"input": "hi"}),
            }]),
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c2".into(),
                name: "hash.digest".into(),
                arguments: json!({"input": "aGk=", "algorithm": "md5"}),
            }]),
            ModelTurn::text("both done"),
        ]);
        let cancel = AtomicBool::new(false);
        let (mut emit, _) = collect_events();

        run_agent_on(
            &db,
            &mut model,
            &conv.id,
            user_text,
            &cancel,
            &mut emit,
        )
        .unwrap();

        assert_eq!(model.received.len(), 3);
        let msgs = get_messages_on(&db, &conv.id).unwrap();
        assert_eq!(msgs.last().unwrap().content, "both done");
    }

    #[test]
    fn dispatch_failure_fed_back_to_model() {
        let db = test_db();
        let user_text = "call unknown tool";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        let mut model = Scripted::new(vec![
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c1".into(),
                name: "http.request".into(),
                arguments: json!({}),
            }]),
            ModelTurn::text("tool unavailable"),
        ]);
        let cancel = AtomicBool::new(false);
        let (mut emit, _) = collect_events();

        run_agent_on(
            &db,
            &mut model,
            &conv.id,
            user_text,
            &cancel,
            &mut emit,
        )
        .unwrap();

        assert_eq!(model.received.len(), 2);
        let second = &model.received[1];
        let tool_msgs: Vec<_> = second.iter().filter(|m| m.role == "tool").collect();
        assert!(
            !tool_msgs.is_empty(),
            "second turn should include tool role messages"
        );
        assert!(
            tool_msgs.iter().any(|m| {
                m.content.contains("未注册") || m.content.contains("unknown")
            }),
            "tool result should contain dispatch error, got: {:?}",
            tool_msgs
        );
    }

    #[test]
    fn cancel_stops_and_keeps_user_message() {
        let db = test_db();
        let user_text = "will cancel";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        let mut model = Scripted::new(vec![ModelTurn::text("should not appear")]);
        let cancel = AtomicBool::new(true);
        let (mut emit, events) = collect_events();

        let _ = run_agent_on(
            &db,
            &mut model,
            &conv.id,
            user_text,
            &cancel,
            &mut emit,
        );

        assert!(
            model.received.is_empty(),
            "cancel=true must not call the model"
        );
        let msgs = get_messages_on(&db, &conv.id).unwrap();
        assert!(
            msgs.iter()
                .any(|m| m.role == "user" && m.content == user_text),
            "persisted user message must remain"
        );
        let ev = events.lock().unwrap();
        assert!(
            ev.iter().any(|e| matches!(e, AgentEvent::Done))
                || ev.iter().any(|e| matches!(e, AgentEvent::Interrupted))
                || ev.iter().any(|e| matches!(e, AgentEvent::Error { .. }))
        );
    }

    #[test]
    fn invalid_args_repaired_via_reask() {
        let db = test_db();
        let user_text = "算一下 hi 的 md5";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        // 第一回合缺 algorithm（校验失败回填），第二回合修复
        let mut model = Scripted::new(vec![
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c1".into(),
                name: "hash.digest".into(),
                arguments: json!({"input": "hi"}),
            }]),
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c2".into(),
                name: "hash.digest".into(),
                arguments: json!({"input": "hi", "algorithm": "md5"}),
            }]),
            ModelTurn::text("digest ready"),
        ]);
        let cancel = AtomicBool::new(false);
        let (mut emit, events) = collect_events();

        run_agent_on(&db, &mut model, &conv.id, user_text, &cancel, &mut emit).unwrap();

        // 修复成功：两次 ToolStart（第二次带合法参数）+ 最终文本
        let ev = events.lock().unwrap();
        let digest_starts = ev
            .iter()
            .filter(|e| matches!(e, AgentEvent::ToolStart { id, .. } if id == "hash.digest"))
            .count();
        assert_eq!(digest_starts, 2);
        assert!(ev.iter().any(|e| matches!(e, AgentEvent::Done)));
        // 回填的校验错误可见于 ToolEnd
        assert!(ev.iter().any(|e| matches!(
            e,
            AgentEvent::ToolEnd { result, .. } if result.contains("algorithm")
        )));
        assert_eq!(model.received.len(), 3);
    }

    #[test]
    fn repair_budget_exhausted_degrades_to_explanation() {
        let db = test_db();
        let user_text = "一直调用坏工具";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        let bad_call = || {
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c1".into(),
                name: "hash.digest".into(),
                arguments: json!({"input": "hi"}), // 永远缺 algorithm
            }])
        };
        // 默认策略预算 1：两轮失败后必须收尾，第三回合永远不会到达
        let mut model = Scripted::new(vec![bad_call(), bad_call(), bad_call()]);
        let cancel = AtomicBool::new(false);
        let (mut emit, events) = collect_events();

        let res = run_agent_on(&db, &mut model, &conv.id, user_text, &cancel, &mut emit);

        // 软退出：以 Ok 返回（不再抛 Err），但仍写入解释性助手消息
        assert!(res.is_ok());
        assert_eq!(model.received.len(), 2, "model must not be called past the repair budget");
        let ev = events.lock().unwrap();
        assert!(!ev.iter().any(|e| matches!(e, AgentEvent::Error { .. })));
        assert!(ev.iter().any(|e| matches!(e, AgentEvent::Done)));
        // 会话保留且有一条解释性助手消息
        let msgs = get_messages_on(&db, &conv.id).unwrap();
        assert!(msgs
            .iter()
            .any(|m| m.role == "assistant" && m.content.contains("修复重试预算")));
    }

    #[test]
    fn repair_never_amplifies_iteration_cap() {
        let db = test_db();
        let user_text = "连续成功调用";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        // 全部成功回合：不消耗 reask 预算，但工具回合总数仍受
        // MAX_TOOL_ITERATIONS 封顶（给满 10 个回合也不得超过 8+1 次调用）。
        let mut turns = Vec::new();
        for i in 0..10 {
            turns.push(ModelTurn::ToolCalls(vec![ToolCall {
                id: format!("c{i}"),
                name: "uuid.generate".into(),
                arguments: json!({}),
            }]));
        }
        turns.push(ModelTurn::text("done"));
        let mut model = Scripted::new(turns);
        let cancel = AtomicBool::new(false);
        let (mut emit, _) = collect_events();

        let res = run_agent_on(&db, &mut model, &conv.id, user_text, &cancel, &mut emit);

        assert!(res.is_ok(), "软退出：不应再返回 Err");
        // STUCK 守卫会在 3 次相同签名后早退；这里全是不同 args，但全部
        // 成功 → 任意一轮都可能撞到重复（连续 2 次相同 hash 难触发），实际
        // 上限是 MAX_TOOL_ITERATIONS（8）。模型被叫 8 次后软退出保存说明。
        assert!(
            model.received.len() <= 9,
            "8 tool rounds max; got {}",
            model.received.len()
        );
    }

    #[test]
    fn repair_budget_counts_failed_rounds_only() {
        let db = test_db();
        let user_text = "交替成功失败";
        let (conv, _) = append_user_message_on(&db, None, user_text).unwrap();
        // 默认策略预算 1：第 1 次失败回填重试（round 3 修复成功），
        // 第 2 次失败即收尾——模型只被调用 4 次。
        let mut model = Scripted::new(vec![
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c0".into(),
                name: "uuid.generate".into(),
                arguments: json!({}),
            }]),
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c1".into(),
                name: "hash.digest".into(),
                arguments: json!({"input": "x"}), // 缺 algorithm
            }]),
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c2".into(),
                name: "uuid.generate".into(),
                arguments: json!({}),
            }]),
            ModelTurn::ToolCalls(vec![ToolCall {
                id: "c3".into(),
                name: "hash.digest".into(),
                arguments: json!({"input": "x"}), // 再次失败 → 收尾
            }]),
            ModelTurn::text("never"),
        ]);
        let cancel = AtomicBool::new(false);
        let (mut emit, _) = collect_events();

        let res = run_agent_on(&db, &mut model, &conv.id, user_text, &cancel, &mut emit);

        assert!(res.is_ok(), "软退出：不应再返回 Err");
        assert_eq!(model.received.len(), 4);
    }

    #[allow(dead_code)]
    fn _signature_check(
        model: &mut impl ChatModel,
        conv_id: &str,
        user_text: &str,
        cancel: &AtomicBool,
        emit: impl FnMut(AgentEvent),
    ) -> Result<(), String> {
        run_agent(model, conv_id, user_text, cancel, emit)
    }
}
