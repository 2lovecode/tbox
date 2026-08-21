//! Allowlisted Agent 工具注册表：lookup + schema 校验 + dispatch。
//!
//! 仅注册 `side_effect=None` 的纯计算工具；未注册 id 一律拒绝。

use base64::{engine::general_purpose, Engine as _};
use md5::{Digest, Md5};
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::HashMap;

/// 工具副作用标记。第一期仅 `None`；预留未来变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffect {
    None,
}

/// 注册表中的工具规格（id、展示名、参数 schema、副作用）。
#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub schema: Value,
    pub side_effect: SideEffect,
}

fn required_string_props(props: &[(&str, &str)]) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for (name, description) in props {
        properties.insert(
            (*name).to_string(),
            json!({
                "type": "string",
                "description": description,
            }),
        );
        required.push(Value::String((*name).to_string()));
    }
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

fn tool(
    id: &'static str,
    name: &'static str,
    schema: Value,
) -> ToolSpec {
    ToolSpec {
        id,
        name,
        schema,
        side_effect: SideEffect::None,
    }
}

static TOOLS: Lazy<Vec<ToolSpec>> = Lazy::new(|| {
    vec![
        tool(
            "json.format",
            "JSON 格式化",
            required_string_props(&[("input", "待格式化的 JSON 字符串")]),
        ),
        tool(
            "base64.encode",
            "Base64 编码",
            required_string_props(&[("input", "待编码的明文")]),
        ),
        tool(
            "base64.decode",
            "Base64 解码",
            required_string_props(&[("input", "待解码的 Base64 字符串")]),
        ),
        tool(
            "hash.digest",
            "哈希摘要",
            json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string", "description": "待哈希的文本" },
                    "algorithm": {
                        "type": "string",
                        "description": "算法：md5 或 sha256",
                        "enum": ["md5", "sha256"]
                    }
                },
                "required": ["input", "algorithm"],
                "additionalProperties": false
            }),
        ),
        tool(
            "jwt.parse",
            "JWT 解析（不验签）",
            required_string_props(&[("input", "JWT 字符串")]),
        ),
        tool(
            "timestamp.convert",
            "时间戳转换",
            json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "Unix 秒/毫秒时间戳，或 ISO-8601 时间字符串"
                    },
                    "unit": {
                        "type": "string",
                        "description": "输入单位：seconds、millis，或 iso（默认自动推断）",
                        "enum": ["seconds", "millis", "iso"]
                    }
                },
                "required": ["input"],
                "additionalProperties": false
            }),
        ),
        tool(
            "encoding.convert",
            "URL 编码",
            required_string_props(&[("input", "待 URL 编码的文本")]),
        ),
        tool(
            "xml.format",
            "XML 格式化",
            required_string_props(&[("input", "待格式化的 XML 字符串")]),
        ),
        tool(
            "yaml.format",
            "YAML 格式化",
            required_string_props(&[("input", "待格式化的 YAML 字符串")]),
        ),
        tool(
            "uuid.generate",
            "生成 UUID v4",
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        ),
        tool(
            "cron.explain",
            "Cron 表达式说明",
            required_string_props(&[("input", "标准 5 字段 cron 表达式")]),
        ),
        tool(
            "number.convert",
            "进制转换",
            json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string", "description": "待转换的数字字符串" },
                    "from_base": { "type": "integer", "description": "源进制 2..=36", "minimum": 2, "maximum": 36 },
                    "to_base": { "type": "integer", "description": "目标进制 2..=36", "minimum": 2, "maximum": 36 }
                },
                "required": ["input", "from_base", "to_base"],
                "additionalProperties": false
            }),
        ),
        tool(
            "charset.convert",
            "字符集标签校验",
            required_string_props(&[
                ("input", "文本内容"),
                ("charset", "字符集标签，例如 UTF-8"),
            ]),
        ),
    ]
});

static BY_ID: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| {
    TOOLS
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id, i))
        .collect()
});

/// 按稳定字符串 id 查找工具规格。
pub fn lookup(tool_id: &str) -> Option<&'static ToolSpec> {
    BY_ID.get(tool_id).map(|&i| &TOOLS[i])
}

/// OpenAI Chat Completions `tools` array for the registered allowlist.
pub fn tools_as_openai_json() -> Value {
    let tools: Vec<Value> = TOOLS
        .iter()
        .map(|t| {
            json!({
                "type": "function",
                "function": {
                    "name": t.id,
                    "description": t.name,
                    "parameters": t.schema,
                }
            })
        })
        .collect();
    Value::Array(tools)
}

/// 校验 args 是否满足工具 schema 的必填字段与基本类型（手写，无 jsonschema 依赖）。
fn validate_args(spec: &ToolSpec, args: &Value) -> Result<(), String> {
    let obj = args.as_object().ok_or_else(|| {
        "参数不符合 schema：期望 JSON object".to_string()
    })?;

    let schema_obj = spec
        .schema
        .as_object()
        .ok_or_else(|| "内部错误：schema 非 object".to_string())?;

    if let Some(required) = schema_obj.get("required").and_then(|v| v.as_array()) {
        for req in required {
            let key = req.as_str().unwrap_or("");
            if key.is_empty() {
                continue;
            }
            match obj.get(key) {
                None => {
                    return Err(format!("参数不符合 schema：缺少必填字段 `{key}`"));
                }
                Some(Value::Null) => {
                    return Err(format!("参数不符合 schema：字段 `{key}` 不能为 null"));
                }
                Some(Value::String(s)) if s.is_empty() && key == "input" => {
                    // allow empty string for some tools; schema type still satisfied
                }
                _ => {}
            }
        }
    }

    if let Some(props) = schema_obj.get("properties").and_then(|v| v.as_object()) {
        for (key, prop_schema) in props {
            if let Some(val) = obj.get(key) {
                if let Some(expected) = prop_schema.get("type").and_then(|t| t.as_str()) {
                    let ok = match expected {
                        "string" => val.is_string(),
                        "integer" => val.is_i64() || val.is_u64(),
                        "number" => val.is_number(),
                        "boolean" => val.is_boolean(),
                        "object" => val.is_object(),
                        "array" => val.is_array(),
                        _ => true,
                    };
                    if !ok {
                        return Err(format!(
                            "参数不符合 schema：字段 `{key}` 期望类型 {expected}"
                        ));
                    }
                }
                if let Some(enum_vals) = prop_schema.get("enum").and_then(|e| e.as_array()) {
                    if !enum_vals.iter().any(|e| e == val) {
                        return Err(format!(
                            "参数不符合 schema：字段 `{key}` 不在允许枚举内"
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

fn require_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("参数错误：缺少字符串字段 `{key}`"))
}

/// 调度已注册工具。未注册或参数非法时返回错误，不执行底层逻辑。
pub fn dispatch(tool_id: &str, args: &Value) -> Result<String, String> {
    let spec = lookup(tool_id).ok_or_else(|| {
        format!("未注册的工具: {tool_id}（unknown tool）")
    })?;

    validate_args(spec, args)?;

    match tool_id {
        "json.format" => dispatch_json_format(args),
        "base64.encode" => {
            let input = require_str(args, "input")?;
            Ok(general_purpose::STANDARD.encode(input.as_bytes()))
        }
        "base64.decode" => {
            let input = require_str(args, "input")?;
            let bytes = general_purpose::STANDARD
                .decode(input.trim())
                .map_err(|e| format!("Base64 解码失败: {e}"))?;
            String::from_utf8(bytes).map_err(|e| format!("解码结果非 UTF-8: {e}"))
        }
        "hash.digest" => dispatch_hash(args),
        "jwt.parse" => dispatch_jwt_parse(args),
        "timestamp.convert" => dispatch_timestamp(args),
        "encoding.convert" => {
            let input = require_str(args, "input")?;
            crate::commands::encoding::url_encode(input.to_string())
        }
        "xml.format" => dispatch_xml_format(args),
        "yaml.format" => dispatch_yaml_format(args),
        "uuid.generate" => Ok(uuid::Uuid::new_v4().to_string()),
        "cron.explain" => dispatch_cron_explain(args),
        "number.convert" => dispatch_number_convert(args),
        "charset.convert" => dispatch_charset_convert(args),
        _ => Err(format!("未注册的工具: {tool_id}（unknown tool）")),
    }
}

fn dispatch_json_format(args: &Value) -> Result<String, String> {
    let input = require_str(args, "input")?;
    let result = crate::commands::json::format_json_pretty(input.to_string(), None)?;
    // JsonFormatResult 字段私有，经 Serialize 输出；优先取出 formatted。
    let v = serde_json::to_value(&result).map_err(|e| e.to_string())?;
    v.get("formatted")
        .and_then(|f| f.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "JSON 格式化结果缺少 formatted".to_string())
}

fn dispatch_hash(args: &Value) -> Result<String, String> {
    let input = require_str(args, "input")?;
    let algorithm = require_str(args, "algorithm")?.to_ascii_lowercase();
    match algorithm.as_str() {
        "md5" => {
            let mut hasher = Md5::new();
            hasher.update(input.as_bytes());
            Ok(format!("{:x}", hasher.finalize()))
        }
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            Ok(format!("{:x}", hasher.finalize()))
        }
        other => Err(format!("不支持的哈希算法: {other}（仅 md5 / sha256）")),
    }
}

fn b64url_decode(segment: &str) -> Result<Vec<u8>, String> {
    let mut s = segment.replace('-', "+").replace('_', "/");
    while s.len() % 4 != 0 {
        s.push('=');
    }
    general_purpose::STANDARD
        .decode(&s)
        .map_err(|e| format!("JWT Base64URL 解码失败: {e}"))
}

fn dispatch_jwt_parse(args: &Value) -> Result<String, String> {
    let input = require_str(args, "input")?.trim();
    let parts: Vec<&str> = input.split('.').collect();
    if parts.len() < 2 {
        return Err("非法 JWT：至少需要 header.payload".to_string());
    }
    let header_bytes = b64url_decode(parts[0])?;
    let payload_bytes = b64url_decode(parts[1])?;
    let header: Value = serde_json::from_slice(&header_bytes)
        .map_err(|e| format!("JWT header 不是合法 JSON: {e}"))?;
    let payload: Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("JWT payload 不是合法 JSON: {e}"))?;
    Ok(json!({
        "header": header,
        "payload": payload,
        "verified": false,
    })
    .to_string())
}

fn dispatch_timestamp(args: &Value) -> Result<String, String> {
    use chrono::{DateTime, TimeZone, Utc};

    let input = require_str(args, "input")?.trim();
    let unit = args
        .get("unit")
        .and_then(|v| v.as_str())
        .unwrap_or("auto");

    let dt: DateTime<Utc> = if unit == "iso"
        || (!input.chars().all(|c| c.is_ascii_digit() || c == '-') && input.contains(|c: char| c == 'T' || c == '-' || c == ':'))
    {
        DateTime::parse_from_rfc3339(input)
            .map(|d| d.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S")
                    .map(|n| Utc.from_utc_datetime(&n))
            })
            .map_err(|e| format!("无法解析时间字符串: {e}"))?
    } else {
        let n: i64 = input
            .parse()
            .map_err(|e| format!("无法解析时间戳数字: {e}"))?;
        let secs = match unit {
            "millis" => n.div_euclid(1000),
            "seconds" => n,
            _ => {
                // 自动：13 位左右当毫秒
                if n.abs() >= 1_000_000_000_000 {
                    n.div_euclid(1000)
                } else {
                    n
                }
            }
        };
        Utc.timestamp_opt(secs, 0)
            .single()
            .ok_or_else(|| "时间戳超出范围".to_string())?
    };

    Ok(json!({
        "iso": dt.to_rfc3339(),
        "unix_seconds": dt.timestamp(),
        "unix_millis": dt.timestamp_millis(),
    })
    .to_string())
}

fn dispatch_xml_format(args: &Value) -> Result<String, String> {
    let input = require_str(args, "input")?;
    // 最小可用：用 serde-xml-rs 解析为 Value 再序列化；失败则返回缩进启发式。
    match serde_xml_rs::from_str::<Value>(input) {
        Ok(v) => serde_xml_rs::to_string(&v).map_err(|e| format!("XML 序列化失败: {e}")),
        Err(_) => {
            // 回退：去掉多余空白并保留结构感
            let compact = input.split_whitespace().collect::<Vec<_>>().join(" ");
            if compact.is_empty() {
                return Err("XML 输入为空".to_string());
            }
            Ok(compact)
        }
    }
}

fn dispatch_yaml_format(args: &Value) -> Result<String, String> {
    let input = require_str(args, "input")?;
    let value: Value =
        serde_yaml::from_str(input).map_err(|e| format!("YAML 解析失败: {e}"))?;
    serde_yaml::to_string(&value).map_err(|e| format!("YAML 序列化失败: {e}"))
}

fn dispatch_cron_explain(args: &Value) -> Result<String, String> {
    let input = require_str(args, "input")?.trim();
    let fields: Vec<&str> = input.split_whitespace().collect();
    if fields.len() != 5 {
        return Err("参数错误：cron 需为 5 字段（分 时 日 月 周）".to_string());
    }
    let labels = ["分钟", "小时", "日", "月", "星期"];
    let mut parts = Vec::new();
    for (label, field) in labels.iter().zip(fields.iter()) {
        let desc = if *field == "*" {
            format!("每{label}")
        } else if field.starts_with("*/") {
            format!("每 {} {label}", &field[2..])
        } else {
            format!("{label}={field}")
        };
        parts.push(desc);
    }
    Ok(parts.join("；"))
}

fn dispatch_number_convert(args: &Value) -> Result<String, String> {
    let input = require_str(args, "input")?.trim();
    let from_base = args
        .get("from_base")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "参数不符合 schema：缺少整数字段 `from_base`".to_string())?
        as u32;
    let to_base = args
        .get("to_base")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "参数不符合 schema：缺少整数字段 `to_base`".to_string())?
        as u32;

    if !(2..=36).contains(&from_base) || !(2..=36).contains(&to_base) {
        return Err("进制必须在 2..=36".to_string());
    }

    let value = i128::from_str_radix(input, from_base)
        .map_err(|e| format!("数字解析失败: {e}"))?;

    // 手动转目标进制
    if value == 0 {
        return Ok("0".to_string());
    }
    let negative = value < 0;
    let mut n = value.unsigned_abs();
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    while n > 0 {
        let rem = (n % to_base as u128) as usize;
        out.push(digits[rem] as char);
        n /= to_base as u128;
    }
    if negative {
        out.push('-');
    }
    Ok(out.into_iter().rev().collect())
}

fn dispatch_charset_convert(args: &Value) -> Result<String, String> {
    let input = require_str(args, "input")?;
    let charset = require_str(args, "charset")?.trim();
    if charset.is_empty() {
        return Err("参数不符合 schema：`charset` 不能为空".to_string());
    }
    // 最小可用：校验标签非空；UTF-8 / utf8 原样返回文本。
    let normalized = charset.replace('_', "-").to_ascii_lowercase();
    if normalized == "utf-8" || normalized == "utf8" {
        return Ok(input.to_string());
    }
    // 尝试用 encoding_rs 识别标签；无法识别则报错，识别成功则按 UTF-8 往返说明。
    match encoding_rs::Encoding::for_label(charset.as_bytes()) {
        Some(enc) => {
            let (cow, _, had_errors) = enc.decode(input.as_bytes());
            if had_errors {
                return Err(format!("按 {charset} 解码出现替换错误"));
            }
            Ok(cow.into_owned())
        }
        None => Err(format!("未知字符集标签: {charset}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn unknown_tool_is_rejected() {
        let err = dispatch("http.request", &json!({})).unwrap_err();
        assert!(err.contains("未注册") || err.contains("unknown"));
    }

    #[test]
    fn invalid_args_do_not_run() {
        let err = dispatch("base64.encode", &json!({})).unwrap_err();
        assert!(err.contains("schema") || err.contains("参数"));
    }

    #[test]
    fn base64_roundtrip() {
        let encoded = dispatch("base64.encode", &json!({"input": "hi"})).unwrap();
        let decoded = dispatch("base64.decode", &json!({"input": encoded})).unwrap();
        assert_eq!(decoded, "hi");
    }

    #[test]
    fn all_allowlisted_tools_lookupable() {
        let ids = [
            "json.format",
            "base64.encode",
            "base64.decode",
            "hash.digest",
            "jwt.parse",
            "timestamp.convert",
            "encoding.convert",
            "xml.format",
            "yaml.format",
            "uuid.generate",
            "cron.explain",
            "number.convert",
            "charset.convert",
        ];
        for id in ids {
            let spec = lookup(id).unwrap_or_else(|| panic!("missing {id}"));
            assert_eq!(spec.side_effect, SideEffect::None);
            assert_eq!(spec.id, id);
        }
        assert!(lookup("http.request").is_none());
    }
}
