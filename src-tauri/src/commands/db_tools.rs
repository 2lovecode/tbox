use mysql::prelude::*;
use mysql::{Pool, Opts};
use postgres::{Client as PgClient, NoTls};
use rusqlite::Connection;

/// 测试MySQL连接
#[tauri::command]
pub async fn test_mysql_connection(
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String
) -> Result<String, String> {
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        username, password, host, port, database
    );

    let pool = Pool::new(Opts::from_url(&url)
        .map_err(|e| format!("MySQL URL解析失败: {}", e))?
    )
        .map_err(|e| format!("MySQL连接失败: {}", e))?;

    let mut conn = pool.get_conn()
        .map_err(|e| format!("获取MySQL连接失败: {}", e))?;

    let version: String = conn.query_first("SELECT VERSION()")
        .map_err(|e| format!("查询MySQL版本失败: {}", e))?
        .ok_or_else(|| "无法获取MySQL版本".to_string())?;

    Ok(format!("MySQL连接成功！版本: {}", version))
}

/// 测试PostgreSQL连接
#[tauri::command]
pub fn test_postgres_connection(
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String
) -> Result<String, String> {
    let mut client = PgClient::connect(
        &format!(
            "host={} port={} user={} password={} dbname={}",
            host, port, username, password, database
        ),
        NoTls
    ).map_err(|e| format!("PostgreSQL连接失败: {}", e))?;

    let row = client.query_one("SELECT version()", &[])
        .map_err(|e| format!("查询PostgreSQL版本失败: {}", e))?;

    let version: String = row.get(0);

    Ok(format!("PostgreSQL连接成功！版本: {}", version))
}

/// 测试SQLite连接
#[tauri::command]
pub fn test_sqlite_connection(file_path: String) -> Result<String, String> {
    let _conn = Connection::open(&file_path)
        .map_err(|e| format!("SQLite连接失败: {}", e))?;

    // SQLite是文件数据库，只要能打开就是连接成功
    Ok(format!("SQLite连接成功！文件: {}", file_path))
}

/// 执行MySQL查询
#[tauri::command]
pub async fn execute_mysql_query(
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
    query: String
) -> Result<serde_json::Value, String> {
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        username, password, host, port, database
    );

    let pool = Pool::new(Opts::from_url(&url)
        .map_err(|e| format!("MySQL URL解析失败: {}", e))?
    )
        .map_err(|e| format!("MySQL连接失败: {}", e))?;

    let mut conn = pool.get_conn()
        .map_err(|e| format!("获取MySQL连接失败: {}", e))?;

    // 简化实现：只查询行数
    let result: String = conn.query_first(&query)
        .map_err(|e| format!("查询执行失败: {}", e))?
        .unwrap_or_else(|| "无结果".to_string());

    Ok(serde_json::json!({
        "result": result
    }))
}

/// 执行PostgreSQL查询
#[tauri::command]
pub fn execute_postgres_query(
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
    query: String
) -> Result<serde_json::Value, String> {
    let mut client = PgClient::connect(
        &format!(
            "host={} port={} user={} password={} dbname={}",
            host, port, username, password, database
        ),
        NoTls
    ).map_err(|e| format!("PostgreSQL连接失败: {}", e))?;

    let rows = client.query(&query, &[])
        .map_err(|e| format!("查询执行失败: {}", e))?;

    let mut result = Vec::new();
    for row in rows {
        let mut values = Vec::new();
        for i in 0..row.len() {
            let value: String = row.try_get(i)
                .unwrap_or_else(|_| "NULL".to_string());
            values.push(value);
        }
        result.push(values);
    }

    Ok(serde_json::json!({
        "rows": result
    }))
}

/// 执行SQLite查询
#[tauri::command]
pub fn execute_sqlite_query(
    file_path: String,
    query: String
) -> Result<serde_json::Value, String> {
    let conn = Connection::open(&file_path)
        .map_err(|e| format!("SQLite连接失败: {}", e))?;

    let mut stmt = conn.prepare(&query)
        .map_err(|e| format!("SQL准备失败: {}", e))?;

    let column_names: Vec<String> = stmt.column_names()
        .iter()
        .map(|&s| s.to_string())
        .collect();

    let mut rows = Vec::new();
    let mut row_data = stmt.query([])
        .map_err(|e| format!("查询执行失败: {}", e))?;

    while let Some(row) = row_data.next()
        .map_err(|e| format!("读取行失败: {}", e))? {
        let mut values = Vec::new();
        for i in 0..row.as_ref().column_count() {
            let value = row.get_ref(i)
                .and_then(|v| Ok(v.as_str()?))
                .unwrap_or_else(|_| "NULL")
                .to_string();
            values.push(value);
        }
        rows.push(values);
    }

    Ok(serde_json::json!({
        "columns": column_names,
        "rows": rows
    }))
}
