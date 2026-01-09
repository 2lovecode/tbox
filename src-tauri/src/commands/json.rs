use serde::Serialize;
use serde_json::Value;

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

    // 解析JSON
    let value: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析错误: {}", e))?;

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
