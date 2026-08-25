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

    loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = append_assistant_message_on(conn, conv_id, "", None);
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
            ModelTurn::Text(text) => {
                emit(AgentEvent::Token {
                    text: text.clone(),
                });
                append_assistant_message_on(conn, conv_id, &text, None)?;
                emit(AgentEvent::Done);
                return Ok(());
            }
            ModelTurn::ToolCalls(calls) => {
                if tool_iterations >= MAX_TOOL_ITERATIONS {
                    let msg = format!(
                        "exceeded max tool iterations ({MAX_TOOL_ITERATIONS})"
                    );
                    emit(AgentEvent::Error {
                        message: msg.clone(),
                    });
                    emit(AgentEvent::Done);
                    return Err(msg);
                }
                tool_iterations += 1;

                let mut any_success = false;
                for call in calls {
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
                    msgs.push(ModelMessage::tool(result));
                }

                if !any_success {
                    if repair_rounds >= repair_budget {
                        // 修复预算耗尽：按既有语义回填最终错误并生成文字
                        // 说明（spec: Repair-and-reask Loop），应用不崩溃。
                        let explanation = format!(
                            "工具调用连续失败，修复重试预算（{repair_budget} 次）已耗尽。请检查请求内容或换一种表述后重试。"
                        );
                        emit(AgentEvent::Error {
                            message: explanation.clone(),
                        });
                        append_assistant_message_on(conn, conv_id, &explanation, None)?;
                        emit(AgentEvent::Token { text: explanation });
                        emit(AgentEvent::Done);
                        return Err("repair budget exhausted".to_string());
                    }
                    repair_rounds += 1;
                }
            }
        }
    }
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
        let mut model = Scripted::new(vec![ModelTurn::Text("hi there".into())]);
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
            ModelTurn::Text("encoded".into()),
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
            ModelTurn::Text("both done".into()),
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
            ModelTurn::Text("tool unavailable".into()),
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
        let mut model = Scripted::new(vec![ModelTurn::Text("should not appear".into())]);
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
            ModelTurn::Text("digest ready".into()),
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

        assert!(res.is_err());
        assert_eq!(model.received.len(), 2, "model must not be called past the repair budget");
        let ev = events.lock().unwrap();
        assert!(ev.iter().any(|e| matches!(e, AgentEvent::Error { .. })));
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
        turns.push(ModelTurn::Text("done".into()));
        let mut model = Scripted::new(turns);
        let cancel = AtomicBool::new(false);
        let (mut emit, _) = collect_events();

        let res = run_agent_on(&db, &mut model, &conv.id, user_text, &cancel, &mut emit);

        assert!(res.is_err());
        assert!(
            model.received.len() <= 9,
            "8 tool rounds max + the final errored round; got {}",
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
            ModelTurn::Text("never".into()),
        ]);
        let cancel = AtomicBool::new(false);
        let (mut emit, _) = collect_events();

        let res = run_agent_on(&db, &mut model, &conv.id, user_text, &cancel, &mut emit);

        assert!(res.is_err());
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
