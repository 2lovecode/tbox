//! `genai`-backed [`ChatModel`](super::llm::ChatModel) for multi-protocol LLM access.

use std::path::PathBuf;
use std::sync::Arc;

use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest, Tool, ToolCall as GenaiToolCall};
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
            _ => AuthData::None,
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
        if local_model.is_none() {
            return Err(AgentError::LlmUnavailable);
        }
        let model = local_model
            .as_ref()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "local".into());
        return Ok((
            AdapterKind::OpenAI,
            "http://127.0.0.1:11435/v1/".into(),
            model,
            None,
        ));
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

fn protocol_to_adapter(protocol: LlmProtocol) -> AdapterKind {
    match protocol {
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
    fn local_without_model_is_unavailable() {
        let cfg = LlmConfig {
            provider: "local".into(),
            protocol: None,
            base_url: String::new(),
            model: String::new(),
            has_api_key: false,
        };
        assert!(GenaiChatModel::from_config(&cfg, None).is_err());
    }

    #[test]
    fn normalize_openai_endpoint_adds_v1() {
        assert_eq!(
            normalize_endpoint("https://api.openai.com", LlmProtocol::OpenaiChat),
            "https://api.openai.com/v1/"
        );
    }
}
