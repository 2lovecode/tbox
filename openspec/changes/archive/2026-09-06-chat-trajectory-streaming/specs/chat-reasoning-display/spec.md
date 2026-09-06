## ADDED Requirements

### Requirement: True Streaming Preferential Emission
模型后端在支持流式时 SHALL 在生成过程中增量发出 `reasoning` / `token`（或等价 delta）事件，而不是仅在整段生成结束后再切块。Agent 循环 MUST 优先消费流式接口；当协议或客户端无法真流式时 MUST 降级为整段完成后的分块发出，并配合 `stream_meta.mode=fallback`（见 agent-chat）。半截工具调用 JSON MUST NOT 被调度执行；完整 `tool_calls` 回合结束后再进入既有校验与 dispatch。

#### Scenario: Live deltas during generation
- **WHEN** 支持流式的后端（如本地嵌入引擎或 OpenAI 兼容 SSE）正在生成长回复
- **THEN** 前端在生成完成前持续收到多个增量 `token` 和/或 `reasoning` 事件并逐步渲染

#### Scenario: Unsupported protocol falls back
- **WHEN** 当前协议无法提供真流式
- **THEN** 系统在发出文本前下发 `fallback` 元数据，再以分块方式发出完整思考与正文，行为对用户仍为逐步渲染且不报错

#### Scenario: Partial tool JSON not executed
- **WHEN** 流式过程中工具调用参数尚未形成完整可解析调用
- **THEN** 系统不执行该工具，待本轮模型输出完整后再按既有 Agent 循环处理

## MODIFIED Requirements

### Requirement: Reasoning Event Streaming
Agent 循环 SHALL 通过 `agent-event` 事件向前端下发思考内容（`reasoning` 事件类型），前端在流式期间能区分「思考中」与「正文输出中」两种阶段。思考增量 MUST 作为助手回合时间线中的思考步骤展示（见 agent-chat 轨迹要求），不得仅在有正文前短暂显示标题而无可展开内容。

#### Scenario: Reasoning streamed during turn
- **WHEN** 一轮包含思考内容的回复正在生成
- **THEN** 前端先收到或交错收到 `reasoning` 事件（思考文本），并与随后的 `token` 事件区分；时间线中思考步骤内容随增量可见

#### Scenario: Turn without reasoning
- **WHEN** 一轮回复没有思考内容
- **THEN** 前端只收到 `token` 等既有事件，不收到空的 `reasoning` 事件，也不渲染空思考步骤

### Requirement: Collapsed-by-Default Reasoning Display
前端 SHALL 在助手回合时间线中展示「思考过程」步骤；模型未提供思考内容时 MUST NOT 渲染该步骤。流式进行中思考步骤 MUST 默认展开；回合结束后与历史重开时 MUST 默认收起，点击标题 MUST 可切换展开/收起。

#### Scenario: Reasoning expanded while streaming
- **WHEN** 思考内容正在流式到达且回合尚未结束
- **THEN** 思考步骤默认展开并显示已到达的思考文本

#### Scenario: Reasoning collapsed by default
- **WHEN** 一条带思考内容的助手消息渲染完成
- **THEN** 时间线中出现「思考过程」步骤，内容默认不展开，仅显示摘要标题

#### Scenario: Toggle reasoning visibility
- **WHEN** 用户点击思考步骤标题
- **THEN** 思考内容在展开与收起之间切换，再次点击恢复

#### Scenario: No reasoning no block
- **WHEN** 助手消息没有思考内容
- **THEN** 不出现「思考过程」步骤

### Requirement: Chunked Streaming Emission
Agent 循环 SHALL 将思考与正文以增量事件流式发出。真流式后端 MUST 在生成过程中发出增量；降级路径 MUST 将完整文本分块经 `reasoning` / `token` 发出（而非整段一次性单事件），前端流式渲染与自动跟随行为保持可用。本要求不排除在 `fallback` 模式下使用分块策略。

#### Scenario: Tokens arrive in chunks
- **WHEN** 模型回合输出多段文本（真流式或降级分块）
- **THEN** 前端在回合收尾前持续接收多个 token/reasoning 事件并逐步渲染

#### Scenario: Fallback still chunked
- **WHEN** 后端以 fallback 模式完成整段生成后下发文本
- **THEN** 文本仍经多个分块事件到达前端，而不是单个巨包事件导致界面一次性跳变（空文本除外）
