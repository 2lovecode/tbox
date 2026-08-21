use serde_xml_rs;
use serde_json::{Value, to_string_pretty};

/// XML格式化
#[tauri::command]
pub fn format_xml(input: String) -> Result<String, String> {
    // 尝试解析XML以验证格式
    let value: Value = serde_xml_rs::from_str(&input)
        .map_err(|e| format!("XML解析失败: {}", e))?;

    // 重新生成格式化的XML
    serde_xml_rs::to_string(&value).map_err(|e| format!("XML序列化失败: {}", e))
}

/// XML压缩
#[tauri::command]
pub fn minify_xml(input: String) -> Result<String, String> {
    let value: Value = serde_xml_rs::from_str(&input)
        .map_err(|e| format!("XML解析失败: {}", e))?;

    // 生成XML后移除所有换行和多余空格
    let xml = serde_xml_rs::to_string(&value)
        .map_err(|e| format!("XML序列化失败: {}", e))?;

    Ok(xml.split_whitespace().collect())
}

/// XML转JSON
#[tauri::command]
pub fn xml_to_json(input: String) -> Result<String, String> {
    let value: Value = serde_xml_rs::from_str(&input)
        .map_err(|e| format!("XML解析失败: {}", e))?;

    to_string_pretty(&value).map_err(|e| format!("JSON序列化失败: {}", e))
}

/// JSON转XML
#[tauri::command]
pub fn json_to_xml(input: String) -> Result<String, String> {
    let value: Value = serde_json::from_str(&input)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    serde_xml_rs::to_string(&value).map_err(|e| format!("XML生成失败: {}", e))
}

/// XML转YAML
#[tauri::command]
pub fn xml_to_yaml(input: String) -> Result<String, String> {
    let value: Value = serde_xml_rs::from_str(&input)
        .map_err(|e| format!("XML解析失败: {}", e))?;

    serde_yaml::to_string(&value).map_err(|e| format!("YAML序列化失败: {}", e))
}

/// YAML转XML
#[tauri::command]
pub fn yaml_to_xml(input: String) -> Result<String, String> {
    let value: Value = serde_yaml::from_str(&input)
        .map_err(|e| format!("YAML解析失败: {}", e))?;

    serde_xml_rs::to_string(&value).map_err(|e| format!("XML生成失败: {}", e))
}

/// XPath查询（简化实现）
#[tauri::command]
pub fn xpath_query(xml: String, _xpath: String) -> Result<String, String> {
    // 解析XML以验证格式
    let value: Value = serde_xml_rs::from_str(&xml)
        .map_err(|e| format!("XML解析失败: {}", e))?;

    // 简化实现：返回整个XML的JSON表示
    // 完整的XPath查询需要额外的XPath库
    to_string_pretty(&value).map_err(|e| format!("JSON序列化失败: {}", e))
}
