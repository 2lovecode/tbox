//! `genai`-backed [`ChatModel`](super::llm::ChatModel) for multi-protocol LLM access.

use std::path::PathBuf;
use std::sync::Arc;

use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest, Tool};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use serde_json::Value;

use super::llm::{AgentError, ChatModel, ModelMessage, ModelTurn, ToolCall};
use crate::commands::llm::{LlmConfig, LlmProtocol};

pub struct GenaiChatModel {
    client: Client,
    model: String,
    tools: Vec<Tool>,
}

impl GenaiChatModel {
    pub fn from_config(cfg: &LlmConfig, local_model: Option<PathBuf>) -> Result<Self, AgentError> {
        if cfg.requires_oauth_preset() {
            return Err(AgentError::LlmUnavailable);
        }

        let protocol = cfg.resolved_protocol();
        let (adapter, endpoint, model, api_key) =
            resolve_target(cfg, local_model, protocol)?;

        let endpoint_arc: Arc<str> = Arc::from(endpoint);
        let auth = match api_key {
            Some(k) if !k.is_empty() => AuthData::from_single(k),
            // The OpenAI adapter requires single-value auth (it always sends
            // `Authorization: Bearer …`); AuthData::None fails with
            // ResolverAuthDataNotSingleValue. Local/Ollama backends ignore
            // the key, so send a placeholder.
            _ => AuthData::from_single("not-set"),
        };

        let resolver = ServiceTargetResolver::from_resolver_fn(
            move |service_target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
                Ok(ServiceTarget {
                    endpoint: Endpoint::from_owned(Arc::clone(&endpoint_arc)),
                    auth: auth.clone(),
                    model: ModelIden::new(adapter, service_target.model.model_name),
                })
            },
        );

        let client = Client::builder()
            .with_service_target_resolver(resolver)
            .build();

        Ok(Self {
            client,
            model,
            tools: tools_from_registry(),
        })
    }
}

pub fn build_from_disk() -> Result<GenaiChatModel, AgentError> {
    let cfg = crate::commands::llm::get_llm_config();
    let local = crate::commands::model_catalog::enabled_model_path();
    GenaiChatModel::from_config(&cfg, local)
}

fn resolve_target(
    cfg: &LlmConfig,
    local_model: Option<PathBuf>,
    protocol: LlmProtocol,
) -> Result<(AdapterKind, String, String, Option<String>), AgentError> {
    if cfg.is_local() {
        // Embedded engine handles the installed-GGUF case (see
        // `build_model_from_disk`); when it has no model, fall back to a
        // local Ollama instance (OpenAI-compatible /v1) and pick one of its
        // installed models.
        if port_open(11434) {
            if let Some(model) = first_ollama_model() {
                return Ok((
                    AdapterKind::OpenAI,
                    "http://127.0.0.1:11434/v1/".into(),
                    model,
                    None,
                ));
            }
        }
        return Err(AgentError::LlmUnavailable);
    }

    if cfg.is_ollama() {
        if cfg.model.trim().is_empty() {
            return Err(AgentError::LlmUnavailable);
        }
        let base = if cfg.base_url.trim().is_empty() {
            "http://127.0.0.1:11434".into()
        } else {
            normalize_ollama_host(&cfg.base_url)
        };
        return Ok((
            AdapterKind::Ollama,
            base,
            cfg.model.clone(),
            None,
        ));
    }

    if cfg.base_url.trim().is_empty() || cfg.model.trim().is_empty() {
        return Err(AgentError::LlmUnavailable);
    }

    let api_key = if cfg.has_api_key {
        crate::commands::llm::read_api_key_for_agent()
            .ok()
            .flatten()
    } else {
        None
    };

    if api_key.is_none() {
        return Err(AgentError::LlmUnavailable);
    }

    let adapter = protocol_to_adapter(protocol);
    let endpoint = normalize_endpoint(&cfg.base_url, protocol);
    Ok((adapter, endpoint, cfg.model.clone(), api_key))
}

/// True when a TCP connect to 127.0.0.1:port succeeds quickly.
fn port_open(port: u16) -> bool {
    use std::net::TcpStream;
    TcpStream::connect(("127.0.0.1", port)).is_ok()
}

/// True when a local Ollama answers on 11434 and has at least one model.
/// Used by `effective_local_backend` for backend visibility.
pub fn ollama_available_with_model() -> bool {
    port_open(11434) && first_ollama_model().is_some()
}

/// First installed model name from a local Ollama instance, if reachable.
/// Uses a raw TCP HTTP request: `reqwest::blocking` builds and drops an
/// internal tokio runtime, which panics when called from an async context.
fn first_ollama_model() -> Option<String> {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", 11434)).ok()?;
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    stream
        .write_all(b"GET /api/tags HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")
        .ok()?;
    let mut body = String::new();
    stream.read_to_string(&mut body).ok()?;
    let json_part = body.split("\r\n\r\n").nth(1)?;
    let v: serde_json::Value = serde_json::from_str(json_part).ok()?;
    v.pointer("/models/0/name")?
        .as_str()
        .map(|s| s.to_string())
}

fn protocol_to_adapter(protocol: LlmProtocol) -> AdapterKind {    match protocol {
        LlmProtocol::OpenaiChat => AdapterKind::OpenAI,
        LlmProtocol::OpenaiResponses => AdapterKind::OpenAIResp,
        LlmProtocol::AnthropicMessages => AdapterKind::Anthropic,
        LlmProtocol::GeminiNative => AdapterKind::Gemini,
        LlmProtocol::OllamaNative => AdapterKind::Ollama,
    }
}

fn normalize_endpoint(base: &str, protocol: LlmProtocol) -> String {
    let b = base.trim().trim_end_matches('/');
    match protocol {
        LlmProtocol::OllamaNative => normalize_ollama_host(b),
        LlmProtocol::AnthropicMessages => format!("{b}/"),
        LlmProtocol::OpenaiResponses | LlmProtocol::OpenaiChat | LlmProtocol::GeminiNative => {
            if b.ends_with("/v1") {
                format!("{b}/")
            } else {
                format!("{b}/v1/")
            }
        }
    }
}

fn normalize_ollama_host(base: &str) -> String {
    let b = base.trim().trim_end_matches('/');
    b.trim_end_matches("/v1").to_string()
}

fn tools_from_registry() -> Vec<Tool> {
    let openai = crate::agent::registry::tools_as_openai_json();
    let Some(arr) = openai.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|t| {
            let name = t.pointer("/function/name")?.as_str()?;
            let desc = t
                .pointer("/function/description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let schema = t
                .pointer("/function/parameters")
                .cloned()
                .unwrap_or_else(|| Value::Object(Default::default()));
            Some(
                Tool::new(name)
                    .with_description(desc)
                    .with_schema(schema),
            )
        })
        .collect()
}

fn to_genai_messages(msgs: &[ModelMessage]) -> Vec<ChatMessage> {
    msgs.iter()
        .map(|m| match m.role.as_str() {
            "system" => ChatMessage::system(m.content.clone()),
            "assistant" => ChatMessage::assistant(m.content.clone()),
            "tool" => ChatMessage::user(format!("[tool result]\n{}", m.content)),
            _ => ChatMessage::user(m.content.clone()),
        })
        .collect()
}


impl ChatModel for GenaiChatModel {
    fn backend_desc(&self) -> super::llm::BackendDesc {
        super::llm::BackendDesc {
            backend: "genai".into(),
            model: self.model.clone(),
        }
    }

    fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String> {
        let chat_req = ChatRequest::new(to_genai_messages(msgs)).with_tools(self.tools.clone());
        let client = self.client.clone();
        let model = self.model.clone();

        let chat_res = tauri::async_runtime::block_on(async move {
            client.exec_chat(&model, chat_req, None).await
        })
        .map_err(|e| format!("LLM 请求失败: {e}"))?;

        let genai_calls = chat_res.tool_calls();
        if !genai_calls.is_empty() {
            let mapped: Vec<ToolCall> = genai_calls
                .iter()
                .map(|c| ToolCall {
                    id: c.call_id.clone(),
                    name: c.fn_name.clone(),
                    arguments: c.fn_arguments.clone(),
                })
                .collect();
            return Ok(ModelTurn::ToolCalls(mapped));
        }

        let text = chat_res
            .first_text()
            .unwrap_or("")
            .to_string();
        Ok(ModelTurn::Text(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_without_model_needs_a_local_backend() {
        let cfg = LlmConfig {
            provider: "local".into(),
            protocol: None,
            base_url: String::new(),
            model: String::new(),
            has_api_key: false,
        };
        // With any local backend up (embedded or Ollama) resolution succeeds
        // (falling back to Ollama's first model); with none it errors.
        let result = resolve_target(&cfg, None, LlmProtocol::OpenaiChat);
        if port_open(11435) || port_open(11434) {
            let (_, endpoint, _, _) = result.expect("local backend should resolve");
            assert!(endpoint.starts_with("http://127.0.0.1:1143"));
        } else {
            assert!(result.is_err());
        }
    }

    #[test]
    fn normalize_openai_endpoint_adds_v1() {
        assert_eq!(
            normalize_endpoint("https://api.openai.com", LlmProtocol::OpenaiChat),
            "https://api.openai.com/v1/"
        );
    }
}
