use chrono::{Utc, TimeZone};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unix时间戳转日期时间
#[tauri::command]
pub fn timestamp_to_datetime(timestamp: i64, unit: String) -> String {
    let seconds = match unit.as_str() {
        "ms" => timestamp / 1000,
        "us" => timestamp / 1_000_000,
        "ns" => timestamp / 1_000_000_000,
        _ => timestamp
    };

    let datetime = Utc.timestamp_opt(seconds, 0).single().unwrap_or(Utc::now());
    format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"))
}

/// 日期时间转Unix时间戳
#[tauri::command]
pub fn datetime_to_timestamp(datetime: String) -> Result<i64, String> {
    // 尝试RFC3339格式
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&datetime) {
        return Ok(dt.timestamp());
    }

    // 尝试简单格式
    let nd = chrono::NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("日期时间解析失败: {}", e))?;

    Ok(nd.timestamp())
}

/// 获取当前时间戳
#[tauri::command]
pub fn current_timestamp(unit: String) -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();

    match unit.as_str() {
        "ms" => duration.as_millis() as i64,
        "us" => duration.as_micros() as i64,
        "ns" => duration.as_nanos() as i64,
        _ => duration.as_secs() as i64
    }
}

/// 批量时间戳转换
#[tauri::command]
pub fn batch_timestamp_convert(timestamps: Vec<String>, unit: String, format: String) -> Vec<String> {
    timestamps.iter()
        .filter_map(|ts| ts.parse::<i64>().ok())
        .map(|ts| {
            let seconds = match unit.as_str() {
                "ms" => ts / 1000,
                "us" => ts / 1_000_000,
                "ns" => ts / 1_000_000_000,
                _ => ts
            };

            let datetime = Utc.timestamp_opt(seconds, 0).single().unwrap_or(Utc::now());
            format!("{}", datetime.format(&format))
        })
        .collect()
}

/// Cron表达式解析
#[tauri::command]
pub fn parse_cron(cron: String) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = cron.split_whitespace().collect();

    if parts.len() < 5 || parts.len() > 6 {
        return Err("Cron表达式格式错误，应为5或6个部分".to_string());
    }

    let minute = parse_cron_field(parts[0], 0, 59)?;
    let hour = parse_cron_field(parts[1], 0, 23)?;
    let day_of_month = parse_cron_field(parts[2], 1, 31)?;
    let month = parse_cron_field(parts[3], 1, 12)?;
    let day_of_week = parse_cron_field(parts[4], 0, 6)?;

    let result = serde_json::json!({
        "minute": minute,
        "hour": hour,
        "dayOfMonth": day_of_month,
        "month": month,
        "dayOfWeek": day_of_week,
        "description": format_cron_description(&minute, &hour, &day_of_month, &month, &day_of_week)
    });

    Ok(result)
}

fn parse_cron_field(field: &str, min: u32, max: u32) -> Result<String, String> {
    if field == "*" {
        return Ok(format!("每{}-{}之间的任意值", min, max));
    }

    if field.contains('/') {
        let parts: Vec<&str> = field.split('/').collect();
        if parts.len() == 2 {
            let base = if parts[0] == "*" { format!("{}-{}", min, max) } else { parts[0].to_string() };
            let step = parts[1].parse::<u32>()
                .map_err(|_| format!("无效的步长值: {}", parts[1]))?;
            return Ok(format!("从{}开始，每隔{}", base, step));
        }
    }

    if field.contains('-') {
        let parts: Vec<&str> = field.split('-').collect();
        if parts.len() == 2 {
            let start = parts[0].parse::<u32>()
                .map_err(|_| format!("无效的范围起始值: {}", parts[0]))?;
            let end = parts[1].parse::<u32>()
                .map_err(|_| format!("无效的范围结束值: {}", parts[1]))?;
            return Ok(format!("从{}到{}", start, end));
        }
    }

    if field.contains(',') {
        let values: Vec<&str> = field.split(',').collect();
        return Ok(format!("指定值: {}", field));
    }

    Ok(field.to_string())
}

fn format_cron_description(minute: &str, hour: &str, dom: &str, month: &str, dow: &str) -> String {
    format!("{} {} {} {} {}", minute, hour, dom, month, dow)
}

/// 日期计算
#[tauri::command]
pub fn date_calculation(date: String, days: i32) -> Result<String, String> {
    let dt = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("日期解析失败: {}", e))?;

    let result = if days >= 0 {
        dt + chrono::Duration::days(days as i64)
    } else {
        dt - chrono::Duration::days((-days) as i64)
    };

    Ok(format!("{}", result.format("%Y-%m-%d")))
}

/// 日期差计算
#[tauri::command]
pub fn date_diff(date1: String, date2: String) -> Result<i64, String> {
    let dt1 = chrono::NaiveDate::parse_from_str(&date1, "%Y-%m-%d")
        .map_err(|e| format!("日期1解析失败: {}", e))?;
    let dt2 = chrono::NaiveDate::parse_from_str(&date2, "%Y-%m-%d")
        .map_err(|e| format!("日期2解析失败: {}", e))?;

    Ok((dt2 - dt1).num_days())
}

/// 格式化时间差
#[tauri::command]
pub fn format_duration(seconds: i64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{}天", days));
    }
    if hours > 0 || days > 0 {
        parts.push(format!("{}小时", hours));
    }
    if minutes > 0 || hours > 0 || days > 0 {
        parts.push(format!("{}分钟", minutes));
    }
    parts.push(format!("{}秒", secs));

    parts.join(" ")
}
