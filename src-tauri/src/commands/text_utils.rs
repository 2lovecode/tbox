use regex::Regex;
use std::collections::HashSet;

/// 正则表达式测试
#[tauri::command]
pub fn regex_test(pattern: String, text: String, flags: String) -> Result<Vec<String>, String> {
    let mut regex = String::new();

    // 解析flags
    if flags.contains('i') {
        regex.push_str("(?i)");
    }
    if flags.contains('m') {
        regex.push_str("(?m)");
    }
    if flags.contains('s') {
        regex.push_str("(?s)");
    }

    regex.push_str(&pattern);

    let re = Regex::new(&regex)
        .map_err(|e| format!("正则表达式错误: {}", e))?;

    let matches: Vec<String> = re.find_iter(&text)
        .map(|m| m.as_str().to_string())
        .collect();

    Ok(matches)
}

/// 正则表达式替换
#[tauri::command]
pub fn regex_replace(pattern: String, text: String, replacement: String, flags: String) -> Result<String, String> {
    let mut regex = String::new();

    if flags.contains('i') {
        regex.push_str("(?i)");
    }
    if flags.contains('m') {
        regex.push_str("(?m)");
    }
    if flags.contains('s') {
        regex.push_str("(?s)");
    }

    regex.push_str(&pattern);

    let re = Regex::new(&regex)
        .map_err(|e| format!("正则表达式错误: {}", e))?;

    let result = re.replace_all(&text, &replacement).to_string();
    Ok(result)
}

/// 文本对比
#[tauri::command]
pub fn text_compare(text1: String, text2: String) -> serde_json::Value {
    let lines1: Vec<&str> = text1.lines().collect();
    let lines2: Vec<&str> = text2.lines().collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = Vec::new();

    let max_len = lines1.len().max(lines2.len());

    for i in 0..max_len {
        let line1 = lines1.get(i).map(|s| *s).unwrap_or("");
        let line2 = lines2.get(i).map(|s| *s).unwrap_or("");

        if line1 == line2 {
            if !line1.is_empty() {
                unchanged.push((i + 1, line1.to_string()));
            }
        } else {
            if !line1.is_empty() && i < lines1.len() {
                removed.push((i + 1, line1.to_string()));
            }
            if !line2.is_empty() && i < lines2.len() {
                added.push((i + 1, line2.to_string()));
            }
            if !line1.is_empty() && !line2.is_empty() {
                modified.push((i + 1, line1.to_string(), line2.to_string()));
            }
        }
    }

    serde_json::json!({
        "added": added,
        "removed": removed,
        "modified": modified,
        "unchanged": unchanged
    })
}

/// 文本去重
#[tauri::command]
pub fn text_deduplicate(text: String, ignore_case: bool) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for line in lines {
        let key = if ignore_case {
            line.to_lowercase()
        } else {
            line.to_string()
        };

        if seen.insert(key) {
            result.push(line);
        }
    }

    result.join("\n")
}

/// 文本排序
#[tauri::command]
pub fn text_sort(text: String, order: String, ignore_case: bool) -> String {
    let mut lines: Vec<&str> = text.lines().collect();

    lines.sort_by(|a, b| {
        let (a, b) = if ignore_case {
            (a.to_lowercase(), b.to_lowercase())
        } else {
            (a.to_string(), b.to_string())
        };

        if order == "desc" {
            b.cmp(&a)
        } else {
            a.cmp(&b)
        }
    });

    lines.join("\n")
}

/// 文本反转
#[tauri::command]
pub fn text_reverse(text: String, mode: String) -> String {
    match mode.as_str() {
        "line" => {
            // 逐行反转
            text.lines()
                .rev()
                .collect::<Vec<&str>>()
                .join("\n")
        }
        "char" => {
            // 字符级反转
            text.chars().rev().collect()
        }
        _ => text
    }
}

/// 文本统计
#[tauri::command]
pub fn text_statistics(text: String) -> serde_json::Value {
    let lines = text.lines().count();
    let chars = text.chars().count();
    let bytes = text.len();
    let words = text.split_whitespace().count();

    // 统计中文字符
    let chinese_chars = text.chars()
        .filter(|c| {
            let cp = *c as u32;
            (0x4E00..=0x9FFF).contains(&cp)
        })
        .count();

    // 统计空白字符
    let whitespace = text.chars()
        .filter(|c| c.is_whitespace())
        .count();

    serde_json::json!({
        "lines": lines,
        "chars": chars,
        "bytes": bytes,
        "words": words,
        "chineseChars": chinese_chars,
        "whitespace": whitespace
    })
}

/// 驼峰命名转换
#[tauri::command]
pub fn convert_naming(text: String, from: String, to: String) -> String {
    match (from.as_str(), to.as_str()) {
        ("camelCase", "snake_case") => camel_to_snake(&text),
        ("camelCase", "kebab-case") => camel_to_kebab(&text),
        ("snake_case", "camelCase") => snake_to_camel(&text),
        ("snake_case", "PascalCase") => snake_to_pascal(&text),
        ("kebab-case", "camelCase") => kebab_to_camel(&text),
        ("camelCase", "PascalCase") => camel_to_pascal(&text),
        ("PascalCase", "camelCase") => pascal_to_camel(&text),
        _ => text
    }
}

// 驼峰转下划线
fn camel_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

// 驼峰转短横线
fn camel_to_kebab(s: &str) -> String {
    camel_to_snake(s).replace('_', "-")
}

// 下划线转驼峰
fn snake_to_camel(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    let mut result = parts[0].to_string();
    for part in &parts[1..] {
        if !part.is_empty() {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                result.extend(first.to_uppercase());
                result.extend(chars);
            }
        }
    }
    result
}

// 下划线转帕斯卡
fn snake_to_pascal(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    let mut result = String::new();
    for part in parts {
        if !part.is_empty() {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                result.extend(first.to_uppercase());
                result.extend(chars);
            }
        }
    }
    result
}

// 短横线转驼峰
fn kebab_to_camel(s: &str) -> String {
    let s = s.replace('-', "_");
    snake_to_camel(&s)
}

// 驼峰转帕斯卡
fn camel_to_pascal(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

// 帕斯卡转驼峰
fn pascal_to_camel(s: &str) -> String {
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        let mut result = String::new();
        result.extend(first.to_lowercase());
        result.extend(chars);
        result
    } else {
        s.to_string()
    }
}

/// 文本大小写转换
#[tauri::command]
pub fn convert_case(text: String, mode: String) -> String {
    match mode.as_str() {
        "upper" => text.to_uppercase(),
        "lower" => text.to_lowercase(),
        "title" => {
            text.split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => {
                            first.to_uppercase().collect::<String>() + chars.as_str()
                        }
                    }
                })
                .collect::<Vec<String>>()
                .join(" ")
        }
        "capitalize" => {
            let mut chars = text.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + chars.as_str()
                }
            }
        }
        _ => text
    }
}
