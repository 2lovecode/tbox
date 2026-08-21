//! ChatModel abstraction, backend resolution, and OpenAI-compatible client.

use serde_json::Value;
use std::path::PathBuf;

use crate::commands::llm::{get_llm_config, LlmConfig, LlmProvider};
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
}

/// Resolve which backend to use. Local without an installed model MUST NOT
/// fall back to cloud silently.
pub fn resolve_backend(
    cfg: &LlmConfig,
    local_model: Option<PathBuf>,
) -> Result<ReadyLlm, AgentError> {
    match cfg.provider {
        LlmProvider::Local => {
            if local_model.is_none() {
                return Err(AgentError::LlmUnavailable);
            }
            Ok(ReadyLlm::OpenAiCompat {
                base_url: "http://127.0.0.1:11435/v1".into(),
                model: local_model
                    .as_ref()
                    .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| "local".into()),
                api_key: None,
            })
        }
        LlmProvider::Openai | LlmProvider::Deepseek | LlmProvider::Custom => {
            if cfg.base_url.trim().is_empty() || cfg.model.trim().is_empty() || !cfg.has_api_key {
                return Err(AgentError::LlmUnavailable);
            }
            let api_key = crate::commands::llm::read_api_key_for_agent()
                .ok()
                .flatten();
            Ok(ReadyLlm::OpenAiCompat {
                base_url: cfg.base_url.trim_end_matches('/').to_string(),
                model: cfg.model.clone(),
                api_key,
            })
        }
        LlmProvider::Anthropic => {
            // Agent tool-calling prefers OpenAI-compatible endpoints in v1.
            Err(AgentError::LlmUnavailable)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_without_model_does_not_hit_cloud() {
        let cfg = LlmConfig {
            provider: LlmProvider::Local,
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
            provider: LlmProvider::Openai,
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
            has_api_key: true,
        };
        assert!(resolve_backend(&cfg, None).is_ok());
    }
}
