use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Serialize)]
pub struct JsonFormatResult {
    formatted: String,
    is_valid: bool,
}

#[derive(Serialize)]
pub struct JsonEscapeResult {
    escaped: String,
}

#[derive(Serialize)]
pub struct JsonUnescapeResult {
    unescaped: String,
    is_valid: bool,
}

/// 美化JSON（格式化）
#[tauri::command]
pub fn format_json_pretty(json_str: String, indent_size: Option<usize>) -> Result<JsonFormatResult, String> {
    let indent_size = indent_size.unwrap_or(2);

    let value = parse_json_value_lenient(&json_str)?;

    // 格式化输出（默认使用2个空格缩进）
    let formatted = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("JSON格式化错误: {}", e))?;

    // 如果指定了自定义缩进大小，替换默认的2个空格
    let formatted = if indent_size != 2 {
        formatted
            .split('\n')
            .map(|line| {
                // 计算当前行的缩进级别（默认每级2个空格）
                let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
                let indent_level = leading_spaces / 2;
                // 生成新的缩进
                let new_indent = " ".repeat(indent_level * indent_size);
                format!("{}{}", new_indent, line.trim_start())
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        formatted
    };

    Ok(JsonFormatResult {
        formatted,
        is_valid: true,
    })
}

/// 宽松解析：合法 JSON、JSON 字符串包裹、已转义 / 多重转义均可；
/// 小模型多写的尾部字符（如多余 `}`）会被忽略，只取第一个完整值。
pub fn parse_json_value_lenient(input: &str) -> Result<Value, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("JSON解析错误: 输入为空".to_string());
    }

    let mut last_err = String::new();
    let mut candidate = trimmed.to_string();
    // 小模型常在 tool args 里对用户原文再套 1～3 层 `\`；每轮失败后剥一层。
    for _ in 0..8 {
        match parse_first_json_value(&candidate) {
            Ok(Value::String(s)) => {
                let st = s.trim();
                if st.starts_with('{') || st.starts_with('[') {
                    if let Ok(inner) = parse_first_json_value(st) {
                        return Ok(inner);
                    }
                }
                return Ok(Value::String(s));
            }
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = e;
                let next = unescape_json_text(&candidate);
                if next == candidate {
                    break;
                }
                candidate = next;
            }
        }
    }
    let hint = if trimmed.contains('\\') {
        "；若这是用户粘贴的转义 JSON，请把原文原样放入 input，不要再多写反斜杠"
    } else {
        ""
    };
    Err(format!("JSON解析错误: {last_err}{hint}"))
}

/// 只反序列化第一个 JSON 值，允许其后有多余字符（LLM 常见多括号/尾巴）。
fn parse_first_json_value(input: &str) -> Result<Value, String> {
    let mut de = serde_json::Deserializer::from_str(input.trim());
    match Value::deserialize(&mut de) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_string()),
    }
}

fn unescape_json_text(escaped: &str) -> String {
    escaped
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

/// 压缩JSON（去除所有空格和换行）
#[tauri::command]
pub fn compress_json(json_str: String) -> Result<JsonFormatResult, String> {
    // 先验证JSON是否有效
    let value: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析错误: {}", e))?;
    
    // 压缩输出（无空格）
    let compressed = serde_json::to_string(&value)
        .map_err(|e| format!("JSON压缩错误: {}", e))?;
    
    Ok(JsonFormatResult {
        formatted: compressed,
        is_valid: true,
    })
}

/// JSON转义（将JSON字符串转义为可以在代码中使用的字符串）
#[tauri::command]
pub fn escape_json(json_str: String) -> Result<JsonEscapeResult, String> {
    // 验证JSON是否有效
    let _: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析错误: {}", e))?;
    
    // 转义JSON字符串
    let escaped = json_str
        .replace('\\', "\\\\")  // 反斜杠
        .replace('"', "\\\"")   // 双引号
        .replace('\n', "\\n")   // 换行
        .replace('\r', "\\r")   // 回车
        .replace('\t', "\\t");  // 制表符
    
    Ok(JsonEscapeResult {
        escaped,
    })
}

/// JSON去转义（将转义的JSON字符串还原）
#[tauri::command]
pub fn unescape_json(escaped_str: String) -> Result<JsonUnescapeResult, String> {
    // 去转义
    let unescaped = escaped_str
        .replace("\\n", "\n")   // 换行
        .replace("\\r", "\r")   // 回车
        .replace("\\t", "\t")   // 制表符
        .replace("\\\"", "\"")  // 双引号
        .replace("\\\\", "\\"); // 反斜杠（最后处理，避免重复转义）
    
    // 验证去转义后的JSON是否有效
    let is_valid = serde_json::from_str::<Value>(&unescaped).is_ok();
    
    Ok(JsonUnescapeResult {
        unescaped,
        is_valid,
    })
}

/// 验证JSON是否有效
#[tauri::command]
pub fn validate_json(json_str: String) -> Result<bool, String> {
    match serde_json::from_str::<Value>(&json_str) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("JSON无效: {}", e)),
    }
}

/// 获取JSON结构信息
#[derive(Serialize)]
pub struct JsonInfo {
    is_valid: bool,
    size: usize,
    depth: usize,
    key_count: usize,
    value_types: Vec<String>,
}

#[tauri::command]
pub fn get_json_info(json_str: String) -> Result<JsonInfo, String> {
    let value: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析错误: {}", e))?;
    
    let size = json_str.len();
    let (depth, key_count, value_types) = analyze_json_value(&value, 0);
    
    Ok(JsonInfo {
        is_valid: true,
        size,
        depth,
        key_count,
        value_types: value_types.into_iter().collect(),
    })
}

fn analyze_json_value(value: &Value, current_depth: usize) -> (usize, usize, std::collections::HashSet<String>) {
    let mut max_depth = current_depth;
    let mut key_count = 0;
    let mut value_types = std::collections::HashSet::new();
    
    match value {
        Value::Object(map) => {
            value_types.insert("object".to_string());
            key_count += map.len();
            for (_, v) in map {
                let (d, k, t) = analyze_json_value(v, current_depth + 1);
                max_depth = max_depth.max(d);
                key_count += k;
                value_types.extend(t);
            }
        }
        Value::Array(arr) => {
            value_types.insert("array".to_string());
            for v in arr {
                let (d, k, t) = analyze_json_value(v, current_depth + 1);
                max_depth = max_depth.max(d);
                key_count += k;
                value_types.extend(t);
            }
        }
        Value::String(_) => {
            value_types.insert("string".to_string());
        }
        Value::Number(_) => {
            value_types.insert("number".to_string());
        }
        Value::Bool(_) => {
            value_types.insert("boolean".to_string());
        }
        Value::Null => {
            value_types.insert("null".to_string());
        }
    }
    
    (max_depth, key_count, value_types)
}

// ==================== JSON转Query参数功能 ====================

#[derive(Serialize)]
pub struct JsonToQueryResult {
    query_string: String,
    encoded: String,
}

/// 将JSON对象转换为URL Query参数
#[tauri::command]
pub fn json_to_query_params(json_str: String) -> Result<JsonToQueryResult, String> {
    let value: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析错误: {}", e))?;

    let mut pairs = Vec::new();
    flatten_json_value(&value, String::new(), &mut pairs);

    let query_string = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let encoded = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding_encode(k), urlencoding_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    Ok(JsonToQueryResult {
        query_string,
        encoded,
    })
}

/// 把 JSON 值递归铺平为 `(path, value_str)` 对。路径风格：`a[b][0][c]`，
/// 与既有 `json_to_query_params` 共享，可作为平铺查询参数的内部表示。
pub(crate) fn flatten_json_value(value: &Value, prefix: String, pairs: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let new_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}[{}]", prefix, k)
                };
                flatten_json_value(v, new_key, pairs);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_key = format!("{}[{}]", prefix, i);
                flatten_json_value(v, new_key, pairs);
            }
        }
        Value::String(s) => {
            pairs.push((prefix, s.clone()));
        }
        Value::Number(n) => {
            pairs.push((prefix, n.to_string()));
        }
        Value::Bool(b) => {
            pairs.push((prefix, b.to_string()));
        }
        Value::Null => {
            pairs.push((prefix, String::new()));
        }
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

// ==================== JSON对比功能 ====================

#[derive(Serialize)]
pub struct JsonDiffChange {
    path: String,
    old_value: Value,
    new_value: Value,
}

#[derive(Serialize)]
pub struct JsonDiffResult {
    added: Vec<String>,
    removed: Vec<String>,
    modified: Vec<JsonDiffChange>,
    unchanged: Vec<String>,
}

/// 对比两个JSON的差异
#[tauri::command]
pub fn compare_json(json1: String, json2: String) -> Result<JsonDiffResult, String> {
    // 解析两个JSON
    let value1: Value = serde_json::from_str(&json1)
        .map_err(|e| format!("第一个JSON解析错误: {}", e))?;
    let value2: Value = serde_json::from_str(&json2)
        .map_err(|e| format!("第二个JSON解析错误: {}", e))?;

    // 执行对比
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = Vec::new();

    compare_values(&value1, &value2, "$", &mut added, &mut removed, &mut modified, &mut unchanged);

    Ok(JsonDiffResult {
        added,
        removed,
        modified,
        unchanged,
    })
}

fn compare_values(
    v1: &Value,
    v2: &Value,
    path: &str,
    added: &mut Vec<String>,
    removed: &mut Vec<String>,
    modified: &mut Vec<JsonDiffChange>,
    unchanged: &mut Vec<String>,
) {
    match (v1, v2) {
        // 两个都是对象
        (Value::Object(obj1), Value::Object(obj2)) => {
            // 获取所有键的集合
            let keys1: HashSet<_> = obj1.keys().collect();
            let keys2: HashSet<_> = obj2.keys().collect();

            // 找出被删除的键（在obj1中但不在obj2中）
            for key in keys1.difference(&keys2) {
                removed.push(format!("{}/{}", path, key));
            }

            // 找出新增的键（在obj2中但不在obj1中）
            for key in keys2.difference(&keys1) {
                added.push(format!("{}/{}", path, key));
            }

            // 比较共同的键
            for key in keys1.intersection(&keys2) {
                let new_path = format!("{}/{}", path, key);
                compare_values(
                    &obj1[*key],
                    &obj2[*key],
                    &new_path,
                    added,
                    removed,
                    modified,
                    unchanged,
                );
            }
        }

        // 两个都是数组
        (Value::Array(arr1), Value::Array(arr2)) => {
            let len1 = arr1.len();
            let len2 = arr2.len();
            let max_len = len1.max(len2);

            for i in 0..max_len {
                let new_path = format!("{}/[{}]", path, i);

                match (arr1.get(i), arr2.get(i)) {
                    (Some(val1), Some(val2)) => {
                        // 两个数组都有这个索引，递归比较
                        compare_values(
                            val1,
                            val2,
                            &new_path,
                            added,
                            removed,
                            modified,
                            unchanged,
                        );
                    }
                    (None, Some(_)) => {
                        // arr2比arr1长，这是新增的元素
                        added.push(new_path);
                    }
                    (Some(_), None) => {
                        // arr1比arr2长，这是删除的元素
                        removed.push(new_path);
                    }
                    (None, None) => {
                        // 不可能发生
                    }
                }
            }
        }

        // 其他类型（字符串、数字、布尔、null）
        _ => {
            if v1 == v2 {
                unchanged.push(path.to_string());
            } else {
                modified.push(JsonDiffChange {
                    path: path.to_string(),
                    old_value: v1.clone(),
                    new_value: v2.clone(),
                });
            }
        }
    }
}

// ==================== Agent dispatch 包装层（register-app-layer-tools） ====================
//
// 这些函数不是 Tauri command，只供 `src-tauri/src/agent/registry.rs` 的 dispatch
// 调用；返回值统一为 `Result<String, String>` 与既有 dispatch 协议一致。
// 前端 JsonToQuery.vue 仍走 `json_to_query_params`（双字段 JsonToQueryResult）
// 保持契约不变。

/// Agent dispatch: JSON 对象 → URL-encoded query string。
/// 内部复用 `json_to_query_params` 的 `encoded` 字段；空对象返回空字符串。
pub fn json_to_query_dispatch(input: &str) -> Result<String, String> {
    let result = json_to_query_params(input.to_string())?;
    Ok(result.encoded)
}

/// Agent dispatch: 嵌套 JSON 展平为 `{path: value}` 对象。数组下标走 `[i]`、
/// 嵌套对象走 `[key]`，与 `json.to_query_dispatch` 同一路径风格。
pub fn json_flatten_dispatch(input: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(input).map_err(|e| format!("JSON 解析错误: {e}"))?;
    let mut pairs: Vec<(String, String)> = Vec::new();
    flatten_json_value(&value, String::new(), &mut pairs);
    let mut obj = serde_json::Map::new();
    for (k, v) in pairs {
        obj.insert(k, Value::String(v));
    }
    serde_json::to_string(&Value::Object(obj)).map_err(|e| format!("序列化失败: {e}"))
}

/// Agent dispatch: URL 字符串 → 结构化 JSON。path 拆为段数组；
/// query 重复键合并为 JSON 数组；空组件（如无 port）省略。
pub fn url_parse_dispatch(input: &str) -> Result<String, String> {
    let parsed = url::Url::parse(input.trim())
        .map_err(|e| format!("URL 解析失败: {e}"))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL 缺少 host".to_string())?
        .to_string();

    let mut out = serde_json::Map::new();
    out.insert("scheme".into(), Value::String(parsed.scheme().to_string()));
    out.insert("host".into(), Value::String(host));
    if let Some(port) = parsed.port() {
        out.insert("port".into(), Value::Number(port.into()));
    }
    let path_segments: Vec<Value> = parsed
        .path_segments()
        .map(|it| it.filter(|s| !s.is_empty()).map(|s| Value::String(s.to_string())).collect())
        .unwrap_or_default();
    out.insert("path".into(), Value::Array(path_segments));

    let mut query_map = serde_json::Map::new();
    for (k, v) in parsed.query_pairs() {
        let key = k.into_owned();
        let val = Value::String(v.into_owned());
        match query_map.remove(&key) {
            Some(Value::Array(mut arr)) => {
                arr.push(val);
                query_map.insert(key, Value::Array(arr));
            }
            Some(prev) => {
                query_map.insert(key, Value::Array(vec![prev, val]));
            }
            None => {
                query_map.insert(key, val);
            }
        }
    }
    out.insert("query".into(), Value::Object(query_map));

    if let Some(frag) = parsed.fragment() {
        out.insert("fragment".into(), Value::String(frag.to_string()));
    }

    serde_json::to_string(&Value::Object(out)).map_err(|e| format!("序列化失败: {e}"))
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;

    #[test]
    fn json_to_query_simple() {
        assert_eq!(json_to_query_dispatch(r#"{"aa":"bb"}"#).unwrap(), "aa=bb");
    }

    #[test]
    fn json_to_query_nested_and_array() {
        let out = json_to_query_dispatch(r#"{"a":{"b":"c"},"x":["1","2"]}"#).unwrap();
        // 嵌套对象走 [b]，数组走 [0]/[1]，key/value 均 URL 编码
        assert!(out.contains("a%5Bb%5D=c"), "got: {out}");
        assert!(out.contains("x%5B0%5D=1"), "got: {out}");
        assert!(out.contains("x%5B1%5D=2"), "got: {out}");
    }

    #[test]
    fn json_to_query_invalid_json() {
        assert!(json_to_query_dispatch("not json").is_err());
    }

    #[test]
    fn json_flatten_simple() {
        let out = json_flatten_dispatch(r#"{"a":1}"#).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["a"], "1");
    }

    #[test]
    fn json_flatten_nested_and_array() {
        let out = json_flatten_dispatch(r#"{"a":{"b":1},"c":[10,20]}"#).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["a[b]"], "1");
        assert_eq!(v["c[0]"], "10");
        assert_eq!(v["c[1]"], "20");
    }

    #[test]
    fn json_flatten_empty_object() {
        let out = json_flatten_dispatch("{}").unwrap();
        assert_eq!(out, "{}");
    }

    #[test]
    fn url_parse_full() {
        let out = url_parse_dispatch("https://api.x.com:8080/v1/x?k=v&k=w#frag").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["scheme"], "https");
        assert_eq!(v["host"], "api.x.com");
        assert_eq!(v["port"], 8080);
        assert_eq!(v["path"], serde_json::json!(["v1", "x"]));
        // 重复键合并为数组
        assert_eq!(v["query"]["k"], serde_json::json!(["v", "w"]));
        assert_eq!(v["fragment"], "frag");
    }

    #[test]
    fn url_parse_no_port_no_fragment() {
        let out = url_parse_dispatch("https://x.com/path").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert!(v.get("port").is_none());
        assert!(v.get("fragment").is_none());
    }

    #[test]
    fn url_parse_invalid() {
        assert!(url_parse_dispatch("not a url").is_err());
    }

    #[test]
    fn url_parse_decodes_query() {
        let out = url_parse_dispatch("https://x.com/?k=hi%20world").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["query"]["k"], "hi world");
    }

    #[test]
    fn format_pretty_accepts_escaped_json() {
        let escaped = r#"{\"a\":1,\"b\":\"x\"}"#;
        let result = format_json_pretty(escaped.to_string(), None).unwrap();
        assert!(result.formatted.contains("\"a\": 1"), "got: {}", result.formatted);
    }

    #[test]
    fn format_pretty_accepts_double_escaped_json() {
        // 模型在 tool args 里再套一层 escape 后的常见形态
        let double = r#"{\\\"query\\\":{\\\"x\\\":1}}"#;
        let result = format_json_pretty(double.to_string(), None).unwrap();
        assert!(result.formatted.contains("\"query\""), "got: {}", result.formatted);
    }

    #[test]
    fn format_pretty_accepts_triple_escaped_json() {
        // 小模型失败重试时再套一层 → 相对用户转义原文再 double 两次
        let once = r#"{\"query\":{\"x\":1}}"#;
        let double: String = once
            .chars()
            .flat_map(|c| if c == '\\' { vec!['\\', '\\'] } else { vec![c] })
            .collect();
        let triple: String = double
            .chars()
            .flat_map(|c| if c == '\\' { vec!['\\', '\\'] } else { vec![c] })
            .collect();
        let result = format_json_pretty(triple, None).unwrap();
        assert!(result.formatted.contains("\"query\""), "got: {}", result.formatted);
    }

    #[test]
    fn format_pretty_accepts_escaped_es_query_like_run() {
        let escaped = r#"{\"query\":{\"function_score\":{\"boost_mode\":\"replace\",\"functions\":[{\"filter\":{\"term\":{\"keywd\":\"将台\"}},\"weight\":3},{\"filter\":{\"bool\":{\"minimum_should_match\":\"1\",\"should\":{\"prefix\":{\"keywd.keyword\":\"将台\"}}}},\"weight\":2},{\"filter\":{\"bool\":{\"minimum_should_match\":\"1\",\"should\":{\"wildcard\":{\"keywd.keyword\":{\"wildcard\":\"*将台*\"}}}}},\"weight\":1},{\"filter\":{\"match_all\":{}},\"weight\":0}],\"query\":{\"bool\":{\"filter\":{\"term\":{\"is_del\":0}},\"minimum_should_match\":\"1\",\"must_not\":{\"term\":{\"manual_weight\":0}},\"should\":{\"term\":{\"synonym_wd\":\"将台\"}}}},\"score_mode\":\"first\"}},\"size\":10,\"sort\":[{\"_score\":{\"order\":\"desc\"}},{\"manual_weight\":{\"missing\":2,\"order\":\"desc\"}},{\"type_weight\":{\"order\":\"desc\"}},{\"house_counts\":{\"order\":\"desc\"}},{\"hits_counts\":{\"order\":\"desc\"}},{\"length\":{\"order\":\"asc\"}},{\"id\":{\"order\":\"asc\"}}]}"#;
        let result = format_json_pretty(escaped.to_string(), None).unwrap();
        assert!(result.formatted.contains("function_score"), "got: {}", result.formatted);
        assert!(result.formatted.contains("将台"), "got: {}", result.formatted);
    }

    #[test]
    fn parse_lenient_unwraps_json_string() {
        let wrapped = r#""{\"a\":1}""#;
        let v = parse_json_value_lenient(wrapped).unwrap();
        assert_eq!(v["a"], 1);
    }

    #[test]
    fn parse_lenient_ignores_trailing_braces() {
        // 小模型在 tool args 末尾多写一个 }
        let with_trail = r#"{"query":{"x":1}}}"#;
        let v = parse_json_value_lenient(with_trail).unwrap();
        assert_eq!(v["query"]["x"], 1);
    }

    #[test]
    fn format_pretty_ignores_trailing_chars_like_run() {
        let mut body = String::from(
            r#"{"query":{"function_score":{"boost_mode":"replace","functions":[{"filter":{"term":{"keywd":"朝阳"}},"weight":3}],"query":{"bool":{"must":{"regexp":{"keywd.keyword":{"value":"朝"}}}}}},"size":10,"sort":[{"_score":{"order":"desc"}}]}"#,
        );
        body.push('}'); // 多余尾括号，对应本次轨迹 trailing characters
        let result = format_json_pretty(body, None).unwrap();
        assert!(result.formatted.contains("function_score"), "got: {}", result.formatted);
    }
}
