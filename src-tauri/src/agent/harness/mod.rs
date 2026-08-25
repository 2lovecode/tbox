//! 可插拔 Agent harness 策略层：所有后端的对话回合统一经由这里
//! 构建系统提示、解析模型输出、执行校验与修复重试（reask）。
//!
//! 档位（见 openspec/changes/agent-tool-harness/design.md D1）：
//! - `SmallLocalStrategy`：embedded 小模型（目录内 0.5B/1.5B）——强化提示、
//!   修复预算 2、默认请求约束解码。
//! - `DefaultStrategy`：云端 / Ollama ——简洁提示、修复预算 1、无约束解码，
//!   但共享同一套容错解析与 schema 校验。

pub mod eval;
pub mod grammar;
pub mod parse;
pub mod prompt;

use crate::agent::llm::ToolCall;

/// 一次解析后的模型回合：工具调用 + 剩余纯文本。
#[derive(Debug, Clone, Default)]
pub struct ParsedTurn {
    pub calls: Vec<ToolCall>,
    pub text: String,
}

/// 可插拔 harness 策略。
pub trait HarnessStrategy {
    fn name(&self) -> &'static str;
    /// 构建本轮系统提示（含 Skill 检索与工具说明）。
    fn build_system_prompt(&self, user_text: &str) -> String;
    /// 容错解析模型输出。
    fn parse_completion(&self, text: &str) -> ParsedTurn {
        let (calls, text) = parse::parse_tool_calls(text);
        ParsedTurn { calls, text }
    }
    /// 修复重试（reask）预算：连续失败回合的上限。
    fn repair_budget(&self) -> usize;
    /// 是否请求约束解码（仅 embedded 引擎支持，其他后端忽略）。
    fn constrained(&self) -> bool;
}

/// embedded 小模型强化策略。
pub struct SmallLocalStrategy;

impl HarnessStrategy for SmallLocalStrategy {
    fn name(&self) -> &'static str {
        "small-local"
    }

    fn build_system_prompt(&self, user_text: &str) -> String {
        prompt::build_small_prompt(user_text)
    }

    fn repair_budget(&self) -> usize {
        2
    }

    // 约束解码默认关闭：实测 0.5B + grammar 后意图命中率从 ~63% 跌至 ~12%
    // （语法迫使模型过早进 tool_call 形态，破坏自由回答能力）。
    // 保留能力与 `TBOX_DISABLE_TOOL_GRAMMAR=1` 反向开关（打开 grammar），
    // 待更大模型/更优语法时再默认启用——见 design.md D3 / 任务 4.3 备注。
    fn constrained(&self) -> bool {
        false
    }
}

/// 云端 / Ollama 默认策略（提示与旧版等价，解析容错共享）。
pub struct DefaultStrategy;

impl HarnessStrategy for DefaultStrategy {
    fn name(&self) -> &'static str {
        "default"
    }

    fn build_system_prompt(&self, user_text: &str) -> String {
        prompt::build_default_prompt(user_text)
    }

    fn repair_budget(&self) -> usize {
        1
    }

    fn constrained(&self) -> bool {
        false
    }
}

/// 按后端与模型名选择策略：embedded 引擎一律小模型强化档，
/// 其余（genai / 云端 / Ollama）走默认档。
pub fn strategy_for(backend: &str, _model: &str) -> Box<dyn HarnessStrategy> {
    if backend == "embedded" {
        Box::new(SmallLocalStrategy)
    } else {
        Box::new(DefaultStrategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_backend_gets_small_strategy() {
        let s = strategy_for("embedded", "qwen2.5-0.5b-instruct-q4_k_m");
        assert_eq!(s.name(), "small-local");
        assert_eq!(s.repair_budget(), 2);
        // constrained 默认 false：4.3 实测 0.5B + grammar 意图率从 63% 跌至 12%。
        assert!(!s.constrained());
    }

    #[test]
    fn other_backends_get_default() {
        for backend in ["genai", "openai", "ollama", "default"] {
            let s = strategy_for(backend, "gpt-4o-mini");
            assert_eq!(s.name(), "default");
            assert_eq!(s.repair_budget(), 1);
            assert!(!s.constrained());
        }
    }

    #[test]
    fn strategies_share_tolerant_parse() {
        let embedded = strategy_for("embedded", "m");
        let default = strategy_for("openai", "gpt");
        for s in [embedded.as_ref(), default.as_ref()] {
            let turn = s.parse_completion(
                "```json\n<tool_call>{\"name\": \"uuid.generate\", \"arguments\": {}}</tool_call>\n```",
            );
            assert_eq!(turn.calls.len(), 1);
            assert_eq!(turn.calls[0].name, "uuid.generate");
        }
    }
}
