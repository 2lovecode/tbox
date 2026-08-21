# Task 1.1 — 会话 SQLite 与纯函数 API

来源：`openspec/changes/add-chat-home-agent/plan.md` Task 1.1

## Spec binding

`openspec/changes/add-chat-home-agent/specs/agent-chat/spec.md` — Requirement: Conversation History

- 空会话在发出第一条用户消息之前 MUST NOT 写入持久化存储。
- 第一条用户消息 MUST 被持久化，并用于生成短标题。
- 删除会话 MUST 同时移除其消息。

## Files

- Create: `src-tauri/src/commands/conversation.rs`
- Modify: `src-tauri/src/commands/mod.rs`（`pub mod conversation;`）
- Modify: `src-tauri/src/commands/tool.rs` 的 `init_db_if_needed`（或 `conversation` 内 `ensure_conversation_schema`，由启动路径调用）
- Test: `conversation.rs` 内 `#[cfg(test)]`，用临时 DB，不要写用户 `~/.toolbox/tools.db`

## Interfaces（必须按此签名）

```rust
pub struct Conversation { pub id: String, pub title: String, pub updated_at: i64 }
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub tool_calls_json: Option<String>,
    pub created_at: i64,
}
pub fn append_user_message(conversation_id: Option<String>, content: &str) -> Result<(Conversation, ChatMessage), String>
pub fn list_conversations() -> Result<Vec<Conversation>, String>
pub fn get_messages(conversation_id: &str) -> Result<Vec<ChatMessage>, String>
pub fn delete_conversation(conversation_id: &str) -> Result<(), String>
```

- `append_user_message(None, …)` 创建会话；标题为 content 截断（≤40 字）
- **没有** `create_empty_conversation` 落库函数
- 测试可用 `append_user_message_on(&Connection, …)` 等注入连接的辅助函数

## Tests（先写再实现，必须看到 RED）

```rust
#[test]
fn first_user_message_creates_conversation() { /* 见 plan.md */ }

#[test]
fn delete_removes_messages() { /* 见 plan.md */ }
```

再补一条：没有任何 `append_user_message` 时 `list_conversations_on` 为空（证明没有隐式建空会话）。

## Schema

```sql
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
  created_at INTEGER NOT NULL,
  FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
```

`id` 用 UUID 字符串。启用 SQLite foreign_keys。

## Verify

```
cd src-tauri && cargo test --lib commands::conversation
cd src-tauri && cargo check
```

## Commit

只 add：

- `src-tauri/src/commands/conversation.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/tool.rs`（若改了 init）
- `openspec/changes/add-chat-home-agent/tasks.md`（勾 1.1）

工作区很脏。**禁止** `git add .` 或加入其它未跟踪文件。

```
git commit -m "feat: persist chat conversations on first user message"
```

勾选 tasks.md：`- [x] 1.1 ...`
