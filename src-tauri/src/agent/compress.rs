//! Session Runtime compression: truncate tool results, threshold batch compress, circuit breaker.

use crate::agent::context_budget::{
    estimate_tokens, ContextBudget, DEFAULT_COMPRESS_THRESHOLD, TOOL_RESULT_CHAR_BUDGET,
};
use crate::agent::llm::ModelMessage;

pub const COMPRESSED_MARKER: &str = "[COMPRESSED]";
pub const DEFAULT_COMPRESS_FAILURE_LIMIT: u32 = 3;

#[derive(Debug, Clone, Default)]
pub struct CompressState {
    pub consecutive_failures: u32,
    pub tripped: bool,
    pub last_compressed: bool,
}

impl CompressState {
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_compressed = true;
    }

    pub fn record_failure(&mut self, limit: u32) -> bool {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        if self.consecutive_failures >= limit {
            self.tripped = true;
        }
        self.tripped
    }

    pub fn reset_trip_if_allowed(&mut self) {
        // Keep tripped until process restart unless explicitly cleared.
    }
}

/// Truncate a single tool result for Runtime; full text stays in Audit/trajectory.
pub fn truncate_tool_result_for_runtime(full: &str, max_chars: usize) -> (String, bool) {
    if full.chars().count() <= max_chars {
        return (full.to_string(), false);
    }
    let preview: String = full.chars().take(max_chars).collect();
    let omitted = full.chars().count().saturating_sub(max_chars);
    (
        format!(
            "{preview}\n…[truncated {omitted} chars; full result kept in conversation audit]"
        ),
        true,
    )
}

fn is_compressed_tool(m: &ModelMessage) -> bool {
    m.role == "tool" && m.content.contains(COMPRESSED_MARKER)
}

/// Replace unmarked old tool results with a short summary when over threshold.
pub fn compress_runtime_tool_results(
    msgs: &mut [ModelMessage],
    budget: &ContextBudget,
    threshold: f64,
    state: &mut CompressState,
) -> Result<usize, String> {
    if state.tripped {
        return Err("context compression circuit breaker open".into());
    }
    if budget.ratio < threshold {
        return Ok(0);
    }

    let mut compressed = 0usize;
    for m in msgs.iter_mut() {
        if m.role != "tool" || is_compressed_tool(m) {
            continue;
        }
        if m.content.chars().count() <= 200 {
            continue;
        }
        let preview: String = m.content.chars().take(180).collect();
        m.content = format!(
            "{COMPRESSED_MARKER} summary: {preview}…"
        );
        compressed += 1;
    }

    if compressed == 0 && budget.ratio >= threshold {
        // Nothing to compress but still over — count as soft failure toward breaker
        if state.record_failure(DEFAULT_COMPRESS_FAILURE_LIMIT) {
            return Err("context compression circuit breaker open".into());
        }
        return Ok(0);
    }

    if compressed > 0 {
        state.record_success();
    }
    Ok(compressed)
}

/// Apply L1 truncate to the latest tool message content before push.
pub fn prepare_tool_result_runtime(full: &str) -> String {
    truncate_tool_result_for_runtime(full, TOOL_RESULT_CHAR_BUDGET).0
}

pub fn should_compress(budget: &ContextBudget) -> bool {
    budget.ratio >= DEFAULT_COMPRESS_THRESHOLD
}

/// Recompute budget after mutation (parts from msgs only).
pub fn recompute_budget(msgs: &[ModelMessage], limit: u32, memory_tokens: u32) -> ContextBudget {
    use crate::agent::context_budget::{build_budget, estimate_messages_parts};
    let parts = estimate_messages_parts(msgs, memory_tokens);
    build_budget(parts, limit, DEFAULT_COMPRESS_THRESHOLD)
}

#[allow(dead_code)]
pub fn estimate_tool_slice(msgs: &[ModelMessage]) -> u32 {
    msgs.iter()
        .filter(|m| m.role == "tool")
        .map(|m| estimate_tokens(&m.content))
        .fold(0u32, |a, b| a.saturating_add(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::context_budget::BudgetParts;

    #[test]
    fn truncate_keeps_short() {
        let (out, truncated) = truncate_tool_result_for_runtime("hi", 100);
        assert_eq!(out, "hi");
        assert!(!truncated);
    }

    #[test]
    fn truncate_long() {
        let long = "x".repeat(5000);
        let (out, truncated) = truncate_tool_result_for_runtime(&long, 100);
        assert!(truncated);
        assert!(out.len() < long.len());
        assert!(out.contains("truncated"));
    }

    #[test]
    fn no_compress_below_threshold() {
        let mut msgs = vec![ModelMessage::tool("y".repeat(500))];
        let budget = ContextBudget {
            limit: 10_000,
            used: 100,
            ratio: 0.01,
            parts: BudgetParts::default(),
            near_limit: false,
        };
        let mut state = CompressState::default();
        let n = compress_runtime_tool_results(&mut msgs, &budget, 0.8, &mut state).unwrap();
        assert_eq!(n, 0);
        assert!(!msgs[0].content.contains(COMPRESSED_MARKER));
    }

    #[test]
    fn batch_compress_above_threshold() {
        let mut msgs = vec![
            ModelMessage::system("sys"),
            ModelMessage::tool("z".repeat(500)),
            ModelMessage::tool("w".repeat(500)),
        ];
        let budget = ContextBudget {
            limit: 1000,
            used: 900,
            ratio: 0.9,
            parts: BudgetParts::default(),
            near_limit: true,
        };
        let mut state = CompressState::default();
        let n = compress_runtime_tool_results(&mut msgs, &budget, 0.8, &mut state).unwrap();
        assert_eq!(n, 2);
        assert!(msgs[1].content.contains(COMPRESSED_MARKER));
        assert!(msgs[2].content.contains(COMPRESSED_MARKER));
        // system untouched
        assert_eq!(msgs[0].content, "sys");
    }

    #[test]
    fn circuit_breaker_trips() {
        let mut state = CompressState::default();
        let budget = ContextBudget {
            limit: 100,
            used: 99,
            ratio: 0.99,
            parts: BudgetParts::default(),
            near_limit: true,
        };
        // Only short tool msgs → compress count 0 → failures accumulate
        let mut msgs = vec![ModelMessage::tool("ok")];
        for _ in 0..DEFAULT_COMPRESS_FAILURE_LIMIT {
            let _ = compress_runtime_tool_results(&mut msgs, &budget, 0.8, &mut state);
        }
        assert!(state.tripped);
        let err = compress_runtime_tool_results(&mut msgs, &budget, 0.8, &mut state);
        assert!(err.is_err());
    }
}
