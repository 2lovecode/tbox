//! 本地密码库。页面与 Agent 读写同一份 `~/.toolbox/passwords.json`。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub id: String,
    pub name: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

static OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

fn store_path() -> PathBuf {
    if let Ok(guard) = OVERRIDE.lock() {
        if let Some(path) = guard.as_ref() {
            return path.clone();
        }
    }
    crate::agent::llama_log::toolbox_dir().join("passwords.json")
}

fn load(path: &Path) -> Vec<PasswordEntry> {
    let raw = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    serde_json::from_slice(&raw).unwrap_or_default()
}

fn save_all(path: &Path, entries: &[PasswordEntry]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("创建密码库目录失败: {e}"))?;
    }
    let raw = serde_json::to_vec_pretty(entries).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| format!("写入密码库失败: {e}"))
}

#[tauri::command]
pub fn password_list() -> Result<Vec<PasswordEntry>, String> {
    Ok(load(&store_path()))
}

#[tauri::command]
pub fn password_get(id: String) -> Result<PasswordEntry, String> {
    load(&store_path())
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| "找不到该密码条目".to_string())
}

#[tauri::command]
pub fn password_save(
    id: Option<String>,
    name: String,
    username: String,
    password: String,
    url: String,
    notes: String,
) -> Result<PasswordEntry, String> {
    if name.trim().is_empty() || password.is_empty() {
        return Err("请填写名称和密码".to_string());
    }
    let path = store_path();
    let mut entries = load(&path);
    let now = chrono::Local::now().to_rfc3339();
    if let Some(id) = id.filter(|s| !s.is_empty()) {
        let entry = entries
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| "找不到要更新的条目".to_string())?;
        entry.name = name;
        entry.username = username;
        entry.password = password;
        entry.url = url;
        entry.notes = notes;
        let saved = entry.clone();
        save_all(&path, &entries)?;
        Ok(saved)
    } else {
        let entry = PasswordEntry {
            id: format!("{}", chrono::Utc::now().timestamp_millis()),
            name,
            username,
            password,
            url,
            notes,
            created_at: now,
        };
        entries.push(entry.clone());
        save_all(&path, &entries)?;
        Ok(entry)
    }
}

#[tauri::command]
pub fn password_delete(id: String) -> Result<String, String> {
    let path = store_path();
    let entries = load(&path);
    let next: Vec<_> = entries.into_iter().filter(|e| e.id != id).collect();
    save_all(&path, &next)?;
    Ok("已删除".to_string())
}

#[tauri::command]
pub fn password_import(entries: Vec<PasswordEntry>) -> Result<usize, String> {
    let path = store_path();
    let n = entries.len();
    save_all(&path, &entries)?;
    Ok(n)
}

#[tauri::command]
pub fn password_generate(length: Option<u32>) -> String {
    let n = length.unwrap_or(16).clamp(4, 128) as usize;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

#[cfg(test)]
pub fn set_store_override(path: Option<PathBuf>) {
    *OVERRIDE.lock().expect("password store lock") = path;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn save_and_list_round_trip() {
        let mut path = std::env::temp_dir();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        path.push(format!("tbox-passwords-{unique}.json"));
        set_store_override(Some(path.clone()));
        let saved = password_save(
            None,
            "mail".into(),
            "a".into(),
            "secret".into(),
            "".into(),
            "".into(),
        )
        .unwrap();
        let listed = password_list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].password, "secret");
        password_delete(saved.id).unwrap();
        assert!(password_list().unwrap().is_empty());
        set_store_override(None);
        let _ = std::fs::remove_file(path);
    }
}
