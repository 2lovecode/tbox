//! Session event log — append-only trajectory ledger (DeepSeek Harness style).
//!
//! 设计原则（spec: Session Event Log）：
//! - **日志即事实源**：每次交互（用户输入、模型请求/回复、工具调用/结果、
//!   上下文注入、压缩）都追加一条 `SessionEvent`，历史永不修改。
//! - **日志重建一致性**：发起模型请求前从日志推导上下文骨架并与实际
//!   请求比对，结果记录在 `request/header.desync_check`。
//! - 存储走既有 SQLite（`session_events` 表），随会话 FK 级联删除。

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// 一条会话事件。`seq` 会话内全局递增；`time` Unix 毫秒。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEvent {
    pub seq: i64,
    pub conversation_id: String,
    /// Unix milliseconds.
    pub time: i64,
    /// 事件类型（turn/start, step/start, user/message, request/header,
    /// assistant/message, tool/call, tool/result, context/snapshot,
    /// compact/checkpoint, step/end, turn/end）。
    #[serde(rename = "type")]
    pub kind: String,
    pub data: Value,
}

const EVENT_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS session_events (
  seq INTEGER NOT NULL,
  conversation_id TEXT NOT NULL,
  time INTEGER NOT NULL,
  type TEXT NOT NULL,
  data TEXT NOT NULL,
  PRIMARY KEY (conversation_id, seq),
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_session_events_conv ON session_events(conversation_id, seq);
";

pub fn ensure_session_event_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(EVENT_SCHEMA_SQL)?;
    Ok(())
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 追加一条事件；`seq` 由 `MAX(seq)+1` 分配（会话内递增）。
/// 幂等依赖调用方单线程写（Agent 循环串行执行）。
pub fn append_event_on(
    conn: &Connection,
    conversation_id: &str,
    kind: &str,
    data: Value,
) -> Result<SessionEvent, String> {
    ensure_session_event_schema(conn).map_err(|e| e.to_string())?;
    let time = now_millis();
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let next_seq: i64 = tx
        .query_row(
            "SELECT COALESCE(MAX(seq), 0) + 1 FROM session_events WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO session_events (seq, conversation_id, time, type, data) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![next_seq, conversation_id, time, kind, data.to_string()],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(SessionEvent {
        seq: next_seq,
        conversation_id: conversation_id.to_string(),
        time,
        kind: kind.to_string(),
        data,
    })
}

/// 列出某会话全部事件（按 seq 升序，无分页——单会话事件量 << 数千）。
pub fn list_events_on(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<SessionEvent>, String> {
    ensure_session_event_schema(conn).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT seq, conversation_id, time, type, data FROM session_events
             WHERE conversation_id = ?1 ORDER BY seq ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![conversation_id], |row| {
            let data_raw: String = row.get(4)?;
            let data: Value = serde_json::from_str(&data_raw).unwrap_or(Value::Null);
            Ok(SessionEvent {
                seq: row.get(0)?,
                conversation_id: row.get(1)?,
                time: row.get(2)?,
                kind: row.get(3)?,
                data,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut events = Vec::new();
    for row in rows {
        events.push(row.map_err(|e| e.to_string())?);
    }
    Ok(events)
}

// ---------------------------------------------------------------------------
// 事件构造便捷函数（保持 kind 字符串单点定义）
// ---------------------------------------------------------------------------

pub fn ev_turn_start(turn_no: usize) -> Value {
    json!({ "turn_no": turn_no })
}

pub fn ev_step_start(turn_no: usize, step_no: usize) -> Value {
    json!({ "turn_no": turn_no, "step_no": step_no })
}

pub fn ev_user_message(content: &str) -> Value {
    json!({ "content": content })
}

/// 系统提示词正文（不含工具目录；工具见 `tools/catalog`）。
pub fn ev_system_message(content: &str) -> Value {
    json!({
        "content": content,
        "title": "静态系统提示词",
        "change": "initial",
    })
}

/// 静态系统提示词相对上一份快照发生变更。
pub fn ev_system_prompt_updated(previous_content: &str, content: &str) -> Value {
    json!({
        "content": content,
        "previous_content": previous_content,
        "title": "静态系统提示词已更新",
        "change": "prompt",
    })
}

/// 工具目录：系统提示词族分条落库（与静态正文分开展示）。
pub fn ev_tools_catalog(tools: &Value) -> Value {
    json!({
        "title": "工具已加载",
        "change": "initial",
        "tools": tools,
    })
}

/// 工具目录相对上一份快照发生变更。
pub fn ev_tools_catalog_updated(previous_tools: &Value, tools: &Value) -> Value {
    json!({
        "title": "工具已更新",
        "change": "updated",
        "tools": tools,
        "previous_tools": previous_tools,
    })
}

/// Skill L1 正文：系统提示词族分条落库（与静态正文分开展示）。
pub fn ev_skills_snapshot(skills: &[(String, String)]) -> Value {
    let arr: Vec<Value> = skills
        .iter()
        .map(|(id, body)| {
            json!({
                "id": id,
                "body": body,
            })
        })
        .collect();
    let chars: usize = skills.iter().map(|(_, b)| b.chars().count()).sum();
    json!({
        "source": "skills",
        "chars": chars,
        "skills": arr,
        "title": if skills.is_empty() {
            "技能".to_string()
        } else {
            format!("已加载 {} 个技能", skills.len())
        },
    })
}

/// Skill L0 目录模块（Agent Skills：name+description 常驻）。
pub fn ev_skills_catalog(entries: &[crate::agent::skills::SkillCatalogEntry]) -> Value {
    json!({
        "title": "技能目录已加载",
        "change": "initial",
        "skills": entries,
    })
}

pub fn ev_skills_catalog_updated(
    previous: &[crate::agent::skills::SkillCatalogEntry],
    entries: &[crate::agent::skills::SkillCatalogEntry],
) -> Value {
    json!({
        "title": "技能目录已更新",
        "change": "updated",
        "skills": entries,
        "previous_skills": previous,
    })
}

/// 从已有事件取最近系统提示词正文 + 最近工具目录（兼容旧 system/message.tools）。
pub fn last_system_snapshot(events: &[SessionEvent]) -> Option<(String, Value)> {
    let mut content: Option<String> = None;
    let mut tools: Option<Value> = None;
    for ev in events.iter().rev() {
        if content.is_none() && ev.kind == "system/message" {
            content = Some(
                ev.data
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            );
            // 旧日志：tools 挂在 system/message 上
            if tools.is_none() {
                if let Some(t) = ev.data.get("tools") {
                    tools = Some(t.clone());
                }
            }
        }
        if tools.is_none() && ev.kind == "tools/catalog" {
            tools = Some(
                ev.data
                    .get("tools")
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new())),
            );
        }
        if content.is_some() && tools.is_some() {
            break;
        }
    }
    content.map(|c| (c, tools.unwrap_or_else(|| Value::Array(Vec::new()))))
}

/// 比较两份 tools JSON 是否语义相同（序列化后比对）。
pub fn tools_equal(a: &Value, b: &Value) -> bool {
    a == b
}

/// 最近一次 `skills/catalog` 的条目 JSON（用于变更检测）。
pub fn last_skills_catalog(events: &[SessionEvent]) -> Option<Value> {
    for ev in events.iter().rev() {
        if ev.kind == "skills/catalog" {
            return Some(
                ev.data
                    .get("skills")
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new())),
            );
        }
    }
    None
}

pub fn ev_request_header(
    model: &str,
    backend: &str,
    message_count: usize,
    tool_ids: &[String],
    stream_mode: Option<&str>,
    desync: &Value,
    delta: &Value,
) -> Value {
    json!({
        "model": model,
        "backend": backend,
        "message_count": message_count,
        "tools": tool_ids,
        "stream_mode": stream_mode,
        "desync_check": desync,
        "delta": delta,
    })
}

/// 将运行时 messages 切片序列化为 request/header.delta.messages。
pub fn messages_delta_value(msgs: &[crate::agent::llm::ModelMessage]) -> Value {
    Value::Array(
        msgs.iter()
            .map(|m| {
                let mut obj = serde_json::Map::new();
                obj.insert("role".into(), Value::String(m.role.clone()));
                if !m.content.is_empty() {
                    obj.insert("content".into(), Value::String(m.content.clone()));
                }
                if !m.tool_calls.is_empty() {
                    obj.insert(
                        "tool_calls".into(),
                        Value::Array(
                            m.tool_calls
                                .iter()
                                .map(|c| {
                                    json!({
                                        "id": c.id,
                                        "name": c.name,
                                        "args": c.arguments,
                                    })
                                })
                                .collect(),
                        ),
                    );
                }
                if let Some(id) = &m.tool_call_id {
                    obj.insert("tool_call_id".into(), Value::String(id.clone()));
                }
                if let Some(name) = &m.tool_name {
                    obj.insert("tool_name".into(), Value::String(name.clone()));
                }
                Value::Object(obj)
            })
            .collect(),
    )
}

pub fn ev_assistant_message(reasoning: Option<&str>, text: Option<&str>, tool_calls: &Value) -> Value {
    json!({
        "reasoning": reasoning,
        "text": text,
        "tool_calls": tool_calls,
    })
}

pub fn ev_tool_call(tool_id: &str, args: &Value) -> Value {
    json!({ "tool_id": tool_id, "args": args })
}

pub fn ev_tool_result(tool_id: &str, ok: bool, result: &str) -> Value {
    json!({ "tool_id": tool_id, "ok": ok, "result": result })
}

pub fn ev_context_snapshot(source: &str, chars: usize) -> Value {
    json!({ "source": source, "chars": chars })
}

pub fn ev_compact_checkpoint(count: usize, message: &str, strategies: &[String]) -> Value {
    json!({
        "kind": "compress",
        "count": count,
        "message": message,
        "strategies": strategies,
    })
}

pub fn user_delta_value(user_text: &str) -> Value {
    json!({
        "messages": [{ "role": "user", "content": user_text }]
    })
}

pub fn wrap_delta_messages(messages: Value) -> Value {
    json!({ "messages": messages })
}

/// 压缩后的 delta：带 compressed/strategy，messages 为 Runtime 中 system 之后的视图。
pub fn compressed_delta_value(
    strategy: &str,
    reset: bool,
    after_system: &[crate::agent::llm::ModelMessage],
) -> Value {
    json!({
        "compressed": true,
        "strategy": strategy,
        "reset": reset,
        "kept": ["system"],
        "messages": messages_delta_value(after_system),
    })
}

pub fn ev_step_end(turn_no: usize, step_no: usize) -> Value {
    json!({ "turn_no": turn_no, "step_no": step_no })
}

pub fn ev_turn_end(turn_no: usize) -> Value {
    json!({ "turn_no": turn_no })
}

// ---------------------------------------------------------------------------
// 日志重建一致性校验（log-reconstruction desync check）
// ---------------------------------------------------------------------------

/// 从事件日志推导模型上下文骨架（仅角色序列，用于计数比对）。
///
/// 实际请求始终以单条 system 打头；日志里每回合可有一条 `system/message`，
/// 推导时最多前置一条 `"system"`（与 msgs[0] 对齐）。`context/snapshot` 仅元数据，不计角色。
pub fn derive_message_skeleton(events: &[SessionEvent]) -> Vec<String> {
    let mut roles: Vec<String> = Vec::new();
    if events.iter().any(|e| e.kind == "system/message") {
        roles.push("system".into());
    }
    for ev in events {
        match ev.kind.as_str() {
            // user 输入 → 一条 user 消息（本轮输入在循环开始时已入库为 user 消息，
            // 故推导时计入）
            "user/message" => roles.push("user".into()),
            // assistant 完整回复：文本 → 一条 assistant；工具调用 → 一条
            // assistant(tool_calls) + 每个工具一条 tool 结果（tool/result 事件补）
            "assistant/message" => {
                let has_tools = ev
                    .data
                    .get("tool_calls")
                    .and_then(|v| v.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if has_tools {
                    roles.push("assistant(tools)".into());
                } else {
                    roles.push("assistant".into());
                }
            }
            "tool/result" => roles.push("tool".into()),
            _ => {}
        }
    }
    roles
}

/// 比较推导骨架与实际请求消息列表（仅比较角色序列）。
///
/// `runtime_compressed` 为 true 时表示本回合 Runtime 已做过上下文压缩（keep_recent /
/// reset 等）：审计日志仍保留完整历史，与 Runtime 视图故意不一致。本步及同回合
/// 后续步均应传 true，避免误报；两侧计数仍写入便于排查。
pub fn desync_check_value(
    events: &[SessionEvent],
    actual_messages: &[Value],
    runtime_compressed: bool,
) -> Value {
    let derived = derive_message_skeleton(events);
    let actual_roles: Vec<String> = actual_messages
        .iter()
        .map(|m| {
            let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("");
            // 实际请求里 tool_calls 可能是 bool 标记（loop 比对用）或数组
            let has_tools = match m.get("tool_calls") {
                Some(Value::Bool(true)) => true,
                Some(Value::Array(a)) if !a.is_empty() => true,
                _ => false,
            };
            if role == "assistant" && has_tools {
                "assistant(tools)".to_string()
            } else {
                role.to_string()
            }
        })
        .collect();
    let derived_cmp: Vec<&str> = derived.iter().map(|s| s.as_str()).collect();
    let actual_cmp: Vec<&str> = actual_roles.iter().map(|s| s.as_str()).collect();
    let matches = runtime_compressed || derived_cmp == actual_cmp;
    json!({
        "derived_messages": derived_cmp.len(),
        "actual_messages": actual_cmp.len(),
        "derived_roles": derived_cmp,
        "actual_roles": actual_cmp,
        "match": matches,
        "compressed": runtime_compressed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::conversation::append_user_message_on;

    fn test_db() -> Connection {
        let mut dir = std::env::temp_dir();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("tbox-session-log-test-{unique}"));
        std::fs::create_dir_all(&dir).unwrap();
        let conn = Connection::open(dir.join("tools.db")).unwrap();
        ensure_session_event_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn append_assigns_increasing_seq_and_roundtrips() {
        let db = test_db();
        let (conv, _) = append_user_message_on(&db, None, "hi").unwrap();
        let e1 = append_event_on(&db, &conv.id, "turn/start", ev_turn_start(1)).unwrap();
        let e2 = append_event_on(&db, &conv.id, "step/start", ev_step_start(1, 1)).unwrap();
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        assert!(e2.time >= e1.time);

        let events = list_events_on(&db, &conv.id).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "turn/start");
        assert_eq!(events[1].kind, "step/start");
        assert_eq!(events[1].data["step_no"], 1);
    }

    #[test]
    fn last_system_snapshot_and_change_events() {
        let tools_a = json!([{"type":"function","function":{"name":"a","description":"A","parameters":{}}}]);
        let tools_b = json!([{"type":"function","function":{"name":"b","description":"B","parameters":{}}}]);
        let events = vec![
            SessionEvent {
                seq: 1,
                conversation_id: "c".into(),
                time: 0,
                kind: "system/message".into(),
                data: ev_system_message("sys-v1"),
            },
            SessionEvent {
                seq: 2,
                conversation_id: "c".into(),
                time: 0,
                kind: "tools/catalog".into(),
                data: ev_tools_catalog(&tools_a),
            },
        ];
        let (c, t) = last_system_snapshot(&events).unwrap();
        assert_eq!(c, "sys-v1");
        assert!(tools_equal(&t, &tools_a));
        assert!(!tools_equal(&tools_a, &tools_b));
        let upd = ev_system_prompt_updated("sys-v1", "sys-v2");
        assert_eq!(upd["title"], "静态系统提示词已更新");
        assert_eq!(upd["previous_content"], "sys-v1");
        let tools_upd = ev_tools_catalog_updated(&tools_a, &tools_b);
        assert_eq!(tools_upd["title"], "工具已更新");
    }

    #[test]
    fn append_only_never_rewrites_history() {
        let db = test_db();
        let (conv, _) = append_user_message_on(&db, None, "hi").unwrap();
        let _ = append_event_on(&db, &conv.id, "user/message", ev_user_message("v1")).unwrap();
        let _ = append_event_on(&db, &conv.id, "user/message", ev_user_message("v2")).unwrap();
        let events = list_events_on(&db, &conv.id).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data["content"], "v1"); // 原始事件未被改写
        assert_eq!(events[1].data["content"], "v2");
    }

    #[test]
    fn derive_skeleton_counts_roles() {
        let events = vec![
            SessionEvent { seq: 1, conversation_id: "c".into(), time: 0, kind: "user/message".into(), data: ev_user_message("hi") },
            SessionEvent { seq: 2, conversation_id: "c".into(), time: 0, kind: "system/message".into(), data: ev_system_message("you are helpful") },
            SessionEvent { seq: 3, conversation_id: "c".into(), time: 0, kind: "assistant/message".into(), data: ev_assistant_message(None, None, &json!([{"id":"t1"}])) },
            SessionEvent { seq: 4, conversation_id: "c".into(), time: 0, kind: "tool/result".into(), data: ev_tool_result("t1", true, "r") },
        ];
        let roles = derive_message_skeleton(&events);
        assert_eq!(roles, vec!["system", "user", "assistant(tools)", "tool"]);
    }

    #[test]
    fn context_snapshot_does_not_count_as_system_role() {
        let events = vec![
            SessionEvent { seq: 1, conversation_id: "c".into(), time: 0, kind: "user/message".into(), data: ev_user_message("hi") },
            SessionEvent { seq: 2, conversation_id: "c".into(), time: 0, kind: "context/snapshot".into(), data: ev_context_snapshot("memory", 12) },
            SessionEvent { seq: 3, conversation_id: "c".into(), time: 0, kind: "system/message".into(), data: ev_system_message("sys") },
        ];
        let roles = derive_message_skeleton(&events);
        assert_eq!(roles, vec!["system", "user"]);
    }

    #[test]
    fn desync_check_detects_mismatch() {
        let events = vec![SessionEvent {
            seq: 1,
            conversation_id: "c".into(),
            time: 0,
            kind: "user/message".into(),
            data: ev_user_message("hi"),
        }];
        let actual_match = vec![json!({"role": "user", "content": "hi"})];
        let v = desync_check_value(&events, &actual_match, false);
        assert_eq!(v["match"], true);

        let actual_bad = vec![
            json!({"role": "user", "content": "hi"}),
            json!({"role": "user", "content": "extra"}),
        ];
        let v2 = desync_check_value(&events, &actual_bad, false);
        assert_eq!(v2["match"], false);

        let v3 = desync_check_value(&events, &actual_bad, true);
        assert_eq!(v3["match"], true);
        assert_eq!(v3["compressed"], true);
    }
}
