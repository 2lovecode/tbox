//! Tauri commands that drive one agent chat turn and stream events to the UI.

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::agent::llm::{ChatModel, ModelMessage, ModelTurn, AgentError, StreamMode};
use crate::agent::r#loop::{run_agent, AgentEvent};
use crate::agent::run_registry::RunRegistry;

pub const AGENT_EVENT: &str = "agent-event";
pub const AGENT_RUN_STATUS: &str = "agent-run-status";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunStatusPayload {
    pub conversation_id: String,
    pub status: String, // "running" | "idle"
}

fn emit_run_status(app: &AppHandle, conversation_id: &str, status: &str) {
    let _ = app.emit(
        AGENT_RUN_STATUS,
        AgentRunStatusPayload {
            conversation_id: conversation_id.to_string(),
            status: status.to_string(),
        },
    );
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similar_key: Option<String>,
}

fn empty_payload(conversation_id: String, event_type: &str) -> AgentEventPayload {
    AgentEventPayload {
        conversation_id,
        event_type: event_type.into(),
        text: None,
        id: None,
        args: None,
        result: None,
        message: None,
        mode: None,
        budget: None,
        count: None,
        request_id: None,
        tool_id: None,
        command: None,
        cwd: None,
        similar_key: None,
    }
}

fn wire_event(conversation_id: String, ev: AgentEvent) -> AgentEventPayload {
    match ev {
        AgentEvent::StreamMeta { mode } => AgentEventPayload {
            mode: Some(match mode {
                StreamMode::Live => "live".into(),
                StreamMode::Fallback => "fallback".into(),
            }),
            ..empty_payload(conversation_id, "stream_meta")
        },
        AgentEvent::Reasoning { text } => AgentEventPayload {
            text: Some(text),
            ..empty_payload(conversation_id, "reasoning")
        },
        AgentEvent::Token { text } => AgentEventPayload {
            text: Some(text),
            ..empty_payload(conversation_id, "token")
        },
        AgentEvent::ToolStart { id, args } => AgentEventPayload {
            id: Some(id),
            args: Some(args),
            ..empty_payload(conversation_id, "tool_start")
        },
        AgentEvent::ToolEnd { id, result } => AgentEventPayload {
            id: Some(id),
            result: Some(result),
            ..empty_payload(conversation_id, "tool_end")
        },
        AgentEvent::ToolApprovalRequired {
            request_id,
            tool_id,
            command,
            cwd,
            similar_key,
        } => AgentEventPayload {
            request_id: Some(request_id),
            tool_id: Some(tool_id),
            command: Some(command),
            cwd: Some(cwd),
            similar_key: Some(similar_key),
            ..empty_payload(conversation_id, "tool_approval_required")
        },
        AgentEvent::ContextBudget { budget } => AgentEventPayload {
            budget: serde_json::to_value(&budget).ok(),
            ..empty_payload(conversation_id, "context_budget")
        },
        AgentEvent::Compress { message, count } => AgentEventPayload {
            message: Some(message),
            count: Some(count),
            ..empty_payload(conversation_id, "compress")
        },
        AgentEvent::Error { message } => AgentEventPayload {
            message: Some(message),
            ..empty_payload(conversation_id, "error")
        },
        AgentEvent::Interrupted => empty_payload(conversation_id, "interrupted"),
        AgentEvent::Done => empty_payload(conversation_id, "done"),
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
        Ok(ModelTurn::text(format!(
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
    registry: State<'_, Arc<RunRegistry>>,
    conversationId: String,
    content: String,
) -> Result<(), String> {
    llm_is_ready()?;

    let registry = Arc::clone(&registry);
    let cancel = registry.try_begin(&conversationId)?;
    emit_run_status(&app, &conversationId, "running");

    let conv_id = conversationId.clone();
    let user_text = content;
    let use_mock = mock_agent_enabled();
    let app_for_worker = app.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let cancel_flag = cancel;
        let emit = |ev: AgentEvent| {
            let payload = wire_event(conv_id.clone(), ev);
            let _ = app_for_worker.emit(AGENT_EVENT, payload);
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
                ),
                Err(AgentError::LlmUnavailable) => {
                    let payload = wire_event(
                        conv_id.clone(),
                        AgentEvent::Error {
                            message: LLM_UNAVAILABLE.into(),
                        },
                    );
                    let _ = app_for_worker.emit(AGENT_EVENT, payload);
                    Err(LLM_UNAVAILABLE.into())
                }
                Err(AgentError::Other(e)) => Err(e),
            }
        };

        if let Err(e) = result {
            let payload = wire_event(conv_id.clone(), AgentEvent::Error { message: e });
            let _ = app_for_worker.emit(AGENT_EVENT, &payload);
        }

        registry.finish(&conv_id);
        crate::agent::tool_approval::clear_session(&conv_id);
        emit_run_status(&app_for_worker, &conv_id, "idle");
    });

    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn cancel_chat_turn(
    registry: State<'_, Arc<RunRegistry>>,
    conversationId: String,
) -> Result<(), String> {
    registry.cancel(&conversationId);
    crate::agent::tool_approval::clear_session(&conversationId);
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn resolve_tool_approval(requestId: String, decision: String) -> Result<(), String> {
    use crate::agent::tool_approval::ApprovalDecision;
    let d = match decision.as_str() {
        "deny" => ApprovalDecision::Deny,
        "allow" => ApprovalDecision::Allow,
        "allow_similar" => ApprovalDecision::AllowSimilar,
        other => return Err(format!("未知决策: {other}")),
    };
    crate::agent::tool_approval::resolve(&requestId, d)
}

#[tauri::command]
pub fn get_shell_prefs() -> crate::agent::os_shell::ShellPrefs {
    crate::agent::os_shell::load_shell_prefs()
}

#[tauri::command]
pub fn save_shell_prefs(
    prefs: crate::agent::os_shell::ShellPrefs,
) -> Result<crate::agent::os_shell::ShellPrefs, String> {
    crate::agent::os_shell::save_shell_prefs(prefs)
}

#[tauri::command]
pub fn probe_shell_commands() -> Vec<crate::agent::os_shell::CommandAvailability> {
    crate::agent::os_shell::probe_commands()
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
        "accelBackend": crate::agent::embedded_engine::accel_backend_label(),
    })
}

#[tauri::command]
pub fn list_skills() -> Vec<crate::agent::skills::SkillInfo> {
    crate::agent::skills::list_skills()
}

#[tauri::command]
pub fn set_skill_enabled(skill_id: String, enabled: bool) -> Result<crate::agent::skills::SkillInfo, String> {
    crate::agent::skills::set_skill_enabled(&skill_id, enabled)
}

#[tauri::command]
pub fn create_skill(
    name: String,
    description: String,
    keywords: Vec<String>,
    tool_ids: Vec<String>,
    body: String,
) -> Result<crate::agent::skills::SkillInfo, String> {
    crate::agent::skills::create_skill(name, description, keywords, tool_ids, body)
}

#[tauri::command]
pub fn update_skill(
    id: String,
    name: String,
    description: String,
    keywords: Vec<String>,
    tool_ids: Vec<String>,
    body: String,
) -> Result<crate::agent::skills::SkillInfo, String> {
    crate::agent::skills::update_skill(id, name, description, keywords, tool_ids, body)
}

#[tauri::command]
pub fn delete_skill(id: String) -> Result<(), String> {
    crate::agent::skills::delete_skill(id)
}

#[tauri::command]
pub fn import_skill(raw: String, name: Option<String>) -> Result<crate::agent::skills::SkillInfo, String> {
    crate::agent::skills::import_skill(raw, name)
}

#[tauri::command]
pub fn list_skill_versions(id: String) -> Result<Vec<crate::agent::skills::SkillVersionInfo>, String> {
    crate::agent::skills::list_skill_versions(&id)
}

#[tauri::command]
pub fn restore_skill_version(
    id: String,
    seq: u64,
) -> Result<crate::agent::skills::SkillInfo, String> {
    crate::agent::skills::restore_skill_version(&id, seq)
}

#[tauri::command]
pub fn restore_skill_default(id: String) -> Result<crate::agent::skills::SkillInfo, String> {
    crate::agent::skills::restore_skill_default(&id)
}

#[tauri::command]
pub fn get_llama_engine_log_settings() -> crate::agent::llama_log::LlamaEngineLogSettingsView {
    crate::agent::llama_log::settings_view()
}

#[tauri::command]
pub fn save_llama_engine_log_settings(
    settings: crate::agent::llama_log::LlamaEngineLogSettings,
) -> Result<crate::agent::llama_log::LlamaEngineLogSettingsView, String> {
    crate::agent::llama_log::save_settings(settings)?;
    Ok(crate::agent::llama_log::settings_view())
}
