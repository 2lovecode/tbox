# Task 3.2 — 接会话列表

来源：`openspec/changes/add-chat-home-agent/plan.md` Task 3.2  
Spec：Conversation History（空会话不进历史；首条消息落库并出标题；可删除）

## Files

- Create: `src/stores/conversations.ts`
- Modify: `src/layout/SideBar.vue`
- Modify: `src/views/HomePage.vue`
- Modify: `openspec/changes/add-chat-home-agent/tasks.md`（勾 3.2）

## Interfaces

Store（Pinia，对齐现有 `settings.ts` / `llm.ts` 风格）:

- `items: Conversation[]` — 来自 `invoke('list_conversations')`
- `activeId: string | null`
- `draftId: string | null` — 仅前端临时 id；**未**调用 `append_user_message` 前不得进入 `items`
- `messages: ChatMessage[]` — 当前会话消息
- `newChat()` — 设新 `draftId`、清空 `messages`、`activeId=null`；不 invoke 创建空会话
- `loadList()` — `list_conversations`
- `openConversation(id)` — `get_conversation_messages` + 设 activeId、清 draftId
- `deleteConversation(id)` — `delete_conversation` 后刷新 list
- `sendFirst(content)` 或 `appendUser(content)` — `invoke('append_user_message', { conversationId: activeId ?? null, content })`；用返回的 conversation 替换 draft、刷新 list、把 user message 放进 messages

后端命令（已注册）:

- `list_conversations`
- `get_conversation_messages(conversationId: string)`
- `delete_conversation(conversationId: string)`
- `append_user_message(conversationId: Option<String>, content: String)` — 检查 Rust 侧参数名（camelCase vs snake）；前端按实际签名传参

SideBar：展示 `items`（标题、点开、删除）；新建调用 `newChat()`；空 draft **不**出现在历史。

HomePage：启用输入框；发送时若无 activeId 走首条消息路径；展示 messages（user/assistant 文本即可，工具卡片留给 4.1）。

## Verify

- 类型：本任务新增文件无新的明显 TS 错误。仓库里 SettingsModal/Spotlight 等既有 `vue-tsc` 错误可忽略，不要为修它们扩大范围。
- 逻辑：`newChat` 后 `items` 不变；`append_user_message` 成功后 list 有标题；delete 后消失。

## Commit

只 add 上述文件。Never `git add .`

```
git commit -m "feat: wire conversation list and first-message persist"
```

勾选 tasks.md 3.2。

## Report

写到 `.superpowers/sdd/plan/task-7-report.md`（本控制器编号；对应 plan 3.2）
