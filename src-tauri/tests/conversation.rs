//! Integration coverage for conversation persistence when lib unit-test binaries
//! cannot start on Windows (Tauri native DLL chain).

use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tbox_lib::commands::conversation::{
    append_user_message_on, delete_conversation_on, get_messages_on, list_conversations_on,
};

fn test_db() -> Connection {
    let mut dir = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("tbox-conversation-itest-{unique}"));
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
fn append_existing_conversation_is_transactional() {
    let db = test_db();
    let (conv, _) = append_user_message_on(&db, None, "first").unwrap();
    let before = conv.updated_at;

    std::thread::sleep(std::time::Duration::from_millis(1100));

    let (updated, msg) = append_user_message_on(&db, Some(conv.id.clone()), "second").unwrap();
    assert_eq!(updated.id, conv.id);
    assert!(updated.updated_at >= before);
    assert_eq!(msg.content, "second");
    assert_eq!(get_messages_on(&db, &conv.id).unwrap().len(), 2);
}
