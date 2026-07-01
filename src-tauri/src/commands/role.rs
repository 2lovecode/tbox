use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::with_connection;

// 角色结构体
#[derive(Serialize, Clone, Deserialize)]
pub struct Role {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub icon: String,
    pub is_system: bool,
}

// 工具结构体（与 tool.rs 中的定义一致，这里仅用于类型标注）
// 实际的 Tool 结构体由 tool.rs 模块提供

/// 获取所有角色
#[tauri::command]
pub fn get_roles() -> Result<Vec<Role>, String> {
    with_connection(|conn| {
        let mut stmt = conn.prepare("
        SELECT id, name, display_name, description, icon, is_system
        FROM roles
        ORDER BY id
    ")?;

        let roles = stmt
            .query_map([], |row| {
                Ok(Role {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    description: row.get(3)?,
                    icon: row.get(4)?,
                    is_system: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(roles)
    })
}

/// 根据角色 ID 获取对应的工具列表
///
/// 注意：此函数返回的 Tool 结构体中的 tags 和 category 字段可能为空
/// 如需完整的工具信息，前端可以调用 get_all_tools 并在本地过滤
#[tauri::command]
pub fn get_tools_by_role(role_id: u32) -> Result<Vec<super::tool::Tool>, String> {
    with_connection(|conn| {
        let mut stmt = conn.prepare("
        SELECT t.id, t.name, t.description, t.icon, t.gradient
        FROM tools t
        INNER JOIN tool_roles tr ON t.id = tr.tool_id
        WHERE tr.role_id = ?1
        ORDER BY t.id
    ")?;

        let tools = stmt
            .query_map(params![role_id], |row| {
                let tool_id: u32 = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let icon: String = row.get(3)?;
                let gradient: String = row.get(4)?;

                // Tag and category lookups are best-effort: a tool without any
                // tags or categories still gets returned, but real DB errors
                // are propagated rather than silently swallowed.
                let tags = load_tags(conn, tool_id)?;
                let category = load_primary_category(conn, tool_id)?;

                Ok(super::tool::Tool {
                    id: tool_id,
                    name,
                    description,
                    icon,
                    category,
                    tags,
                    gradient,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tools)
    })
}

fn load_tags(conn: &Connection, tool_id: u32) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "
            SELECT tg.name
            FROM tool_tags tt
            JOIN tags tg ON tt.tag_id = tg.id
            WHERE tt.tool_id = ?1
        ",
    )?;
    let rows = stmt.query_map(params![tool_id], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>()
}

fn load_primary_category(
    conn: &Connection,
    tool_id: u32,
) -> Result<Option<super::tool::ToolCategory>, rusqlite::Error> {
    conn.query_row(
        "
                SELECT c.id, c.name
                FROM tool_categories tc
                JOIN categories c ON tc.category_id = c.id
                WHERE tc.tool_id = ?1
                LIMIT 1
            ",
        params![tool_id],
        |row| {
            Ok(super::tool::ToolCategory {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        },
    )
    .optional()
}

/// 获取用户配置目录
fn get_config_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("无法获取系统配置目录")?;
    Ok(config_dir.join("tbox"))
}

/// 设置用户选择的角色
#[tauri::command]
pub fn set_user_role(role_ids: Vec<u32>) -> Result<(), String> {
    let app_config_dir = get_config_dir()?;

    // 创建目录（如不存在）
    fs::create_dir_all(&app_config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let config_file = app_config_dir.join("user_roles.json");
    let json = serde_json::to_string(&role_ids)
        .map_err(|e| format!("序列化失败: {}", e))?;

    fs::write(&config_file, json)
        .map_err(|e| format!("写入配置失败: {}", e))?;

    Ok(())
}

/// 获取用户选择的角色
#[tauri::command]
pub fn get_user_role() -> Result<Vec<u32>, String> {
    let app_config_dir = get_config_dir()?;
    let config_file = app_config_dir.join("user_roles.json");

    if !config_file.exists() {
        // 默认返回空列表（用户未选择角色）
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&config_file)
        .map_err(|e| format!("读取配置失败: {}", e))?;

    let role_ids: Vec<u32> = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    Ok(role_ids)
}
