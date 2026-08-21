use serde_json::{Value, to_string_pretty};

/// YAML格式化
#[tauri::command]
pub fn format_yaml(input: String) -> Result<String, String> {
    let value: Value = serde_yaml::from_str(&input)
        .map_err(|e| format!("YAML解析失败: {}", e))?;

    serde_yaml::to_string(&value).map_err(|e| format!("YAML序列化失败: {}", e))
}

/// YAML转JSON
#[tauri::command]
pub fn yaml_to_json(input: String) -> Result<String, String> {
    let value: Value = serde_yaml::from_str(&input)
        .map_err(|e| format!("YAML解析失败: {}", e))?;

    to_string_pretty(&value).map_err(|e| format!("JSON序列化失败: {}", e))
}

/// JSON转YAML
#[tauri::command]
pub fn json_to_yaml(input: String) -> Result<String, String> {
    let value: Value = serde_json::from_str(&input)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    serde_yaml::to_string(&value).map_err(|e| format!("YAML序列化失败: {}", e))
}

/// YAML验证
#[tauri::command]
pub fn validate_yaml(input: String) -> Result<bool, String> {
    match serde_yaml::from_str::<Value>(&input) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("YAML格式错误: {}", e))
    }
}

/// YAML合并
#[tauri::command]
pub fn merge_yaml(yaml1: String, yaml2: String) -> Result<String, String> {
    let mut value1: Value = serde_yaml::from_str(&yaml1)
        .map_err(|e| format!("YAML1解析失败: {}", e))?;

    let value2: Value = serde_yaml::from_str(&yaml2)
        .map_err(|e| format!("YAML2解析失败: {}", e))?;

    // 简单的合并策略：如果是对象，合并键值
    if let (Some(obj1), Some(obj2)) = (value1.as_object_mut(), value2.as_object()) {
        for (key, value) in obj2 {
            obj1.insert(key.clone(), value.clone());
        }
    }

    serde_yaml::to_string(&value1).map_err(|e| format!("YAML序列化失败: {}", e))
}
