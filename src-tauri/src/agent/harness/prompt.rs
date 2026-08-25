//! 提示工程：注册表工具的中文单行摘要 + 小模型四段式系统提示。

use crate::agent::registry::{self, ToolSpec};
use crate::agent::skills::retrieve_skills;

/// 小模型系统提示字符预算（超出即测试告警断言）。
/// 0.5B 上下文 4096 token，中文约 1 字 ≈ 1 token，留一半给对话与工具结果。
pub const SMALL_PROMPT_CHAR_BUDGET: usize = 6000;

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
    ]
    .join("\n")
}

/// 单个 Skill 注入正文的字符上限：保留能力描述与首个问法样本，
/// 防止 3 条 Skill 撑爆小模型 prompt 预算（超限部分截断）。
const SKILL_BODY_CHAR_CAP: usize = 500;

/// 截断 Skill 正文：优先在段落/句子边界断开，避免截断 `<tool_call>` 块中段。
fn truncate_skill_body(body: &str) -> String {
    if body.chars().count() <= SKILL_BODY_CHAR_CAP {
        return body.to_string();
    }
    let truncated: String = body.chars().take(SKILL_BODY_CHAR_CAP).collect();
    // 回退到最后一个换行或句号，避免撕裂 JSON 示例
    match truncated.rfind(['\n', '。']) {
        Some(pos) if pos > SKILL_BODY_CHAR_CAP / 2 => truncated[..=pos].to_string(),
        _ => truncated,
    }
}

/// 小模型四段式系统提示：角色任务 + Skill + 工具摘要 + 少样本。
pub fn build_small_prompt(user_text: &str) -> String {
    let mut prompt = String::new();
    prompt.push_str("你是 TBox 工具助手。用户提出计算类请求（编码、解码、哈希、解析、格式化、转换、生成）时，你必须调用下述工具完成，不要自己心算。\n");
    prompt.push_str("调用规则：只输出一个 <tool_call> 块，格式为 <tool_call>{\"name\": \"工具id\", \"arguments\": {...}}</tool_call>；不得编造参数名；与工具无关的请求直接回答。\n");

    let skills = retrieve_skills(user_text, 3);
    if !skills.is_empty() {
        prompt.push_str("\n相关技能说明：\n");
        for sk in &skills {
            prompt.push_str(&format!(
                "### {}\n{}\n\n",
                sk.tool_id,
                truncate_skill_body(&sk.body)
            ));
        }
    }

    prompt.push_str("\n可调用工具（id：用途｜参数）：\n");
    prompt.push_str(&render_tool_summary());
    prompt.push_str("\n\n用法示例：\n");
    prompt.push_str(&few_shot_block());
    prompt
}

/// 提示是否在预算内（测试与评测用告警断言）。
pub fn prompt_within_budget(prompt: &str) -> bool {
    prompt.chars().count() <= SMALL_PROMPT_CHAR_BUDGET
}

/// 默认策略提示：与旧版 `build_system_prompt` 行为等价（云端/Ollama 路径）。
pub fn build_default_prompt(user_text: &str) -> String {
    let skills = retrieve_skills(user_text, 3);
    let mut prompt = String::from(
        "You are the TBox local agent. Prefer registered pure-compute tools when helpful.\n",
    );
    if !skills.is_empty() {
        prompt.push_str("\nRelevant skills:\n");
        for sk in &skills {
            prompt.push_str(&format!("### {}\n{}\n\n", sk.tool_id, sk.body));
        }
    }
    prompt
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
        assert!(p.contains("示例 3")); // 负例
        assert!(p.contains("不得编造参数名"));
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
        assert!(jwt.contains("Relevant skills:"));
    }
}
