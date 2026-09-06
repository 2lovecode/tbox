//! Tauri commands for user memory management.

use crate::agent::memory::{
    ensure_memory_schema, list_active_memories, load_memory_settings, save_memory_settings,
    soft_delete_memory, undo_last_update, MemoryItem, MemorySettings,
};
use crate::db::open_connection;

#[tauri::command]
pub fn list_user_memories() -> Result<Vec<MemoryItem>, String> {
    let conn = open_connection()?;
    ensure_memory_schema(&conn).map_err(|e| e.to_string())?;
    list_active_memories(&conn)
}

#[tauri::command]
pub fn delete_user_memory(id: String) -> Result<(), String> {
    let conn = open_connection()?;
    soft_delete_memory(&conn, &id)
}

#[tauri::command]
pub fn undo_user_memory_update(id: String) -> Result<Option<MemoryItem>, String> {
    let conn = open_connection()?;
    undo_last_update(&conn, &id)
}

#[tauri::command]
pub fn get_memory_settings() -> MemorySettings {
    load_memory_settings()
}

#[tauri::command]
pub fn save_memory_settings_cmd(settings: MemorySettings) -> Result<MemorySettings, String> {
    save_memory_settings(&settings)?;
    Ok(load_memory_settings())
}
