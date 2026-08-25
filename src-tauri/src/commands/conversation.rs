use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use serde::Serialize;
use uuid::Uuid;

use crate::db::open_connection;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub tool_calls_json: Option<String>,
    /// 思考/推理内容（模型未提供则序列化时省略）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    pub created_at: i64,
}

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  tool_calls_json TEXT,
  reasoning TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL,
  FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
";

pub fn ensure_conversation_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(SCHEMA_SQL)?;
    migrate_add_reasoning(conn)?;
    Ok(())
}

/// 向后兼容 migration：为旧库的 messages 表幂等补齐 `reasoning` 列
/// （duplicate column 错误视为已存在，静默通过）。
fn migrate_add_reasoning(conn: &Connection) -> Result<(), rusqlite::Error> {
    // 幂等：列已存在时 SQLite 报 duplicate column，视为成功。
    match conn.execute(
        "ALTER TABLE messages ADD COLUMN reasoning TEXT NOT NULL DEFAULT ''",
        [],
    ) {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("duplicate column") => Ok(()),
        Err(e) => Err(e),
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("系统时钟早于 Unix epoch")
        .as_secs() as i64
}

fn truncate_title(content: &str) -> String {
    content.chars().take(40).collect()
}

fn row_to_conversation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        id: row.get(0)?,
        title: row.get(1)?,
        updated_at: row.get(2)?,
    })
}

fn row_to_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatMessage> {
    let reasoning: String = row.get(6)?;
    Ok(ChatMessage {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        tool_calls_json: row.get(4)?,
        reasoning: if reasoning.is_empty() {
            None
        } else {
            Some(reasoning)
        },
        created_at: row.get(5)?,
    })
}

pub fn append_user_message_on(
    conn: &Connection,
    conversation_id: Option<String>,
    content: &str,
) -> Result<(Conversation, ChatMessage), String> {
    ensure_conversation_schema(conn).map_err(|e| e.to_string())?;

    let ts = now_ts();
    let message_id = Uuid::new_v4().to_string();

    match conversation_id {
        None => {
            let conv_id = Uuid::new_v4().to_string();
            let title = truncate_title(content);
            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT INTO conversations (id, title, updated_at) VALUES (?1, ?2, ?3)",
                params![conv_id, title, ts],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT INTO messages (id, conversation_id, role, content, tool_calls_json, reasoning, created_at)
                 VALUES (?1, ?2, 'user', ?3, NULL, '', ?4)",
                params![message_id, conv_id, content, ts],
            )
            .map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;

            let conversation = Conversation {
                id: conv_id.clone(),
                title,
                updated_at: ts,
            };
            let message = ChatMessage {
                id: message_id,
                conversation_id: conv_id,
                role: "user".to_string(),
                content: content.to_string(),
                tool_calls_json: None,
                reasoning: None,
                created_at: ts,
            };
            Ok((conversation, message))
        }
        Some(conv_id) => {
            let mut exists = conn
                .prepare("SELECT id, title, updated_at FROM conversations WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            let conversation = exists
                .query_row(params![conv_id], row_to_conversation)
                .map_err(|_| format!("会话不存在: {conv_id}"))?;

            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            tx.execute(
                "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
                params![ts, conv_id],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT INTO messages (id, conversation_id, role, content, tool_calls_json, reasoning, created_at)
                 VALUES (?1, ?2, 'user', ?3, NULL, '', ?4)",
                params![message_id, conv_id, content, ts],
            )
            .map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;

            let message = ChatMessage {
                id: message_id,
                conversation_id: conv_id.clone(),
                role: "user".to_string(),
                content: content.to_string(),
                tool_calls_json: None,
                reasoning: None,
                created_at: ts,
            };
            let updated = Conversation {
                updated_at: ts,
                ..conversation
            };
            Ok((updated, message))
        }
    }
}

pub fn append_assistant_message_on(
    conn: &Connection,
    conversation_id: &str,
    content: &str,
    tool_calls_json: Option<&str>,
    reasoning: Option<&str>,
) -> Result<ChatMessage, String> {
    ensure_conversation_schema(conn).map_err(|e| e.to_string())?;

    let mut exists = conn
        .prepare("SELECT id FROM conversations WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    exists
        .query_row(params![conversation_id], |_| Ok(()))
        .map_err(|_| format!("会话不存在: {conversation_id}"))?;

    let ts = now_ts();
    let message_id = Uuid::new_v4().to_string();
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![ts, conversation_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO messages (id, conversation_id, role, content, tool_calls_json, reasoning, created_at)
         VALUES (?1, ?2, 'assistant', ?3, ?4, ?5, ?6)",
        params![message_id, conversation_id, content, tool_calls_json, reasoning.unwrap_or(""), ts],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(ChatMessage {
        id: message_id,
        conversation_id: conversation_id.to_string(),
        role: "assistant".to_string(),
        content: content.to_string(),
        tool_calls_json: tool_calls_json.map(|s| s.to_string()),
        reasoning: reasoning.map(|s| s.to_string()).filter(|s| !s.is_empty()),
        created_at: ts,
    })
}

pub fn list_conversations_on(conn: &Connection) -> Result<Vec<Conversation>, String> {
    ensure_conversation_schema(conn).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, title, updated_at FROM conversations ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], row_to_conversation)
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_messages_on(conn: &Connection, conversation_id: &str) -> Result<Vec<ChatMessage>, String> {
    ensure_conversation_schema(conn).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, tool_calls_json, created_at, reasoning
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![conversation_id], row_to_message)
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn delete_conversation_on(conn: &Connection, conversation_id: &str) -> Result<(), String> {
    ensure_conversation_schema(conn).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM conversations WHERE id = ?1",
        params![conversation_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn append_user_message(
    conversationId: Option<String>,
    content: String,
) -> Result<(Conversation, ChatMessage), String> {
    let conn = open_connection()?;
    append_user_message_on(&conn, conversationId, &content)
}

#[tauri::command]
pub fn list_conversations() -> Result<Vec<Conversation>, String> {
    let conn = open_connection()?;
    list_conversations_on(&conn)
}

pub fn get_messages(conversation_id: &str) -> Result<Vec<ChatMessage>, String> {
    let conn = open_connection()?;
    get_messages_on(&conn, conversation_id)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn get_conversation_messages(conversationId: String) -> Result<Vec<ChatMessage>, String> {
    get_messages(&conversationId)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn delete_conversation(conversationId: String) -> Result<(), String> {
    let conn = open_connection()?;
    delete_conversation_on(&conn, &conversationId)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_db() -> Connection {
        let mut dir = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("tbox-conversation-test-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        let path: PathBuf = dir.join("tools.db");
        Connection::open(path).unwrap()
    }

    #[test]
    fn first_user_message_creates_conversation() {
        let db = test_db();
        let (conv, msg) = append_user_message_on(&db, None, "请把 hello 做 Base64").unwrap();
        assert!(!conv.id.is_empty());
        assert!(conv.title.contains("Base64") || conv.title.contains("hello"));
        assert_eq!(msg.role, "user");
        assert_eq!(list_conversations_on(&db).unwrap().len(), 1);
    }

    #[test]
    fn delete_removes_messages() {
        let db = test_db();
        let (conv, _) = append_user_message_on(&db, None, "hi").unwrap();
        delete_conversation_on(&db, &conv.id).unwrap();
        assert!(list_conversations_on(&db).unwrap().is_empty());
        assert!(get_messages_on(&db, &conv.id).unwrap().is_empty());
    }

    #[test]
    fn list_empty_without_any_message() {
        let db = test_db();
        assert!(list_conversations_on(&db).unwrap().is_empty());
    }
#[test]
fn reasoning_roundtrip_and_migration() {
    let db = test_db();
    // 走 ensure 前，先模拟旧 schema（无 reasoning 列）
    db.execute_batch(
        "CREATE TABLE messages (
           id TEXT PRIMARY KEY,
           conversation_id TEXT NOT NULL,
           role TEXT NOT NULL,
           content TEXT NOT NULL,
           tool_calls_json TEXT,
           created_at INTEGER NOT NULL
         );",
    )
    .unwrap();
    ensure_conversation_schema(&db).unwrap();
    // 幂等：再跑一次不报错
    ensure_conversation_schema(&db).unwrap();

    let (conv, _) = append_user_message_on(&db, None, "hi").unwrap();
    let saved = append_assistant_message_on(&db, &conv.id, "answer", None, Some("思考中…")).unwrap();
    assert_eq!(saved.reasoning.as_deref(), Some("思考中…"));

    let msgs = get_messages_on(&db, &conv.id).unwrap();
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].reasoning, None);
    assert_eq!(msgs[1].reasoning.as_deref(), Some("思考中…"));
}
}
