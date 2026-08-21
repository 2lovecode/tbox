use serde_json::Value;
use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct JsonDiffResult {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<ModificationDetail>,
    pub unchanged: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct ModificationDetail {
    pub path: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

/// 对比两个JSON的差异
#[tauri::command]
pub fn compare_json(json1: String, json2: String) -> Result<JsonDiffResult, String> {
    let value1: Value = serde_json::from_str(&json1)
        .map_err(|e| format!("JSON1解析失败: {}", e))?;
    let value2: Value = serde_json::from_str(&json2)
        .map_err(|e| format!("JSON2解析失败: {}", e))?;

    let mut diff = JsonDiffResult {
        added: Vec::new(),
        removed: Vec::new(),
        modified: Vec::new(),
        unchanged: Vec::new(),
    };

    compare_values(&value1, &value2, &mut diff, "$");

    Ok(diff)
}

fn compare_values(v1: &Value, v2: &Value, diff: &mut JsonDiffResult, path: &str) {
    match (v1, v2) {
        // 都是对象
        (Value::Object(obj1), Value::Object(obj2)) => {
            let keys1: std::collections::HashSet<&String> = obj1.keys().collect();
            let keys2: std::collections::HashSet<&String> = obj2.keys().collect();

            // 找出新增的key
            for key in keys2.difference(&keys1) {
                diff.added.push(format!("{}/{}", path, key));
            }

            // 找出删除的key
            for key in keys1.difference(&keys2) {
                diff.removed.push(format!("{}/{}", path, key));
            }

            // 递归比较共同的key
            for key in keys1.intersection(&keys2) {
                let new_path = format!("{}/{}", path, key);
                compare_values(&obj1[*key], &obj2[*key], diff, &new_path);
            }
        }
        // 都是数组
        (Value::Array(arr1), Value::Array(arr2)) => {
            let len1 = arr1.len();
            let len2 = arr2.len();
            let min_len = len1.min(len2);

            // 比较共同索引的元素
            for i in 0..min_len {
                let new_path = format!("{}/[{}]", path, i);
                compare_values(&arr1[i], &arr2[i], diff, &new_path);
            }

            // 记录新增的元素
            for i in min_len..len2 {
                diff.added.push(format!("{}/[{}]", path, i));
            }

            // 记录删除的元素
            for i in min_len..len1 {
                diff.removed.push(format!("{}/[{}]", path, i));
            }
        }
        // 不同类型或值不相等
        _ => {
            if v1 != v2 {
                diff.modified.push(ModificationDetail {
                    path: path.to_string(),
                    old_value: v1.clone(),
                    new_value: v2.clone(),
                });
            } else {
                diff.unchanged.push(path.to_string());
            }
        }
    }
}

/// 格式化JSON差异为可读文本
#[tauri::command]
pub fn format_json_diff(json1: String, json2: String) -> Result<String, String> {
    let diff = compare_json(json1, json2)?;

    let mut result = String::new();
    result.push_str("=== JSON对比结果 ===\n\n");

    if !diff.added.is_empty() {
        result.push_str("✅ 新增字段:\n");
        for path in &diff.added {
            result.push_str(&format!("  + {}\n", path));
        }
        result.push('\n');
    }

    if !diff.removed.is_empty() {
        result.push_str("❌ 删除字段:\n");
        for path in &diff.removed {
            result.push_str(&format!("  - {}\n", path));
        }
        result.push('\n');
    }

    if !diff.modified.is_empty() {
        result.push_str("🔄 修改字段:\n");
        for mod_detail in &diff.modified {
            result.push_str(&format!("  ~ {}\n", mod_detail.path));
            result.push_str(&format!("    旧值: {}\n", mod_detail.old_value));
            result.push_str(&format!("    新值: {}\n", mod_detail.new_value));
        }
        result.push('\n');
    }

    if diff.unchanged.is_empty() && diff.added.is_empty() && diff.removed.is_empty() && diff.modified.is_empty() {
        result.push_str("两个JSON完全相同\n");
    } else if !diff.unchanged.is_empty() {
        result.push_str(&format!("ℹ️  未变化字段: {} 个\n", diff.unchanged.len()));
    }

    Ok(result)
}

/// 简单的JSON路径查询（用于测试）
#[tauri::command]
pub fn query_json_path(json: String, path: String) -> Result<serde_json::Value, String> {
    let value: Value = serde_json::from_str(&json)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    // 简单的路径解析，支持 $.key 或 $.key[0].nestedKey
    let result = query_json_path_helper(&value, &path)?;

    Ok(result)
}

fn query_json_path_helper(value: &Value, path: &str) -> Result<serde_json::Value, String> {
    let path = path.trim_start_matches("$.");
    if path.is_empty() {
        return Ok(value.clone());
    }

    let mut current = value;

    for segment in path.split('.') {
        let segment = segment.trim();

        // 处理数组索引 [0]
        if let Some(bracket_start) = segment.find('[') {
            let key = &segment[..bracket_start];
            let index_str = &segment[bracket_start + 1..segment.len() - 1];

            if !key.is_empty() {
                current = current.get(key)
                    .ok_or_else(|| format!("路径不存在: {}", key))?;
            }

            let index: usize = index_str.parse()
                .map_err(|_| format!("无效的数组索引: {}", index_str))?;

            current = current.get(index)
                .ok_or_else(|| format!("数组索引超出范围: {}", index))?;
        } else {
            current = current.get(segment)
                .ok_or_else(|| format!("路径不存在: {}", segment))?;
        }
    }

    Ok(current.clone())
}
