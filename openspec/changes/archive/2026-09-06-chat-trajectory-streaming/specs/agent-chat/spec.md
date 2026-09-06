## ADDED Requirements

### Requirement: Assistant Turn Trajectory Display
聊天界面 SHALL 将同一助手回合内的思考、工具调用与正文按发生顺序渲染为一条时间线（轨迹）。流式进行中各步骤 MUST 默认展开并随事件增量更新；回合结束后（及历史重开）思考与工具步骤 MUST 默认收起，正文 MUST 保持展开。用户可手动切换某步展开/收起；重开会话后 MUST 恢复上述默认折叠规则。

#### Scenario: Streaming turn shows ordered steps expanded
- **WHEN** 一轮包含思考与工具调用的回复正在流式生成
- **THEN** 界面按顺序展示思考、工具（参数与结果状态）、正文，且各步骤默认展开并随事件更新

#### Scenario: Completed turn collapses process steps
- **WHEN** 该回合完成或用户重新打开含轨迹的历史会话
- **THEN** 思考与工具步骤默认收起，助手正文保持展开可见

#### Scenario: Manual expand does not persist across reopen
- **WHEN** 用户在已完成回合中手动展开某思考或工具步骤后关闭并重新打开该会话
- **THEN** 该步骤再次以默认收起状态展示

### Requirement: Trajectory Persistence
带过程步骤的助手消息 SHALL 将有序轨迹持久化到本地 SQLite（`messages` 新增 `trajectory_json` 列，或等价结构化字段）；历史会话重开时 MUST 按原顺序恢复时间线。已存在的数据库 MUST 通过向后兼容 migration 加列；旧消息无轨迹时 MUST 由 `reasoning`、`tool_calls_json`、`content` 合成一条降级时间线（顺序：思考 → 工具 → 正文）。

#### Scenario: Reopen restores trajectory order
- **WHEN** 用户重新打开一条含多步工具与思考的历史会话
- **THEN** 时间线步骤顺序与生成时一致，工具调用可见而不丢失

#### Scenario: Legacy message without trajectory
- **WHEN** 旧库消息仅有 content/reasoning/tool_calls 而无轨迹字段
- **THEN** 界面仍展示合成时间线，应用不崩溃

### Requirement: Stream Mode Indication
Agent 循环在每轮模型调用开始时 SHALL 经 `agent-event` 下发流式模式元数据：`live`（真流式）或 `fallback`（整段完成后分块）。当模式为 `fallback` 时，前端 MUST 在该助手回合时间线上提供轻量可见提示（例如「整段生成」角标）；`live` 时 MUST NOT 强制显示该提示。

#### Scenario: Fallback mode shows badge
- **WHEN** 当前后端不支持真流式并以分块降级推送
- **THEN** 前端在本回合时间线显示 fallback 轻提示，且仍逐步渲染文本

#### Scenario: Live mode without fallback badge
- **WHEN** 后端以真流式推送 token/reasoning
- **THEN** 不显示「整段生成」类 fallback 提示

## MODIFIED Requirements

### Requirement: Agent Tool Loop
系统 SHALL 在 Rust 侧执行 Agent 循环：系统提示构建、输出解析、参数校验与修复重试由统一 harness 策略层承担（见 agent-tool-harness 能力）；策略按当前后端/模型插拔，所有后端 MUST 经由该层。循环将系统提示、检索到的 Skill、会话消息发给当前 LLM；若模型返回 tool call，则仅调度注册表中的工具并把结果回填；参数校验不合格或调用被拒绝时 MUST 在修复预算内回填错误并重试（reask），而不是直接终止回合。前端 MUST 以流式事件按时间线展示 token、reasoning 与工具调用（名称、参数、结果或失败），且工具步骤 MUST 出现在同一助手回合轨迹内（不得仅在回合进行中短暂显示、结束后从历史中消失）。

#### Scenario: Single tool call then answer
- **WHEN** 用户发送需要 Base64 解码的请求且本地或云端 LLM 可用
- **THEN** 对话中出现对应工具调用记录与结果，并跟有助手对结果的说明

#### Scenario: Sequential tool calls
- **WHEN** 用户请求先 Base64 解码再计算哈希，且两个工具均已注册
- **THEN** 系统按循环依次执行这两个工具，并在同一轮对话轨迹中按序展示两次调用与最终回复

#### Scenario: Invalid tool call repaired via reask
- **WHEN** 模型返回的工具调用参数不符合 JSON Schema
- **THEN** 系统不调用底层 command，将校验错误回填给模型并重新请求；修复预算内得到合法调用则继续执行，预算耗尽则按错误语义收尾

#### Scenario: Cancel in-flight turn
- **WHEN** 用户在 Agent 循环尚未完成时取消发送
- **THEN** 循环停止；已持久化的消息与已收到的轨迹步骤保留；进行中的助手输出标记为中断而非删除整条会话

#### Scenario: Tool calls visible after turn completes
- **WHEN** 含工具调用的助手回合结束并写入存储
- **THEN** 刷新或重开该会话后，工具步骤仍出现在该回合时间线中

### Requirement: Message Visual Hierarchy
消息区 UI SHALL 统一视觉规范：用户/助手气泡区分对齐与配色，助手消息支持流式输出指示（输出中状态），工具调用步骤 MUST 展示工具名、状态（运行中/完成/失败）、参数与结果（等宽字体、可滚动、限高折叠），并嵌入助手回合时间线而非与消息流脱节的孤立区域；错误提示条 MUST 与消息流视觉区分且不遮挡输入区。

#### Scenario: Streaming assistant indicator
- **WHEN** 助手回复正在流式输出
- **THEN** 消息区呈现输出中状态指示，结束后指示消失

#### Scenario: Tool call card presentation
- **WHEN** 一轮对话包含工具调用
- **THEN** 工具步骤按顺序展示工具名、状态徽标、参数与结果，参数/结果以等宽字体限高展示并可滚动

#### Scenario: Error bar stays visible
- **WHEN** 本轮出现错误提示
- **THEN** 错误条在输入区上方可见，不随消息滚动隐藏，不遮挡输入框
