use serde::Serialize;
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
