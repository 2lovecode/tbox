use serde::Serialize;
use rusqlite::{params, Connection, Result};
use crate::utils2::get_db_path;

// 新增：工具分类结构体
#[derive(Serialize)]
pub struct ToolCategory {
    id: u32,
    name: String,
}

#[derive(Serialize)]
pub struct Tool {
    id: u32,
    name: String,
    description: String,
    icon: String,
    category: Option<ToolCategory>,
    tags: Vec<String>,
    gradient: String,
}

#[derive(Serialize)]
pub struct Category {
    id: u32,
    name: String,
    description: String,
    order: u32,
    count: u32,
}

#[derive(Serialize)]
pub struct Tag {
    id: u32,
    name: String,
    description: String,
    count: u32,
}

pub fn init_db_if_needed() -> Result<()> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;

    if db_path.exists() {
        // 已存在数据库文件，检查并添加缺失的工具
        add_missing_tools(&conn)?;
        return Ok(());
    }

    // 创建 categories 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            sort INTEGER NOT NULL
        )",
        [],
    )?;
    // 创建 tags 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL
        )",
        [],
    )?;

    // 创建 tools 表（删除了 tags 字段）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tools (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            icon TEXT NOT NULL,
            gradient TEXT NOT NULL
        )",
        [],
    )?;


    // 创建工具和分类的关联表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tool_categories (
            tool_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            FOREIGN KEY (tool_id) REFERENCES tools(id),
            FOREIGN KEY (category_id) REFERENCES categories(id)
            PRIMARY KEY (tool_id, category_id)
        )",
        [],
    )?;

    // 创建工具和标签的关联表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tool_tags (
            tool_id INTEGER,
            tag_id INTEGER,
            FOREIGN KEY (tool_id) REFERENCES tools(id),
            FOREIGN KEY (tag_id) REFERENCES tags(id),
            PRIMARY KEY (tool_id, tag_id)
        )",
        [],
    )?;
    let categories = vec![
        (1, "图片处理", "图片处理工具", 1),
        (2, "视频处理", "视频处理工具", 2),
        (3, "音频处理", "音频处理工具", 3),
        (4, "文件管理", "文档处理工具", 4),
        (5, "开发工具", "开发工具", 5),
        (6, "安全工具", "安全工具", 6),
        (7, "网络工具", "网络工具", 7),
        (8, "设计工具", "设计工具", 8),
    ];
    let tags = vec![
        (1, "图片处理", "图片处理"),
        (2, "优化", "优化"),
        (3, "视频处理", "视频处理"),
        (4, "转换", "转换"),
        (5, "安全工具", "安全工具"),
        (6, "加密", "加密"),
        (7, "文件管理", "文件管理"),
        (8, "PDF", "PDF"),
        (9, "设计工具", "设计工具"),
        (10, "测量", "测量"),
        (11, "开发工具", "开发工具"),
        (12, "编程", "编程"),
        (13, "恢复", "恢复"),
        (14, "网络工具", "网络工具"),
        (15, "测速", "测速"),
    ];

    let tools = vec![
        (1, "图片压缩工具", "快速压缩图片大小而不损失质量，支持JPG、PNG等格式。", "fas fa-compress-arrows-alt", "linear-gradient(135deg, #4361ee, #4895ef)"),
        (2, "视频格式转换", "支持多种视频格式转换，包括MP4, AVI, MOV等常见格式。", "fas fa-video", "linear-gradient(135deg, #f72585, #b5179e)"),
        (3, "密码管理器", "安全存储和管理所有密码，自动生成强密码，保护您的账户安全。", "fas fa-key", "linear-gradient(135deg, #4cc9f0, #4895ef)"),
        (4, "PDF工具箱", "合并、分割、压缩PDF文件，添加水印和密码保护等实用功能。", "fas fa-file-pdf", "linear-gradient(135deg, #2ec4b6, #1a936f)"),
        (5, "屏幕标尺", "在屏幕上测量元素尺寸，支持像素、厘米、英寸等多种单位。", "fas fa-ruler-combined", "linear-gradient(135deg, #ff9e00, #ff5400)"),
        (6, "代码格式化", "美化您的代码，支持多种编程语言，提高代码可读性和规范性。", "fas fa-code", "linear-gradient(135deg, #7209b7, #560bad)"),
        (7, "文件恢复工具", "恢复误删除的文件，支持多种文件系统和存储设备。", "fas fa-undo", "linear-gradient(135deg, #3a0ca3, #4cc9f0)"),
        (8, "网络测速", "测试您的网络下载、上传速度和延迟，提供详细分析报告。", "fas fa-wifi", "linear-gradient(135deg, #4361ee, #3a0ca3)"),
        (9, "JSON处理工具", "JSON美化、压缩、转义、去转义、验证和信息查看等实用功能。", "fas fa-brackets-curly", "linear-gradient(135deg, #06ffa5, #00d4ff)"),
    ];

    let tool_tags = vec![
        (1, 1),
        (1, 2),
        (2, 2),
        (2, 3),
        (3, 5),
        (3, 6),
        (4, 7),
        (4, 8),
        (5, 8),
        (6, 9),
        (6, 10),
        (7, 7),
        (7, 13),
        (8, 14),
        (8, 15),
        (9, 11),  // 开发工具
        (9, 12),  // 编程
    ];

    let tool_categories = vec![
        (1, 1),
        (2, 2),
        (3, 6),
        (4, 4),
        (5, 5),
        (6, 5),
        (7, 7),
        (8, 8),
        (9, 5),  // JSON工具属于开发工具
    ];
    for (id, name, description, sort) in categories {
        conn.execute(
            "INSERT INTO categories (id, name, description, sort)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                id,
                name,
                description,
                sort
            ],
        )?;
    }
    for (id, name, description) in tags {
        conn.execute(
            "INSERT INTO tags (id, name, description)
             VALUES (?1, ?2, ?3)",
            params![
                id,
                name,
                description
            ],
        )?;
    }

    for (id, name, description, icon, gradient) in tools {
        conn.execute(
            "INSERT INTO tools (id, name, description, icon, gradient)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                name,
                description,
                icon,
                gradient
            ],
        )?;
    }

    for (tool_id, category_id) in tool_categories {
        conn.execute(
            "INSERT INTO tool_categories (tool_id, category_id) VALUES (?1, ?2)",
            params![tool_id, category_id],
        )?;
    }

    for (tool_id, tag_id) in tool_tags {
        conn.execute(
            "INSERT INTO tool_tags (tool_id, tag_id) VALUES (?1, ?2)",
            params![tool_id, tag_id],
        )?;
    }

    Ok(())
}

// 添加缺失的工具到现有数据库
fn add_missing_tools(conn: &Connection) -> Result<()> {
    // 检查工具ID 9（JSON处理工具）是否存在
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM tools WHERE id = 9")?;
    let count: i32 = stmt.query_row([], |row| row.get(0))?;

    if count == 0 {
        // 添加JSON处理工具
        conn.execute(
            "INSERT INTO tools (id, name, description, icon, gradient)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                9,
                "JSON处理工具",
                "JSON美化、压缩、转义、去转义、验证和信息查看等实用功能。",
                "fas fa-brackets-curly",
                "linear-gradient(135deg, #06ffa5, #00d4ff)"
            ],
        )?;

        // 添加工具分类关联
        conn.execute(
            "INSERT OR IGNORE INTO tool_categories (tool_id, category_id) VALUES (9, 5)",
            [],
        )?;

        // 添加工具标签关联
        conn.execute(
            "INSERT OR IGNORE INTO tool_tags (tool_id, tag_id) VALUES (9, 11)",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO tool_tags (tool_id, tag_id) VALUES (9, 12)",
            [],
        )?;
    }

    Ok(())
}


#[tauri::command]
pub fn get_all_tools() -> Result<Vec<Tool>, String> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("
        SELECT t.id, t.name, t.description, t.icon, t.gradient
        FROM tools t
    ").map_err(|e| e.to_string())?;

    let tool_iter = stmt
        .query_map([], |row| {
            let tool_id: u32 = row.get(0)?;
            
            // 获取工具的标签
            let mut tag_stmt = conn.prepare("
                SELECT tg.name 
                FROM tool_tags tt
                JOIN tags tg ON tt.tag_id = tg.id
                WHERE tt.tool_id = ?1
            ").map_err(|_| rusqlite::Error::QueryReturnedNoRows)?;
            
            let tags: Vec<String> = tag_stmt
                .query_map(params![tool_id], |row| row.get(0))
                .map_err(|_| rusqlite::Error::QueryReturnedNoRows)?
                .collect::<Result<Vec<String>, _>>()
                .map_err(|_| rusqlite::Error::QueryReturnedNoRows)?;
            
            // 获取工具的分类（只取第一个分类，因为Tool结构只支持一个分类）
            let category: Option<ToolCategory> = {
                let mut cat_stmt = conn.prepare("
                    SELECT c.id, c.name
                    FROM tool_categories tc
                    JOIN categories c ON tc.category_id = c.id
                    WHERE tc.tool_id = ?1
                    LIMIT 1
                ").map_err(|_| rusqlite::Error::QueryReturnedNoRows)?;
                
                cat_stmt
                    .query_row(params![tool_id], |row| {
                        let id: u32 = row.get(0)?;
                        let name: String = row.get(1)?;
                        Ok(ToolCategory { id, name })
                    })
                    .ok()
            };

            Ok(Tool {
                id: tool_id,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?, 
                category,
                tags,
                gradient: row.get(4)?,
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
pub fn get_categories() -> Result<Vec<Category>, String> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("
        SELECT c.id, c.name, c.description, c.sort, COUNT(tc.tool_id) as count
        FROM categories c
        LEFT JOIN tool_categories tc ON c.id = tc.category_id
        GROUP BY c.id, c.name, c.description, c.sort
        ORDER BY c.sort
    ").map_err(|e| e.to_string())?;

    let category_iter = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                order: row.get(3)?,
                count: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut categories = Vec::new();
    for category in category_iter {
        categories.push(category.map_err(|e| e.to_string())?);
    }

    Ok(categories)
}