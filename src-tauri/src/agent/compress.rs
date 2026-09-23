//! Session Runtime compression: strategy waterfall + circuit breaker.
//!
//! Order when over threshold: keep_recent → summarize_tools → reset.
//! Leading system prompt is always preserved.

use crate::agent::context_budget::{
    estimate_tokens, ContextBudget, DEFAULT_COMPRESS_THRESHOLD, TOOL_RESULT_CHAR_BUDGET,
};
use crate::agent::llm::ModelMessage;

pub const COMPRESSED_MARKER: &str = "[COMPRESSED]";
pub const DEFAULT_COMPRESS_FAILURE_LIMIT: u32 = 3;
/// Keep this many non-system messages from the tail (strategy keep_recent).
pub const KEEP_RECENT_NON_SYSTEM: usize = 12;
/// Never summarize the newest tool result — it is often the answer the model
/// must quote on the next turn. Compressing it caused the model to echo
/// `[COMPRESSED] summary: …` (truncated JSON) to the user.
pub const KEEP_RECENT_TOOLS_FULL: usize = 1;

pub const STRATEGY_KEEP_RECENT: &str = "keep_recent";
pub const STRATEGY_SUMMARIZE_TOOLS: &str = "summarize_tools";
pub const STRATEGY_RESET: &str = "reset";

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
}

#[derive(Debug, Clone, Default)]
pub struct CompressOutcome {
    pub strategies_applied: Vec<String>,
    /// Messages dropped or tool results summarized.
    pub count: usize,
    pub last_strategy: Option<String>,
}

impl CompressOutcome {
    pub fn is_empty(&self) -> bool {
        self.strategies_applied.is_empty()
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

fn has_tool_calls(m: &ModelMessage) -> bool {
    !m.tool_calls.is_empty()
}

/// Waterfall: keep_recent → summarize_tools → reset. Enough after any step → stop.
pub fn apply_context_strategies(
    msgs: &mut Vec<ModelMessage>,
    budget: &ContextBudget,
    threshold: f64,
    state: &mut CompressState,
    user_text: &str,
    memory_tokens: u32,
) -> Result<CompressOutcome, String> {
    if state.tripped {
        return Err("context compression circuit breaker open".into());
    }
    if budget.ratio < threshold {
        return Ok(CompressOutcome::default());
    }

    let limit = budget.limit;
    let mut outcome = CompressOutcome::default();

    // 1) keep_recent
    let dropped = strategy_keep_recent(msgs, KEEP_RECENT_NON_SYSTEM);
    if dropped > 0 {
        outcome.strategies_applied.push(STRATEGY_KEEP_RECENT.into());
        outcome.count = outcome.count.saturating_add(dropped);
        outcome.last_strategy = Some(STRATEGY_KEEP_RECENT.into());
        let current = recompute_budget(msgs, limit, memory_tokens);
        if current.ratio < threshold {
            state.record_success();
            return Ok(outcome);
        }
    }

    // 2) summarize_tools
    let summarized = strategy_summarize_tools(msgs);
    if summarized > 0 {
        outcome.strategies_applied.push(STRATEGY_SUMMARIZE_TOOLS.into());
        outcome.count = outcome.count.saturating_add(summarized);
        outcome.last_strategy = Some(STRATEGY_SUMMARIZE_TOOLS.into());
        let current = recompute_budget(msgs, limit, memory_tokens);
        if current.ratio < threshold {
            state.record_success();
            return Ok(outcome);
        }
    }

    // 3) reset
    let discarded = strategy_reset(msgs, user_text);
    if discarded > 0 {
        outcome.strategies_applied.push(STRATEGY_RESET.into());
        outcome.count = outcome.count.saturating_add(discarded);
        outcome.last_strategy = Some(STRATEGY_RESET.into());
        state.record_success();
        return Ok(outcome);
    }

    // Nothing helped
    if state.record_failure(DEFAULT_COMPRESS_FAILURE_LIMIT) {
        return Err("context compression circuit breaker open".into());
    }
    Ok(outcome)
}

/// Drop older non-system messages; keep system + last `keep` non-system (pair-safe).
fn strategy_keep_recent(msgs: &mut Vec<ModelMessage>, keep: usize) -> usize {
    if msgs.is_empty() {
        return 0;
    }
    let system = msgs
        .iter()
        .find(|m| m.role == "system")
        .cloned()
        .unwrap_or_else(|| ModelMessage::system(""));
    let rest: Vec<ModelMessage> = msgs.iter().filter(|m| m.role != "system").cloned().collect();
    if rest.len() <= keep {
        return 0;
    }

    let mut cut = rest.len() - keep;
    // If window would start mid tool-pair, include the preceding assistant(tool_calls).
    if cut > 0 && rest[cut].role == "tool" {
        if rest[cut - 1].role == "assistant" && has_tool_calls(&rest[cut - 1]) {
            cut -= 1;
        } else {
            // Orphan tools at window start — skip them toward newer messages.
            while cut < rest.len() && rest[cut].role == "tool" {
                cut += 1;
            }
        }
    }

    let dropped = cut.min(rest.len());
    let kept_tail: Vec<ModelMessage> = rest[dropped..].to_vec();
    msgs.clear();
    msgs.push(system);
    msgs.extend(kept_tail);
    dropped
}

fn strategy_summarize_tools(msgs: &mut [ModelMessage]) -> usize {
    let tool_indices: Vec<usize> = msgs
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == "tool")
        .map(|(i, _)| i)
        .collect();
    let protect: std::collections::HashSet<usize> = tool_indices
        .iter()
        .rev()
        .take(KEEP_RECENT_TOOLS_FULL)
        .copied()
        .collect();

    let mut compressed = 0usize;
    for (i, m) in msgs.iter_mut().enumerate() {
        if m.role != "tool" || is_compressed_tool(m) || protect.contains(&i) {
            continue;
        }
        if m.content.chars().count() <= 200 {
            continue;
        }
        let orig_chars = m.content.chars().count();
        // Do NOT paste a truncated JSON/body preview — models copy it into the
        // user-facing reply and produce broken output.
        m.content = format!(
            "{COMPRESSED_MARKER} prior tool result omitted ({orig_chars} chars). \
             Full text remains in conversation audit. Do not quote this stub to the user; \
             re-call the tool if you still need the payload."
        );
        compressed += 1;
    }
    compressed
}

fn strategy_reset(msgs: &mut Vec<ModelMessage>, user_text: &str) -> usize {
    let discarded = msgs.iter().filter(|m| m.role != "system").count();
    if discarded == 0 {
        return 0;
    }
    let system = msgs
        .iter()
        .find(|m| m.role == "system")
        .cloned()
        .unwrap_or_else(|| ModelMessage::system(""));
    msgs.clear();
    msgs.push(system);
    msgs.push(ModelMessage::user(user_text));
    discarded
}

/// Apply L1 truncate to the latest tool message content before push.
pub fn prepare_tool_result_runtime(full: &str) -> String {
    truncate_tool_result_for_runtime(full, TOOL_RESULT_CHAR_BUDGET).0
}

pub fn should_compress(budget: &ContextBudget) -> bool {
    budget.ratio >= DEFAULT_COMPRESS_THRESHOLD
}

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
    use crate::agent::llm::ToolCall;
    use serde_json::json;

    fn hot_budget() -> ContextBudget {
        ContextBudget {
            limit: 1000,
            used: 900,
            ratio: 0.9,
            parts: BudgetParts::default(),
            near_limit: true,
        }
    }

    fn cool_budget() -> ContextBudget {
        ContextBudget {
            limit: 10_000,
            used: 100,
            ratio: 0.01,
            parts: BudgetParts::default(),
            near_limit: false,
        }
    }

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
    fn no_op_below_threshold() {
        let mut msgs = vec![
            ModelMessage::system("sys"),
            ModelMessage::user("hi"),
            ModelMessage::tool("y".repeat(500)),
        ];
        let mut state = CompressState::default();
        let out = apply_context_strategies(&mut msgs, &cool_budget(), 0.8, &mut state, "hi", 0)
            .unwrap();
        assert!(out.is_empty());
        assert_eq!(msgs.len(), 3);
    }

    #[test]
    fn keep_recent_drops_old_keeps_system() {
        let mut msgs = vec![ModelMessage::system("sys")];
        for i in 0..20 {
            msgs.push(ModelMessage::user(format!("u{i}")));
        }
        // Direct keep_recent path (short messages; waterfall may also reset under hot budget)
        let dropped = strategy_keep_recent(&mut msgs, 5);
        assert_eq!(dropped, 15);
        assert_eq!(msgs[0].role, "system");
        assert_eq!(msgs.len(), 6); // system + 5
        assert_eq!(msgs[1].content, "u15");
    }

    #[test]
    fn waterfall_summarize_when_keep_not_enough() {
        // Few messages but huge tools → keep_recent no-op, summarize applies
        let mut msgs = vec![
            ModelMessage::system("sys"),
            ModelMessage::user("hi"),
            ModelMessage::tool("z".repeat(5000)),
            ModelMessage::tool("w".repeat(5000)),
        ];
        let mut state = CompressState::default();
        let out =
            apply_context_strategies(&mut msgs, &hot_budget(), 0.8, &mut state, "hi", 0).unwrap();
        assert!(out.strategies_applied.contains(&STRATEGY_SUMMARIZE_TOOLS.to_string())
            || out.strategies_applied.contains(&STRATEGY_RESET.to_string()));
        assert_eq!(msgs[0].content, "sys");
    }

    #[test]
    fn summarize_tools_keeps_newest_full_and_omits_preview() {
        let old = format!("{{\"old\":{}}}", "x".repeat(400));
        let newest = format!("{{\"newest\":{}}}", "z".repeat(400));
        let mut msgs = vec![
            ModelMessage::system("sys"),
            ModelMessage::user("hi"),
            ModelMessage::tool(old.clone()),
            ModelMessage::tool(newest.clone()),
        ];
        let n = strategy_summarize_tools(&mut msgs);
        assert_eq!(n, 1, "older tool compresses; newest stays full");
        assert!(msgs[2].content.contains(COMPRESSED_MARKER));
        assert!(
            !msgs[2].content.contains("\"old\""),
            "stub must not embed truncated JSON body"
        );
        assert_eq!(msgs[3].content, newest, "newest kept full");
    }

    #[test]
    fn summarize_tools_protects_only_newest() {
        let a = "a".repeat(500);
        let mut msgs = vec![
            ModelMessage::system("sys"),
            ModelMessage::user("hi"),
            ModelMessage::tool(a.clone()),
        ];
        let n = strategy_summarize_tools(&mut msgs);
        assert_eq!(n, 0, "sole tool is newest and stays full");
        assert_eq!(msgs[2].content, a);
    }

    #[test]
    fn reset_last_resort() {
        let mut msgs = vec![
            ModelMessage::system("sys"),
            ModelMessage::user("old"),
            ModelMessage::assistant_tool_calls(vec![ToolCall {
                id: "c1".into(),
                name: "t".into(),
                arguments: json!({}),
            }]),
            ModelMessage::tool_result("c1", "t", "ok"),
        ];
        let discarded = strategy_reset(&mut msgs, "new ask");
        assert_eq!(discarded, 3);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].content, "sys");
        assert_eq!(msgs[1].content, "new ask");
    }

    #[test]
    fn circuit_breaker_trips() {
        let mut state = CompressState::default();
        let mut msgs = vec![ModelMessage::system("sys")];
        for _ in 0..DEFAULT_COMPRESS_FAILURE_LIMIT {
            let _ = apply_context_strategies(&mut msgs, &hot_budget(), 0.8, &mut state, "u", 0);
        }
        assert!(state.tripped);
        let err = apply_context_strategies(&mut msgs, &hot_budget(), 0.8, &mut state, "u", 0);
        assert!(err.is_err());
    }
}
