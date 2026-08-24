//! Agent tool loop: retrieve skills -> model -> dispatch -> feed results; max 8 tool iterations.

use super::llm::{ChatModel, ModelMessage, ModelTurn};
use super::registry;
use super::skills::retrieve_skills;
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

fn build_system_prompt(user_text: &str) -> String {
    let skills = retrieve_skills(user_text, 3);
    let mut prompt = String::from(
        "You are the TBox local agent. Prefer registered pure-compute tools when helpful.\n",
    );
    if !skills.is_empty() {
        prompt.push_str("\nRelevant skills:\n");
        for sk in &skills {
            prompt.push_str(&format!("### {}\n{}\n\n", sk.tool_id, sk.body));
        }
    }
    prompt
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

    let mut msgs = Vec::new();
    msgs.push(ModelMessage::system(build_system_prompt(user_text)));
    msgs.extend(history_to_model_messages(conn, conv_id)?);

    let mut tool_iterations = 0usize;

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

                for call in calls {
                    let tool_id = call.name.clone();
                    emit(AgentEvent::ToolStart {
                        id: tool_id.clone(),
                        args: call.arguments.clone(),
                    });
                    let result = match registry::dispatch(&call.name, &call.arguments) {
                        Ok(s) => s,
                        Err(e) => e,
                    };
                    emit(AgentEvent::ToolEnd {
                        id: tool_id,
                        result: result.clone(),
                    });
                    msgs.push(ModelMessage::tool(result));
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
