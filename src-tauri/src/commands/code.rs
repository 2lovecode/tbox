use serde::Serialize;

#[derive(Serialize)]
pub struct FormatCodeResult {
    formatted: String,
    language: String,
}

#[tauri::command]
pub fn format_json(code: String) -> Result<FormatCodeResult, String> {
    let parsed: serde_json::Value = serde_json::from_str(&code)
        .map_err(|e| format!("JSON解析错误: {}", e))?;

    let formatted = serde_json::to_string_pretty(&parsed)
        .map_err(|e| format!("JSON格式化错误: {}", e))?;

    Ok(FormatCodeResult {
        formatted,
        language: "json".to_string(),
    })
}

#[tauri::command]
pub fn format_code(
    code: String,
    language: String,
    indent_size: Option<usize>,
    use_tabs: Option<bool>,
) -> Result<FormatCodeResult, String> {
    let indent_size = indent_size.unwrap_or(2);
    let use_tabs = use_tabs.unwrap_or(false);
    let indent = if use_tabs {
        "\t".to_string()
    } else {
        " ".repeat(indent_size)
    };

    let formatted = match language.as_str() {
        "json" => {
            let parsed: serde_json::Value = serde_json::from_str(&code)
                .map_err(|e| format!("JSON解析错误: {}", e))?;
            serde_json::to_string_pretty(&parsed)
                .map_err(|e| format!("JSON格式化错误: {}", e))?
        }
        "javascript" | "typescript" => {
            // 简单的JavaScript/TypeScript格式化
            format_javascript(&code, &indent)
        }
        _ => {
            // 通用格式化（基础缩进）
            format_generic(&code, &indent)
        }
    };

    Ok(FormatCodeResult {
        formatted,
        language,
    })
}

fn format_javascript(code: &str, indent: &str) -> String {
    let mut formatted = String::new();
    let mut current_indent: usize = 0;
    let lines: Vec<&str> = code.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            formatted.push('\n');
            continue;
        }

        // 减少缩进
        if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
            current_indent = current_indent.saturating_sub(1);
        }

        // 添加缩进
        formatted.push_str(&indent.repeat(current_indent));
        formatted.push_str(trimmed);
        formatted.push('\n');

        // 增加缩进
        if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
            current_indent += 1;
        }
    }

    formatted.trim().to_string()
}

fn format_generic(code: &str, indent: &str) -> String {
    let mut formatted = String::new();
    let mut current_indent: usize = 0;
    let lines: Vec<&str> = code.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            formatted.push('\n');
            continue;
        }

        // 简单的缩进处理
        if trimmed.starts_with('}') || trimmed.starts_with("end") || trimmed.starts_with("endif") {
            current_indent = current_indent.saturating_sub(1);
        }

        formatted.push_str(&indent.repeat(current_indent));
        formatted.push_str(trimmed);
        formatted.push('\n');

        if trimmed.ends_with('{') || trimmed.contains(':') {
            current_indent += 1;
        }
    }

    formatted.trim().to_string()
}
