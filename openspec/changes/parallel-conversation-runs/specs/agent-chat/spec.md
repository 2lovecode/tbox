## ADDED Requirements

### Requirement: Parallel Conversation Agent Turns
系统 SHALL 允许不同会话同时各有一轮 in-flight Agent 回合。同一会话 MUST 最多一轮；若该会话已在运行中再次发起 `send_chat_turn`，系统 MUST 拒绝并返回错误码 `turn_in_progress`，MUST NOT 启动第二轮。进程内运行注册表 SHALL 按 `conversation_id` 持有状态与取消标志；应用重启后注册表清空。

#### Scenario: Two conversations run concurrently
- **WHEN** 会话 A 正在生成且用户切换到会话 B 并成功发送新消息
- **THEN** A 与 B 的 Agent 回合同时进行，互不取消对方

#### Scenario: Same conversation rejected while running
- **WHEN** 会话 A 仍在 `running` 时再次对其调用 `send_chat_turn`
- **THEN** 命令失败，错误包含 `turn_in_progress`，且不启动第二轮

### Requirement: Per-Conversation Cancel
系统 SHALL 通过 `cancel_chat_turn(conversationId)` 仅取消指定会话的 in-flight 回合。取消未知或未运行会话 MUST 返回成功（no-op）。前端 MUST 仅在当前打开且正在运行的会话上通过 composer 停止按钮调用取消；MUST NOT 在侧栏提供取消入口。

#### Scenario: Cancel only active conversation
- **WHEN** 会话 A 与 B 均在运行，用户在打开 B 时点击停止
- **THEN** 仅 B 被中断，A 继续运行

#### Scenario: Cancel unknown id is no-op
- **WHEN** 对不存在或不在运行的 `conversationId` 调用 `cancel_chat_turn`
- **THEN** 命令成功返回，不抛错

### Requirement: Agent Run Status Events
系统 SHALL 在回合开始时发出 `agent-run-status`（`conversationId`，`status=running`），并在 Done、Interrupted 或致命 Error 收尾时发出 `status=idle`。既有 `agent-event` 流式事件 MUST 继续携带 `conversationId`，且无论该会话是否为当前打开会话 MUST 继续派发。

#### Scenario: Status running then idle
- **WHEN** 用户对某会话成功发起 Agent 回合并随后完成
- **THEN** 前端先后收到该会话的 `running` 与 `idle` 状态事件

### Requirement: Sidebar Run Indicators
系统 SHALL 在侧栏每条历史会话标题左侧展示状态图标：空闲为消息图标；该会话 `running` 时为旋转/脉冲运行图标（颜色使用 warning 语义）。整行点击仍打开会话；图标 MUST NOT 作为独立取消控件。

#### Scenario: Running icon while generating
- **WHEN** 某已列出会话处于 Agent 运行中
- **THEN** 该行左侧显示运行中图标，其他空闲会话显示消息图标

### Requirement: Reconnect Live Buffer After Switch
系统 SHALL 在用户离开仍在运行的会话时保持后台生成与内存 live 缓冲；用户切回该会话时 MUST 展示已缓冲内容，若仍在运行则继续追加流式更新。回合结束后 Rust 侧已持久化的助手消息为真相来源；前端 MUST 清除该会话 live 缓冲并将侧栏状态恢复为空闲。

#### Scenario: Switch away and back while running
- **WHEN** 会话 A 生成中用户切到 B 再切回 A，且 A 尚未结束
- **THEN** A 展示已生成内容并继续流式更新

### Requirement: Delete Cancels Running Turn
当用户删除仍在 `running` 的会话时，系统 SHALL 先取消该会话回合，再删除存储与列表项，并清除该会话的运行状态指示。

#### Scenario: Delete running conversation
- **WHEN** 用户确认删除一条正在生成的会话
- **THEN** 该回合被取消，会话从列表与存储移除，侧栏不再显示其运行图标
