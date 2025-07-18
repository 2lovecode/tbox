use serde::Serialize;
use rusqlite::{params, Connection, Result, Error};
use crate::utils2::get_db_path;

#[derive(Serialize)]
pub struct Tool {
    id: u32,
    name: String,
    description: String,
    icon: String,
    category: Option<String>,
    tags: Vec<String>,
    gradient: String,
}

pub fn init_db_if_needed() -> Result<()> {
    let db_path = get_db_path();

    if db_path.exists() {
        // 已存在数据库文件，跳过初始化
        return Ok(());
    }

    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tools (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            icon TEXT NOT NULL,
            category TEXT,
            tags TEXT NOT NULL,
            gradient TEXT NOT NULL
        )",
        [],
    )?;

    let tools = vec![
        (1, "图片压缩工具", "快速压缩图片大小而不损失质量，支持JPG、PNG等格式。", "fas fa-compress-arrows-alt", Some("image"), vec!["图片处理", "优化"], "linear-gradient(135deg, #4361ee, #4895ef)"),
        (2, "视频格式转换", "支持多种视频格式转换，包括MP4, AVI, MOV等常见格式。", "fas fa-video", Some("video"), vec!["视频处理", "转换"], "linear-gradient(135deg, #f72585, #b5179e)"),
        (3, "密码管理器", "安全存储和管理所有密码，自动生成强密码，保护您的账户安全。", "fas fa-key", Some("security"), vec!["安全工具", "加密"], "linear-gradient(135deg, #4cc9f0, #4895ef)"),
        (4, "PDF工具箱", "合并、分割、压缩PDF文件，添加水印和密码保护等实用功能。", "fas fa-file-pdf", Some("file"), vec!["文件管理", "PDF"], "linear-gradient(135deg, #2ec4b6, #1a936f)"),
        (5, "屏幕标尺", "在屏幕上测量元素尺寸，支持像素、厘米、英寸等多种单位。", "fas fa-ruler-combined", Some("dev"), vec!["设计工具", "测量"], "linear-gradient(135deg, #ff9e00, #ff5400)"),
        (6, "代码格式化", "美化您的代码，支持多种编程语言，提高代码可读性和规范性。", "fas fa-code", Some("dev"), vec!["开发工具", "编程"], "linear-gradient(135deg, #7209b7, #560bad)"),
        (7, "文件恢复工具", "恢复误删除的文件，支持多种文件系统和存储设备。", "fas fa-undo", Some("file"), vec!["文件管理", "恢复"], "linear-gradient(135deg, #3a0ca3, #4cc9f0)"),
        (8, "网络测速", "测试您的网络下载、上传速度和延迟，提供详细分析报告。", "fas fa-wifi", Some("network"), vec!["网络工具", "测速"], "linear-gradient(135deg, #4361ee, #3a0ca3)"),
    ];

    for (id, name, description, icon, category, tags, gradient) in tools {
        let tags_json = serde_json::to_string(&tags)
            .map_err(|e| Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
        conn.execute(
            "INSERT INTO tools (id, name, description, icon, category, tags, gradient)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                name,
                description,
                icon,
                category,
                tags_json,
                gradient
            ],
        )?;
    }

    Ok(())
}


#[tauri::command]
pub fn get_all_tools() -> Result<Vec<Tool>, String> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, description, icon, category, tags, gradient FROM tools")
        .map_err(|e| e.to_string())?;

    let tool_iter = stmt
        .query_map([], |row| {
            let tags_json: String = row.get(5)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            Ok(Tool {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                category: row.get::<_, Option<String>>(4)?,
                tags,
                gradient: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut tools = Vec::new();
    for tool in tool_iter {
        tools.push(tool.map_err(|e| e.to_string())?);
    }

    Ok(tools)
}


#[tauri::command]
pub fn get_categories() -> String {
    format!("a{}a", "b")
}