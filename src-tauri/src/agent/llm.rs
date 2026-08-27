//! ChatModel abstraction, backend resolution, and OpenAI-compatible client.

use serde_json::Value;
use std::path::PathBuf;

use crate::commands::llm::{get_llm_config, LlmConfig, LlmProtocol};
use crate::commands::model_catalog;

/// One message sent to the model (system / user / assistant / tool).
///
/// 工具协议要求「assistant 发起 tool_calls → role=tool 回填结果」成对出现：
/// `tool_calls` 仅 assistant 角色使用；`tool_call_id`/`tool_name` 仅 tool
/// 角色使用。缺失 assistant tool_calls 消息会让模型看不到自己的调用而
/// 重复发起（exceeded max tool iterations）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModelMessage {
    pub role: String,
    pub content: String,
    /// assistant 角色：模型请求的工具调用（普通文本回复为空）。
    pub tool_calls: Vec<ToolCall>,
    /// tool 角色：对应的调用 id 与工具名（回填结果时携带）。
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
}

impl ModelMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            ..Default::default()
        }
    }

    /// assistant 发起工具调用的消息（content 可为空）。
    pub fn assistant_tool_calls(calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: String::new(),
            tool_calls: calls,
            ..Default::default()
        }
    }

    /// 兼容旧测试：不带 id 的 tool 结果。
    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.into(),
            ..Default::default()
        }
    }

    /// 回填工具结果：携带调用 id 与工具名，后端按协议正确配对。
    pub fn tool_result(id: impl Into<String>, name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.into(),
            tool_call_id: Some(id.into()),
            tool_name: Some(name.into()),
            ..Default::default()
        }
    }

    /// 纯文本视图：把 tool_calls / tool 结果序列化为文本（embedded 引擎
    /// 的 Qwen 文本协议用）。
    pub fn flatten_content(&self) -> String {
        if !self.tool_calls.is_empty() {
            let calls: Vec<String> = self
                .tool_calls
                .iter()
                .map(|c| {
                    format!(
                        "<tool_call>{{\"name\": \"{}\", \"arguments\": {}}}</tool_call>",
                        c.name, c.arguments
                    )
                })
                .collect();
            let head = if self.content.is_empty() {
                String::new()
            } else {
                format!("{}\n", self.content)
            };
            return format!("{head}{}", calls.join(""));
        }
        if let Some(name) = &self.tool_name {
            return format!("[tool result: {name}]\n{}", self.content);
        }
        self.content.clone()
    }
}

/// One tool call requested by the model.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Single model turn: plain text (with optional reasoning) or tool calls.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelTurn {
    Text {
        text: String,
        /// 思考/推理内容（模型未提供则为 None）。
        reasoning: Option<String>,
    },
    ToolCalls(Vec<ToolCall>),
}

impl ModelTurn {
    /// 无思考内容的纯文本回复（测试与不回 reasoning 的后端使用）。
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            reasoning: None,
        }
    }
}

/// 剥离模型正文里内联的思考标记（`<think>…</think>`，Qwen/DeepSeek 风格），
/// 返回 (reasoning, 正文)。支持：
/// - 多个 think 块（内容以换行拼接）
/// - 被截断未闭合的 `<think>…`（生成中断时），其后全部内容视为思考
/// 无 think 标记时原样返回（reasoning = None）。
pub fn split_think_tags(text: &str) -> (Option<String>, String) {
    const OPEN: &str = "<think>";
    const CLOSE: &str = "</think>";
    if !text.contains(OPEN) {
        return (None, text.to_string());
    }
    let mut reasoning_parts: Vec<String> = Vec::new();
    let mut body = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(OPEN) {
        body.push_str(&rest[..i]);
        let after = &rest[i + OPEN.len()..];
        match after.find(CLOSE) {
            Some(j) => {
                reasoning_parts.push(after[..j].to_string());
                rest = &after[j + CLOSE.len()..];
            }
            None => {
                // 未闭合（截断）：其后全部内容归入思考。
                reasoning_parts.push(after.to_string());
                rest = "";
                break;
            }
        }
    }
    body.push_str(rest);
    let reasoning = reasoning_parts
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let reasoning = if reasoning.is_empty() { None } else { Some(reasoning) };
    (reasoning, body.trim().to_string())
}

/// 把文本切成小块供事件流式发出（体验层伪流式：模型回合本身非流式，
/// 但分块 emit 让前端逐步渲染）。空文本返回单个空块以保留事件语义。
pub fn chunk_text(text: &str, chunk_chars: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return vec![String::new()];
    }
    chars
        .chunks(chunk_chars.max(1))
        .map(|c| c.iter().collect())
        .collect()
}

/// 模型后端描述：harness 策略选档依据（backend 如 "embedded" / "genai"）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendDesc {
    pub backend: String,
    pub model: String,
}

impl Default for BackendDesc {
    fn default() -> Self {
        Self {
            backend: "default".to_string(),
            model: String::new(),
        }
    }
}

/// Swappable chat model backend (real LLM or Scripted mock).
pub trait ChatModel {
    fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String>;

    /// 后端/模型描述，供 harness 选择策略档位；默认非 embedded 档。
    fn backend_desc(&self) -> BackendDesc {
        BackendDesc::default()
    }
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
                // assistant 工具调用：带 tool_calls 结构（与 role=tool 配对）
                if m.role == "assistant" && !m.tool_calls.is_empty() {
                    let calls: Vec<Value> = m
                        .tool_calls
                        .iter()
                        .map(|c| {
                            serde_json::json!({
                                "id": c.id,
                                "type": "function",
                                "function": {
                                    "name": c.name,
                                    "arguments": c.arguments.to_string(),
                                }
                            })
                        })
                        .collect();
                    return serde_json::json!({
                        "role": "assistant",
                        "content": if m.content.is_empty() { Value::Null } else { Value::String(m.content.clone()) },
                        "tool_calls": calls,
                    });
                }
                // 工具结果：role=tool + tool_call_id 配对
                if m.role == "tool" {
                    return serde_json::json!({
                        "role": "tool",
                        "tool_call_id": m.tool_call_id.clone().unwrap_or_default(),
                        "content": m.content,
                    });
                }
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
    let reasoning = choice
        .get("reasoning_content")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let text = choice
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(ModelTurn::Text { text, reasoning })
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
                // assistant 工具调用 → tool_use 块（与 tool_result 配对）
                if m.role == "assistant" && !m.tool_calls.is_empty() {
                    let blocks: Vec<Value> = m
                        .tool_calls
                        .iter()
                        .map(|c| {
                            serde_json::json!({
                                "type": "tool_use",
                                "id": c.id,
                                "name": c.name,
                                "input": c.arguments,
                            })
                        })
                        .collect();
                    return serde_json::json!({
                        "role": "assistant",
                        "content": blocks,
                    });
                }
                // 工具结果 → user 侧 tool_result 块（Anthropic 协议要求）
                if m.role == "tool" {
                    return serde_json::json!({
                        "role": "user",
                        "content": [serde_json::json!({
                            "type": "tool_result",
                            "tool_use_id": m.tool_call_id.clone().unwrap_or_default(),
                            "content": m.content,
                        })],
                    });
                }
                serde_json::json!({
                    "role": m.role,
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
    let mut thinking_parts = Vec::new();
    let mut tool_calls = Vec::new();
    for block in content {
        match block.get("type").and_then(|v| v.as_str()) {
            Some("thinking") => {
                if let Some(t) = block.get("thinking").and_then(|v| v.as_str()) {
                    thinking_parts.push(t.to_string());
                }
            }
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
    let reasoning = if thinking_parts.is_empty() {
        None
    } else {
        Some(thinking_parts.join("\n"))
    };
    Ok(ModelTurn::Text {
        text: text_parts.join(""),
        reasoning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_think_basic() {
        let (r, body) = split_think_tags("<think>推理过程</think>最终答案");
        assert_eq!(r.as_deref(), Some("推理过程"));
        assert_eq!(body, "最终答案");
    }

    #[test]
    fn split_think_unclosed_on_truncation() {
        let (r, body) = split_think_tags("<think>只写到一半");
        assert_eq!(r.as_deref(), Some("只写到一半"));
        assert_eq!(body, "");
    }

    #[test]
    fn split_think_multiple_blocks() {
        let (r, body) = split_think_tags("<think>a</think>中段<think>b</think>尾");
        assert_eq!(r.as_deref(), Some("a\nb"));
        assert_eq!(body, "中段尾");
    }

    #[test]
    fn split_think_absent_returns_original() {
        let (r, body) = split_think_tags("普通回复，无思考");
        assert!(r.is_none());
        assert_eq!(body, "普通回复，无思考");
    }

    #[test]
    fn chunk_text_splits_and_keeps_empty() {
        let chunks = chunk_text("abcdef", 2);
        assert_eq!(chunks, vec!["ab", "cd", "ef"]);
        assert_eq!(chunk_text("", 4), vec![String::new()]);
    }

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
