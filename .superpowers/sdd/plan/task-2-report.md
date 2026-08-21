# Task 1.2 Report — 注册会话 Tauri 命令

**Status:** DONE  
**Branch:** `add-chat-home-agent`  
**Commit:** `ad0a18f` — feat: expose conversation commands to the frontend

## Summary

在 Task 1.1 纯函数 API 基础上，为 `Conversation` / `ChatMessage` 增加 `Serialize`，用 `#[tauri::command]` 暴露四个 invoke 命令，并在 `lib.rs` 的 `generate_handler!` 中注册。内部 `get_messages(&str)` 保持不变；前端 invoke 名为 `get_conversation_messages`。`append_user_message` 参数 `conversationId`（驼峰）与 `content: String` 与 brief 一致。

## Files Changed

| File | Action |
|------|--------|
| `src-tauri/src/commands/conversation.rs` | Added `Serialize`, `#[tauri::command]` wrappers |
| `src-tauri/src/lib.rs` | Registered four conversation commands |
| `openspec/changes/add-chat-home-agent/tasks.md` | Checked 1.2 |

## Tauri Commands Registered

| Invoke name | Rust symbol | Notes |
|-------------|-------------|-------|
| `list_conversations` | `list_conversations` | Returns `Vec<Conversation>` |
| `get_conversation_messages` | `get_conversation_messages` | Wraps internal `get_messages` |
| `delete_conversation` | `delete_conversation` | Takes `conversation_id: String` |
| `append_user_message` | `append_user_message` | `conversationId: Option<String>`, `content: String` |

## Implementation Notes

- **Serialize:** `Conversation`, `ChatMessage` derive `Serialize` for JSON responses.
- **No schema/business changes:** `*_on` helpers and SQL unchanged from 1.1.
- **get_messages:** Remains internal `pub fn get_messages(conversation_id: &str)`; not registered as Tauri command.
- **append_user_message:** Parameter named `conversationId` (with `#[allow(non_snake_case)]`) so frontend `invoke('append_user_message', { conversationId, content })` deserializes without extra serde struct. `content` changed from `&str` to owned `String` for Tauri IPC.
- **delete_conversation / get_conversation_messages:** Take owned `String` for conversation id (Tauri convention).

## Verify

**Command:**
```
cd src-tauri && cargo check
```

**Result:** Exit code **0** — `Finished dev profile` (one pre-existing dead_code warning in `db.rs`; no new errors).

## Self-Review

- [x] Command names match brief verbatim.
- [x] `get_conversation_messages` wraps `get_messages`, does not rename internal API.
- [x] No SQLite schema or persistence rule changes.
- [x] Only brief-listed files modified for commit scope.
- [x] Task 1.1 unit tests untouched; compile check sufficient per TDD note.

## Frontend Usage (reference)

```typescript
await invoke('list_conversations')
await invoke('get_conversation_messages', { conversationId: id })
await invoke('delete_conversation', { conversationId: id })
await invoke('append_user_message', { conversationId: null, content: 'hello' })
```
