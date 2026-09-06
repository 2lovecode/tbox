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
    // 小模型常把 JSON 对象直接塞进 string 字段且不转义，例如
    // `"input": "{"query":"go"}"` —— 标准 JSON 解析失败。尝试按字段名抢救。
    repair_unescaped_input_object(payload)
}

/// 抢救 `"input": {…}` / `"input": "{…}"`（内层引号未转义）为合法 arguments。
fn repair_unescaped_input_object(payload: &str) -> Option<(String, serde_json::Value)> {
    let name = extract_json_string_field(payload, "name")?;
    // 定位 "input" : 之后的值起点
    let key = "\"input\"";
    let idx = payload.find(key)?;
    let after_key = &payload[idx + key.len()..];
    let colon = after_key.find(':')?;
    let mut rest = after_key[colon + 1..].trim_start();
    // 可选的开引号（模型写成 `"input": "{...}"`）
    if rest.starts_with('"') {
        rest = rest[1..].trim_start();
    }
    if !rest.starts_with('{') {
        return None;
    }
    let obj = extract_braces_lenient(rest)?;
    let args = serde_json::json!({ "input": obj });
    Some((name, args))
}

fn extract_json_string_field(payload: &str, field: &str) -> Option<String> {
    let key = format!("\"{field}\"");
    let idx = payload.find(&key)?;
    let after = &payload[idx + key.len()..];
    let colon = after.find(':')?;
    let mut rest = after[colon + 1..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    rest = &rest[1..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(n) = chars.next() {
                out.push(n);
            }
            continue;
        }
        if c == '"' {
            break;
        }
        out.push(c);
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// 仅按花括号深度截取（忽略字符串状态）——专用于引号已损坏的 payload。
fn extract_braces_lenient(input: &str) -> Option<&str> {
    let bytes = input.as_bytes();
    if bytes.first() != Some(&b'{') {
        return None;
    }
    let mut depth: i32 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&input[..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// 流式剥离 `<tool_call>`：标签内内容不对外发出，避免 Live 路径泄漏到 UI。
#[derive(Debug, Default)]
pub struct ToolCallStreamFilter {
    buf: String,
    in_tool: bool,
}

impl ToolCallStreamFilter {
    pub fn push(&mut self, chunk: &str, emit: &mut dyn FnMut(String)) {
        self.buf.push_str(chunk);
        loop {
            if self.in_tool {
                if let Some(i) = self.buf.find("</tool_call>") {
                    self.buf = self.buf[i + "</tool_call>".len()..].to_string();
                    self.in_tool = false;
                    continue;
                }
                // 保留可能的部分闭合标签
                let keep = partial_tag_suffix(&self.buf, "</tool_call>");
                if self.buf.len() > keep {
                    self.buf.drain(..self.buf.len() - keep);
                }
                break;
            } else if let Some(i) = self.buf.find("<tool_call>") {
                let before = self.buf[..i].to_string();
                self.buf = self.buf[i + "<tool_call>".len()..].to_string();
                self.in_tool = true;
                if !before.is_empty() {
                    emit(before);
                }
                continue;
            } else {
                let keep = partial_tag_suffix(&self.buf, "<tool_call>");
                if self.buf.len() > keep {
                    let emit_s = self.buf[..self.buf.len() - keep].to_string();
                    self.buf.drain(..self.buf.len() - keep);
                    if !emit_s.is_empty() {
                        emit(emit_s);
                    }
                }
                break;
            }
        }
    }

    /// 回合结束：若仍在 tool 块内则丢弃；否则冲刷缓冲。
    pub fn finish(mut self, emit: &mut dyn FnMut(String)) {
        if self.in_tool {
            return;
        }
        if !self.buf.is_empty() {
            emit(std::mem::take(&mut self.buf));
        }
    }
}

fn partial_tag_suffix(s: &str, tag: &str) -> usize {
    let max = s.len().min(tag.len().saturating_sub(1));
    for len in (1..=max).rev() {
        if s.ends_with(&tag[..len]) {
            return len;
        }
    }
    0
}

/// 剥离已闭合/未闭合的 `<tool_call>` 块，避免泄漏到用户可见正文。
pub fn strip_tool_call_markup(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("<tool_call>") {
        out.push_str(&rest[..start]);
        let after = &rest[start + "<tool_call>".len()..];
        match after.find("</tool_call>") {
            Some(end) => {
                rest = &after[end + "</tool_call>".len()..];
            }
            None => {
                // 未闭合：丢弃其后全部
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out.trim().to_string()
}

/// 从模型输出解析 `<tool_call>` 块。返回（工具调用列表, 剩余文本）。
/// 无法解析的块隐藏为旁注；无块时输出为纯文本。
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
                        // 隐藏畸变块，旁注说明（不把 payload 漏给用户）
                        rest.push_str(&remainder[..start]);
                        rest.push_str("[工具调用格式异常，已忽略]\n");
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
    use serde_json::json;

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

/// 回归：用户实测 `"input": "{"query":"go"}"`（内层 JSON 未转义），
/// 不得把整段 `<tool_call>…` 当助手正文展示。
#[test]
fn unescaped_input_object_recovered_not_leaked_as_text() {
    let raw = r#"<tool_call>{"name": "json.to_query", "arguments": {"input": "{"query":"go"}"}}</tool_call>"#;
    let (calls, rest) = parse_tool_calls(raw);
    assert_eq!(calls.len(), 1, "must recover tool call");
    assert_eq!(calls[0].name, "json.to_query");
    assert_eq!(
        calls[0].arguments.get("input").and_then(|v| v.as_str()),
        Some(r#"{"query":"go"}"#)
    );
    assert!(
        !rest.contains("<tool_call>") && !rest.contains("json.to_query"),
        "tool markup must not leak into rest: {rest:?}"
    );
}

#[test]
fn tool_call_stream_filter_holds_back_markup() {
    let mut f = ToolCallStreamFilter::default();
    let mut out = String::new();
    let mut emit = |s: String| out.push_str(&s);
    f.push("先说明一下", &mut emit);
    f.push("<tool_call>{\"name\":", &mut emit);
    f.push(" \"x\"}</tool_call>结尾", &mut emit);
    f.finish(&mut emit);
    assert_eq!(out, "先说明一下结尾");
    assert!(!out.contains("tool_call"));
}

#[test]
fn strip_tool_call_markup_drops_blocks() {
    let s = strip_tool_call_markup("A<tool_call>{\"name\":\"x\"}</tool_call>B");
    assert_eq!(s, "AB");
    assert_eq!(strip_tool_call_markup("<tool_call>only"), "");
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
        assert!(!rest.contains("not json"));
        assert!(rest.contains("工具调用格式异常"));
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
