//! Context token budget estimation (deterministic heuristic).

use serde::Serialize;

use crate::agent::llm::ModelMessage;
use crate::commands::llm::{active_profile, DEFAULT_N_CTX};

/// Default cloud context window when provider does not expose one.
pub const DEFAULT_CLOUD_CTX: u32 = 128_000;

/// Compress when used/limit >= this ratio (default 80%).
pub const DEFAULT_COMPRESS_THRESHOLD: f64 = 0.8;

/// Max tool-result chars kept in Runtime without truncation.
pub const TOOL_RESULT_CHAR_BUDGET: usize = 4_000;

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BudgetParts {
    pub system: u32,
    pub skills: u32,
    pub tools: u32,
    pub messages: u32,
    pub tool_results: u32,
    pub memory: u32,
}

impl BudgetParts {
    pub fn sum(&self) -> u32 {
        self.system
            .saturating_add(self.skills)
            .saturating_add(self.tools)
            .saturating_add(self.messages)
            .saturating_add(self.tool_results)
            .saturating_add(self.memory)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextBudget {
    pub limit: u32,
    pub used: u32,
    pub ratio: f64,
    pub parts: BudgetParts,
    /// True when ratio >= compress threshold.
    pub near_limit: bool,
}

/// Rough chars→tokens: CJK denser than ASCII. Deterministic, not billing-accurate.
pub fn estimate_tokens(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    let mut tokens = 0u32;
    for ch in text.chars() {
        if ch <= '\u{007f}' {
            // ~4 chars per token for ASCII runs — count 1 per 4, min 1 per char group
            tokens = tokens.saturating_add(1);
        } else {
            // CJK / other: ~1–2 chars per token; count 1 per char
            tokens = tokens.saturating_add(1);
        }
    }
    // ASCII was over-counted 1:1; scale down Latin-heavy text
    let ascii = text.chars().filter(|c| *c <= '\u{007f}').count() as u32;
    let non_ascii = text.chars().count() as u32 - ascii;
    let ascii_tokens = (ascii + 3) / 4;
    ascii_tokens.saturating_add(non_ascii).max(1)
}

pub fn resolve_context_limit() -> u32 {
    if let Some(p) = active_profile() {
        if p.provider == "local" {
            return p.n_ctx.unwrap_or(DEFAULT_N_CTX);
        }
        if let Some(n) = p.n_ctx {
            return n.clamp(512, 32768);
        }
        return DEFAULT_CLOUD_CTX;
    }
    DEFAULT_N_CTX
}

pub fn estimate_messages_parts(msgs: &[ModelMessage], memory_tokens: u32) -> BudgetParts {
    let mut parts = BudgetParts {
        memory: memory_tokens,
        ..Default::default()
    };
    for (i, m) in msgs.iter().enumerate() {
        let t = estimate_tokens(&m.flatten_content());
        if i == 0 && m.role == "system" {
            // First system may include skills; attribute all to system for simplicity
            // unless content has skill markers — keep skills=0 unless caller splits.
            parts.system = parts.system.saturating_add(t);
            continue;
        }
        match m.role.as_str() {
            "system" => parts.system = parts.system.saturating_add(t),
            "tool" => parts.tool_results = parts.tool_results.saturating_add(t),
            _ => parts.messages = parts.messages.saturating_add(t),
        }
        // tool schemas not in msgs — tools part stays 0 unless set by caller
    }
    parts
}

pub fn build_budget(parts: BudgetParts, limit: u32, threshold: f64) -> ContextBudget {
    let used = parts.sum();
    let ratio = if limit == 0 {
        0.0
    } else {
        used as f64 / limit as f64
    };
    ContextBudget {
        limit,
        used,
        ratio,
        parts,
        near_limit: ratio >= threshold,
    }
}

pub fn budget_from_messages(
    msgs: &[ModelMessage],
    memory_tokens: u32,
    tools_json_tokens: u32,
) -> ContextBudget {
    let mut parts = estimate_messages_parts(msgs, memory_tokens);
    parts.tools = tools_json_tokens;
    build_budget(parts, resolve_context_limit(), DEFAULT_COMPRESS_THRESHOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_sum_and_ratio() {
        let parts = BudgetParts {
            system: 100,
            skills: 50,
            tools: 20,
            messages: 200,
            tool_results: 130,
            memory: 50,
        };
        assert_eq!(parts.sum(), 550);
        let b = build_budget(parts, 1000, 0.8);
        assert_eq!(b.used, 550);
        assert!((b.ratio - 0.55).abs() < 1e-9);
        assert!(!b.near_limit);
        let b2 = build_budget(
            BudgetParts {
                messages: 850,
                ..Default::default()
            },
            1000,
            0.8,
        );
        assert!(b2.near_limit);
    }

    #[test]
    fn estimate_nonempty() {
        assert!(estimate_tokens("hello world") > 0);
        assert!(estimate_tokens("你好世界") >= 4);
    }
}
