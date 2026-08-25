//! 容错解析层：代码围栏剥离、全角标点归一、arguments 字符串化 JSON、
//! 相近工具名建议。所有归一只在原始解析失败后尝试，避免破坏合法数据。

use crate::agent::llm::ToolCall;
use crate::agent::registry;

/// 剥离包裹整个输出的代码围栏（``` / ```json ... ```）。
pub fn strip_code_fences(text: &str) -> String {
    let t = text.trim();
    let Some(rest) = t.strip_prefix("```") else {
        return t.to_string();
    };
    // 跳过语言标记行
    let rest = match rest.find('\n') {
        Some(nl) => &rest[nl + 1..],
        None => rest,
    };
    let rest = rest.trim();
    rest.strip_suffix("```").unwrap_or(rest).trim().to_string()
}

/// 全角引号/冒号/花括号归一（仅对解析失败的 payload 使用）。
pub fn normalize_payload(s: &str) -> String {
    s.trim()
        .replace('“', "\"")
        .replace('”', "\"")
        .replace('‘', "'")
        .replace('’', "'")
        .replace('：', ":")
        .replace('｛', "{")
        .replace('｝', "}")
}


/// 从一段文本里取出首个花括号深度匹配的 JSON 对象子串（含两侧大括号）。
fn extract_balanced_json_object(input: &str) -> Option<&str> {
    let bytes = input.as_bytes();
    let mut start = None;
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if escape { escape = false; continue; }
        if in_string {
            if b == b'\\' { escape = true; continue; }
            if b == b'"' { in_string = false; }
            continue;
        }
        match b {
            b'\"' => in_string = true,
            b'{' => {
                if depth == 0 { start = Some(i); }
                depth += 1;
            }
            b'}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(s) = start {
                            return Some(&input[s..=i]);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// 尝试把 payload 解析为 (name, arguments)；arguments 为字符串化 JSON 时二次解析。
fn parse_payload(payload: &str) -> Option<(String, serde_json::Value)> {
    for candidate in [payload, &normalize_payload(payload)] {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(candidate) {
            let name = v.get("name")?.as_str()?.to_string();
            let args = v.get("arguments").cloned()?;
            // arguments 可能是字符串化 JSON：再解一层（失败则原样保留）。
            let args = match args.as_str() {
                Some(s) => serde_json::from_str::<serde_json::Value>(s)
                    .or_else(|_| serde_json::from_str(&normalize_payload(s)))
                    .unwrap_or(args),
                None => args,
            };
            return Some((name, args));
        }
    }
    None
}

/// 从模型输出解析 `<tool_call>` 块。返回（工具调用列表, 剩余文本）。
/// 无法解析的块原样保留为文本；无块时输出为纯文本。
pub fn parse_tool_calls(text: &str) -> (Vec<ToolCall>, String) {
    let fenced_stripped = strip_code_fences(text);
    let mut calls = Vec::new();
    let mut rest = String::new();
    let mut remainder = fenced_stripped.as_str();
    while let Some(start) = remainder.find("<tool_call>") {
        let after_start = &remainder[start + "<tool_call>".len()..];
        match after_start.find("</tool_call>") {
            Some(end) => {
                let payload = after_start[..end].trim();
                match parse_payload(payload) {
                    Some((name, args)) => {
                        calls.push(ToolCall {
                            id: format!("call_{}", calls.len()),
                            name,
                            arguments: args,
                        });
                        rest.push_str(&remainder[..start]);
                        remainder = &after_start[end + "</tool_call>".len()..];
                    }
                    None => {
                        // 兜底：payload 大括号未匹配 / 多余转义时，按花括号深度
                        // 切出最内层完整 JSON 对象再试一次（实测 0.5B 偶发
                        // 嵌套引号导致最外层解析失败，但内层仍然合法）。
                        let mut recovered = None;
                        if let Some(slice) = extract_balanced_json_object(payload) {
                            if let Some((n, a)) = parse_payload(slice) {
                                if !a.is_null() {
                                    recovered = Some((n, a));
                                }
                            }
                        }
                        if let Some((name, args)) = recovered {
                            calls.push(ToolCall {
                                id: format!("call_{}", calls.len()),
                                name,
                                arguments: args,
                            });
                            rest.push_str(&remainder[..start]);
                            remainder = &after_start[end + "</tool_call>".len()..];
                        } else {
                            // 彻底失败：把畸变块隐藏进 `rest` 旁注，避免把
                            // 半个 `<tool_call>{...}` 直接展示给用户。
                            rest.push_str(&remainder[..start]);
                            rest.push_str("[工具调用格式异常，已忽略]\n");
                            remainder = &after_start[end + "</tool_call>".len()..];
                        }
                    }
                }
            }
            None => {
                // 未闭合的 <tool_call>：0.5B 常在 JSON 结束即停（EOS/截断），
                // 不输出 </tool_call>。若剩余 payload 能解析为合法调用则接受，
                // 否则按原设计保留为纯文本。
                let payload = after_start.trim();
                let mut recovered = parse_payload(payload).filter(|(_, args)| !args.is_null());
                if recovered.is_none() {
                    if let Some(slice) = extract_balanced_json_object(payload) {
                        recovered = parse_payload(slice).filter(|(_, args)| !args.is_null());
                    }
                }
                match recovered {
                    Some((name, args)) => {
                        calls.push(ToolCall {
                            id: format!("call_{}", calls.len()),
                            name,
                            arguments: args,
                        });
                        rest.push_str(&remainder[..start]);
                    }
                    None => {
                        // 隐藏畸变块，旁注说明
                        rest.push_str(&remainder[..start]);
                        rest.push_str("[工具调用格式异常，已忽略]\n");
                        rest.push_str(payload);
                    }
                }
                remainder = "";
                break;
            }
        }
    }
    rest.push_str(remainder);
    // 兜底：0.5B 有时直接输出裸 JSON（{"name": …, "arguments": …}）
    // 不带 <tool_call> 包裹（实测见 DB 记录）。仅当输出以 { 开头且能解析
    // 为合法调用时接受，避免误吞正常 JSON 内容回答。
    if calls.is_empty() {
        let trimmed = rest.trim();
        // 模型常在裸 JSON 后面追加 `</tool_call>` 或换行；允许尾巴是它们。
        let core = trimmed
            .trim_end_matches("</tool_call>")
            .trim_end();
        if core.starts_with('{') && core.ends_with('}') {
            if let Some((name, args)) = parse_payload(core) {
                if !args.is_null() {
                    return (
                        vec![ToolCall {
                            id: "call_0".to_string(),
                            name,
                            arguments: args,
                        }],
                        String::new(),
                    );
                }
            }
        }
    }
    (calls, rest.trim().to_string())
}

/// 简易 Levenshtein 编辑距离（工具名都很短，O(n*m) 足够）。
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// 编辑距离 ≤2 的注册表工具名建议（含 id 与短名段匹配）。
pub fn suggest_tool_names(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    for spec in registry::all_tools() {
        if edit_distance(name, spec.id) <= 2 {
            out.push(spec.id.to_string());
            continue;
        }
        for seg in spec.id.split('.') {
            if !seg.is_empty() && edit_distance(name, seg) <= 2 {
                out.push(spec.id.to_string());
                break;
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// 校验一次工具调用：未注册时给出相近建议；已注册时做 schema 形状校验。
/// 校验失败时返回可直接回填给模型的错误文本。
pub fn validate_call(name: &str, args: &serde_json::Value) -> Result<(), String> {
    match registry::lookup(name) {
        None => {
            let suggestions = suggest_tool_names(name);
            match suggestions.is_empty() {
                false => Err(format!(
                    "未注册的工具: {name}。你是否想调用：{}？",
                    suggestions.join("、")
                )),
                true => Err(format!("未注册的工具: {name}（unknown tool）")),
            }
        }
        Some(spec) => registry::validate_args(spec, args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

/// 回归：用户 DB 实测模型输出 `<tool_call>{"name": "json.to_query",
/// "arguments": {"input":"[\"{\"aa\":\"bb\"}\""]}</tool_call>`（括号
/// 嵌套/转义混乱），原 parser 解析失败导致整段 raw 文本直显给用户。
/// 修复后应能从首个平衡 JSON 对象恢复出 name + arguments。
#[test]
fn unbalanced_payload_recovered_by_brace_depth() {
    let raw = "<tool_call>{\"name\": \"json.to_query\", \"arguments\": {\"input\":\"[\\\"{\\\"aa\\\":\\\"bb\\\"}\\\"]\"}}</tool_call>";
    let (calls, rest) = parse_tool_calls(raw);
    assert_eq!(calls.len(), 1, "balanced object must be recovered");
    assert_eq!(calls[0].name, "json.to_query");
    assert!(calls[0].arguments.get("input").is_some());
    assert!(rest.is_empty(), "raw block hidden, no rest leak");
}

/// 回归：未闭合的 `<tool_call>` 后即便 JSON 也不完整（缺 args），仍应
/// 隐藏而不是把半截块漏给用户。
#[test]
fn unterminated_block_with_no_args_is_hidden() {
    let raw = "<tool_call>{\"name\": \"x\"}";
    let (calls, rest) = parse_tool_calls(raw);
    assert!(calls.is_empty());
    assert!(rest.contains("工具调用格式异常"));
}

#[test]
fn bare_json_tool_call_recovered() {
    let (calls, rest) = parse_tool_calls(
        "{\"name\": \"json.to_query\", \"arguments\": {\"input\": \"{\\\"aa\\\":\\\"bb\\\"}\"}}",
    );
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "json.to_query");
    assert!(rest.is_empty());
    // 正常 JSON 内容回答不被误吞：与注册工具无关
    let (calls, rest) = parse_tool_calls("配置文件内容是 {\"port\": 8080}");
    assert!(calls.is_empty());
    assert!(rest.contains("8080"));
}
    use serde_json::json;

    #[test]
    fn plain_text_untouched() {
        let (calls, rest) = parse_tool_calls("你好，世界");
        assert!(calls.is_empty());
        assert_eq!(rest, "你好，世界");
    }

    #[test]
    fn fenced_tool_call_recovered() {
        let raw = "```json\n<tool_call>{\"name\": \"base64.encode\", \"arguments\": {\"input\": \"hi\"}}</tool_call>\n```";
        let (calls, _) = parse_tool_calls(raw);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "base64.encode");
        assert_eq!(calls[0].arguments, json!({"input": "hi"}));
    }

    #[test]
    fn fullwidth_payload_normalized() {
        let raw = "<tool_call>{“name”： “base64.encode”, “arguments”： {“input”： “hi”}}</tool_call>";
        let (calls, _) = parse_tool_calls(raw);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "base64.encode");
        assert_eq!(calls[0].arguments, json!({"input": "hi"}));
    }

    #[test]
    fn stringified_arguments_parsed() {
        let raw = r#"<tool_call>{"name": "hash.digest", "arguments": "{\"input\": \"hi\", \"algorithm\": \"md5\"}"}</tool_call>"#;
        let (calls, _) = parse_tool_calls(raw);
        assert_eq!(calls[0].arguments, json!({"input": "hi", "algorithm": "md5"}));
    }

    #[test]
    fn valid_payload_not_corrupted_by_normalization() {
        // 合法 payload 中的中文内容必须原样保留（归一只在失败后尝试）。
        let raw = r#"<tool_call>{"name": "charset.convert", "arguments": {"input": "你好，世界", "charset": "UTF-8"}}</tool_call>"#;
        let (calls, _) = parse_tool_calls(raw);
        assert_eq!(calls[0].arguments["input"], "你好，世界");
    }

    #[test]
    fn leading_chatter_stripped_from_rest() {
        let raw = "让我查一下\n<tool_call>{\"name\": \"uuid.generate\", \"arguments\": {}}</tool_call>\n";
        let (calls, rest) = parse_tool_calls(raw);
        assert_eq!(calls.len(), 1);
        assert_eq!(rest, "让我查一下");
    }

    #[test]
    fn malformed_block_hidden_with_hint() {
        // 畸变 <tool_call> 不应再把半截 JSON 展示给用户，改为隐藏 + 旁注。
        let (calls, rest) = parse_tool_calls("<tool_call>{not json}</tool_call>以及后续");
        assert!(calls.is_empty());
        assert!(!rest.contains("{not json}"));
        assert!(rest.contains("工具调用格式异常"));
        assert!(rest.contains("以及后续"));
    }

    #[test]
    fn unterminated_tag_is_text() {
        let (calls, _) = parse_tool_calls("<tool_call>{\"name\": \"x\"}");
        assert!(calls.is_empty());
    }

    #[test]
    fn unterminated_tag_with_valid_payload_accepted() {
        // 0.5B 实测：生成到 JSON 结束即停，缺 </tool_call>
        let raw = "<tool_call>{\"name\": \"json.to_query\", \"arguments\": {\"input\": \"{\\\"aa\\\":\\\"bb\\\"}\"}}";
        let (calls, rest) = parse_tool_calls(raw);
        assert_eq!(calls.len(), 1, "unterminated but valid payload must parse");
        assert_eq!(calls[0].name, "json.to_query");
        assert_eq!(rest, "");

        // 带引导语 + 未闭合
        let raw2 = "我来帮你转换\n<tool_call>{\"name\": \"uuid.generate\", \"arguments\": {}}";
        let (calls2, rest2) = parse_tool_calls(raw2);
        assert_eq!(calls2.len(), 1);
        assert_eq!(rest2, "我来帮你转换");
    }

    #[test]
    fn unterminated_garbage_still_text() {
        let (calls, rest) = parse_tool_calls("<tool_call>not json at all");
        assert!(calls.is_empty());
        assert!(rest.contains("not json"));
    }

    #[test]
    fn near_miss_tool_suggested() {
        let err = validate_call("base64.encod", &json!({"input": "hi"})).unwrap_err();
        assert!(err.contains("base64.encode"), "got: {err}");
    }

    #[test]
    fn unknown_tool_without_suggestion() {
        let err = validate_call("http.request", &json!({})).unwrap_err();
        assert!(err.contains("未注册"));
    }

    #[test]
    fn schema_violation_detected() {
        let err = validate_call("hash.digest", &json!({"input": "hi"})).unwrap_err();
        assert!(err.contains("algorithm"));
        assert!(validate_call("hash.digest", &json!({"input": "hi", "algorithm": "md5"})).is_ok());
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("base64.encode", "base64.encod"), 1);
        assert_eq!(edit_distance("abc", "abc"), 0);
    }
}
