use std::process::Command;

/// 查看端口占用情况
#[tauri::command]
pub fn check_port_usage(port: u16) -> Result<serde_json::Value, String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("netstat")
            .args(["-ano"])
            .output()
    } else {
        Command::new("lsof")
            .args(["-i", &format!("-P{}", port)])
            .output()
    }.map_err(|e| format!("执行命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    let mut processes = Vec::new();

    for line in lines {
        if line.contains(&format!(":{}", port)) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if cfg!(target_os = "windows") {
                if parts.len() >= 5 {
                    processes.push(serde_json::json!({
                        "protocol": parts[0],
                        "localAddress": parts[1],
                        "foreignAddress": parts[2],
                        "state": parts[3],
                        "pid": parts[4]
                    }));
                }
            } else {
                processes.push(serde_json::json!({
                    "info": line
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "port": port,
        "inUse": !processes.is_empty(),
        "processes": processes
    }))
}

/// 杀死进程
#[tauri::command]
pub fn kill_process(pid: u32) -> Result<bool, String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output()
    } else {
        Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
    }.map_err(|e| format!("执行命令失败: {}", e))?;

    Ok(output.status.success())
}

/// 查看系统进程列表
#[tauri::command]
pub fn list_processes() -> Result<Vec<serde_json::Value>, String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("tasklist")
            .args(["/fo", "csv"])
            .output()
    } else {
        Command::new("ps")
            .args(["-eo", "pid,comm,%mem,%cpu"])
            .output()
    }.map_err(|e| format!("执行命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    let mut processes = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if i == 0 { continue; } // 跳过标题行

        if cfg!(target_os = "windows") {
            // CSV格式: "imagename","pid","sessionname","memusage"
            let parts: Vec<&str> = line.split("\",\"").collect();
            if parts.len() >= 2 {
                let name = parts[0].trim_start_matches('"');
                let pid = parts[1].replace("\"", "").parse::<u32>().unwrap_or(0);
                processes.push(serde_json::json!({
                    "name": name,
                    "pid": pid
                }));
            }
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let pid = parts[0].parse::<u32>().unwrap_or(0);
                processes.push(serde_json::json!({
                    "pid": pid,
                    "name": parts[1],
                    "mem": parts[2],
                    "cpu": parts[3]
                }));
            }
        }
    }

    Ok(processes)
}

/// 获取系统信息
#[tauri::command]
pub fn get_system_info() -> Result<serde_json::Value, String> {
    let os = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "ARM64"
    } else {
        "Unknown"
    };

    Ok(serde_json::json!({
        "os": os,
        "arch": arch,
        "version": "unknown"
    }))
}

/// 文件查找
#[tauri::command]
pub fn find_files(directory: String, pattern: String) -> Result<Vec<String>, String> {
    use std::fs;
    use std::path::Path;

    let mut results = Vec::new();

    let search_dir = Path::new(&directory);
    if !search_dir.exists() {
        return Err("目录不存在".to_string());
    }

    fn find_recursive(dir: &std::path::Path, pattern: &str, results: &mut Vec<String>) {
        if let Ok(entries) = dir.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    find_recursive(&path, pattern, results);
                } else {
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    if file_name.contains(pattern) {
                        if let Some(path_str) = path.to_str() {
                            results.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    find_recursive(search_dir, &pattern, &mut results);

    Ok(results)
}

/// 计算文件Hash
#[tauri::command]
pub fn calculate_file_hash(file_path: String, algorithm: String) -> Result<String, String> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(&file_path)
        .map_err(|e| format!("打开文件失败: {}", e))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let hash = match algorithm.as_str() {
        "md5" => {
            use md5::{Md5, Digest};
            let mut hasher = Md5::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        "sha1" => {
            use sha1::{Sha1, Digest};
            let mut hasher = Sha1::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        "sha256" => {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        _ => return Err("不支持的哈希算法".to_string())
    };

    Ok(hash)
}
