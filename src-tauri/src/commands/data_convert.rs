use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct JsonField {
    pub name: String,
    pub type_name: String,
    pub description: Option<String>,
}

/// JSON转实体类 - Java
#[tauri::command]
pub fn json_to_java(json_str: String, class_name: String) -> Result<String, String> {
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    let fields = extract_fields(&json_value);
    let mut result = String::new();

    // 生成导入
    result.push_str("import com.fasterxml.jackson.annotation.JsonProperty;\n");
    result.push_str("import java.util.List;\n");
    result.push_str("import java.util.Map;\n\n");

    result.push_str(&format!("public class {} {{\n", class_name));

    for field in &fields {
        let java_type = json_type_to_java_type(&field.type_name);
        result.push_str(&format!(
            "    @JsonProperty(\"{}\")\n",
            field.name
        ));
        result.push_str(&format!(
            "    private {} {};\n\n",
            java_type, field.name
        ));
    }

    // 生成getter和setter
    for field in &fields {
        let java_type = json_type_to_java_type(&field.type_name);
        let capitalized_name = capitalize_first(&field.name);

        result.push_str(&format!(
            "    public {} get{}() {{\n",
            java_type, capitalized_name
        ));
        result.push_str(&format!("        return this.{};\n", field.name));
        result.push_str("    }\n\n");

        result.push_str(&format!(
            "    public void set{}({} {}) {{\n",
            capitalized_name, java_type, field.name
        ));
        result.push_str(&format!("        this.{} = {};\n", field.name, field.name));
        result.push_str("    }\n\n");
    }

    result.push_str("}\n");

    Ok(result)
}

/// JSON转实体类 - C#
#[tauri::command]
pub fn json_to_csharp(json_str: String, class_name: String) -> Result<String, String> {
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    let fields = extract_fields(&json_value);
    let mut result = String::new();

    result.push_str("using System;\n");
    result.push_str("using System.Collections.Generic;\n");
    result.push_str("using Newtonsoft.Json;\n\n");

    result.push_str(&format!("public class {}\n{{\n", class_name));

    for field in &fields {
        let csharp_type = json_type_to_csharp_type(&field.type_name);
        result.push_str(&format!(
            "    [JsonProperty(\"{}\")]\n",
            field.name
        ));
        result.push_str(&format!(
            "    public {} {} {{ get; set; }}\n",
            csharp_type, to_pascal_case(&field.name)
        ));
    }

    result.push_str("}\n");

    Ok(result)
}

/// JSON转实体类 - Go
#[tauri::command]
pub fn json_to_go(json_str: String, struct_name: String) -> Result<String, String> {
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    let fields = extract_fields(&json_value);
    let mut result = String::new();

    result.push_str(&format!("type {} struct {{\n", struct_name));

    for field in &fields {
        let go_type = json_type_to_go_type(&field.type_name);
        result.push_str(&format!(
            "    {} {} `json:\"{}\"`\n",
            to_pascal_case(&field.name),
            go_type,
            field.name
        ));
    }

    result.push_str("}\n");

    Ok(result)
}

/// JSON转实体类 - Python
#[tauri::command]
pub fn json_to_python(json_str: String, class_name: String) -> Result<String, String> {
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    let fields = extract_fields(&json_value);
    let mut result = String::new();

    result.push_str(&format!("class {}:\n", class_name));

    for field in &fields {
        let python_type = json_type_to_python_type(&field.type_name);
        result.push_str(&format!(
            "    {}: {}  # type: {}\n",
            field.name, "None", python_type
        ));
    }

    result.push_str("\n");
    result.push_str("    def __init__(self");
    for field in &fields {
        result.push_str(&format!(", {}", field.name));
    }
    result.push_str("):\n");

    for field in &fields {
        result.push_str(&format!("        self.{} = {}\n", field.name, field.name));
    }

    Ok(result)
}

/// JSON转实体类 - TypeScript
#[tauri::command]
pub fn json_to_typescript(json_str: String, interface_name: String) -> Result<String, String> {
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    let fields = extract_fields(&json_value);
    let mut result = String::new();

    result.push_str(&format!("export interface {} {{\n", interface_name));

    for field in &fields {
        let ts_type = json_type_to_typescript_type(&field.type_name);
        result.push_str(&format!(
            "    {}?: {};\n",
            field.name, ts_type
        ));
    }

    result.push_str("}\n");

    Ok(result)
}

// ============ 辅助函数 ============

/// 从JSON值中提取字段
fn extract_fields(json: &serde_json::Value) -> Vec<JsonField> {
    let mut fields = Vec::new();

    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            let type_name = get_json_type(value);
            fields.push(JsonField {
                name: key.clone(),
                type_name,
                description: None,
            });
        }
    }

    fields
}

/// 获取JSON值的类型
fn get_json_type(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "string".to_string(),
        serde_json::Value::Bool(_) => "boolean".to_string(),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                "integer".to_string()
            } else {
                "number".to_string()
            }
        }
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(arr) => {
            if !arr.is_empty() {
                let inner_type = get_json_type(&arr[0]);
                format!("array<{}>", inner_type)
            } else {
                "array".to_string()
            }
        }
        serde_json::Value::Object(_) => "object".to_string(),
    }
}

/// JSON类型转Java类型
fn json_type_to_java_type(json_type: &str) -> &str {
    match json_type {
        "string" => "String",
        "integer" => "Integer",
        "number" => "Double",
        "boolean" => "Boolean",
        "array" => "List<Object>",
        t if t.starts_with("array<") => {
            let inner = &t[6..t.len()-1];
            let java_inner = json_type_to_java_type(inner);
            if java_inner == "Object" {
                "List<Object>"
            } else {
                "List<String>"
            }
        }
        "object" => "Map<String, Object>",
        _ => "Object",
    }
}

/// JSON类型转C#类型
fn json_type_to_csharp_type(json_type: &str) -> &str {
    match json_type {
        "string" => "string",
        "integer" => "int",
        "number" => "double",
        "boolean" => "bool",
        "array" => "List<object>",
        t if t.starts_with("array<") => "List<string>",
        "object" => "Dictionary<string, object>",
        _ => "object",
    }
}

/// JSON类型转Go类型
fn json_type_to_go_type(json_type: &str) -> &str {
    match json_type {
        "string" => "string",
        "integer" => "int",
        "number" => "float64",
        "boolean" => "bool",
        "array" => "[]interface{}",
        t if t.starts_with("array<") => "[]string",
        "object" => "map[string]interface{}",
        _ => "interface{}",
    }
}

/// JSON类型转Python类型
fn json_type_to_python_type(json_type: &str) -> &str {
    match json_type {
        "string" => "str",
        "integer" => "int",
        "number" => "float",
        "boolean" => "bool",
        "array" => "list",
        "object" => "dict",
        _ => "Any",
    }
}

/// JSON类型转TypeScript类型
fn json_type_to_typescript_type(json_type: &str) -> &str {
    match json_type {
        "string" => "string",
        "integer" => "number",
        "number" => "number",
        "boolean" => "boolean",
        "array" => "any[]",
        "object" => "Record<string, any>",
        _ => "any",
    }
}

/// 首字母大写
fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// 转为PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| capitalize_first(word))
        .collect()
}
