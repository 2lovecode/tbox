//! 提示工程：注册表工具的中文单行摘要 + 小模型四段式系统提示。

use crate::agent::registry::{self, ToolSpec};

/// 小模型系统提示字符预算（超出即测试告警断言）。
/// 0.5B 上下文 4096 token，中文约 1 字 ≈ 1 token，留一半给对话与工具结果。
pub const SMALL_PROMPT_CHAR_BUDGET: usize = 6000;
/// 轻量档更紧的预算。
pub const LITE_PROMPT_CHAR_BUDGET: usize = 4500;

fn type_label(t: &str) -> &'static str {
    match t {
        "string" => "字符串",
        "integer" => "整数",
        "number" => "数字",
        "boolean" => "布尔",
        "object" => "对象",
        "array" => "数组",
        _ => "任意",
    }
}

/// 单个工具的单行中文摘要：`- id：用途｜参数：名(类型,必填)：说明`。
pub fn render_tool_line(spec: &ToolSpec) -> String {
    let mut params = String::new();
    if let Some(props) = spec.schema.pointer("/properties").and_then(|v| v.as_object()) {
        let required: Vec<String> = spec
            .schema
            .pointer("/required")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|r| r.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let mut parts = Vec::new();
        for (name, p) in props {
            let typ = p.get("type").and_then(|t| t.as_str()).unwrap_or("any");
            let req = if required.iter().any(|r| r == name) {
                "必填"
            } else {
                "可选"
            };
            let desc = p.get("description").and_then(|d| d.as_str()).unwrap_or("");
            let enum_note = p
                .get("enum")
                .and_then(|e| e.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("/")
                })
                .map(|e| format!("，取值 {e}"))
                .unwrap_or_default();
            parts.push(format!("{name}({typ_label},{req})：{desc}{enum_note}", typ_label = type_label(typ)));
        }
        params = parts.join("；");
    }
    if params.is_empty() {
        format!("- {}：{}（无参数）", spec.id, spec.name)
    } else {
        format!("- {}：{}｜参数：{}", spec.id, spec.name, params)
    }
}

/// 全部注册工具的摘要清单（与 tools_as_openai_json 同源：同一注册表）。
pub fn render_tool_summary() -> String {
    registry::all_tools()
        .iter()
        .map(render_tool_line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// 小模型少样本示例：覆盖编码类与计算类意图，加一个无需工具的负例。
fn few_shot_block() -> String {
    [
        "示例 1：",
        "用户：帮我把 hi 编成 Base64",
        "助手：<tool_call>{\"name\": \"base64.encode\", \"arguments\": {\"input\": \"hi\"}}</tool_call>",
        "工具结果：aGk=",
        "助手：hi 的 Base64 编码是 aGk=",
        "",
        "示例 2：",
        "用户：算一下 hi 的 md5",
        "助手：<tool_call>{\"name\": \"hash.digest\", \"arguments\": {\"input\": \"hi\", \"algorithm\": \"md5\"}}</tool_call>",
        "工具结果：764efa883dda1e11db47671c4a3bbd9e",
        "助手：hi 的 md5 摘要是 764efa883dda1e11db47671c4a3bbd9e",
        "",
        "示例 3：",
        "用户：把 {\"aa\":\"bb\"} 转成 query string",
        "助手：<tool_call>{\"name\": \"json.to_query\", \"arguments\": {\"input\": \"{\\\"aa\\\":\\\"bb\\\"}\"}}</tool_call>",
        "工具结果：aa=bb",
        "助手：转换结果是 aa=bb",
        "",
        "示例 4（无需工具时直接回答，不要输出 tool_call）：",
        "用户：你好",
        "助手：你好！我是 TBox 工具助手，可以帮你做编码、哈希、格式化等计算任务。",
        "",
        "示例 5（被问有哪些功能时，简短概括，不要罗列全部工具清单，更不要重复同一个词）：",
        "用户：你有什么功能",
        "助手：我可以做这些：JSON/Base64/XML/YAML 处理、编码转换、哈希计算、JWT 解析、时间戳转换、UUID 生成、Cron 表达式说明、数制转换等。直接告诉我你的需求即可。",
        "",
        "示例 6（问当前时间：input 必须为 now，禁止编造日期）：",
        "用户：看下当前时间",
        "助手：<tool_call>{\"name\": \"timestamp.convert\", \"arguments\": {\"input\": \"now\"}}</tool_call>",
        "工具结果：{\"iso\":\"2026-09-23T05:20:34+00:00\",\"unix_seconds\":1758600034,\"unix_millis\":1758600034000}",
        "助手：当前 UTC 时间是 2026-09-23T05:20:34+00:00（Unix 1758600034）。",
        "",
        "示例 7（JSON 转义/反转义/美化：必须用 json.format，禁止 charset.convert）：",
        "用户：转义下 {\\\"a\\\":1}",
        "助手：<tool_call>{\"name\": \"json.format\", \"arguments\": {\"input\": \"{\\\"a\\\":1}\"}}</tool_call>",
        "工具结果：{\n  \"a\": 1\n}",
        "助手：格式化结果：\n{\n  \"a\": 1\n}",
    ]
    .join("\n")
}

/// 轻量档少样本：保留 1 个正例 + 1 个负例。
fn lite_few_shot_block() -> String {
    [
        "示例 1：",
        "用户：帮我把 hi 编成 Base64",
        "助手：<tool_call>{\"name\": \"base64.encode\", \"arguments\": {\"input\": \"hi\"}}</tool_call>",
        "工具结果：aGk=",
        "助手：hi 的 Base64 编码是 aGk=",
        "",
        "示例 2（无需工具时直接回答，不要输出 tool_call）：",
        "用户：你好",
        "助手：你好！我是 TBox 工具助手，可以帮你做编码、哈希、格式化等计算任务。",
        "",
        "示例 3（当前时间用 now，禁止编造日期）：",
        "用户：看下当前时间",
        "助手：<tool_call>{\"name\": \"timestamp.convert\", \"arguments\": {\"input\": \"now\"}}</tool_call>",
    ]
    .join("\n")
}

fn push_role_and_rules(prompt: &mut String) {
    prompt.push_str("你是 TBox 工具助手。用户提出计算类请求（编码、解码、哈希、解析、格式化、转换、生成）时，你必须调用下述工具完成，不要自己心算。\n");
    prompt.push_str("调用规则：只输出一个 <tool_call> 块，格式为 <tool_call>{\"name\": \"工具id\", \"arguments\": {...}}</tool_call>；不得编造参数名；与工具无关的请求直接回答。\n");
    prompt.push_str("重要：需要工具时本轮只输出 <tool_call>，不要模仿示例里的「工具结果：」或「助手：」行，也不要编造工具返回值；等系统回传真实结果后再用自然语言回答用户。\n");
}

fn push_tool_directory(prompt: &mut String) {
    prompt.push_str("\n可调用工具（id：用途｜参数）：\n");
    prompt.push_str(&render_tool_summary());
}

/// 小模型强化提示：角色 → 工具目录（常驻）→ 少样本。
/// Skill 采用渐进式披露：L0/L1 由 loop 与系统提示词分装，不在此拼正文。
pub fn build_small_prompt(_user_text: &str) -> String {
    let mut prompt = String::new();
    push_role_and_rules(&mut prompt);
    push_tool_directory(&mut prompt);
    prompt.push_str("\n\n用法示例：\n");
    prompt.push_str(&few_shot_block());
    prompt
}

/// 轻量档提示：同布局，缩短少样本。
pub fn build_lite_prompt(_user_text: &str) -> String {
    let mut prompt = String::new();
    push_role_and_rules(&mut prompt);
    push_tool_directory(&mut prompt);
    prompt.push_str("\n\n用法示例：\n");
    prompt.push_str(&lite_few_shot_block());
    prompt
}

/// 提示是否在预算内（测试与评测用告警断言）。
pub fn prompt_within_budget(prompt: &str) -> bool {
    prompt.chars().count() <= SMALL_PROMPT_CHAR_BUDGET
}

pub fn lite_prompt_within_budget(prompt: &str) -> bool {
    prompt.chars().count() <= LITE_PROMPT_CHAR_BUDGET
}

/// 默认策略提示：云端/Ollama；Skill 正文不写进 system（由 loop 渐进装载）。
pub fn build_default_prompt(_user_text: &str) -> String {
    String::from(
        "You are the TBox local agent. Prefer registered pure-compute tools when helpful.\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_summary_covers_all_registered_tools() {
        let summary = render_tool_summary();
        for spec in registry::all_tools() {
            assert!(summary.contains(spec.id), "missing {}", spec.id);
        }
        assert!(summary.contains("base64.encode：Base64 编码"));
    }

    #[test]
    fn tool_line_marks_required_and_enum() {
        let spec = registry::lookup("hash.digest").unwrap();
        let line = render_tool_line(spec);
        assert!(line.contains("algorithm(字符串,必填)"));
        assert!(line.contains("md5/sha256"));
    }

    #[test]
    fn small_prompt_has_all_sections() {
        let p = build_small_prompt("帮我解析这段 JWT");
        assert!(p.contains("TBox 工具助手"));
        assert!(p.contains("<tool_call>"));
        assert!(p.contains("base64.encode"));
        assert!(p.contains("示例 4")); // 负例
        assert!(p.contains("不得编造参数名"));
        assert!(p.contains("可调用工具"));
        // Skill 正文不再写入 system（渐进式披露由 loop 装载）
        assert!(!p.contains("相关技能说明"));
        assert!(!p.contains("Loaded skill instructions"));
    }

    #[test]
    fn small_prompt_keeps_directory_without_skills() {
        let p = build_small_prompt("zzzz-no-skill-match-xxxxx");
        assert!(p.contains("可调用工具"));
        assert!(!p.contains("相关技能说明"));
    }

    #[test]
    fn lite_prompt_within_char_budget() {
        let p = build_lite_prompt("base64 编码 hi");
        assert!(
            lite_prompt_within_budget(&p),
            "lite prompt chars = {} > budget {LITE_PROMPT_CHAR_BUDGET}",
            p.chars().count()
        );
        assert!(p.contains("示例 2"));
        assert!(!p.contains("示例 5"));
    }

    #[test]
    fn small_prompt_within_char_budget() {
        let p = build_small_prompt("base64 编码 hi 然后算哈希 JWT 时间戳 cron uuid xml yaml json");
        assert!(
            prompt_within_budget(&p),
            "prompt chars = {} > budget {SMALL_PROMPT_CHAR_BUDGET}",
            p.chars().count()
        );
    }

    #[test]
    fn default_prompt_matches_legacy_format() {
        let p = build_default_prompt("hello");
        assert!(p.starts_with("You are the TBox local agent."));
        let jwt = build_default_prompt("帮我解析 JWT");
        // Skill 检索可能因关键词而命中；未命中时也应是合法默认提示。
        assert!(jwt.starts_with("You are the TBox local agent."));
    }
}
