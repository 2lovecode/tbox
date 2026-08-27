//! Agent tool loop: harness strategy → model → dispatch → feed results;
//! max 8 tool iterations with a bounded repair-and-reask budget.

use super::harness;
use super::llm::{ChatModel, ModelMessage, ModelTurn};
use super::registry;
use crate::commands::conversation::{append_assistant_message_on, get_messages_on};
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
    Token { text: String },
    Reasoning { text: String },
    ToolStart { id: String, args: Value },
    ToolEnd { id: String, result: String },
    Error { message: String },
    Interrupted,
    Done,
}

fn history_to_model_messages(conn: &Connection, conv_id: &str) -> Result<Vec<ModelMessage>, String> {
    let history = get_messages_on(conn, conv_id)?;
    Ok(history
        .into_iter()
        .map(|m| ModelMessage {
            role: m.role,
            content: m.content,
            ..Default::default()
        })
        .collect())
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

    let mut msgs = Vec::new();
    msgs.push(ModelMessage::system(strategy.build_system_prompt(user_text)));
    msgs.extend(history_to_model_messages(conn, conv_id)?);

    let mut tool_iterations = 0usize;
    let mut repair_rounds = 0usize;
    let mut last_call_signature: Option<String> = None;
    let mut stuck_rounds: usize = 0;
    /// 连续相同签名调用达到此上限即视为卡住，提前结束回合。
    const STUCK_ROUNDS_LIMIT: usize = 2;

    loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = append_assistant_message_on(conn, conv_id, "", None, None);
            emit(AgentEvent::Interrupted);
            return Ok(());
        }

        let turn = match model.complete(&msgs) {
            Ok(t) => t,
            Err(e) => {
                emit(AgentEvent::Error {
                    message: e.clone(),
                });
                return Err(e);
            }
        };

        match turn {
            ModelTurn::Text { text, reasoning } => {
                // 内联 <think> 标签（Qwen/DeepSeek 风格）剥离为 reasoning，
                // 与后端已分离的 reasoning（reasoning_content / thinking 块）合并。
                let (inline_reasoning, body) = super::llm::split_think_tags(&text);
                let reasoning = match (
                    reasoning.filter(|r| !r.trim().is_empty()),
                    inline_reasoning,
                ) {
                    (Some(a), Some(b)) => Some(format!("{a}\n{b}")),
                    (r, i) => r.or(i),
                };

                // 分块流式发出：思考先于正文，均按块 emit（spec: Chunked
                // Streaming Emission）；持久化仍写完整文本。
                const CHUNK_CHARS: usize = 24;
                if let Some(r) = reasoning.as_ref() {
                    for chunk in super::llm::chunk_text(r, CHUNK_CHARS) {
                        emit(AgentEvent::Reasoning { text: chunk });
                    }
                }
                for chunk in super::llm::chunk_text(&body, CHUNK_CHARS) {
                    emit(AgentEvent::Token { text: chunk });
                }
                append_assistant_message_on(
                    conn,
                    conv_id,
                    &body,
                    None,
                    reasoning.as_deref(),
                )?;
                emit(AgentEvent::Done);
                return Ok(());
            }
            ModelTurn::ToolCalls(calls) => {
                // 重复调用守卫：连续 N 轮发起相同（按 name+args 签名）的调用
                // 是小模型「卡住」的明确信号，提前结束比硬撞 8 轮上限更友好。
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
                    return finish_with_explanation(
                        conn,
                        conv_id,
                        &calls,
                        "模型重复发起相同的工具调用，疑似陷入循环。已基于最近一次结果给出说明。",
                        &mut emit,
                    );
                }

                if tool_iterations >= MAX_TOOL_ITERATIONS {
                    return finish_with_explanation(
                        conn,
                        conv_id,
                        &calls,
                        &format!(
                            "已连续发起 {MAX_TOOL_ITERATIONS} 轮工具调用但模型未给出总结回复。基于最近一次结果给出说明。"
                        ),
                        &mut emit,
                    );
                }
                tool_iterations += 1;

                // 协议配对：先回放 assistant 的 tool_calls 消息，再逐个
                // 回填 role=tool 结果（缺失配对会让模型重复发起调用）。
                msgs.push(ModelMessage::assistant_tool_calls(calls.clone()));

                let mut any_success = false;
                let mut last_results: Vec<(String, String)> = Vec::new();
                for call in &calls {
                    let tool_id = call.name.clone();
                    emit(AgentEvent::ToolStart {
                        id: tool_id.clone(),
                        args: call.arguments.clone(),
                    });
                    // dispatch 前校验：未注册给出相近工具名建议；参数不合
                    // schema 不触发底层 command，错误回填进 reask 循环。
                    let result = match harness::parse::validate_call(
                        &call.name,
                        &call.arguments,
                    )
                    .and_then(|()| registry::dispatch(&call.name, &call.arguments))
                    {
                        Ok(s) => {
                            any_success = true;
                            s
                        }
                        Err(e) => e,
                    };
                    emit(AgentEvent::ToolEnd {
                        id: tool_id,
                        result: result.clone(),
                    });
                    last_results.push((call.name.clone(), result.clone()));
                    msgs.push(ModelMessage::tool_result(
                        call.id.clone(),
                        call.name.clone(),
                        result,
                    ));
                }

                if !any_success {
                    if repair_rounds >= repair_budget {
                        // 修复预算耗尽：按既有语义回填最终错误并生成文字
                        // 说明（spec: Repair-and-reask Loop），应用不崩溃。
                        return finish_with_explanation(
                            conn,
                            conv_id,
                            &calls,
                            &format!(
                                "工具调用连续失败，修复重试预算（{repair_budget} 次）已耗尽。请检查请求内容或换一种表述后重试。"
                            ),
                            &mut emit,
                        );
                    }
                    repair_rounds += 1;
                }
            }
        }
    }
}

/// 工具循环在以下情况被调用来「软退出」：超过最大轮次、修复预算耗尽、
/// 或小模型陷入重复调用循环。把工具结果摘要成一段说明文本，作为
/// assistant 最终回复持久化（不再只发 Error 事件被前端当 toast 处理）。
fn finish_with_explanation(
    conn: &Connection,
    conv_id: &str,
    calls: &[crate::agent::llm::ToolCall],
    prefix: &str,
    emit: &mut dyn FnMut(AgentEvent),
) -> Result<(), String> {
    let mut body = prefix.to_string();
    if !calls.is_empty() {
        body.push_str("\n\n最近一次工具调用：\n");
        for c in calls {
            body.push_str(&format!(
                "- `{}` 参数 `{}`\n",
                c.name,
                c.arguments
            ));
        }
    }
    append_assistant_message_on(conn, conv_id, &body, None, None)?;
    emit(AgentEvent::Token { text: body.clone() });
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
    }

    /// 回归：assistant tool_calls 消息必须与 role=tool 结果配对回填，
    /// 否则模型看不到自己的调用而重复发起（exceeded max tool iterations）。
    #[test]
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
/// 不再硬撞 MAX_TOOL_ITERATIONS；结果以 assistant 文本回复持久化。
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
    // 最后一条应是 finish_with_explanation 写入的 assistant 文本
    let last = msgs.last().expect("assistant message persisted");
    assert_eq!(last.role, "assistant");
    assert!(last.content.contains("重复发起"));
    assert!(last.content.contains("base64.encode"));
    // 模型被叫了 3 次（基线 + 第 1 次重复 + 第 2 次重复时软退出）
    assert_eq!(model.received.len(), 3);
    let evs = events.lock().unwrap();
    // 循环以 Token + Done 收尾（而非 Error）
    assert!(matches!(evs.last(), Some(AgentEvent::Done)));
    assert!(!evs.iter().any(|e| matches!(e, AgentEvent::Error { .. })));
}

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
