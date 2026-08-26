use csv::{ReaderBuilder, WriterBuilder};
use serde_json::Value;

/// CSV转JSON
#[tauri::command]
pub fn csv_to_json(input: String, has_header: bool) -> Result<String, String> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(has_header)
        .from_reader(input.as_bytes());

    let mut result = Vec::new();

    if has_header {
        // 带表头：生成对象数组
        for result_row in rdr.deserialize() {
            let row: serde_json::Map<String, Value> = result_row
                .map_err(|e| format!("CSV解析失败: {}", e))?;

            result.push(Value::Object(row));
        }
    } else {
        // 无表头：生成二维数组
        for result_row in rdr.records() {
            let row = result_row.map_err(|e| format!("CSV解析失败: {}", e))?;

            let arr: Vec<Value> = row.iter()
                .map(|s| Value::String(s.to_string()))
                .collect();

            result.push(Value::Array(arr));
        }
    }

    serde_json::to_string_pretty(&result)
        .map_err(|e| format!("JSON序列化失败: {}", e))
}

/// JSON转CSV
#[tauri::command]
pub fn json_to_csv(input: String, has_header: bool) -> Result<String, String> {
    let value: Value = serde_json::from_str(&input)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    let mut output = Vec::new();

    {
        let mut wtr = WriterBuilder::new()
            .has_headers(has_header)
            .from_writer(&mut output);

        if let Some(array) = value.as_array() {
            if !array.is_empty() {
                if let Some(_first_obj) = array[0].as_object() {
                    // 获取所有可能的字段
                    let mut headers = Vec::new();
                    for obj in array {
                        if let Some(o) = obj.as_object() {
                            for key in o.keys() {
                                if !headers.contains(key) {
                                    headers.push(key.clone());
                                }
                            }
                        }
                    }

                    // 写入表头
                    if has_header {
                        wtr.write_record(&headers)
                            .map_err(|e| format!("写入CSV表头失败: {}", e))?;
                    }

                    // 写入数据
                    for obj in array {
                        if let Some(o) = obj.as_object() {
                            let row: Vec<String> = headers.iter()
                                .map(|h| o.get(h).and_then(|v| v.as_str()).unwrap_or("").to_string())
                                .collect();

                            wtr.write_record(&row)
                                .map_err(|e| format!("写入CSV行失败: {}", e))?;
                        }
                    }
                }
            }
        }

        wtr.flush()
            .map_err(|e| format!("刷新CSV失败: {}", e))?;
    }

    String::from_utf8(output)
        .map_err(|e| format!("UTF-8转换失败: {}", e))
}

/// CSV格式化
#[tauri::command]
pub fn format_csv(input: String, delimiter: String) -> Result<String, String> {
    let delimiter_char = delimiter.chars().next().unwrap_or(',');

    let mut rdr = ReaderBuilder::new()
        .delimiter(delimiter_char as u8)
        .from_reader(input.as_bytes());

    let mut result = String::new();
    let mut records = rdr.records();

    // 写入表头
    if let Some(Ok(headers)) = records.next() {
        let header_vec: Vec<&str> = headers.iter().collect();
        result.push_str(&header_vec.join(&format!("{} ", delimiter_char)));
        result.push('\n');
    }

    // 写入数据
    for record in records {
        let row = record.map_err(|e| format!("CSV解析失败: {}", e))?;
        let row_vec: Vec<&str> = row.iter().collect();
        result.push_str(&row_vec.join(&format!("{} ", delimiter_char)));
        result.push('\n');
    }

    Ok(result)
}

/// CSV转Excel（简化实现，返回CSV）
#[tauri::command]
pub fn csv_to_excel(input: String) -> Result<String, String> {
    // 简化实现：返回CSV数据
    // 完整实现需要使用calamine或rust_xlsxwriter库
    Ok(input)
}

/// 统计CSV信息
#[tauri::command]
pub fn csv_stats(input: String) -> Result<serde_json::Value, String> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input.as_bytes());

    let headers = rdr.headers()
        .map_err(|e| format!("读取CSV表头失败: {}", e))?;

    let column_count = headers.len();
    let mut row_count = 0usize;

    for _ in rdr.records() {
        row_count += 1;
    }

    Ok(serde_json::json!({
        "columns": column_count,
        "rows": row_count,
        "total_cells": column_count * row_count
    }))
}
