/// 生成Cron表达式
#[tauri::command]
pub fn generate_cron(
    minute: String,
    hour: String,
    day_of_month: String,
    month: String,
    day_of_week: String
) -> String {
    format!("{} {} {} {} {}", minute, hour, day_of_month, month, day_of_week)
}

/// 获取Cron表达式的下次执行时间（简化实现）
#[tauri::command]
pub fn get_next_cron_time(cron: String) -> Result<String, String> {
    // 简化实现：返回提示信息
    // 实际项目中应该使用cron或scheduling库
    Ok(format!("下次执行时间计算功能（Cron: {}）", cron))
}

/// Cron表达式转自然语言描述
#[tauri::command]
pub fn cron_to_natural_language(cron: String) -> Result<String, String> {
    let parts: Vec<&str> = cron.split_whitespace().collect();

    if parts.len() < 5 {
        return Err("无效的Cron表达式".to_string());
    }

    let mut description = String::from("在");

    // 解析分钟
    if parts[0] == "*" {
        description.push_str("每分钟");
    } else if parts[0].contains('/') {
        let step: u32 = parts[0].split('/').last().unwrap().parse().unwrap_or(1);
        description.push_str(&format!("每{}分钟", step));
    } else {
        description.push_str(&format!("第{}分钟", parts[0]));
    }

    // 解析小时
    if parts[1] == "*" {
        description.push_str("");
    } else if parts[1].contains('/') {
        let step: u32 = parts[1].split('/').last().unwrap().parse().unwrap_or(1);
        description.push_str(&format!("，每{}小时", step));
    } else {
        description.push_str(&format!("的第{}小时", parts[1]));
    }

    description.push_str("执行");

    Ok(description)
}

/// 验证Cron表达式
#[tauri::command]
pub fn validate_cron(cron: String) -> Result<bool, String> {
    let parts: Vec<&str> = cron.split_whitespace().collect();

    if parts.len() < 5 || parts.len() > 6 {
        return Err("Cron表达式应为5或6个部分".to_string());
    }

    let ranges = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 6)];

    for (i, part) in parts.iter().enumerate().take(5) {
        if *part == "*" {
            continue;
        }

        if part.contains('/') {
            let parts2: Vec<&str> = part.split('/').collect();
            if parts2.len() != 2 {
                return Err(format!("第{}部分格式错误", i + 1));
            }
            continue;
        }

        if part.contains('-') {
            let parts2: Vec<&str> = part.split('-').collect();
            if parts2.len() != 2 {
                return Err(format!("第{}部分范围格式错误", i + 1));
            }
            continue;
        }

        if part.contains(',') {
            for num in part.split(',') {
                if let Ok(n) = num.parse::<u32>() {
                    if n < ranges[i].0 || n > ranges[i].1 {
                        return Err(format!("第{}部分数值超出范围", i + 1));
                    }
                }
            }
            continue;
        }

        if let Ok(n) = part.parse::<u32>() {
            if n < ranges[i].0 || n > ranges[i].1 {
                return Err(format!("第{}部分数值超出范围", i + 1));
            }
        }
    }

    Ok(true)
}
