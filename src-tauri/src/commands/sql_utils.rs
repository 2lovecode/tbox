/// SQL格式化
#[tauri::command]
pub fn format_sql(input: String) -> Result<String, String> {
    let sql = input.trim();

    // 简单的SQL格式化实现
    let mut formatted = String::new();
    let mut indent = 0;
    let keywords = [
        "SELECT", "FROM", "WHERE", "AND", "OR", "ORDER BY", "GROUP BY",
        "HAVING", "LIMIT", "OFFSET", "JOIN", "LEFT JOIN", "RIGHT JOIN",
        "INNER JOIN", "OUTER JOIN", "ON", "AS", "INSERT INTO", "VALUES",
        "UPDATE", "SET", "DELETE FROM", "CREATE TABLE", "ALTER TABLE",
        "DROP TABLE", "UNION", "UNION ALL", "CASE", "WHEN", "THEN", "ELSE", "END"
    ];

    let lines: Vec<&str> = sql.lines().collect();
    if lines.is_empty() {
        return Ok(String::new());
    }

    // 处理单行SQL
    if lines.len() == 1 {
        let tokens = tokenize_sql(sql);
        let mut result = Vec::new();

        for token in tokens {
            let upper = token.to_uppercase();

            // 检查是否是关键字
            if keywords.iter().any(|&k| k == upper) {
                match upper.as_str() {
                    "SELECT" | "FROM" | "WHERE" | "ORDER BY" | "GROUP BY" | "HAVING" => {
                        result.push("\n".to_string());
                        result.push("  ".repeat(indent));
                        result.push(token.to_string());
                        result.push("\n".to_string());
                        result.push("  ".repeat(indent + 1));
                    }
                    "AND" | "OR" | "ON" => {
                        result.push("\n".to_string());
                        result.push("  ".repeat(indent + 1));
                        result.push(token.to_string());
                    }
                    "JOIN" | "LEFT JOIN" | "RIGHT JOIN" | "INNER JOIN" | "OUTER JOIN" => {
                        result.push("\n".to_string());
                        result.push("  ".repeat(indent));
                        result.push(token.to_string());
                    }
                    _ => {
                        result.push(format!(" {} ", token));
                    }
                }
            } else {
                result.push(token.to_string());
            }
        }

        formatted = result.join("");
    } else {
        // 多行SQL，基本格式化
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            formatted.push_str(&"  ".repeat(indent));
            formatted.push_str(trimmed);
            formatted.push('\n');

            // 调整缩进
            let upper = trimmed.to_uppercase();
            if upper.contains("SELECT") || upper.contains("FROM") {
                indent = 1;
            }
        }
    }

    Ok(formatted.trim().to_string())
}

fn tokenize_sql(sql: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut string_char = ' ';

    for ch in sql.chars() {
        if in_string {
            current.push(ch);
            if ch == string_char {
                in_string = false;
            }
        } else if ch == '\'' || ch == '"' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            in_string = true;
            string_char = ch;
            current.push(ch);
        } else if ch == ',' || ch == '(' || ch == ')' || ch == ';' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            tokens.push(ch.to_string());
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// SQL压缩
#[tauri::command]
pub fn minify_sql(input: String) -> String {
    input.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// SQL转义
#[tauri::command]
pub fn escape_sql(input: String, db_type: String) -> String {
    match db_type.as_str() {
        "mysql" => input.replace("\\", "\\\\").replace("'", "\\'").replace("\"", "\\\""),
        "postgres" => input.replace("'", "''").replace("\\", "\\\\"),
        "mssql" => input.replace("'", "''"),
        "sqlite" => input.replace("'", "''"),
        _ => input.replace("'", "\\'")
    }
}

/// SQL反转义
#[tauri::command]
pub fn unescape_sql(input: String, db_type: String) -> String {
    match db_type.as_str() {
        "mysql" => input.replace("\\'", "'").replace("\\\"", "\"").replace("\\\\", "\\"),
        "postgres" => input.replace("''", "'").replace("\\\\", "\\"),
        "mssql" => input.replace("''", "'"),
        "sqlite" => input.replace("''", "'"),
        _ => input.replace("\\'", "'").replace("\\\"", "\"")
    }
}
