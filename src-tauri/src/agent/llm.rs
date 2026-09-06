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

    /// 优先真流式；默认实现走 `complete` + 分块（Fallback）。
    fn complete_streaming(
        &mut self,
        msgs: &[ModelMessage],
        _cancel: &std::sync::atomic::AtomicBool,
        on_delta: &mut dyn FnMut(StreamDelta),
    ) -> Result<ModelTurn, String> {
        let turn = self.complete(msgs)?;
        emit_turn_as_fallback_chunks(&turn, on_delta);
        Ok(turn)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMode {
    Live,
    Fallback,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamDelta {
    Meta { mode: StreamMode },
    Reasoning { text: String },
    Text { text: String },
}

const FALLBACK_CHUNK_CHARS: usize = 24;

/// Emit a completed turn as Fallback meta + chunked reasoning/text deltas.
pub fn emit_turn_as_fallback_chunks(turn: &ModelTurn, on_delta: &mut dyn FnMut(StreamDelta)) {
    on_delta(StreamDelta::Meta {
        mode: StreamMode::Fallback,
    });
    match turn {
        ModelTurn::Text { text, reasoning } => {
            if let Some(r) = reasoning.as_ref().filter(|s| !s.trim().is_empty()) {
                for chunk in chunk_text(r, FALLBACK_CHUNK_CHARS) {
                    if !chunk.is_empty() {
                        on_delta(StreamDelta::Reasoning { text: chunk });
                    }
                }
            }
            for chunk in chunk_text(text, FALLBACK_CHUNK_CHARS) {
                if !chunk.is_empty() {
                    on_delta(StreamDelta::Text { text: chunk });
                }
            }
        }
        ModelTurn::ToolCalls(_) => {
            // 工具轮不在流式阶段推正文；loop 侧用 ToolStart/End。
        }
    }
}

/// 增量剥离 `<think>…</think>`，供真流式路径使用。
#[derive(Debug, Default)]
pub struct ThinkStreamParser {
    buf: String,
    in_think: bool,
    reasoning_acc: String,
    body_acc: String,
}

impl ThinkStreamParser {
    /// Push a text chunk; emits Reasoning/Text deltas with tags stripped.
    pub fn push(&mut self, chunk: &str, on_delta: &mut dyn FnMut(StreamDelta)) {
        self.buf.push_str(chunk);
        loop {
            if self.in_think {
                if let Some(i) = self.buf.find("</think>") {
                    let reason = self.buf[..i].to_string();
                    self.buf = self.buf[i + "</think>".len()..].to_string();
                    self.in_think = false;
                    if !reason.is_empty() {
                        self.reasoning_acc.push_str(&reason);
                        on_delta(StreamDelta::Reasoning { text: reason });
                    }
                    continue;
                }
                let keep = partial_suffix_overlap(&self.buf, "</think>");
                if self.buf.len() > keep {
                    let emit = self.buf[..self.buf.len() - keep].to_string();
                    self.buf.drain(..self.buf.len() - keep);
                    if !emit.is_empty() {
                        self.reasoning_acc.push_str(&emit);
                        on_delta(StreamDelta::Reasoning { text: emit });
                    }
                }
                break;
            } else if let Some(i) = self.buf.find("<think>") {
                let before = self.buf[..i].to_string();
                self.buf = self.buf[i + "<think>".len()..].to_string();
                self.in_think = true;
                if !before.is_empty() {
                    self.body_acc.push_str(&before);
                    on_delta(StreamDelta::Text { text: before });
                }
                continue;
            } else {
                let keep = partial_suffix_overlap(&self.buf, "<think>");
                if self.buf.len() > keep {
                    let emit = self.buf[..self.buf.len() - keep].to_string();
                    self.buf.drain(..self.buf.len() - keep);
                    if !emit.is_empty() {
                        self.body_acc.push_str(&emit);
                        on_delta(StreamDelta::Text { text: emit });
                    }
                }
                break;
            }
        }
    }

    /// Flush remaining buffer (unclosed think → reasoning). Returns (reasoning, body).
    pub fn finish(mut self, on_delta: &mut dyn FnMut(StreamDelta)) -> (Option<String>, String) {
        if self.in_think {
            if !self.buf.is_empty() {
                self.reasoning_acc.push_str(&self.buf);
                on_delta(StreamDelta::Reasoning {
                    text: std::mem::take(&mut self.buf),
                });
            }
        } else if !self.buf.is_empty() {
            self.body_acc.push_str(&self.buf);
            on_delta(StreamDelta::Text {
                text: std::mem::take(&mut self.buf),
            });
        }
        let reasoning = {
            let t = self.reasoning_acc.trim().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        };
        (reasoning, self.body_acc.trim().to_string())
    }
}

fn partial_suffix_overlap(s: &str, tag: &str) -> usize {
    let max = s.len().min(tag.len().saturating_sub(1));
    for len in (1..=max).rev() {
        if s.ends_with(&tag[..len]) {
            return len;
        }
    }
    0
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
        let body = self.build_request_body(msgs, false);
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

    fn complete_streaming(
        &mut self,
        msgs: &[ModelMessage],
        cancel: &std::sync::atomic::AtomicBool,
        on_delta: &mut dyn FnMut(StreamDelta),
    ) -> Result<ModelTurn, String> {
        let url = format!(
            "{}/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let body = self.build_request_body(msgs, true);
        let client = reqwest::blocking::Client::new();
        let mut req = client.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let resp = match req.send() {
            Ok(r) => r,
            Err(_) => {
                let turn = self.complete(msgs)?;
                emit_turn_as_fallback_chunks(&turn, on_delta);
                return Ok(turn);
            }
        };
        if !resp.status().is_success() {
            let turn = self.complete(msgs)?;
            emit_turn_as_fallback_chunks(&turn, on_delta);
            return Ok(turn);
        }

        on_delta(StreamDelta::Meta {
            mode: StreamMode::Live,
        });

        let mut aggregator = OpenAiStreamAggregator::default();
        let reader = resp;
        use std::io::{BufRead, BufReader};
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            line.clear();
            let n = buf_reader
                .read_line(&mut line)
                .map_err(|e| format!("读取 SSE 失败: {e}"))?;
            if n == 0 {
                break;
            }
            let trimmed = line.trim_end();
            if trimmed.is_empty() || trimmed.starts_with(':') {
                continue;
            }
            let Some(data) = trimmed.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data == "[DONE]" {
                break;
            }
            if let Ok(json) = serde_json::from_str::<Value>(data) {
                aggregator.ingest_chunk(&json, on_delta);
            }
        }
        Ok(aggregator.into_turn())
    }
}

impl OpenAiCompatModel {
    fn build_request_body(&self, msgs: &[ModelMessage], stream: bool) -> Value {
        let messages: Vec<Value> = msgs
            .iter()
            .map(|m| {
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
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "tools": self.tools_json,
            "tool_choice": "auto",
        });
        if stream {
            body["stream"] = Value::Bool(true);
        }
        body
    }
}

#[derive(Default)]
struct OpenAiStreamAggregator {
    text: String,
    reasoning: String,
    /// index -> (id, name, arguments_acc)
    tool_calls: std::collections::BTreeMap<usize, (String, String, String)>,
}

impl OpenAiStreamAggregator {
    fn ingest_chunk(&mut self, json: &Value, on_delta: &mut dyn FnMut(StreamDelta)) {
        let Some(choice) = json.pointer("/choices/0") else {
            return;
        };
        let delta = choice.get("delta").unwrap_or(choice);
        if let Some(r) = delta
            .get("reasoning_content")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            self.reasoning.push_str(r);
            on_delta(StreamDelta::Reasoning {
                text: r.to_string(),
            });
        }
        if let Some(t) = delta
            .get("content")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            self.text.push_str(t);
            on_delta(StreamDelta::Text {
                text: t.to_string(),
            });
        }
        if let Some(arr) = delta.get("tool_calls").and_then(|v| v.as_array()) {
            for c in arr {
                let idx = c.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let entry = self.tool_calls.entry(idx).or_insert_with(|| {
                    (
                        c.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("call")
                            .to_string(),
                        String::new(),
                        String::new(),
                    )
                });
                if let Some(id) = c.get("id").and_then(|v| v.as_str()) {
                    if !id.is_empty() {
                        entry.0 = id.to_string();
                    }
                }
                if let Some(name) = c.pointer("/function/name").and_then(|v| v.as_str()) {
                    entry.1.push_str(name);
                }
                if let Some(args) = c.pointer("/function/arguments").and_then(|v| v.as_str()) {
                    entry.2.push_str(args);
                }
            }
        }
    }

    fn into_turn(self) -> ModelTurn {
        if !self.tool_calls.is_empty() {
            let calls: Vec<ToolCall> = self
                .tool_calls
                .into_values()
                .map(|(id, name, args)| ToolCall {
                    id,
                    name,
                    arguments: serde_json::from_str(&args)
                        .unwrap_or_else(|_| serde_json::json!({})),
                })
                .collect();
            return ModelTurn::ToolCalls(calls);
        }
        ModelTurn::Text {
            text: self.text,
            reasoning: if self.reasoning.is_empty() {
                None
            } else {
                Some(self.reasoning)
            },
        }
    }
}

#[cfg(test)]
mod openai_stream_tests {
    use super::*;

    #[test]
    fn sse_aggregator_merges_content_and_tools() {
        let mut agg = OpenAiStreamAggregator::default();
        let mut deltas = Vec::new();
        let mut on = |d: StreamDelta| deltas.push(d);
        agg.ingest_chunk(
            &serde_json::json!({
                "choices": [{"delta": {"content": "hel"}}]
            }),
            &mut on,
        );
        agg.ingest_chunk(
            &serde_json::json!({
                "choices": [{"delta": {"content": "lo"}}]
            }),
            &mut on,
        );
        let turn = agg.into_turn();
        assert_eq!(turn, ModelTurn::text("hello"));
        assert_eq!(deltas.len(), 2);

        let mut agg2 = OpenAiStreamAggregator::default();
        let mut on2 = |_d: StreamDelta| {};
        agg2.ingest_chunk(
            &serde_json::json!({
                "choices": [{"delta": {"tool_calls": [{
                    "index": 0,
                    "id": "c1",
                    "function": {"name": "base64.encode", "arguments": "{\"in"}
                }]}}]
            }),
            &mut on2,
        );
        agg2.ingest_chunk(
            &serde_json::json!({
                "choices": [{"delta": {"tool_calls": [{
                    "index": 0,
                    "function": {"arguments": "put\":\"hi\"}"}
                }]}}]
            }),
            &mut on2,
        );
        match agg2.into_turn() {
            ModelTurn::ToolCalls(calls) => {
                assert_eq!(calls[0].name, "base64.encode");
                assert_eq!(calls[0].arguments["input"], "hi");
            }
            other => panic!("expected tool calls, got {other:?}"),
        }
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
    fn think_stream_parser_closed_and_unclosed() {
        let mut p = ThinkStreamParser::default();
        let mut deltas = Vec::new();
        let mut on = |d: StreamDelta| deltas.push(d);
        p.push("<thi", &mut on);
        p.push("nk>推理", &mut on);
        p.push("</think>答案", &mut on);
        let (r, body) = p.finish(&mut on);
        assert_eq!(r.as_deref(), Some("推理"));
        assert_eq!(body, "答案");
        assert!(deltas.iter().any(|d| matches!(d, StreamDelta::Reasoning { .. })));
        assert!(deltas.iter().any(|d| matches!(d, StreamDelta::Text { .. })));

        let mut p2 = ThinkStreamParser::default();
        let mut deltas2 = Vec::new();
        let mut on2 = |d: StreamDelta| deltas2.push(d);
        p2.push("<think>半截", &mut on2);
        let (r2, body2) = p2.finish(&mut on2);
        assert_eq!(r2.as_deref(), Some("半截"));
        assert_eq!(body2, "");
    }

    #[test]
    fn fallback_streaming_default_emits_meta() {
        struct OnlyComplete;
        impl ChatModel for OnlyComplete {
            fn complete(&mut self, _: &[ModelMessage]) -> Result<ModelTurn, String> {
                Ok(ModelTurn::text("abcdef"))
            }
        }
        let mut m = OnlyComplete;
        let mut deltas = Vec::new();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let turn = m
            .complete_streaming(&[], &cancel, &mut |d| deltas.push(d))
            .unwrap();
        assert!(matches!(turn, ModelTurn::Text { .. }));
        assert!(matches!(
            deltas.first(),
            Some(StreamDelta::Meta {
                mode: StreamMode::Fallback
            })
        ));
        let text: String = deltas
            .iter()
            .filter_map(|d| match d {
                StreamDelta::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, "abcdef");
    }

    #[test]
    fn stream_meta_serde_snake_case() {
        let live = serde_json::to_value(StreamMode::Live).unwrap();
        let fb = serde_json::to_value(StreamMode::Fallback).unwrap();
        assert_eq!(live, serde_json::json!("live"));
        assert_eq!(fb, serde_json::json!("fallback"));
        let ev = crate::agent::r#loop::AgentEvent::StreamMeta {
            mode: StreamMode::Live,
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["kind"], "stream_meta");
        assert_eq!(v["mode"], "live");
    }

    #[test]
    fn local_without_model_does_not_hit_cloud() {
        let cfg = LlmConfig {
            provider: "local".into(),
            protocol: None,
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
            has_api_key: true,
            ..Default::default()
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
            ..Default::default()
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
