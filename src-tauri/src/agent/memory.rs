//! Cross-session user memory: extract, write-time UPDATE, retrieve, inject.

use std::fs;
use std::path::PathBuf;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::context_budget::estimate_tokens;
use crate::db::open_connection;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS user_memories (
  id TEXT PRIMARY KEY,
  key TEXT,
  text TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  source_conversation_id TEXT,
  evidence_excerpt TEXT,
  updated_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS user_memory_revisions (
  id TEXT PRIMARY KEY,
  memory_id TEXT NOT NULL,
  text TEXT NOT NULL,
  key TEXT,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(memory_id) REFERENCES user_memories(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_user_memories_status ON user_memories(status);
";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryItem {
    pub id: String,
    pub key: Option<String>,
    pub text: String,
    pub status: String,
    pub source_conversation_id: Option<String>,
    pub evidence_excerpt: Option<String>,
    pub updated_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryAction {
    Add,
    Update { id: String },
    Delete { id: String },
    Noop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettings {
    pub auto_memory_enabled: bool,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            auto_memory_enabled: true,
        }
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn settings_path() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push(".toolbox");
    dir.push("memory_settings.json");
    dir
}

pub fn load_memory_settings() -> MemorySettings {
    let path = settings_path();
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => MemorySettings::default(),
    }
}

pub fn save_memory_settings(settings: &MemorySettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn ensure_memory_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(SCHEMA)?;
    Ok(())
}

pub fn looks_like_secret(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("token")
            && (lower.contains("sk-") || lower.contains("bearer"))
    {
        return true;
    }
    // OpenAI-like keys
    if text.contains("sk-") && text.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').count() > 20
    {
        return true;
    }
    false
}

fn normalize_key(text: &str) -> Option<String> {
    let t = text.to_lowercase();
    if t.contains("中文") || t.contains("chinese") || t.contains("english") || t.contains("英文")
        || t.contains("语言") || t.contains("language") || t.contains("简洁") || t.contains("concise")
    {
        return Some("preference.language_style".into());
    }
    None
}

/// Heuristic candidate extraction for tests / LLM fallback.
pub fn extract_candidates_heuristic(user_and_assistant: &str) -> Vec<(Option<String>, String)> {
    let mut out = Vec::new();
    let text = user_and_assistant.trim();
    if text.is_empty() {
        return out;
    }
    let lower = text.to_lowercase();
    if lower.contains("用中文") || lower.contains("中文简洁") || lower.contains("always use chinese")
    {
        out.push((
            Some("preference.language_style".into()),
            "用户偏好：中文简洁回答".into(),
        ));
    }
    if lower.contains("用英文") || lower.contains("use english") || lower.contains("in english")
    {
        out.push((
            Some("preference.language_style".into()),
            "用户偏好：英文回答".into(),
        ));
    }
    if lower.contains("不要 emoji") || lower.contains("no emoji") {
        out.push((
            Some("preference.emoji".into()),
            "用户偏好：不要使用 emoji".into(),
        ));
    }
    out
}

fn similarity(a: &str, b: &str) -> f64 {
    let al: Vec<char> = a.chars().collect();
    let bl: Vec<char> = b.chars().collect();
    if al.is_empty() || bl.is_empty() {
        return 0.0;
    }
    let mut overlap = 0usize;
    for c in &al {
        if bl.contains(c) {
            overlap += 1;
        }
    }
    overlap as f64 / al.len().max(bl.len()) as f64
}

pub fn decide_action(
    key: &Option<String>,
    text: &str,
    existing: &[MemoryItem],
) -> MemoryAction {
    if looks_like_secret(text) {
        return MemoryAction::Noop;
    }
    if let Some(k) = key {
        if let Some(hit) = existing.iter().find(|m| m.key.as_deref() == Some(k.as_str()) && m.status == "active")
        {
            if hit.text == text {
                return MemoryAction::Noop;
            }
            return MemoryAction::Update {
                id: hit.id.clone(),
            };
        }
    }
    // Fuzzy match on text
    let mut best: Option<(&MemoryItem, f64)> = None;
    for m in existing.iter().filter(|m| m.status == "active") {
        let s = similarity(&m.text, text);
        if s > 0.55 {
            if best.map(|(_, bs)| s > bs).unwrap_or(true) {
                best = Some((m, s));
            }
        }
    }
    if let Some((m, _)) = best {
        if m.text == text {
            return MemoryAction::Noop;
        }
        // Same topic key preference → UPDATE
        if m.key.is_some() && key == &m.key {
            return MemoryAction::Update {
                id: m.id.clone(),
            };
        }
        if key.is_some() && m.key == *key {
            return MemoryAction::Update {
                id: m.id.clone(),
            };
        }
        // High overlap same key family
        if similarity(&m.text, text) > 0.7 && key.is_some() {
            return MemoryAction::Update {
                id: m.id.clone(),
            };
        }
    }
    MemoryAction::Add
}

pub fn list_active_memories(conn: &Connection) -> Result<Vec<MemoryItem>, String> {
    ensure_memory_schema(conn).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, key, text, status, source_conversation_id, evidence_excerpt, updated_at, created_at
             FROM user_memories WHERE status = 'active' ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(MemoryItem {
                id: row.get(0)?,
                key: row.get(1)?,
                text: row.get(2)?,
                status: row.get(3)?,
                source_conversation_id: row.get(4)?,
                evidence_excerpt: row.get(5)?,
                updated_at: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn apply_action(
    conn: &Connection,
    action: MemoryAction,
    key: Option<String>,
    text: &str,
    source_conversation_id: Option<&str>,
    evidence: Option<&str>,
) -> Result<Option<MemoryItem>, String> {
    ensure_memory_schema(conn).map_err(|e| e.to_string())?;
    if looks_like_secret(text) {
        return Ok(None);
    }
    let ts = now_ts();
    match action {
        MemoryAction::Noop => Ok(None),
        MemoryAction::Add => {
            let id = Uuid::new_v4().to_string();
            let key = key.or_else(|| normalize_key(text));
            conn.execute(
                "INSERT INTO user_memories (id, key, text, status, source_conversation_id, evidence_excerpt, updated_at, created_at)
                 VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6, ?6)",
                params![id, key, text, source_conversation_id, evidence, ts],
            )
            .map_err(|e| e.to_string())?;
            Ok(Some(MemoryItem {
                id,
                key,
                text: text.to_string(),
                status: "active".into(),
                source_conversation_id: source_conversation_id.map(|s| s.to_string()),
                evidence_excerpt: evidence.map(|s| s.to_string()),
                updated_at: ts,
                created_at: ts,
            }))
        }
        MemoryAction::Update { id } => {
            let existing: (Option<String>, String, i64) = conn
                .query_row(
                    "SELECT key, text, updated_at FROM user_memories WHERE id = ?1",
                    params![id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|e| e.to_string())?;
            let rev_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO user_memory_revisions (id, memory_id, text, key, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![rev_id, id, existing.1, existing.0, existing.2],
            )
            .map_err(|e| e.to_string())?;
            let key = key.or(existing.0);
            conn.execute(
                "UPDATE user_memories SET text = ?1, key = ?2, source_conversation_id = ?3, evidence_excerpt = ?4, updated_at = ?5 WHERE id = ?6",
                params![text, key, source_conversation_id, evidence, ts, id],
            )
            .map_err(|e| e.to_string())?;
            Ok(Some(MemoryItem {
                id,
                key,
                text: text.to_string(),
                status: "active".into(),
                source_conversation_id: source_conversation_id.map(|s| s.to_string()),
                evidence_excerpt: evidence.map(|s| s.to_string()),
                updated_at: ts,
                created_at: ts,
            }))
        }
        MemoryAction::Delete { id } => {
            conn.execute(
                "UPDATE user_memories SET status = 'tombstone', updated_at = ?1 WHERE id = ?2",
                params![ts, id],
            )
            .map_err(|e| e.to_string())?;
            Ok(None)
        }
    }
}

pub fn soft_delete_memory(conn: &Connection, id: &str) -> Result<(), String> {
    apply_action(
        conn,
        MemoryAction::Delete { id: id.to_string() },
        None,
        "",
        None,
        None,
    )?;
    Ok(())
}

pub fn undo_last_update(conn: &Connection, memory_id: &str) -> Result<Option<MemoryItem>, String> {
    ensure_memory_schema(conn).map_err(|e| e.to_string())?;
    let rev: Option<(String, String, Option<String>, i64)> = conn
        .query_row(
            "SELECT id, text, key, updated_at FROM user_memory_revisions
             WHERE memory_id = ?1 ORDER BY updated_at DESC LIMIT 1",
            params![memory_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let Some((rev_id, text, key, _)) = rev else {
        return Ok(None);
    };
    let ts = now_ts();
    conn.execute(
        "UPDATE user_memories SET text = ?1, key = ?2, updated_at = ?3 WHERE id = ?4",
        params![text, key, ts, memory_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM user_memory_revisions WHERE id = ?1",
        params![rev_id],
    )
    .map_err(|e| e.to_string())?;
    let item = conn
        .query_row(
            "SELECT id, key, text, status, source_conversation_id, evidence_excerpt, updated_at, created_at
             FROM user_memories WHERE id = ?1",
            params![memory_id],
            |row| {
                Ok(MemoryItem {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    text: row.get(2)?,
                    status: row.get(3)?,
                    source_conversation_id: row.get(4)?,
                    evidence_excerpt: row.get(5)?,
                    updated_at: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(Some(item))
}

trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Ingest candidates with write-time decide.
pub fn ingest_candidates(
    conn: &Connection,
    candidates: &[(Option<String>, String)],
    conversation_id: &str,
    evidence: &str,
) -> Result<usize, String> {
    let existing = list_active_memories(conn)?;
    let mut n = 0usize;
    for (key, text) in candidates {
        if looks_like_secret(text) {
            continue;
        }
        let action = decide_action(key, text, &existing);
        if matches!(action, MemoryAction::Noop) {
            continue;
        }
        if apply_action(
            conn,
            action,
            key.clone(),
            text,
            Some(conversation_id),
            Some(evidence),
        )?
        .is_some()
        {
            n += 1;
        }
    }
    Ok(n)
}

/// Top-k retrieve by simple keyword overlap with query; respect char/token budget.
pub fn retrieve_for_inject(
    conn: &Connection,
    query: &str,
    max_items: usize,
    max_tokens: u32,
) -> Result<(String, u32), String> {
    let items = list_active_memories(conn)?;
    let mut scored: Vec<(i32, &MemoryItem)> = items
        .iter()
        .map(|m| {
            let s = (similarity(&m.text, query) * 1000.0) as i32;
            (s, m)
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let mut lines = Vec::new();
    let mut tokens = 0u32;
    for (_, m) in scored.into_iter().take(max_items.max(1) * 2) {
        if lines.len() >= max_items {
            break;
        }
        let line = format!("- {}", m.text);
        let t = estimate_tokens(&line);
        if tokens.saturating_add(t) > max_tokens && !lines.is_empty() {
            break;
        }
        tokens = tokens.saturating_add(t);
        lines.push(line);
    }
    if lines.is_empty() {
        return Ok((String::new(), 0));
    }
    let block = format!("## User memory\n{}", lines.join("\n"));
    let tokens = estimate_tokens(&block);
    Ok((block, tokens))
}

pub fn memory_token_cap(context_limit: u32) -> u32 {
    let five_pct = context_limit.saturating_mul(5) / 100;
    five_pct.max(64).min(512)
}

/// Run heuristic extract+ingest after a turn (non-fatal).
pub fn extract_and_ingest_session_delta(
    conversation_id: &str,
    delta_text: &str,
) -> Result<usize, String> {
    if !load_memory_settings().auto_memory_enabled {
        return Ok(0);
    }
    let candidates = extract_candidates_heuristic(delta_text);
    if candidates.is_empty() {
        return Ok(0);
    }
    let conn = open_connection().map_err(|e| e.to_string())?;
    let excerpt: String = delta_text.chars().take(240).collect();
    ingest_candidates(&conn, &candidates, conversation_id, &excerpt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure_memory_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn secrets_rejected() {
        assert!(looks_like_secret("my api_key is sk-abc"));
        let conn = mem_db();
        let action = decide_action(&None, "api_key sk-test123456789012345", &[]);
        assert_eq!(action, MemoryAction::Noop);
        let r = apply_action(
            &conn,
            MemoryAction::Add,
            None,
            "password=secret123",
            None,
            None,
        )
        .unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn preference_update_replaces() {
        let conn = mem_db();
        let key = Some("preference.language_style".into());
        apply_action(
            &conn,
            MemoryAction::Add,
            key.clone(),
            "用户偏好：中文简洁回答",
            Some("c1"),
            Some("用中文"),
        )
        .unwrap();
        let existing = list_active_memories(&conn).unwrap();
        let action = decide_action(&key, "用户偏好：英文回答", &existing);
        assert!(matches!(action, MemoryAction::Update { .. }));
        apply_action(
            &conn,
            action,
            key,
            "用户偏好：英文回答",
            Some("c2"),
            Some("用英文"),
        )
        .unwrap();
        let list = list_active_memories(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].text.contains("英文"));
        // revision exists
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM user_memory_revisions",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
        let restored = undo_last_update(&conn, &list[0].id).unwrap().unwrap();
        assert!(restored.text.contains("中文"));
    }

    #[test]
    fn basic_recall_add_three() {
        let conn = mem_db();
        for (k, t) in [
            ("preference.emoji", "用户偏好：不要使用 emoji"),
            ("preference.language_style", "用户偏好：中文简洁回答"),
            ("note.role", "用户是后端工程师"),
        ] {
            apply_action(
                &conn,
                MemoryAction::Add,
                Some(k.into()),
                t,
                Some("c"),
                None,
            )
            .unwrap();
        }
        assert_eq!(list_active_memories(&conn).unwrap().len(), 3);
    }

    #[test]
    fn inject_respects_token_cap() {
        let conn = mem_db();
        for i in 0..20 {
            apply_action(
                &conn,
                MemoryAction::Add,
                Some(format!("k{i}")),
                &format!("很长的用户记忆条目编号 {i} {}", "详".repeat(40)),
                None,
                None,
            )
            .unwrap();
        }
        let (_block, tokens) = retrieve_for_inject(&conn, "记忆", 8, 80).unwrap();
        assert!(tokens <= 80 + 20); // small slack for block header
    }

    #[test]
    fn heuristic_extract_chinese_then_english() {
        let c1 = extract_candidates_heuristic("请用中文简洁回答");
        assert!(!c1.is_empty());
        let c2 = extract_candidates_heuristic("以后请用英文回答 use english");
        assert!(c2.iter().any(|(_, t)| t.contains("英文")));
    }
}
