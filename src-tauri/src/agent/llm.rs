//! ChatModel abstraction and model turn types for the agent loop.

use serde_json::Value;

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
    /// Call instance id (for event correlation).
    pub id: String,
    /// Registry tool id, e.g. base64.encode.
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
