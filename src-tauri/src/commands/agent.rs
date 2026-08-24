//! Tauri commands that drive one agent chat turn and stream events to the UI.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::agent::llm::{ChatModel, ModelMessage, ModelTurn, AgentError};
use crate::agent::r#loop::{run_agent, AgentEvent};

pub const AGENT_EVENT: &str = "agent-event";

/// Shared cancel flag for the in-flight agent turn.
pub struct AgentCancel(pub Arc<AtomicBool>);

impl Default for AgentCancel {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEventPayload {
    pub conversation_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

fn wire_event(conversation_id: String, ev: AgentEvent) -> AgentEventPayload {
    match ev {
        AgentEvent::Token { text } => AgentEventPayload {
            conversation_id,
            event_type: "token".into(),
            text: Some(text),
            id: None,
            args: None,
            result: None,
            message: None,
        },
        AgentEvent::ToolStart { id, args } => AgentEventPayload {
            conversation_id,
            event_type: "tool_start".into(),
            text: None,
            id: Some(id),
            args: Some(args),
            result: None,
            message: None,
        },
        AgentEvent::ToolEnd { id, result } => AgentEventPayload {
            conversation_id,
            event_type: "tool_end".into(),
            text: None,
            id: Some(id),
            args: None,
            result: Some(result),
            message: None,
        },
        AgentEvent::Error { message } => AgentEventPayload {
            conversation_id,
            event_type: "error".into(),
            text: None,
            id: None,
            args: None,
            result: None,
            message: Some(message),
        },
        AgentEvent::Interrupted => AgentEventPayload {
            conversation_id,
            event_type: "interrupted".into(),
            text: None,
            id: None,
            args: None,
            result: None,
            message: None,
        },
        AgentEvent::Done => AgentEventPayload {
            conversation_id,
            event_type: "done".into(),
            text: None,
            id: None,
            args: None,
            result: None,
            message: None,
        },
    }
}

/// Error code when no usable LLM is configured.
pub const LLM_UNAVAILABLE: &str = "llm_unavailable";

/// Whether the current provider can run an agent turn.
pub fn llm_is_ready() -> Result<(), String> {
    if mock_agent_enabled() {
        return Ok(());
    }
    match crate::agent::llm::build_model_from_disk() {
        Ok(_) => Ok(()),
        Err(AgentError::LlmUnavailable) => Err(LLM_UNAVAILABLE.to_string()),
        Err(AgentError::Other(e)) => Err(e),
    }
}

fn mock_agent_enabled() -> bool {
    std::env::var("TBOX_AGENT_MOCK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Development / UI mock model: optional one tool call, then a short reply.
struct DevMockModel {
    user_text: String,
    emitted_tool: bool,
}

impl ChatModel for DevMockModel {
    fn complete(&mut self, _msgs: &[ModelMessage]) -> Result<ModelTurn, String> {
        if !self.emitted_tool
            && (self.user_text.contains("Base64")
                || self.user_text.to_lowercase().contains("base64"))
        {
            self.emitted_tool = true;
            let input = self
                .user_text
                .split_whitespace()
                .last()
                .unwrap_or("hello")
                .to_string();
            return Ok(ModelTurn::ToolCalls(vec![crate::agent::llm::ToolCall {
                id: "call_mock_1".into(),
                name: "base64.encode".into(),
                arguments: serde_json::json!({ "input": input }),
            }]));
        }
        Ok(ModelTurn::Text(format!(
            "（开发模式）已处理你的请求。配置云端 LLM 或下载本地模型后，将使用真实推理。\n原文：{}",
            self.user_text
        )))
    }
}

#[tauri::command]
pub fn check_llm_ready() -> Result<bool, String> {
    match llm_is_ready() {
        Ok(()) => Ok(true),
        Err(code) if code == LLM_UNAVAILABLE => Ok(false),
        Err(e) => Err(e),
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn send_chat_turn(
    app: AppHandle,
    cancel_state: State<'_, AgentCancel>,
    conversationId: String,
    content: String,
) -> Result<(), String> {
    llm_is_ready()?;

    cancel_state.0.store(false, Ordering::SeqCst);
    let cancel = Arc::clone(&cancel_state.0);
    let conv_id = conversationId.clone();
    let user_text = content;
    let use_mock = mock_agent_enabled();

    tauri::async_runtime::spawn_blocking(move || {
        let cancel_flag = cancel;
        let emit = |ev: AgentEvent| {
            let payload = wire_event(conv_id.clone(), ev);
            let _ = app.emit(AGENT_EVENT, payload);
        };

        let result = if use_mock {
            let mut model = DevMockModel {
                user_text: user_text.clone(),
                emitted_tool: false,
            };
            run_agent(
                &mut model,
                &conv_id,
                &user_text,
                cancel_flag.as_ref(),
                emit,
            )
        } else {
            match crate::agent::llm::build_model_from_disk() {
                Ok(mut model) => run_agent(
                    model.as_mut(),
                    &conv_id,
                    &user_text,
                    cancel_flag.as_ref(),
                    emit,
                ),                Err(AgentError::LlmUnavailable) => {
                    let payload = wire_event(
                        conv_id.clone(),
                        AgentEvent::Error {
                            message: LLM_UNAVAILABLE.into(),
                        },
                    );
                    let _ = app.emit(AGENT_EVENT, payload);
                    Err(LLM_UNAVAILABLE.into())
                }
                Err(AgentError::Other(e)) => Err(e),
            }
        };

        if let Err(e) = result {
            let payload = wire_event(conv_id, AgentEvent::Error { message: e });
            let _ = app.emit(AGENT_EVENT, &payload);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_chat_turn(cancel_state: State<'_, AgentCancel>) -> Result<(), String> {
    cancel_state.0.store(true, Ordering::SeqCst);
    Ok(())
}

/// Engine load status + which local backend is currently effective, for the
/// settings page (`engine:status` events carry the same data on change).
#[tauri::command]
pub fn get_engine_status() -> serde_json::Value {
    use crate::agent::llm::effective_local_backend;
    let cfg = crate::commands::llm::get_llm_config();
    let backend = if cfg.is_local() {
        match effective_local_backend() {
            Some(crate::agent::llm::LocalBackend::Embedded) => "embedded",
            Some(crate::agent::llm::LocalBackend::Ollama) => "ollama",
            None => "unavailable",
        }
    } else {
        "cloud"
    };
    serde_json::json!({
        "engine": crate::agent::embedded_engine::engine().status(),
        "effectiveBackend": backend,
    })
}
