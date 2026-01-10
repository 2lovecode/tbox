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

    // 检查 tools 表是否存在
    let mut check_table = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='tools'")?;
    let table_exists = check_table.exists([])?;

    if !table_exists {
        // 表不存在，创建所有表
        create_all_tables(&conn)?;
    } else {
        // 表已存在，检查并添加缺失的工具
        add_missing_tools(&conn)?;
    }

    Ok(())
}

fn create_all_tables(conn: &Connection) -> Result<()> {
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
        (16, "编码", "编码"),
        (17, "哈希", "哈希"),
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
        (10, "Base64工具", "Base64编码和解码工具，支持中文字符和各种格式。", "fas fa-exchange-alt", "linear-gradient(135deg, #ffd60a, #ff9500)"),
        (11, "哈希生成器", "生成MD5、SHA-1、SHA-256等多种哈希值，用于数据校验和安全验证。", "fas fa-hashtag", "linear-gradient(135deg, #7b2cbf, #9d4edd)"),
        // 新增工具
        (12, "JSON转实体类", "将JSON自动转换为Java、C#、Go、Python、TypeScript实体类。", "fas fa-file-code", "linear-gradient(135deg, #4361ee, #4895ef)"),
        (13, "JSON对比工具", "对比两个JSON的差异，高亮显示新增、删除和修改的字段。", "fas fa-not-equal", "linear-gradient(135deg, #f72585, #b5179e)"),
        (14, "JWT工具", "解析和生成JWT Token，支持签名验证。", "fas fa-key", "linear-gradient(135deg, #4cc9f0, #4895ef)"),
        (15, "正则表达式测试", "实时测试正则表达式，查看匹配结果。", "fas fa-regexp", "linear-gradient(135deg, #7209b7, #560bad)"),
        (16, "时间戳转换", "Unix时间戳与日期时间互转，支持多种时间单位。", "fas fa-clock", "linear-gradient(135deg, #ff9e00, #ff5400)"),
        (17, "HTTP请求工具", "发送HTTP请求，支持GET/POST/PUT/DELETE等方法。", "fas fa-paper-plane", "linear-gradient(135deg, #3a0ca3, #4cc9f0)"),
        (18, "文本处理工具", "文本对比、去重、排序、正则替换等文本处理功能。", "fas fa-font", "linear-gradient(135deg, #4361ee, #3a0ca3)"),
        (19, "编码转换工具", "URL、Unicode、Base58/62等多种编码转换。", "fas fa-exchange-alt", "linear-gradient(135deg, #06ffa5, #00d4ff)"),
        // 更多新工具
        (20, "XML工具", "XML格式化、压缩、JSON/YAML转换等功能。", "fas fa-code", "linear-gradient(135deg, #e76f51, #f4a261)"),
        (21, "YAML工具", "YAML格式化、验证、JSON转换和合并。", "fas fa-file-code", "linear-gradient(135deg, #2a9d8f, #264653)"),
        (22, "国密算法工具", "SM2/SM3/SM4国密算法加密解密和签名。", "fas fa-lock", "linear-gradient(135deg, #e63946, #457b9d)"),
        (23, "SQL工具", "SQL格式化和转义，支持多种数据库。", "fas fa-database", "linear-gradient(135deg, #1d3557, #457b9d)"),
        (24, "数据库工具", "MySQL、PostgreSQL、SQLite连接测试。", "fas fa-server", "linear-gradient(135deg, #0077b6, #00b4d8)"),
        (25, "图片工具", "图片格式转换、Base64编码、图片信息查看。", "fas fa-image", "linear-gradient(135deg, #9b5de5, #f15bb5)"),
        (26, "CSV工具", "CSV与JSON转换、格式化和统计。", "fas fa-file-csv", "linear-gradient(135deg, #00bbf9, #00f5d4)"),
        (27, "日志分析工具", "日志分析、错误统计、过滤和级别提取。", "fas fa-file-lines", "linear-gradient(135deg, #fb8500, #ffb703)"),
        // 2025-01-10 新增工具
        (28, "颜色工具", "RGB/HEX/HSL/HSV颜色格式转换，生成随机颜色，调整亮度。", "fas fa-palette", "linear-gradient(135deg, #f72585, #7209b7)"),
        (29, "二维码工具", "生成二维码，解析二维码，生成条形码（CODE128、EAN13等）。", "fas fa-qrcode", "linear-gradient(135deg, #4361ee, #4cc9f0)"),
        (30, "UUID工具", "生成v4/v7 UUID，批量生成，验证，Base64转换，版本检测。", "fas fa-fingerprint", "linear-gradient(135deg, #7209b7, #3a0ca3)"),
        (31, "Cron工具", "Cron表达式构建、验证和自然语言解释。", "fas fa-calendar-check", "linear-gradient(135deg, #2ec4b6, #06ffa5)"),
        (32, "数字工具", "进制转换（十进制、十六进制、二进制、八进制），科学计数法，罗马数字。", "fas fa-calculator", "linear-gradient(135deg, #ff9e00, #ff5400)"),
        (33, "字符编码工具", "编码检测，UTF-8/GBK转换，URL编解码，HTML实体，Punycode。", "fas fa-language", "linear-gradient(135deg, #4895ef, #00bbf9)"),
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
        (10, 4),  // 转换
        (10, 16), // 编码
        (11, 5),  // 安全工具
        (11, 17), // 哈希
        // 新增工具的tags
        (12, 11), (12, 12),  // JSON转实体类
        (13, 11), (13, 12),  // JSON对比
        (14, 5), (14, 6),    // JWT
        (15, 11), (15, 12),  // 正则
        (16, 11), (16, 12),  // 时间戳
        (17, 14),            // HTTP
        (18, 11), (18, 12),  // 文本工具
        (19, 4), (19, 16),   // 编码工具
        // 更多新工具的tags
        (20, 11), (20, 12),  // XML工具
        (21, 11), (21, 12),  // YAML工具
        (22, 5), (22, 6),    // 国密
        (23, 11), (23, 12),  // SQL工具
        (24, 7),            // 数据库工具
        (25, 1), (25, 4),    // 图片工具
        (26, 4), (26, 11),   // CSV工具
        (27, 11), (27, 12),  // 日志分析
        // 2025-01-10 新增工具的tags
        (28, 9), (28, 10),   // 颜色工具 - 设计工具 + 测量
        (29, 4), (29, 11),   // 二维码工具 - 转换 + 开发工具
        (30, 11), (30, 12),  // UUID工具 - 开发工具 + 编程
        (31, 11), (31, 12),  // Cron工具 - 开发工具 + 编程
        (32, 11), (32, 12),  // 数字工具 - 开发工具 + 编程
        (33, 4), (33, 16),   // 字符编码工具 - 转换 + 编码
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
        (10, 5), // Base64工具属于开发工具
        (11, 5), // 哈希生成器属于开发工具
        // 新增工具的categories
        (12, 5), // JSON转实体类属于开发工具
        (13, 5), // JSON对比属于开发工具
        (14, 5), // JWT属于开发工具
        (15, 5), // 正则属于开发工具
        (16, 5), // 时间戳属于开发工具
        (17, 7), // HTTP属于网络工具
        (18, 5), // 文本工具属于开发工具
        (19, 5), // 编码工具属于开发工具
        // 更多新工具的categories
        (20, 5), // XML工具属于开发工具
        (21, 5), // YAML工具属于开发工具
        (22, 5), // 国密属于开发工具
        (23, 5), // SQL工具属于开发工具
        (24, 7), // 数据库工具属于网络工具
        (25, 1), // 图片工具属于图片处理
        (26, 5), // CSV工具属于开发工具
        (27, 5), // 日志分析属于开发工具
        // 2025-01-10 新增工具的categories
        (28, 8), // 颜色工具属于设计工具
        (29, 5), // 二维码工具属于开发工具
        (30, 5), // UUID工具属于开发工具
        (31, 5), // Cron工具属于开发工具
        (32, 5), // 数字工具属于开发工具
        (33, 5), // 字符编码工具属于开发工具
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
    // 首先检查并添加缺失的标签
    let required_tags = vec![
        (16, "编码", "编码"),
        (17, "哈希", "哈希"),
    ];

    for (id, name, description) in required_tags {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM tags WHERE id = ?1")?;
        let count: i32 = stmt.query_row(params![id], |row| row.get(0))?;

        if count == 0 {
            conn.execute(
                "INSERT INTO tags (id, name, description) VALUES (?1, ?2, ?3)",
                params![id, name, description],
            )?;
        }
    }

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

    // 检查工具ID 10（Base64工具）是否存在
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM tools WHERE id = 10")?;
    let count: i32 = stmt.query_row([], |row| row.get(0))?;

    if count == 0 {
        // 添加Base64工具
        conn.execute(
            "INSERT INTO tools (id, name, description, icon, gradient)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                10,
                "Base64工具",
                "Base64编码和解码工具，支持中文字符和各种格式。",
                "fas fa-exchange-alt",
                "linear-gradient(135deg, #ffd60a, #ff9500)"
            ],
        )?;

        // 添加工具分类关联
        conn.execute(
            "INSERT OR IGNORE INTO tool_categories (tool_id, category_id) VALUES (10, 5)",
            [],
        )?;

        // 添加工具标签关联
        conn.execute(
            "INSERT OR IGNORE INTO tool_tags (tool_id, tag_id) VALUES (10, 4)",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO tool_tags (tool_id, tag_id) VALUES (10, 16)",
            [],
        )?;
    }

    // 检查工具ID 11（哈希生成器）是否存在
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM tools WHERE id = 11")?;
    let count: i32 = stmt.query_row([], |row| row.get(0))?;

    if count == 0 {
        // 添加哈希生成器
        conn.execute(
            "INSERT INTO tools (id, name, description, icon, gradient)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                11,
                "哈希生成器",
                "生成MD5、SHA-1、SHA-256等多种哈希值，用于数据校验和安全验证。",
                "fas fa-hashtag",
                "linear-gradient(135deg, #7b2cbf, #9d4edd)"
            ],
        )?;

        // 添加工具分类关联
        conn.execute(
            "INSERT OR IGNORE INTO tool_categories (tool_id, category_id) VALUES (11, 5)",
            [],
        )?;

        // 添加工具标签关联
        conn.execute(
            "INSERT OR IGNORE INTO tool_tags (tool_id, tag_id) VALUES (11, 5)",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO tool_tags (tool_id, tag_id) VALUES (11, 17)",
            [],
        )?;
    }

    // 检查并添加新工具 28-33
    let new_tools = vec![
        (28, "颜色工具", "RGB/HEX/HSL/HSV颜色格式转换，生成随机颜色，调整亮度。", "fas fa-palette", "linear-gradient(135deg, #f72585, #7209b7)", 8, vec![9, 10]),
        (29, "二维码工具", "生成二维码，解析二维码，生成条形码（CODE128、EAN13等）。", "fas fa-qrcode", "linear-gradient(135deg, #4361ee, #4cc9f0)", 5, vec![4, 11]),
        (30, "UUID工具", "生成v4/v7 UUID，批量生成，验证，Base64转换，版本检测。", "fas fa-fingerprint", "linear-gradient(135deg, #7209b7, #3a0ca3)", 5, vec![11, 12]),
        (31, "Cron工具", "Cron表达式构建、验证和自然语言解释。", "fas fa-calendar-check", "linear-gradient(135deg, #2ec4b6, #06ffa5)", 5, vec![11, 12]),
        (32, "数字工具", "进制转换（十进制、十六进制、二进制、八进制），科学计数法，罗马数字。", "fas fa-calculator", "linear-gradient(135deg, #ff9e00, #ff5400)", 5, vec![11, 12]),
        (33, "字符编码工具", "编码检测，UTF-8/GBK转换，URL编解码，HTML实体，Punycode。", "fas fa-language", "linear-gradient(135deg, #4895ef, #00bbf9)", 5, vec![4, 16]),
    ];

    for (id, name, description, icon, gradient, category_id, tag_ids) in new_tools {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM tools WHERE id = ?1")?;
        let count: i32 = stmt.query_row(params![id], |row| row.get(0))?;

        if count == 0 {
            // 添加工具
            conn.execute(
                "INSERT INTO tools (id, name, description, icon, gradient)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, name, description, icon, gradient],
            )?;

            // 添加工具分类关联
            conn.execute(
                "INSERT OR IGNORE INTO tool_categories (tool_id, category_id) VALUES (?1, ?2)",
                params![id, category_id],
            )?;

            // 添加工具标签关联
            for tag_id in tag_ids {
                conn.execute(
                    "INSERT OR IGNORE INTO tool_tags (tool_id, tag_id) VALUES (?1, ?2)",
                    params![id, tag_id],
                )?;
            }
        }
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