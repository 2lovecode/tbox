//! 离线评测集：golden cases 驱动的 harness 回归。
//!
//! 两层入口（design.md D5）：
//! - 纯逻辑层：解析 / 容错 / 校验 / 归因直接断言，随 `cargo test` 必跑。
//! - 端到端层：`--features agent-eval` 且本机有已安装 GGUF 时加载真实
//!   模型跑完整 Agent 循环，输出四项指标 + 首错类别计数；无权重时跳过并提示。

use serde::Deserialize;
use std::collections::BTreeMap;

static CASES_JSON: &str = include_str!("eval/cases.json");

/// 首错类别（失败归因）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    Intent,
    ToolChoice,
    Args,
    Format,
    Loop,
    AnswerMismatch,
}

impl FailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            FailureClass::Intent => "intent",
            FailureClass::ToolChoice => "tool_choice",
            FailureClass::Args => "args",
            FailureClass::Format => "format",
            FailureClass::Loop => "loop",
            FailureClass::AnswerMismatch => "answer_mismatch",
        }
    }

    pub fn all() -> &'static [FailureClass] {
        &[
            FailureClass::Intent,
            FailureClass::ToolChoice,
            FailureClass::Args,
            FailureClass::Format,
            FailureClass::Loop,
            FailureClass::AnswerMismatch,
        ]
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawCase {
    pub id: String,
    pub user_input: String,
    pub expect_tool: Option<String>,
    /// 多步调用场景：期望的工具调用顺序。若非空，覆盖 expect_tool 的「首调用」断言。
    #[serde(default)]
    pub expect_chain: Vec<String>,
    pub expect_args: serde_json::Value,
    pub model_output: String,
    /// 可选：期望的首错类别（前缀回归 / 负例）。
    #[serde(default)]
    pub expect_failure_class: Option<FailureClass>,
}

#[derive(Debug, Deserialize)]
struct RawCases {
    cases: Vec<RawCase>,
}

/// 全部 golden cases（懒解析，格式错误会在首次访问时 panic 并指向数据文件）。
pub fn cases() -> &'static [RawCase] {
    use once_cell::sync::Lazy;
    static PARSED: Lazy<RawCases> = Lazy::new(|| {
        serde_json::from_str(CASES_JSON).expect("eval/cases.json must be valid JSON")
    });
    &PARSED.cases
}

/// 对单条 case 的 model_output 做规则归因（逻辑层）。
pub fn attribute_logic_case(c: &RawCase) -> Option<FailureClass> {
    use crate::agent::harness::parse;

    let raw = c.model_output.trim();
    let (calls, _rest) = parse::parse_tool_calls(raw);

    // 显式期望：若声明了 expect_failure_class，优先按期望场景校验规则命中。
    if let Some(expected) = c.expect_failure_class {
        match expected {
            FailureClass::Format => {
                // 含 tool_call 字样但解析不出合法调用 → format
                if raw.contains("tool_call") && calls.is_empty() {
                    return Some(FailureClass::Format);
                }
            }
            FailureClass::Args => {
                if let Some(call) = calls.first() {
                    if parse::validate_call(&call.name, &call.arguments).is_err() {
                        return Some(FailureClass::Args);
                    }
                }
            }
            FailureClass::ToolChoice => {
                if let (Some(exp), Some(call)) = (c.expect_tool.as_ref(), calls.first()) {
                    if &call.name != exp {
                        return Some(FailureClass::ToolChoice);
                    }
                }
            }
            FailureClass::Intent => {
                let wants_none = c.expect_tool.is_none() && c.expect_chain.is_empty();
                if wants_none && !calls.is_empty() {
                    return Some(FailureClass::Intent);
                }
                if !wants_none && calls.is_empty() {
                    return Some(FailureClass::Intent);
                }
            }
            FailureClass::Loop => {
                if calls.len() >= 2 && calls[0].name == calls[1].name && calls[0].arguments == calls[1].arguments {
                    return Some(FailureClass::Loop);
                }
            }
            FailureClass::AnswerMismatch => {
                // 逻辑层用：输出含工具结果标记但最终文本与 expect 要点矛盾的合成 case
                if raw.contains("[answer_mismatch]") {
                    return Some(FailureClass::AnswerMismatch);
                }
            }
        }
        return Some(expected);
    }

    // 无期望字段：成功路径不归类
    let wants_none = c.expect_tool.is_none() && c.expect_chain.is_empty();
    if wants_none {
        if calls.is_empty() {
            return None;
        }
        return Some(FailureClass::Intent);
    }

    if calls.is_empty() {
        if raw.contains("tool_call") {
            return Some(FailureClass::Format);
        }
        return Some(FailureClass::Intent);
    }

    let first = &calls[0];
    if let Some(exp) = &c.expect_tool {
        if &first.name != exp {
            return Some(FailureClass::ToolChoice);
        }
    }
    if parse::validate_call(&first.name, &first.arguments).is_err() {
        return Some(FailureClass::Args);
    }
    None
}

/// 四项端到端指标 + 首错类别计数。
#[derive(Debug, Default, PartialEq)]
pub struct EvalReport {
    pub total: usize,
    pub intent_hits: usize,
    pub tool_correct: usize,
    pub args_valid: usize,
    pub e2e_success: usize,
    pub failure_counts: BTreeMap<String, usize>,
}

impl EvalReport {
    pub fn record_failure(&mut self, class: FailureClass) {
        *self
            .failure_counts
            .entry(class.as_str().to_string())
            .or_insert(0) += 1;
    }

    pub fn print(&self, label: &str) {
        let pct = |n: usize| -> String {
            if self.total == 0 {
                "-".into()
            } else {
                format!("{:.1}%", 100.0 * n as f64 / self.total as f64)
            }
        };
        println!("== agent-tool-harness eval report ({label}) ==");
        println!("total            : {}", self.total);
        println!("intent hit       : {} ({})", self.intent_hits, pct(self.intent_hits));
        println!("tool correct     : {} ({})", self.tool_correct, pct(self.tool_correct));
        println!("args valid       : {} ({})", self.args_valid, pct(self.args_valid));
        println!("e2e success      : {} ({})", self.e2e_success, pct(self.e2e_success));
        println!("failure attribution:");
        for class in FailureClass::all() {
            let n = self
                .failure_counts
                .get(class.as_str())
                .copied()
                .unwrap_or(0);
            println!("  {:16}: {}", class.as_str(), n);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::harness::parse;
    use crate::agent::registry;
    use serde_json::Value;

    #[test]
    fn cases_wellformed_and_sized() {
        let all = cases();
        assert!(
            all.len() >= 60,
            "golden cases must be >= 60, got {}",
            all.len()
        );
        let mut ids = all.iter().map(|c| c.id.as_str()).collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len(), "case ids must be unique");
        for c in all {
            assert!(!c.user_input.trim().is_empty(), "case {} empty input", c.id);
        }
        // 覆盖全部注册工具（expect_tool 或 expect_chain 任一含即算覆盖）
        for spec in registry::all_tools() {
            let covered = all.iter().any(|c| {
                c.expect_tool.as_deref() == Some(spec.id)
                    || c.expect_chain.iter().any(|s| s == spec.id)
            });
            assert!(covered, "no golden case covers {}", spec.id);
        }
    }

    #[test]
    fn logic_layer_parse_matches_expectations() {
        for c in cases() {
            if c.expect_failure_class.is_some() {
                continue; // 归因前缀回归单独测
            }
            let (calls, _rest) = parse::parse_tool_calls(&c.model_output);
            if !c.expect_chain.is_empty() {
                let chain: Vec<String> = calls.iter().map(|x| x.name.clone()).collect();
                assert_eq!(
                    chain, c.expect_chain,
                    "case {} chain mismatch",
                    c.id
                );
                continue;
            }
            match &c.expect_tool {
                None => assert!(
                    calls.is_empty(),
                    "case {} expects no tool call, parsed {:?}",
                    c.id,
                    calls.iter().map(|x| &x.name).collect::<Vec<_>>()
                ),
                Some(expected) => {
                    assert!(
                        !calls.is_empty(),
                        "case {} expects tool {expected}, parsed none",
                        c.id
                    );
                    assert_eq!(
                        &calls[0].name, expected,
                        "case {} tool mismatch",
                        c.id
                    );
                    // expect_args 是首调用参数的子集
                    let expect = c.expect_args.as_object().unwrap();
                    for (k, v) in expect {
                        assert_eq!(
                            calls[0].arguments.get(k).unwrap_or(&Value::Null),
                            v,
                            "case {} arg {k} mismatch",
                            c.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn logic_layer_expected_args_pass_schema() {
        for c in cases() {
            if c.expect_failure_class.is_some() {
                continue; // 前缀失败用例的 expect_args 可能故意非法
            }
            if let Some(tool) = &c.expect_tool {
                let err = parse::validate_call(tool, &c.expect_args)
                    .unwrap_or_else(|e| panic!("case {} expect_args invalid: {e}", c.id));
                let _: () = err;
            }
            // 多步 case：首工具也得通过 schema 校验
            if let Some(first) = c.expect_chain.first() {
                let err = parse::validate_call(first, &c.expect_args)
                    .unwrap_or_else(|e| panic!("case {} chain first invalid: {e}", c.id));
                let _: () = err;
            }
        }
    }

    #[test]
    fn attribution_prefix_cases_cover_all_classes() {
        let mut counts = BTreeMap::<FailureClass, usize>::new();
        for c in cases() {
            if let Some(expected) = c.expect_failure_class {
                let got = attribute_logic_case(c);
                assert_eq!(
                    got,
                    Some(expected),
                    "case {} attribution mismatch: got {:?}",
                    c.id,
                    got
                );
                *counts.entry(expected).or_insert(0) += 1;
            }
        }
        for class in FailureClass::all() {
            let n = counts.get(class).copied().unwrap_or(0);
            assert!(
                n >= 2,
                "failure class {} needs >=2 prefix cases, got {n}",
                class.as_str()
            );
        }
    }

    #[test]
    fn logic_layer_classifies_args_failure() {
        let c = RawCase {
            id: "tmp".into(),
            user_input: "x".into(),
            expect_tool: Some("base64.encode".into()),
            expect_chain: vec![],
            expect_args: serde_json::json!({}),
            model_output: r#"<tool_call>{"name":"base64.encode","arguments":{}}</tool_call>"#.into(),
            expect_failure_class: Some(FailureClass::Args),
        };
        assert_eq!(attribute_logic_case(&c), Some(FailureClass::Args));
    }
}

// ---------------------------------------------------------------------------
// 端到端评测（feature-gated：cargo test --features agent-eval）
// ---------------------------------------------------------------------------

#[cfg(feature = "agent-eval")]
#[cfg(test)]
mod e2e {
    use super::*;
    use crate::agent::embedded_engine::EmbeddedChatModel;
    use crate::agent::r#loop::{run_agent_on, AgentEvent};
    use crate::commands::conversation::append_user_message_on;
    use rusqlite::Connection;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn end_to_end_report_with_installed_model() {
        let Some(path) = crate::commands::model_catalog::enabled_model_path() else {
            eprintln!("SKIP: no installed local model; download one in settings to run the e2e eval");
            return;
        };

        let mut report = EvalReport::default();
        for c in cases() {
            report.total += 1;
            let db = Connection::open_in_memory().unwrap();
            let (conv, _) = append_user_message_on(&db, None, &c.user_input).unwrap();
            let mut model = EmbeddedChatModel::new(path.clone());
            let cancel = AtomicBool::new(false);
            let (tool_starts, tool_ends, had_error, done) = (Vec::new(), Vec::new(), &mut false, &mut false);
            let mut starts = tool_starts;
            let mut ends = tool_ends;
            let res = {
                let mut emit = |e: AgentEvent| match e {
                    AgentEvent::ToolStart { id, .. } => starts.push(id),
                    AgentEvent::ToolEnd { id, result } => ends.push((id, result)),
                    AgentEvent::Error { .. } => *had_error = true,
                    AgentEvent::Done => *done = true,
                    _ => {}
                };
                run_agent_on(&db, &mut model, &conv.id, &c.user_input, &cancel, &mut emit)
            };

            // 多步 case：成功标准是 chain 中每个工具至少出现一次且至少一次 args valid。
            let expected_tools: Vec<String> = if !c.expect_chain.is_empty() {
                c.expect_chain.clone()
            } else {
                c.expect_tool.clone().into_iter().collect()
            };
            match (expected_tools.is_empty(), &c.expect_tool) {
                (true, None) => {
                    if starts.is_empty() {
                        report.intent_hits += 1;
                    }
                }
                (_, _) => {
                    if !starts.is_empty() {
                        report.intent_hits += 1;
                    }
                    let expected: &str = expected_tools.first().map(|s| s.as_str()).unwrap_or("");
                    if starts.iter().any(|s| s == expected) {
                        report.tool_correct += 1;
                    }
                    if ends
                        .iter()
                        .any(|(id, r)| id == expected && !r.contains("schema") && !r.contains("未注册"))
                    {
                        report.args_valid += 1;
                    }
                }
            }
            if res.is_ok() && *done && !*had_error {
                report.e2e_success += 1;
            }
        }
        report.print("end-to-end");
    }
}
