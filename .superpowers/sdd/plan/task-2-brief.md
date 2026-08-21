# Task 1.2 — 注册会话 Tauri 命令

来源：plan.md Task 1.2

## Files

- Modify: `src-tauri/src/commands/conversation.rs`（`#[tauri::command]`）
- Modify: `src-tauri/src/lib.rs` `generate_handler!`

## Interfaces（必须按此命令名）

- `list_conversations`
- `get_conversation_messages`（包装 `get_messages`；Tauri 命令名与内部函数可以不同）
- `delete_conversation`
- `append_user_message` 参数：`conversationId: Option<String>`, `content: String`（serde 驼峰：`conversationId`）

Tauri 命令返回值需可序列化。给 `Conversation` / `ChatMessage` 加 `Serialize`（若尚未加）。`append_user_message` 若目前签名是 `&str`，可保留内部函数、另写 command 包装，或把参数改成 `String` 并保持内部行为不变。

不要改 schema、不要改业务规则。

## Verify

`cd src-tauri && cargo check`

## Commit

只 add 上述文件 + `openspec/changes/add-chat-home-agent/tasks.md` 勾 1.2。禁止 `git add .`。

`git commit -m "feat: expose conversation commands to the frontend"`
