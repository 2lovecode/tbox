use regex::Regex;
use std::collections::HashMap;

/// 分析日志文件
#[tauri::command]
pub fn analyze_logs(log_content: String, pattern: String) -> Result<serde_json::Value, String> {
    let re = Regex::new(&pattern)
        .map_err(|e| format!("正则表达式编译失败: {}", e))?;

    let mut matches = Vec::new();
    let mut stats = HashMap::new();

    for (line_num, line) in log_content.lines().enumerate() {
        if re.is_match(line) {
            matches.push(serde_json::json!({
                "line": line_num + 1,
                "content": line
            }));

            // 统计关键词
            for cap in re.captures_iter(line) {
                for (i, match_str) in cap.iter().enumerate() {
                    if let Some(s) = match_str {
                        let _key = format!("group_{}", i);
                        *stats.entry(s.as_str().to_string()).or_insert(0usize) += 1;
                    }
                }
            }
        }
    }

    // 按出现次数排序
    let mut sorted_stats: Vec<(String, usize)> = stats.into_iter().collect();
    sorted_stats.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(serde_json::json!({
        "total_matches": matches.len(),
        "matches": matches,
        "top_keywords": sorted_stats.into_iter().take(20).collect::<Vec<_>>()
    }))
}

/// 提取日志级别
#[tauri::command]
pub fn extract_log_levels(log_content: String) -> Result<serde_json::Value, String> {
    let mut levels = HashMap::new();

    for line in log_content.lines() {
        let upper = line.to_uppercase();

        let level = if upper.contains("ERROR") {
            "ERROR"
        } else if upper.contains("WARN") {
            "WARN"
        } else if upper.contains("INFO") {
            "INFO"
        } else if upper.contains("DEBUG") {
            "DEBUG"
        } else if upper.contains("TRACE") {
            "TRACE"
        } else {
            continue;
        };

        *levels.entry(level.to_string()).or_insert(0usize) += 1;
    }

    Ok(serde_json::json!({
        "levels": levels,
        "total": levels.values().sum::<usize>()
    }))
}

/// 过滤日志
#[tauri::command]
pub fn filter_logs(
    log_content: String,
    keyword: String,
    case_sensitive: bool,
    invert: bool
) -> String {
    log_content.lines()
        .filter(|line| {
            let matches = if case_sensitive {
                line.contains(&keyword)
            } else {
                line.to_lowercase().contains(&keyword.to_lowercase())
            };

            if invert { !matches } else { matches }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// 统计日志中的错误
#[tauri::command]
pub fn count_errors(log_content: String) -> Result<serde_json::Value, String> {
    let mut errors = Vec::new();
    let mut error_types = HashMap::new();

    for (line_num, line) in log_content.lines().enumerate() {
        let upper = line.to_uppercase();

        if upper.contains("ERROR") || upper.contains("EXCEPTION") || upper.contains("FAILED") {
            errors.push(serde_json::json!({
                "line": line_num + 1,
                "content": line
            }));

            // 尝试提取错误类型
            if let Some(start) = line.find("Exception") {
                if let Some(end) = line[start..].find(':') {
                    let error_type = line[start..start + end].to_string();
                    *error_types.entry(error_type).or_insert(0usize) += 1;
                }
            }
        }
    }

    let mut sorted_errors: Vec<(String, usize)> = error_types.into_iter().collect();
    sorted_errors.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(serde_json::json!({
        "total_errors": errors.len(),
        "errors": errors.into_iter().take(100).collect::<Vec<_>>(),
        "error_types": sorted_errors
    }))
}

/// 提取时间范围内的日志
#[tauri::command]
pub fn extract_logs_by_time(
    log_content: String,
    start_time: String,
    end_time: String
) -> Result<String, String> {
    let mut result = Vec::new();
    let mut in_range = false;

    // 简化实现：查找包含时间字符串的行
    for line in log_content.lines() {
        if line.contains(&start_time) {
            in_range = true;
        }

        if in_range {
            result.push(line);

            if line.contains(&end_time) {
                break;
            }
        }
    }

    Ok(result.join("\n"))
}

/// 高亮日志中的关键信息
#[tauri::command]
pub fn highlight_logs(
    log_content: String,
    keywords: Vec<String>
) -> Result<serde_json::Value, String> {
    let mut highlighted = Vec::new();

    for line in log_content.lines() {
        let mut line_html = line.to_string();

        for keyword in &keywords {
            if line.contains(keyword) {
                line_html = line_html.replace(
                    keyword,
                    &format!("<mark>{}</mark>", keyword)
                );
            }
        }

        highlighted.push(line_html);
    }

    Ok(serde_json::json!({
        "lines": highlighted
    }))
}

/// 查找重复日志
#[tauri::command]
pub fn find_duplicate_logs(log_content: String) -> Result<serde_json::Value, String> {
    let mut log_counts = HashMap::new();
    let mut duplicates = Vec::new();

    for line in log_content.lines() {
        let count = log_counts.entry(line.to_string()).or_insert(0usize);
        *count += 1;
    }

    for (log, count) in log_counts.iter() {
        if *count > 1 {
            duplicates.push(serde_json::json!({
                "content": log,
                "count": count
            }));
        }
    }

    // 按出现次数排序
    duplicates.sort_by(|a, b| {
        b["count"].as_u64().cmp(&a["count"].as_u64())
    });

    Ok(serde_json::json!({
        "duplicates": duplicates.into_iter().take(50).collect::<Vec<_>>()
    }))
}

/// 生成日志报告
#[tauri::command]
pub fn generate_log_report(log_content: String) -> Result<serde_json::Value, String> {
    let total_lines = log_content.lines().count();

    // 提取日志级别
    let levels = extract_log_levels(log_content.clone())?;

    // 统计错误
    let errors = count_errors(log_content)?;

    Ok(serde_json::json!({
        "total_lines": total_lines,
        "levels": levels,
        "errors": errors
    }))
}
