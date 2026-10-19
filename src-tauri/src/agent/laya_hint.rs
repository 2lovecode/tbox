//! 把本轮用户原文压成 Laya 的决策状态。不调用语言模型。

const SHORT_TURN_CHARS: usize = 40;
const STATE_CHARS: usize = 180;

/// `prior` 为历史中最近一条工具的名字和结果首行。
pub fn compress_decision_state(user_text: &str, prior: Option<(&str, &str)>) -> String {
    let original_short = user_text.trim().chars().count() <= SHORT_TURN_CHARS;
    let mut out = strip_payloads(user_text);
    if out.is_empty() {
        out = "有一段JSON".to_string();
    }
    if original_short {
        if let Some((name, note)) = prior {
            let name = name.trim();
            if !name.is_empty() {
                let note: String = note.trim().chars().take(60).collect();
                if note.is_empty() {
                    out.push_str(&format!("。上一轮工具 {name}"));
                } else {
                    out.push_str(&format!("。上一轮工具 {name}：{note}"));
                }
            }
        }
    }
    out.chars().take(STATE_CHARS).collect()
}

/// 去掉 JSON 等载荷行后剩下的意图文本，以及是否见过载荷。
pub fn intent_without_payload(text: &str) -> (String, bool) {
    let mut kept: Vec<String> = Vec::new();
    let mut saw_payload = false;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(intent) = trailing_intent(t) {
            saw_payload = true;
            kept.push(intent.to_string());
            continue;
        }
        if looks_like_payload(t) {
            saw_payload = true;
            continue;
        }
        kept.push(t.to_string());
    }
    (kept.join("\n"), saw_payload)
}

fn strip_payloads(text: &str) -> String {
    let (joined_raw, saw_payload) = intent_without_payload(text);
    let mut joined = joined_raw.replace('\n', " ");
    if saw_payload && joined.is_empty() {
        return "有一段JSON".to_string();
    }
    if saw_payload && !joined.contains("JSON") && !joined.contains("json") {
        joined.push_str("。有一段JSON");
    }
    joined
}

fn trailing_intent(line: &str) -> Option<&str> {
    const TAILS: &[&str] = &[
        "去转义",
        "反转义",
        "转义",
        "转query",
        "转 query",
        "转成 query",
    ];
    for tail in TAILS {
        let Some(rest) = line.trim().strip_suffix(tail) else {
            continue;
        };
        let rest = rest.trim().trim_end_matches(['，', ',', '：', ':', ' ']);
        if rest.is_empty() {
            return None;
        }
        if looks_like_payload(rest) || rest.ends_with('}') || rest.ends_with(']') {
            return Some(*tail);
        }
    }
    None
}

fn looks_like_payload(line: &str) -> bool {
    let t = line.trim();
    if t.starts_with('{')
        || t.starts_with('[')
        || t.starts_with('}')
        || t.starts_with(']')
        || t.starts_with("{\\")
        || t.starts_with("[\\")
        || t.starts_with("\\\"")
        || (t.starts_with('"') && t.contains(':'))
    {
        return true;
    }
    let colon = t.matches(':').count();
    let quotes = t.matches('"').count() + t.matches("\\\"").count();
    t.len() > 80 && colon >= 2 && quotes >= 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_json_keeps_verb() {
        let text = "{\n  \"a\": 1\n}\n转义";
        let state = compress_decision_state(text, None);
        assert!(state.contains("转义"), "{state}");
        assert!(!state.contains("\"a\""), "{state}");
        assert!(state.contains("有一段JSON"), "{state}");
    }

    #[test]
    fn same_line_escaped_json_keeps_unescape() {
        let text = r#"{\"filters\":{\"city\":1}}反转义"#;
        let state = compress_decision_state(text, None);
        assert!(state.contains("反转义"), "{state}");
        assert!(!state.contains("filters"), "{state}");
    }

    #[test]
    fn short_followup_mentions_prior_tool() {
        let state = compress_decision_state("把这个反转义", Some(("json.escape", "JSON解析错误: key")));
        assert!(state.contains("反转义"), "{state}");
        assert!(state.contains("json.escape"), "{state}");
        assert!(state.contains("JSON解析错误"), "{state}");
    }

    #[test]
    fn long_paste_does_not_attach_prior() {
        let text = format!("{}\n转query", "{\"k\":1}".repeat(30));
        let state = compress_decision_state(&text, Some(("json.escape", "err")));
        assert!(!state.contains("json.escape"), "{state}");
        assert!(state.contains("转query") || state.contains("转 query") || state.contains("query"), "{state}");
    }
}
