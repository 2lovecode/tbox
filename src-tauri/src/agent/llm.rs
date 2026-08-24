//! ChatModel abstraction, backend resolution, and OpenAI-compatible client.

use serde_json::Value;
use std::path::PathBuf;

use crate::commands::llm::{get_llm_config, LlmConfig, LlmProtocol};
use crate::commands::model_catalog;

/// One message sent to the model (system / user / assistant / tool).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelMessage {
    pub role: String,
    pub content: String,
}

impl ModelMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.into(),
        }
    }
}

/// One tool call requested by the model.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Single model turn: plain text or tool calls.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelTurn {
    Text(String),
    ToolCalls(Vec<ToolCall>),
}

/// Swappable chat model backend (real LLM or Scripted mock).
pub trait ChatModel {
    fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String>;
}

#[derive(Debug)]
pub enum AgentError {
    LlmUnavailable,
    Other(String),
}

/// Resolved backend for an agent turn.
#[derive(Debug)]
pub enum ReadyLlm {
    OpenAiCompat {
        base_url: String,
        model: String,
        api_key: Option<String>,
    },
    AnthropicMessages {
        base_url: String,
        model: String,
        api_key: String,
    },
}

/// Resolve which backend to use. Local without an installed model MUST NOT
/// fall back to cloud silently.
pub fn resolve_backend(
    cfg: &LlmConfig,
    local_model: Option<PathBuf>,
) -> Result<ReadyLlm, AgentError> {
    if cfg.requires_oauth_preset() {
        return Err(AgentError::LlmUnavailable);
    }

    let protocol = cfg.resolved_protocol();

    if cfg.is_local() {
        if local_model.is_none() {
            return Err(AgentError::LlmUnavailable);
        }
        return Ok(ReadyLlm::OpenAiCompat {
            base_url: "http://127.0.0.1:11435/v1".into(),
            model: local_model
                .as_ref()
                .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "local".into()),
            api_key: None,
        });
    }

    if cfg.is_ollama() {
        if cfg.model.trim().is_empty() {
            return Err(AgentError::LlmUnavailable);
        }
        let base = if cfg.base_url.trim().is_empty() {
            "http://127.0.0.1:11434/v1".into()
        } else {
            let b = cfg.base_url.trim_end_matches('/');
            if b.ends_with("/v1") {
                b.to_string()
            } else {
                format!("{b}/v1")
            }
        };
        return Ok(ReadyLlm::OpenAiCompat {
            base_url: base,
            model: cfg.model.clone(),
            api_key: None,
        });
    }

    if cfg.base_url.trim().is_empty() || cfg.model.trim().is_empty() {
        return Err(AgentError::LlmUnavailable);
    }

    match protocol {
        LlmProtocol::AnthropicMessages => {
            if !cfg.has_api_key {
                return Err(AgentError::LlmUnavailable);
            }
            let api_key = crate::commands::llm::read_api_key_for_agent()
                .ok()
                .flatten()
                .ok_or(AgentError::LlmUnavailable)?;
            Ok(ReadyLlm::AnthropicMessages {
                base_url: cfg.base_url.trim_end_matches('/').to_string(),
                model: cfg.model.clone(),
                api_key,
            })
        }
        LlmProtocol::OpenaiChat | LlmProtocol::OpenaiResponses | LlmProtocol::GeminiNative
        | LlmProtocol::OllamaNative => {
            if !cfg.has_api_key && !cfg.is_ollama() {
                return Err(AgentError::LlmUnavailable);
            }
            let api_key = crate::commands::llm::read_api_key_for_agent()
                .ok()
                .flatten();
            let mut base = cfg.base_url.trim_end_matches('/').to_string();
            if matches!(protocol, LlmProtocol::OpenaiResponses) && !base.ends_with("/v1") {
                base = format!("{base}/v1");
            }
            Ok(ReadyLlm::OpenAiCompat {
                base_url: base,
                model: cfg.model.clone(),
                api_key,
            })
        }
    }
}

pub fn resolve_from_disk() -> Result<ReadyLlm, AgentError> {
    let cfg = get_llm_config();
    let local = model_catalog::enabled_model_path();
    resolve_backend(&cfg, local)
}

/// OpenAI-compatible chat completions client (non-streaming for tool loops).
pub struct OpenAiCompatModel {
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub tools_json: Value,
}

impl OpenAiCompatModel {
    pub fn new(base_url: String, model: String, api_key: Option<String>) -> Self {
        let tools_json = crate::agent::registry::tools_as_openai_json();
        Self {
            base_url,
            model,
            api_key,
            tools_json,
        }
    }
}

impl ChatModel for OpenAiCompatModel {
    fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String> {
        let url = format!(
            "{}/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let messages: Vec<Value> = msgs
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "tools": self.tools_json,
            "tool_choice": "auto",
        });

        let client = reqwest::blocking::Client::new();
        let mut req = client.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let resp = req.send().map_err(|e| format!("LLM 请求失败: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(format!("LLM HTTP {status}: {text}"));
        }
        let json: Value = resp
            .json()
            .map_err(|e| format!("解析 LLM 响应失败: {e}"))?;
        parse_openai_turn(&json)
    }
}

fn parse_openai_turn(json: &Value) -> Result<ModelTurn, String> {
    let choice = json
        .pointer("/choices/0/message")
        .ok_or_else(|| "LLM 响应缺少 choices".to_string())?;
    if let Some(calls) = choice.get("tool_calls").and_then(|v| v.as_array()) {
        if !calls.is_empty() {
            let mut out = Vec::new();
            for c in calls {
                let id = c
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("call")
                    .to_string();
                let name = c
                    .pointer("/function/name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let args_str = c
                    .pointer("/function/arguments")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                let arguments: Value =
                    serde_json::from_str(args_str).unwrap_or_else(|_| serde_json::json!({}));
                out.push(ToolCall {
                    id,
                    name,
                    arguments,
                });
            }
            return Ok(ModelTurn::ToolCalls(out));
        }
    }
    let text = choice
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(ModelTurn::Text(text))
}

/// Anthropic Messages API client (tool use).
pub struct AnthropicMessagesModel {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub tools_json: Value,
}

impl AnthropicMessagesModel {
    pub fn new(base_url: String, model: String, api_key: String) -> Self {
        let tools_json = anthropic_tools_from_openai(crate::agent::registry::tools_as_openai_json());
        Self {
            base_url,
            model,
            api_key,
            tools_json,
        }
    }
}

fn anthropic_tools_from_openai(openai_tools: Value) -> Value {
    let Some(arr) = openai_tools.as_array() else {
        return serde_json::json!([]);
    };
    let mapped: Vec<Value> = arr
        .iter()
        .filter_map(|t| {
            let name = t.pointer("/function/name")?.as_str()?;
            let desc = t
                .pointer("/function/description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let schema = t.pointer("/function/parameters").cloned().unwrap_or_else(|| serde_json::json!({}));
            Some(serde_json::json!({
                "name": name,
                "description": desc,
                "input_schema": schema,
            }))
        })
        .collect();
    serde_json::Value::Array(mapped)
}

impl ChatModel for AnthropicMessagesModel {
    fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let messages: Vec<Value> = msgs
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                serde_json::json!({
                    "role": if m.role == "tool" { "user" } else { m.role.as_str() },
                    "content": m.content,
                })
            })
            .collect();
        let system = msgs
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());
        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": messages,
            "tools": self.tools_json,
        });
        if let Some(s) = system {
            body["system"] = Value::String(s);
        }
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .map_err(|e| format!("Anthropic 请求失败: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(format!("Anthropic HTTP {status}: {text}"));
        }
        let json: Value = resp.json().map_err(|e| format!("解析 Anthropic 响应失败: {e}"))?;
        parse_anthropic_turn(&json)
    }
}

fn parse_anthropic_turn(json: &Value) -> Result<ModelTurn, String> {
    let content = json
        .get("content")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Anthropic 响应缺少 content".to_string())?;
    let mut text_parts = Vec::new();
    let mut tool_calls = Vec::new();
    for block in content {
        match block.get("type").and_then(|v| v.as_str()) {
            Some("text") => {
                if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                    text_parts.push(t.to_string());
                }
            }
            Some("tool_use") => {
                let id = block
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("call")
                    .to_string();
                let name = block
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let arguments = block.get("input").cloned().unwrap_or_else(|| serde_json::json!({}));
                tool_calls.push(ToolCall {
                    id,
                    name,
                    arguments,
                });
            }
            _ => {}
        }
    }
    if !tool_calls.is_empty() {
        return Ok(ModelTurn::ToolCalls(tool_calls));
    }
    Ok(ModelTurn::Text(text_parts.join("")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_without_model_does_not_hit_cloud() {
        let cfg = LlmConfig {
            provider: "local".into(),
            protocol: None,
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
            has_api_key: true,
        };
        let err = resolve_backend(&cfg, None).unwrap_err();
        assert!(matches!(err, AgentError::LlmUnavailable));
    }

    #[test]
    fn cloud_ready_when_configured() {
        let cfg = LlmConfig {
            provider: "openai".into(),
            protocol: Some(LlmProtocol::OpenaiChat),
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
            has_api_key: true,
        };
        assert!(resolve_backend(&cfg, None).is_ok());
    }
}

// ---------------------------------------------------------------------------
// Unified model factory
// ---------------------------------------------------------------------------

/// Which concrete backend a locally-resolved model will run on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalBackend {
    Embedded,
    Ollama,
}

/// Resolve the effective local backend WITHOUT building a model: embedded
/// when a catalog GGUF is installed, else Ollama when its port answers with
/// at least one model, else `None` (local LLM unavailable).
pub fn effective_local_backend() -> Option<LocalBackend> {
    if model_catalog::enabled_model_path().is_some() {
        return Some(LocalBackend::Embedded);
    }
    if super::genai_model::ollama_available_with_model() {
        return Some(LocalBackend::Ollama);
    }
    None
}

/// Build the chat model for the CURRENT config: embedded engine for `local`
/// with an installed GGUF, genai (HTTP) for everything else (including the
/// local→Ollama fallback inside genai's resolver).
pub fn build_model_from_disk() -> Result<Box<dyn ChatModel>, AgentError> {
    let cfg = get_llm_config();
    if cfg.is_local() {
        if let Some(path) = model_catalog::enabled_model_path() {
            return Ok(Box::new(super::embedded_engine::EmbeddedChatModel::new(path)));
        }
        // No installed GGUF: fall through to genai (Ollama fallback) which
        // errors with LlmUnavailable when nothing is reachable.
    }
    let model = super::genai_model::build_from_disk()?;
    Ok(Box::new(model))
}
