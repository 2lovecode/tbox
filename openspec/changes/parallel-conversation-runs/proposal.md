## Why

当前 Agent 回合依赖全局单一取消标志，前端也会丢弃非当前会话的流式事件，用户无法在多个会话间并行生成，也无法从侧栏看出哪些会话仍在运行。需要进程内运行注册表 + 侧栏两态指示，并支持切走后台继续、切回接上进度。

## What Changes

- 多会话可同时运行 Agent 回合（每会话最多一轮 in-flight）
- 侧栏历史行左侧两态图标：空闲 = 消息图标；运行中 = 转圈（`--warning`）
- `cancel_chat_turn` **BREAKING**：必须传入 `conversationId`，仅取消该会话
- 同会话在 `running` 时再次 `send_chat_turn` → 错误码 `turn_in_progress`
- 新增事件 `agent-run-status`（`conversationId` + `running|idle`）
- 切走不取消；切回展示内存中的 live 缓冲并继续流式
- 删除仍在运行的会话：先 cancel 再删
- 取消入口仍仅为当前打开会话的 composer 停止按钮（侧栏无停止）

### Non-goals

- 侧栏取消按钮
- 失败 / 未读角标
- SQLite 持久化 run 表或全局并发上限
- 同会话排队第二轮（仅拒绝）

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-chat`: 多会话并行回合、按会话取消、运行状态事件、侧栏运行指示、切回接流、删除时先取消

## Impact

- 前端：`SideBar.vue`、`HomePage.vue`、新建 `stores/agentRuns.ts`；轻触 `stores/conversations.ts`（删除前 cancel）
- Rust：`agent/run_registry.rs`；`commands/agent.rs`（send/cancel + 状态事件）；`lib.rs` 管理注册表，移除全局 `AgentCancel`
- 契约：**BREAKING** — `cancel_chat_turn` 签名变更
