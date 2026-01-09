use serde::Serialize;
use std::path::PathBuf;
use std::fs;

#[derive(Serialize)]
pub struct FileInfo {
    name: String,
    path: String,
    size: u64,
    is_file: bool,
    modified: Option<String>,
}

#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<FileInfo>, String> {
    let dir_path = PathBuf::from(&path);
    
    if !dir_path.exists() {
        return Err("路径不存在".to_string());
    }

    if !dir_path.is_dir() {
        return Err("路径不是目录".to_string());
    }

    let mut files = Vec::new();

    for entry in fs::read_dir(&dir_path)
        .map_err(|e| format!("无法读取目录: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("无法读取条目: {}", e))?;
        let path = entry.path();
        let metadata = entry.metadata()
            .map_err(|e| format!("无法读取元数据: {}", e))?;

        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let modified = metadata.modified()
            .ok()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs().to_string())
            });

        files.push(FileInfo {
            name,
            path: path.to_string_lossy().to_string(),
            size: if metadata.is_file() { metadata.len() } else { 0 },
            is_file: metadata.is_file(),
            modified,
        });
    }

    files.sort_by(|a, b| {
        // 目录在前，然后按名称排序
        match (a.is_file, b.is_file) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(files)
}

#[tauri::command]
pub async fn get_file_size(file_path: String) -> Result<u64, String> {
    let metadata = fs::metadata(&file_path)
        .map_err(|e| format!("无法读取文件信息: {}", e))?;
    Ok(metadata.len())
}

#[tauri::command]
pub async fn file_exists(file_path: String) -> Result<bool, String> {
    Ok(PathBuf::from(&file_path).exists())
}
