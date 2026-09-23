## 1. Rust 运行注册表

- [x] 1.1 新增 `src-tauri/src/agent/run_registry.rs`（`try_begin` / `cancel` / `finish` / `is_running`），并在 `agent/mod.rs` 导出；验证：`cargo test --lib agent::run_registry`
- [x] 1.2 将 `lib.rs` 管理状态改为 `Arc<RunRegistry>`，移除全局 `AgentCancel`；验证：`cargo check`

## 2. Agent 命令与事件

- [x] 2.1 `send_chat_turn`：`try_begin`、发射 `agent-run-status=running`，worker 收尾 `finish` + `idle`；同会话重复 → `turn_in_progress`；验证：`cargo check`
- [x] 2.2 `cancel_chat_turn(conversationId)` 仅取消该会话；未知/idle no-op；验证：`cargo check` + 单测覆盖 cancel/finish

## 3. 前端运行态 store

- [x] 3.1 新增 `src/stores/agentRuns.ts`（`runStatusById`、`liveById`、`applyAgentEvent`、`clearLive`、`clearConversation`）；验证：`npx vue-tsc --noEmit`

## 4. HomePage 事件路由

- [x] 4.1 监听 `agent-event` 按 `conversationId` 写入 store（不再丢弃非 active）；监听 `agent-run-status`；`turnBusy`/轨迹取自 active；验证：手动双会话 mock 流式
- [x] 4.2 `cancel_chat_turn` 传入当前 `activeId`；`turn_in_progress` 展示错误文案；验证：手动

## 5. 侧栏与删除

- [x] 5.1 `SideBar.vue` 行首两态图标（`fa-message` / `fa-spinner fa-spin` + `--warning`）；验证：视觉检查
- [x] 5.2 `deleteConversation`：若 running 先 cancel 再删并 `clearConversation`；验证：手动删运行中会话

## 6. 验收

- [ ] 6.1 对照设计验收矩阵（并行、仅停当前、切回接流、图标回落、`turn_in_progress`、删运行中）；验证：手动清单全部勾选
